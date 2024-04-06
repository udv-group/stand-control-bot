use std::time::Duration;

use chrono::Utc;
use tokio::{sync::mpsc::Sender, time::sleep};

use crate::db::{models::HostId, Registry};
use anyhow::Result;

use super::notifications::Notification;
use tracing::error;

pub async fn hosts_release_timer(registry: Registry, sender: Sender<Notification>) {
    loop {
        if let Err(err) = release(&registry, &sender).await {
            error!("Release fail: {err}")
        }
        sleep(Duration::from_secs(10)).await;
    }
}

async fn release(registry: &Registry, sender: &Sender<Notification>) -> Result<()> {
    let mut tx = registry.begin().await?;

    let hosts = tx.get_leased_until_hosts(Utc::now()).await?;
    if hosts.is_empty() {
        return Ok(());
    }

    tx.free_hosts(hosts.iter().map(|h| h.id).collect::<Vec<HostId>>().as_ref())
        .await?;
    tx.commit().await?;

    for host in hosts {
        sender
            .send(Notification::HostRelased((host.id, host.user.id)))
            .await?;
    }
    Ok(())
}
