use std::{fmt::Display, ops::Deref};

use chrono::prelude::*;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(sqlx::Type, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct UserId(pub i32);

impl Deref for UserId {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for UserId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[derive(sqlx::Type, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[sqlx(transparent)]
pub struct HostId(pub i32);
impl Display for HostId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for HostId {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for HostId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct LeasedHost {
    #[sqlx(rename = "hid")]
    pub id: HostId,
    pub hostname: String,
    pub ip_address: IpNetwork,
    pub leased_until: Option<DateTime<Utc>>,
    #[sqlx(flatten)]
    pub user: User,
}

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct Host {
    pub id: HostId,
    pub hostname: String,
    pub ip_address: IpNetwork,
    pub leased_until: Option<DateTime<Utc>>,
    pub user_id: Option<UserId>,
}

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct User {
    pub id: UserId,
    pub login: String,
    pub tg_handle: Option<String>,
    pub email: Option<String>,
}
