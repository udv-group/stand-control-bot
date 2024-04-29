use std::{collections::HashMap, time::Duration};

use chrono::Utc;
use stated_dialogues::controller::BotAdapter;
use tokio::time::sleep;

use crate::db::{
    models::{HostId, UserId},
    Registry,
};
use anyhow::Result;

use super::notifications::{Notification, Notifier};
use tracing::error;

pub async fn hosts_release_timer<T: BotAdapter>(registry: Registry, notifier: Notifier<T>) {
    loop {
        if let Err(err) = release(&registry, &notifier).await {
            error!("Release fail: {err}")
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn release<T: BotAdapter>(registry: &Registry, notifier: &Notifier<T>) -> Result<()> {
    let mut tx = registry.begin().await?;

    let hosts = tx.get_leased_until_hosts(Utc::now()).await?;
    if hosts.is_empty() {
        return Ok(());
    }

    tx.free_hosts(hosts.iter().map(|h| h.id).collect::<Vec<HostId>>().as_ref())
        .await?;
    tx.commit().await?;

    let mut notifications: HashMap<UserId, Vec<HostId>> = HashMap::new();
    for host in hosts {
        notifications
            .entry(host.user.id)
            .and_modify(|v| v.push(host.id))
            .or_insert(vec![host.id]);
    }

    for (user_id, hosts_ids) in notifications {
        if let Err(err) = notifier
            .notify(Notification::HostsReleased((hosts_ids.clone(), user_id)))
            .await
        {
            error!(
                "Notification sending error {:?} {:?}: {err}",
                user_id, hosts_ids
            );
        }
    }

    Ok(())
}
