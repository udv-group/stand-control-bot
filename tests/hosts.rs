pub mod support;

use anyhow::{Ok, Result};
use chrono::{TimeDelta, Utc};
use stand_control_bot::{db::models::HostId, logic::hosts::*};

use crate::support::registry::create_registry;

#[tokio::test]
async fn leasing_host_adds_them_to_user_leased_hosts() {
    let (mut gen, registry) = create_registry().await;
    let leased_host = gen.generate_host().await;
    let free = gen.generate_host().await;
    let user = gen.generate_user().await;
    let hosts_service = HostsService::new(registry);

    let leased = hosts_service
        .lease(&user.id, &[leased_host.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    assert_eq!(leased.len(), 1);
    assert_eq!(leased[0].id, leased_host.id);
    assert_eq!(leased[0].user.id, user.id);
    assert!(leased[0].leased_until.is_some());
    assert!(Utc::now() < leased[0].leased_until.unwrap());

    let available_hosts = hosts_service.get_available_hosts().await.unwrap();

    assert_eq!(available_hosts.len(), 1);
    assert_eq!(free.id, available_hosts[0].id);
    assert!(available_hosts[0].leased_until.is_none());
}

#[tokio::test]
async fn leasing_random_host_leases_one_host() {
    let (mut gen, registry) = create_registry().await;
    let host1 = gen.generate_host().await;
    let host2 = gen.generate_host().await;
    let user = gen.generate_user().await;
    let hosts_service = HostsService::new(registry);

    let leased = hosts_service
        .lease_random(&user.id, TimeDelta::seconds(42))
        .await
        .unwrap();

    assert!([host1.id, host2.id].contains(&leased.id));
}

#[tokio::test]
async fn free_host() {
    let (mut gen, registry) = create_registry().await;

    gen.generate_host().await;
    let user = gen.generate_user().await;
    let hosts_service = HostsService::new(registry);

    let leased = hosts_service
        .lease_random(&user.id, TimeDelta::seconds(42))
        .await
        .unwrap();

    hosts_service.free(&user.id, &[leased.id]).await.unwrap();
    let hosts = hosts_service.get_available_hosts().await.unwrap();

    assert!(hosts
        .iter()
        .map(|h| h.id)
        .collect::<Vec<HostId>>()
        .contains(&leased.id));
}

#[tokio::test]
async fn leased_until_read() -> Result<()> {
    let (mut gen, registry) = create_registry().await;

    let host = gen.generate_host().await;
    let user = gen.generate_user().await;

    let mut tx = registry.begin().await?;

    let date = Utc::now();
    let lease_time = 30;

    tx.lease_hosts(&user.id, &[host.id], date + TimeDelta::seconds(lease_time))
        .await?;

    let hosts_ids = tx
        .get_leased_until_hosts(date + TimeDelta::seconds(lease_time - 1))
        .await?;
    assert!(hosts_ids.is_empty());

    let hosts_ids: Vec<HostId> = tx
        .get_leased_until_hosts(date + TimeDelta::seconds(lease_time + 1))
        .await?
        .into_iter()
        .map(|h| h.id)
        .collect();
    assert_eq!(hosts_ids.to_vec(), vec![host.id]);

    Ok(())
}
