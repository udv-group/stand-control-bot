pub mod support;

use std::sync::Arc;

use crate::support::registry::create_registry;
use async_cell::sync::AsyncCell;
use stand_control_bot::logic::notifications::{BotAdapter, Notification, Notifier};

use anyhow::{Context, Result};

struct TestAdapter {
    pub sent: Arc<AsyncCell<Vec<String>>>,
}

impl BotAdapter for TestAdapter {
    async fn send_message(&self, user_id: i64, _msg: String) -> Result<()> {
        let mut sent = self.sent.take().await;
        sent.push(user_id.to_string());

        self.sent.set(sent);
        Ok(())
    }
}

#[tokio::test]
async fn notification_send() -> Result<()> {
    let (mut test_registry, registry) = create_registry().await;

    let sent = AsyncCell::<Vec<String>>::new().into_shared();
    sent.set(Vec::new());

    let test_adapter = TestAdapter { sent: sent.clone() };

    let host = test_registry.generate_host().await;
    let user = test_registry.generate_user().await;

    let notifier = Notifier::new(registry, test_adapter);

    notifier
        .notify(Notification::HostsReleased((vec![host.id], user.id)))
        .await?;
    let user_id = sent
        .try_take()
        .unwrap()
        .pop()
        .with_context(|| "Failed receive expected message")?;
    assert_eq!(user_id, user.tg_handle);

    Ok(())
}
