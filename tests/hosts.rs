pub mod support;

use std::collections::HashSet;

use chrono::{TimeDelta, Utc};
use stand_control_bot::{db::models::HostId, logic::hosts::HostError};
use support::registry::create_service_with_limit;

use crate::support::registry::{create_registry, create_service};

#[tokio::test]
async fn leasing_host_adds_them_to_user_leased_hosts() {
    let (mut gen, hosts_service) = create_service().await;
    let leased_host = gen.generate_host().await;
    let free = gen.generate_host().await;
    let user = gen.generate_user().await;

    let leased = hosts_service
        .lease(&user.id, &[leased_host.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    assert_eq!(leased.len(), 1);
    assert_eq!(leased[0].id, leased_host.id);
    assert_eq!(leased[0].user.id, user.id);
    assert!(Utc::now() < leased[0].leased_until);

    let available_hosts = hosts_service.get_available_hosts().await.unwrap();

    assert_eq!(available_hosts.len(), 1);
    assert_eq!(free.id, available_hosts[0].id);
    assert!(available_hosts[0].leased_until.is_none());
}

#[tokio::test]
async fn leasing_random_host_leases_one_host() {
    let (mut gen, hosts_service) = create_service().await;
    let group = gen.generate_group().await;
    let host1 = gen.generate_host_in_group(&group.id).await;
    let host2 = gen.generate_host_in_group(&group.id).await;
    let user = gen.generate_user().await;

    let leased = hosts_service
        .lease_random(&user.id, TimeDelta::seconds(42), &group.id)
        .await
        .unwrap();

    assert!([host1.id, host2.id].contains(&leased.id));
}

#[tokio::test]
async fn leasing_multiple_hosts() {
    let (mut gen, service) = create_service().await;
    let group = gen.generate_group().await;
    let host1 = gen.generate_host_in_group(&group.id).await;
    let host2 = gen.generate_host_in_group(&group.id).await;
    let host3 = gen.generate_host_in_group(&group.id).await;
    let user = gen.generate_user().await;

    service
        .lease(&user.id, &[host1.id, host2.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    service
        .lease_random(&user.id, TimeDelta::seconds(42), &group.id)
        .await
        .unwrap();

    let available = service.get_available_hosts().await.unwrap();
    assert!(available.is_empty());

    let leased = service.get_leased_hosts(&user.id).await.unwrap();

    assert_eq!(
        HashSet::from([host1.id, host2.id, host3.id]),
        leased.into_iter().map(|h| h.id).collect::<HashSet<_>>()
    )
}

#[tokio::test]
async fn freeing_host_makes_it_available_for_lease() {
    let (mut gen, service) = create_service().await;
    let host1 = gen.generate_host().await;
    let host2 = gen.generate_host().await;
    let user = gen.generate_user().await;

    let available = service.get_available_hosts().await.unwrap();
    assert_eq!(available.len(), 2);

    service
        .lease(&user.id, &[host1.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    let available = service.get_available_hosts().await.unwrap();
    assert_eq!(available.len(), 1);
    assert_eq!(available[0].id, host2.id);

    service.free(&user.id, &[host1.id]).await.unwrap();

    let available = service.get_available_hosts().await.unwrap();
    assert_eq!(available.len(), 2);
    assert_eq!(
        available.into_iter().map(|h| h.id).collect::<HashSet<_>>(),
        HashSet::from([host1.id, host2.id])
    );
}

#[tokio::test]
async fn free_all_frees_hosts_only_for_one_user() {
    let (mut gen, service) = create_service().await;
    let host1 = gen.generate_host().await;
    let host2 = gen.generate_host().await;
    let host3 = gen.generate_host().await;
    let user1 = gen.generate_user().await;
    let user2 = gen.generate_user().await;

    service
        .lease(&user1.id, &[host1.id, host2.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    service
        .lease(&user2.id, &[host3.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    let available = service.get_available_hosts().await.unwrap();
    assert!(available.is_empty());

    service.free_all(&user1.id).await.unwrap();

    let available = service.get_available_hosts().await.unwrap();
    assert_eq!(available.len(), 2);
    let leased_u1 = service.get_leased_hosts(&user1.id).await.unwrap();
    assert!(leased_u1.is_empty());

    let leased_u2 = service.get_leased_hosts(&user2.id).await.unwrap();
    assert_eq!(leased_u2.len(), 1);
}

#[tokio::test]
async fn leased_until_read() {
    let (mut gen, registry) = create_registry().await;

    let host = gen.generate_host().await;
    let user = gen.generate_user().await;

    let mut tx = registry.begin().await.unwrap();

    let date = Utc::now();
    let lease_time = 30;

    tx.lease_hosts(&user.id, &[host.id], date + TimeDelta::seconds(lease_time))
        .await
        .unwrap();

    let hosts_ids = tx
        .get_leased_until_hosts(date + TimeDelta::seconds(lease_time - 1))
        .await
        .unwrap();
    assert!(hosts_ids.is_empty());

    let hosts_ids: Vec<HostId> = tx
        .get_leased_until_hosts(date + TimeDelta::seconds(lease_time + 1))
        .await
        .unwrap()
        .into_iter()
        .map(|h| h.id)
        .collect();
    assert_eq!(hosts_ids.to_vec(), vec![host.id]);
}

#[tokio::test]
async fn lease_limit() {
    let (mut get, service) = create_service_with_limit(2).await;
    let group = get.generate_group().await;
    let host1 = get.generate_host().await;
    let host2 = get.generate_host().await;
    let host3 = get.generate_host().await;
    let user = get.generate_user().await;

    // lease too many at once
    match service
        .lease(
            &user.id,
            &[host1.id, host2.id, host3.id],
            TimeDelta::seconds(42),
        )
        .await
    {
        Ok(_) => panic!("Didn't error on lease limit"),
        Err(e) => match e {
            HostError::LeaseLimit => (),
            _ => panic!("Wrong error type on lease error"),
        },
    };

    service
        .lease(&user.id, &[host1.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    // leasing 2 will go over the limit
    match service
        .lease(&user.id, &[host2.id, host3.id], TimeDelta::seconds(42))
        .await
    {
        Ok(_) => panic!("Didn't error on lease limit"),
        Err(e) => match e {
            HostError::LeaseLimit => (),
            _ => panic!("Wrong error type on lease error"),
        },
    };

    // leasing 2, but one of them is already leased
    match service
        .lease(&user.id, &[host1.id, host2.id], TimeDelta::seconds(42))
        .await
    {
        Ok(_) => panic!("Didn't error on lease limit"),
        Err(e) => match e {
            HostError::AlreadyLeased(_) => (),
            _ => panic!("Wrong error type on lease error"),
        },
    };

    service
        .lease(&user.id, &[host2.id], TimeDelta::seconds(42))
        .await
        .unwrap();

    // leasing random when at limit
    match service
        .lease_random(&user.id, TimeDelta::seconds(42), &group.id)
        .await
    {
        Ok(_) => panic!("Didn't error on lease limit"),
        Err(e) => match e {
            HostError::LeaseLimit => (),
            _ => panic!("Wrong error type on lease error"),
        },
    };
}
