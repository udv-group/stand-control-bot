use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use chrono::{DateTime, Utc};

use tokio::time::sleep;

use crate::db::{
    models::{HostId, UserId},
    Registry,
};
use anyhow::Result;

use super::notifications::{BotAdapter, Notification, Notifier};
use tracing::error;

pub async fn hosts_release_timer<T: BotAdapter>(registry: Registry, notifier: Notifier<T>) {
    let mut release_timer = ReleaseTimer {
        registry,
        notifier,
        last_expiration_soon_notification: HashMap::new(),
        expiration_notify_delay_time: Duration::from_secs(30 * 60),
    };
    loop {
        if let Err(err) = release_timer.release().await {
            error!("Release fail: {err}")
        }
        if let Err(err) = release_timer.notify_soon_release().await {
            error!("Notify soon release fail: {err}")
        }
        sleep(Duration::from_secs(10)).await;
    }
}

struct ReleaseTimer<T: BotAdapter> {
    registry: Registry,
    notifier: Notifier<T>,
    last_expiration_soon_notification: HashMap<UserId, (DateTime<Utc>, HashSet<HostId>)>,
    expiration_notify_delay_time: Duration,
}

impl<T: BotAdapter> ReleaseTimer<T> {
    async fn release(&mut self) -> Result<()> {
        let mut tx = self.registry.begin().await?;

        let expired_hosts = tx.get_leased_until_hosts(Utc::now()).await?;
        if expired_hosts.is_empty() {
            return Ok(());
        }

        tx.free_hosts(
            expired_hosts
                .iter()
                .map(|h| h.id)
                .collect::<Vec<HostId>>()
                .as_ref(),
        )
        .await?;
        tx.commit().await?;

        let mut expired_notifications: HashMap<UserId, Vec<HostId>> = HashMap::new();
        expired_hosts.into_iter().for_each(|host| {
            expired_notifications
                .entry(host.user.id)
                .and_modify(|v| v.push(host.id))
                .or_insert(vec![host.id]);
        });

        for (user_id, hosts_ids) in expired_notifications.into_iter() {
            let notification = Notification::HostsReleased(hosts_ids);

            if let Err(err) = self.notifier.notify(user_id, &notification).await {
                error!(
                    "Notification sending error {:?} {:?}: {err}",
                    user_id, notification
                );
            }
        }

        Ok(())
    }

    async fn notify_soon_release(&mut self) -> Result<()> {
        let mut tx = self.registry.begin().await?;

        let next_expiration_date = Utc::now() + self.expiration_notify_delay_time;
        let expire_soon_hosts = tx.get_leased_until_hosts(next_expiration_date).await?;

        let mut expire_soon_notifications: HashMap<UserId, HashSet<HostId>> = HashMap::new();

        expire_soon_hosts.into_iter().for_each(|host| {
            expire_soon_notifications
                .entry(host.user.id)
                .and_modify(|v| {
                    v.insert(host.id);
                })
                .or_insert(HashSet::from_iter(vec![host.id]));
        });

        for (user_id, hosts_ids) in expire_soon_notifications.into_iter() {
            if let Some((last_notify_time, last_hosts_ids)) =
                self.last_expiration_soon_notification.get(&user_id)
            {
                let delta = Utc::now() - last_notify_time;
                if delta.num_seconds() < self.expiration_notify_delay_time.as_secs() as i64
                    && hosts_ids.eq(last_hosts_ids)
                {
                    continue;
                }
            }

            let notification =
                Notification::ExpirationSoon(hosts_ids.clone().into_iter().collect());
            if let Err(err) = self.notifier.notify(user_id, &notification).await {
                error!(
                    "Notification sending error {:?} {:?}: {err}",
                    user_id, notification
                );
            }
            self.last_expiration_soon_notification
                .insert(user_id, (Utc::now(), hosts_ids));
        }

        Ok(())
    }
}
