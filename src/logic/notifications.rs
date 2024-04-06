use anyhow::{Context, Ok, Result};
use stated_dialogues::controller::BotAdapter;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::db::{
    models::{HostId, UserId},
    Registry,
};

#[derive(Debug)]
pub enum Notification {
    HostRelased((HostId, UserId)),
}

pub struct Notifier<T: BotAdapter> {
    receiver: Receiver<Notification>,
    registry: Registry,
    tg_adapter: T,
}

impl<T> Notifier<T>
where
    T: BotAdapter,
{
    pub fn create(registry: Registry, tg_adapter: T) -> (Sender<Notification>, Self) {
        let (sender, receiver) = mpsc::channel::<Notification>(32);
        (
            sender,
            Self {
                receiver,
                registry,
                tg_adapter,
            },
        )
    }

    pub async fn work(mut self) {
        while let Some(notification) = self.receiver.recv().await {
            if let Err(err) = self.handle(&notification).await {
                tracing::error!("Failed notification handle {:?}: {}", notification, err)
            }
        }
    }

    async fn handle(&self, notification: &Notification) -> Result<()> {
        match notification {
            Notification::HostRelased((host_id, user_id)) => {
                let mut tx = self
                    .registry
                    .begin()
                    .await
                    .with_context(|| "Failed begin transaction")?;

                let host = tx.get_host(host_id).await?;
                let tg_handle = tx
                    .get_user_by_id(user_id)
                    .await
                    .with_context(|| format!("Failed read user {:?}", user_id))?
                    .with_context(|| format!("User ({:?}) doesn't exist", user_id))?
                    .tg_handle
                    .with_context(|| format!("User ({:?}) tg_handle is None", user_id))?;

                self.tg_adapter
                    .send_message(
                        tg_handle.parse()?,
                        format!(
                            "Хост {} ({}) освобожден!",
                            host.hostname,
                            host.ip_address.ip()
                        )
                        .into(),
                    )
                    .await?;
            }
        };

        Ok(())
    }
}
