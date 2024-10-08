use std::future::Future;

use anyhow::{Context, Ok, Result};
use chrono::Utc;

use crate::db::{
    models::{HostId, UserId},
    Registry,
};

#[derive(Debug, Clone)]
pub enum Notification {
    HostsReleased(Vec<HostId>),
    ExpirationSoon(Vec<HostId>),
}

pub trait BotAdapter {
    fn send_message(&self, user_id: i64, msg: String) -> impl Future<Output = Result<()>> + Send;
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

    pub async fn notify(&self, user_id: UserId, notification: &Notification) -> Result<()> {
        let mut tx = self
            .registry
            .begin()
            .await
            .with_context(|| "Failed to begin transaction")?;

        let tg_handle = tx
            .get_user_by_id(&user_id)
            .await
            .with_context(|| format!("Failed to read user {:?}", user_id))?
            .with_context(|| format!("User ({:?}) doesn't exist", user_id))?
            .tg_handle
            .with_context(|| format!("User ({:?}) tg_handle is None", user_id))?;

        let msg = match notification {
            Notification::HostsReleased(hosts_ids) => {
                if hosts_ids.is_empty() {
                    return Ok(());
                }

                let hosts = tx.get_hosts(hosts_ids).await?;

                let mut msg = String::from("Hosts released:");
                msg.extend(hosts.into_iter().enumerate().map(|(idx, host)| {
                    format!(
                        "\n{}. {} ({})",
                        idx + 1,
                        host.hostname,
                        host.ip_address.ip()
                    )
                }));

                msg
            }
            Notification::ExpirationSoon(hosts_ids) => {
                if hosts_ids.is_empty() {
                    return Ok(());
                }

                let hosts = tx.get_hosts(hosts_ids).await?;
                let mut msg = String::from("Hosts expiring soon:");
                msg.extend(hosts.into_iter().enumerate().map(|(idx, host)| {
                    format!(
                        "\n{}. {} ({}) - {} minutes left",
                        idx + 1,
                        host.hostname,
                        host.ip_address.ip(),
                        (host.leased_until.unwrap() - Utc::now()).num_minutes()
                    )
                }));

                msg
            }
        };

        self.tg_adapter
            .send_message(
                tg_handle
                    .parse()
                    .with_context(|| format!("Failed parse chat_id from {}", tg_handle))?,
                msg,
            )
            .await?;

        Ok(())
    }
}
