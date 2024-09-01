use std::collections::HashSet;

use chrono::Utc;
use thiserror::Error;

use crate::db::{
    models::{GroupId, Host, HostId, LeasedHost, UserId},
    Registry,
};

#[derive(Error, Debug)]
pub enum HostError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Host is already leased")]
    AlreadyLeased(Vec<HostId>),

    #[error("Hosts lease limit is reached")]
    LeaseLimit,
}

#[derive(Clone)]
pub struct HostsService {
    registry: Registry,
    lease_limit: usize,
}

impl HostsService {
    pub fn new(registry: Registry, lease_limit: usize) -> Self {
        HostsService {
            registry,
            lease_limit,
        }
    }

    pub async fn get_all_hosts(&self) -> Result<Vec<Host>, HostError> {
        let mut tx = self.registry.begin().await?;
        let hosts = tx.get_all_hosts().await?;
        tx.commit().await?;
        Ok(hosts)
    }

    pub async fn get_available_group_hosts(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<Host>, HostError> {
        let mut tx = self.registry.begin().await?;
        let hosts = tx.get_available_group_hosts(group_id).await?;

        tx.commit().await?;
        Ok(hosts)
    }

    pub async fn get_available_hosts(&self) -> Result<Vec<Host>, HostError> {
        let mut tx = self.registry.begin().await?;
        let hosts = tx.get_available_hosts().await?;
        tx.commit().await?;
        Ok(hosts)
    }

    pub async fn get_leased_hosts(&self, user_id: &UserId) -> Result<Vec<LeasedHost>, HostError> {
        let mut tx = self.registry.begin().await?;
        let hosts = tx.get_leased_hosts(user_id).await?;
        tx.commit().await?;
        Ok(hosts)
    }

    pub async fn lease(
        &self,
        user_id: &UserId,
        hosts_ids: &[HostId],
        lease_for: chrono::TimeDelta,
    ) -> Result<Vec<LeasedHost>, HostError> {
        let mut tx = self.registry.begin().await?;
        let leased: HashSet<_> = tx
            .get_leased_hosts(user_id)
            .await?
            .into_iter()
            .map(|h| h.id)
            .collect();

        let mut hosts_ids_set: HashSet<_> = HashSet::new();
        hosts_ids_set.extend(hosts_ids.to_vec());

        let intersection: HashSet<_> = leased.intersection(&hosts_ids_set).collect();
        if !intersection.is_empty() {
            return Err(HostError::AlreadyLeased(
                intersection.into_iter().cloned().collect(),
            ));
        }

        if leased.len() + hosts_ids_set.len() > self.lease_limit {
            return Err(HostError::LeaseLimit);
        };

        tx.lease_hosts(user_id, hosts_ids, Utc::now() + lease_for)
            .await?;

        let leased = tx.get_leased_hosts(user_id).await?;
        tx.commit().await?;
        Ok(leased)
    }

    pub async fn free(&self, user_id: &UserId, hosts_ids: &[HostId]) -> Result<(), HostError> {
        let mut tx = self.registry.begin().await?;
        tx.free_hosts_for_user(hosts_ids.as_ref(), user_id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn free_all(&self, user_id: &UserId) -> Result<(), HostError> {
        let mut tx = self.registry.begin().await?;

        tx.free_all(user_id).await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn lease_random(
        &self,
        user_id: &UserId,
        lease_for: chrono::TimeDelta,
    ) -> Result<LeasedHost, HostError> {
        let mut tx = self.registry.begin().await?;
        if tx.get_leased_hosts(user_id).await?.len() >= self.lease_limit {
            return Err(HostError::LeaseLimit);
        };
        let host = tx.get_first_available_host().await?;
        tx.lease_hosts(user_id, &[host.id], Utc::now() + lease_for)
            .await?;
        let leased = tx.get_leased_hosts(user_id).await?.into_iter().next();
        tx.commit().await?;
        // TODO: dont unwrap, make another error type
        Ok(leased.unwrap())
    }
}
