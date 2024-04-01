use std::collections::HashSet;

use chrono::Utc;
use thiserror::Error;

use crate::db::{
    models::{Host, HostId, LeasedHost, UserId},
    Registry,
};

#[derive(Error, Debug)]
pub enum HostError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Host is already leased")]
    AlreadyLeased(Vec<HostId>),
}

pub async fn get_available_hosts(registry: &Registry) -> Result<Vec<Host>, HostError> {
    let mut tx = registry.begin().await?;
    let hosts = tx.get_available_hosts().await?;
    tx.commit().await?;
    Ok(hosts)
}

pub async fn lease(
    registry: &Registry,
    user_id: &UserId,
    hosts_ids: &[HostId],
    lease_for: chrono::TimeDelta,
) -> Result<Vec<LeasedHost>, HostError> {
    let mut tx = registry.begin().await?;
    let available: HashSet<_> = tx
        .get_available_hosts()
        .await?
        .into_iter()
        .map(|h| h.id)
        .collect();
    let mut hosts_ids_set: HashSet<_> = HashSet::new();
    hosts_ids_set.extend(hosts_ids.to_vec());

    if !available.is_superset(&hosts_ids_set) {
        return Err(HostError::AlreadyLeased(
            hosts_ids_set.difference(&available).cloned().collect(),
        ));
    }
    tx.lease_hosts(user_id, hosts_ids, Utc::now() + lease_for)
        .await?;

    let leased = tx.get_leased_hosts(user_id).await?;
    tx.commit().await?;
    Ok(leased)
}

pub async fn lease_random(
    registry: &Registry,
    user_id: &UserId,
    lease_for: chrono::TimeDelta,
) -> Result<LeasedHost, HostError> {
    let mut tx = registry.begin().await?;
    let host = tx.get_first_available_host().await?;
    tx.lease_hosts(user_id, &[host.id], Utc::now() + lease_for)
        .await?;
    let leased = tx.get_leased_hosts(user_id).await?.into_iter().next();
    tx.commit().await?;
    // TODO: dont unwrap, make another error type
    Ok(leased.unwrap())
}
