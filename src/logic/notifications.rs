use anyhow::{Context, Ok, Result};
use stated_dialogues::controller::BotAdapter;

use crate::db::{
    models::{HostId, UserId},
    Registry,
};

#[derive(Debug)]
pub enum Notification {
    HostsRelased((Vec<HostId>, UserId)),
}

pub struct Notifier<T: BotAdapter> {
    registry: Registry,
    tg_adapter: T,
}

impl<T> Notifier<T>
where
    T: BotAdapter,
{
    pub fn new(registry: Registry, tg_adapter: T) -> Self {
        Self {
            registry,
            tg_adapter,
        }
    }

    pub async fn notify(&self, notification: Notification) -> Result<()> {
        match notification {
            Notification::HostsRelased((hosts_ids, user_id)) => {
                if hosts_ids.is_empty() {
                    return Ok(());
                }

                let mut tx = self
                    .registry
                    .begin()
                    .await
                    .with_context(|| "Failed begin transaction")?;

                let hosts = tx.get_hosts(&hosts_ids).await?;
                let tg_handle = tx
                    .get_user_by_id(&user_id)
                    .await
                    .with_context(|| format!("Failed read user {:?}", user_id))?
                    .with_context(|| format!("User ({:?}) doesn't exist", user_id))?
                    .tg_handle
                    .with_context(|| format!("User ({:?}) tg_handle is None", user_id))?;

                self.tg_adapter
                    .send_message(
                        tg_handle.parse()?,
                        match hosts.len() {
                            1 => format!(
                                "Host {} ({}) released!",
                                hosts[0].hostname,
                                hosts[0].ip_address.ip()
                            ),
                            _ => {
                                let mut msg = String::from("Hosts released:");
                                hosts.into_iter().enumerate().for_each(|(idx, host)| {
                                    msg.push_str(&format!(
                                        "\n{}. {} ({})",
                                        idx + 1,
                                        host.hostname,
                                        host.ip_address.ip()
                                    ));
                                });
                                msg
                            }
                        }
                        .into(),
                    )
                    .await?;
            }
        };

        Ok(())
    }
}
