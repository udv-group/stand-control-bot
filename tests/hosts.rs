pub mod support;

use chrono::{TimeDelta, Utc};
use stand_control_bot::logic::hosts::*;

use crate::support::registry::create_registry;

#[tokio::test]
async fn leasing_host_adds_them_to_user_leased_hosts() {
    let (mut gen, registry) = create_registry().await;
    let leased_host = gen.generate_host().await;
    let free = gen.generate_host().await;
    let user_id = gen.generate_user().await;

    let leased = lease(
        &registry,
        &user_id,
        &[leased_host.id.clone()],
        TimeDelta::seconds(42),
    )
    .await
    .unwrap();

    assert_eq!(leased.len(), 1);
    assert_eq!(leased[0].id, leased_host.id);
    assert_eq!(leased[0].user.id, user_id);
    assert!(leased[0].leased_until.is_some());
    assert!(Utc::now() < leased[0].leased_until.unwrap());

    let available_hosts = get_available_hosts(&registry).await.unwrap();

    assert_eq!(available_hosts.len(), 1);
    assert_eq!(free.id, available_hosts[0].id);
    assert!(available_hosts[0].leased_until.is_none());
}

#[tokio::test]
async fn leasing_random_host_leases_one_host() {
    let (mut gen, registry) = create_registry().await;
    let host1 = gen.generate_host().await;
    let host2 = gen.generate_host().await;
    let user_id = gen.generate_user().await;

    let leased = lease_random(&registry, &user_id, TimeDelta::seconds(42))
        .await
        .unwrap();

    assert!([host1.id, host2.id].contains(&leased.id));
}
