pub mod models;

use std::ops::Deref;

use chrono::prelude::*;
use sqlx::{PgPool, Postgres, Transaction};

use crate::configuration::DatabaseSettings;
use crate::db::models::{Host, HostId, LeasedHost, User, UserId};

#[derive(Clone)]
pub struct Registry {
    pool: PgPool,
}

impl Registry {
    pub async fn new(settings: &DatabaseSettings) -> sqlx::Result<Self> {
        let pool = PgPool::connect_with(settings.with_db()).await?;
        Ok(Registry { pool })
    }
    pub async fn begin(&self) -> sqlx::Result<RegistryTx> {
        Ok(RegistryTx {
            tx: self.pool.begin().await?,
        })
    }
}
pub struct RegistryTx<'c> {
    tx: Transaction<'c, Postgres>,
}

impl<'c> RegistryTx<'c> {
    pub async fn commit(self) -> sqlx::Result<()> {
        self.tx.commit().await
    }

    pub async fn get_available_hosts(&mut self) -> sqlx::Result<Vec<Host>> {
        sqlx::query_as("SELECT * FROM hosts WHERE user_id is NULL")
            .fetch_all(&mut *self.tx)
            .await
    }
    pub async fn get_first_available_host(&mut self) -> sqlx::Result<Host> {
        sqlx::query_as("SELECT * FROM hosts WHERE user_id is NULL LIMIT 1")
            .fetch_one(&mut *self.tx)
            .await
    }
    pub async fn get_host(&mut self, host_id: &HostId) -> sqlx::Result<Host> {
        sqlx::query_as("SELECT * FROM hosts WHERE id = $1 LIMIT 1")
            .bind(host_id.deref())
            .fetch_one(&mut *self.tx)
            .await
    }
    pub async fn get_leased_hosts(&mut self, user_id: &UserId) -> sqlx::Result<Vec<LeasedHost>> {
        sqlx::query_as(
            r#"
            SELECT hosts.id, hosts.hostname, hosts.ip_address, hosts.leased_until, users.id, users.login, users.tg_handle, users.email 
            FROM hosts JOIN users on hosts.user_id = users.id 
            WHERE hosts.user_id = $1
            "#,
        ).bind(user_id.deref())
        .fetch_all(&mut *self.tx)
        .await
    }
    pub async fn get_leased_until_hosts(
        &mut self,
        until: DateTime<Utc>,
    ) -> sqlx::Result<Vec<LeasedHost>> {
        sqlx::query_as(
            r#"
            SELECT hosts.id, hosts.hostname, hosts.ip_address, hosts.leased_until, users.id, users.login, users.tg_handle, users.email 
            FROM hosts JOIN users on hosts.user_id = users.id
            WHERE hosts.leased_until < $1
            "#,
        ).bind(until).fetch_all(&mut *self.tx).await
    }
    pub async fn lease_hosts(
        &mut self,
        user_id: &UserId,
        hosts_ids: &[HostId],
        untill: DateTime<Utc>,
    ) -> sqlx::Result<()> {
        let ids: Vec<_> = hosts_ids.iter().map(|h| h.0).collect();

        sqlx::query!(
            "UPDATE hosts SET user_id = $1, leased_until = $2 WHERE id = any($3)",
            user_id.deref(),
            untill,
            ids.as_slice(),
        )
        .execute(&mut *self.tx)
        .await?;
        Ok(())
    }
    pub async fn free_hosts(&mut self, hosts_ids: &[HostId]) -> sqlx::Result<()> {
        let ids: Vec<_> = hosts_ids.iter().map(|h| h.0).collect();
        sqlx::query!(
            "UPDATE hosts SET user_id = NULL, leased_until = NULL WHERE id = any($1)",
            ids.as_slice()
        )
        .execute(&mut *self.tx)
        .await?;
        Ok(())
    }
    pub async fn free_all(&mut self, user_id: &UserId) -> sqlx::Result<()> {
        sqlx::query!(
            "UPDATE hosts SET user_id = NULL, leased_until = NULL WHERE user_id = $1",
            user_id.deref()
        )
        .execute(&mut *self.tx)
        .await?;
        Ok(())
    }
    pub async fn get_user_by_id(&mut self, user_id: &UserId) -> sqlx::Result<Option<User>> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id.deref())
            .fetch_optional(&mut *self.tx)
            .await
    }
    pub async fn get_user(&mut self, login: &str) -> sqlx::Result<Option<User>> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE login = $1", login)
            .fetch_optional(&mut *self.tx)
            .await
    }
    pub async fn add_user(
        &mut self,
        login: &str,
        tg_handle: Option<&str>,
        email: Option<&str>,
    ) -> sqlx::Result<UserId> {
        let rec = sqlx::query!(
            "INSERT INTO users (login, tg_handle, email) VALUES ($1, $2, $3) RETURNING id",
            login,
            tg_handle,
            email
        )
        .fetch_one(&mut *self.tx)
        .await?;
        Ok(rec.id.into())
    }
}
