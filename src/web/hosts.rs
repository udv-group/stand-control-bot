use std::ops::Deref;

use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum::Extension;
use axum::{extract::State, response::Html};
use axum_extra::extract::Form;
use axum_login::AuthUser;
use chrono::{DateTime, TimeDelta, Utc};

use serde::Deserialize;

use crate::db::models::{Host, HostId, LeasedHost, User as UserDb};
use crate::logic::hosts::HostsService;
use crate::logic::users::UsersService;

use super::auth::middleware::User;

#[derive(Template, Debug)]
#[template(path = "available_hosts.html", escape = "none")]
struct HostsPage {
    user: UserInfo,
    hosts: Vec<HostInfo>,
    leased: Vec<LeaseInfo>,
    bot_username: String,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    login: String,
    tg_linked: bool,
    link: String,
}
impl From<UserDb> for UserInfo {
    fn from(value: UserDb) -> Self {
        Self {
            login: value.login,
            tg_linked: value.tg_handle.is_some(),
            link: value.link,
        }
    }
}

#[derive(Deserialize, Debug)]
struct HostInfo {
    id: HostId,
    hostname: String,
    ip_address: String,
}
impl From<Host> for HostInfo {
    fn from(value: Host) -> Self {
        Self {
            id: value.id,
            hostname: value.hostname,
            ip_address: value.ip_address.ip().to_string(),
        }
    }
}
#[derive(Deserialize, Debug)]
struct LeaseInfo {
    id: HostId,
    hostname: String,
    ip_address: String,
    leased_until: DateTime<Utc>,
    valid_for: String,
}

impl From<LeasedHost> for LeaseInfo {
    fn from(value: LeasedHost) -> Self {
        Self {
            id: value.id,
            hostname: value.hostname,
            ip_address: value.ip_address.ip().to_string(),
            leased_until: value.leased_until,
            valid_for: format_duration(value.leased_until - Utc::now()),
        }
    }
}

fn format_duration(duration: TimeDelta) -> String {
    let days = duration.num_days();
    let hours = (duration - TimeDelta::days(days)).num_hours();
    let minutes = (duration - TimeDelta::days(days) - TimeDelta::hours(hours)).num_minutes();
    format!("{days} days, {hours} hours, {minutes} minutes")
}

pub async fn get_hosts(
    State(service): State<HostsService>,
    State(users_service): State<UsersService>,
    State(bot_username): State<String>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let hosts = service.get_available_hosts().await.unwrap();
    let leased = service.get_leased_hosts(&user.id().into()).await.unwrap();
    let user_db = users_service
        .get_user(&user.username)
        .await
        .unwrap()
        .unwrap();

    let page = HostsPage {
        user: user_db.into(),
        hosts: hosts.into_iter().map(|h| h.into()).collect(),
        leased: leased.into_iter().map(|h| h.into()).collect(),
        bot_username,
    };

    Html(page.render().unwrap())
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct StrU8(pub u8);

impl Deref for StrU8 {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for StrU8 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<u8>() {
            Ok(v) => Ok(Self(v)),
            Err(_) => Err(format!("Wrong value {value}, can not parse as u8")),
        }
    }
}

#[derive(Deserialize)]
pub struct LeaseForm {
    days: StrU8,
    hours: StrU8,
    #[serde(default)]
    hosts_ids: Vec<HostId>,
}

pub async fn lease_hosts(
    State(service): State<HostsService>,
    Extension(user): Extension<User>,
    Form(data): Form<LeaseForm>,
) -> impl IntoResponse {
    service
        .lease(
            &user.id().into(),
            &data.hosts_ids,
            TimeDelta::hours((*data.hours + *data.days * 24) as i64),
        )
        .await
        .unwrap();

    Redirect::to("/hosts")
}

pub async fn lease_random(
    State(service): State<HostsService>,
    Extension(user): Extension<User>,
    Form(data): Form<LeaseForm>,
) -> impl IntoResponse {
    service
        .lease_random(
            &user.id().into(),
            TimeDelta::hours((*data.hours + *data.days * 24) as i64),
        )
        .await
        .unwrap();

    Redirect::to("/hosts")
}

#[derive(Deserialize)]
pub struct ReleaseForm {
    hosts_ids: Vec<HostId>,
}

pub async fn release_hosts(
    State(service): State<HostsService>,
    Extension(user): Extension<User>,
    Form(data): Form<ReleaseForm>,
) -> impl IntoResponse {
    service
        .free(&user.id().into(), &data.hosts_ids)
        .await
        .unwrap();
    Redirect::to("/hosts")
}

pub async fn release_all(
    State(service): State<HostsService>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    service.free_all(&user.id().into()).await.unwrap();
    Redirect::to("/hosts")
}
