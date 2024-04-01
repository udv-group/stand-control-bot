use crate::helpers::create_registry;
use chrono::{TimeDelta, Utc};
use stand_control_bot::{db::models::HostId, logic::hosts::*};

mod helpers;

#[tokio::test]
async fn leasing_host_adds_them_to_user_leased_hosts() {
    let (mut test_registry, registry) = create_registry().await;
    let leased_id = test_registry.generate_host().await;
    let free_id = test_registry.generate_host().await;
    let user_id = test_registry.generate_user().await;

    let leased = lease(
        &registry,
        &user_id,
        &[leased_id.clone()],
        TimeDelta::seconds(42),
    )
    .await
    .unwrap();

    assert_eq!(leased.len(), 1);
    assert_eq!(leased[0].id, leased_id);
    assert_eq!(leased[0].user.id, user_id);
    assert!(leased[0].leased_until.is_some());
    assert!(Utc::now() < leased[0].leased_until.unwrap());

    let available_hosts = get_available_hosts(&registry).await.unwrap();

    assert_eq!(available_hosts.len(), 1);
    assert_eq!(free_id, available_hosts[0].id);
    assert!(available_hosts[0].leased_until.is_none());
}

#[tokio::test]
async fn leasing_random_host_leases_one_host() {
    let (mut test_registry, registry) = create_registry().await;
    let host_id_1: HostId = test_registry.generate_host().await;
    let host_id_2: HostId = test_registry.generate_host().await;
    let user_id = test_registry.generate_user().await;

    let leased = lease_random(&registry, &user_id, TimeDelta::seconds(42))
        .await
        .unwrap();

    assert!([host_id_1, host_id_2].contains(&leased.id));
}
