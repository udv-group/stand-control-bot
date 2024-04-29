use anyhow::{Context, Ok, Result};
use stated_dialogues::controller::BotAdapter;

use crate::db::{
    models::{HostId, UserId},
    Registry,
};

#[derive(Debug)]
pub enum Notification {
    HostsReleased((Vec<HostId>, UserId)),
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
            Notification::HostsReleased((hosts_ids, user_id)) => {
                if hosts_ids.is_empty() {
                    return Ok(());
                }

                let mut tx = self
                    .registry
                    .begin()
                    .await
                    .with_context(|| "Failed to begin transaction")?;

                let hosts = tx.get_hosts(&hosts_ids).await?;
                let tg_handle = tx
                    .get_user_by_id(&user_id)
                    .await
                    .with_context(|| format!("Failed to read user {:?}", user_id))?
                    .with_context(|| format!("User ({:?}) doesn't exist", user_id))?
                    .tg_handle
                    .with_context(|| format!("User ({:?}) tg_handle is None", user_id))?;

                let mut msg = String::from("Hosts released:");
                msg.extend(hosts.into_iter().enumerate().map(|(idx, host)| {
                    format!(
                        "\n{}. {} ({})",
                        idx + 1,
                        host.hostname,
                        host.ip_address.ip()
                    )
                }));
                self.tg_adapter
                    .send_message(tg_handle.parse()?, msg.into())
                    .await?;
            }
        };

        Ok(())
    }
}
