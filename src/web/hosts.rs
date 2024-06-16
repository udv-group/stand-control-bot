use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Json, Redirect},
    Extension,
};
use axum_extra::extract::{Form, OptionalQuery};
use axum_flash::{Flash, IncomingFlashes};
use axum_login::AuthUser;
use chrono::{DateTime, TimeDelta, Utc};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::db::models::{Host, HostId, LeasedHost};
use crate::logic::hosts::HostsService;
use crate::logic::users::UsersService;

use super::auth::middleware::User;
use super::{flash_redirect, AuthLink};

#[derive(Template, Debug)]
#[template(path = "available_hosts.html", escape = "none")]
struct HostsPage {
    hosts: Vec<HostInfo>,
    leased: Vec<LeaseInfo>,
    user: UserInfo,
    auth_link: String,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    login: String,
    tg_linked: bool,
    link: String,
}
impl From<User> for UserInfo {
    fn from(value: User) -> Self {
        Self {
            login: value.username,
            tg_linked: value.tg_handle.is_some(),
            link: value.link,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
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
    State(AuthLink(auth_link)): State<AuthLink>,
    flashes: IncomingFlashes,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let hosts = service.get_available_hosts().await.unwrap();
    let leased = service.get_leased_hosts(&user.id().into()).await.unwrap();
    let error = flashes.into_iter().next().map(|(_, err)| err.to_owned());
    let page = HostsPage {
        user: user.into(),
        auth_link,
        hosts: hosts.into_iter().map(|h| h.into()).collect(),
        leased: leased.into_iter().map(|h| h.into()).collect(),
        error,
    };

    (flashes, Html(page.render().unwrap()))
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Days(pub i64);

impl Deref for Days {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Days {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<u8>() {
            Ok(v @ 0..=63) => Ok(Self(v as i64)),
            Ok(_) => Err("Value must be between 0 and 63".to_string()),
            Err(_) => Err(format!("Wrong value {value}, can not parse as u8")),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Hours(pub i64);

impl Deref for Hours {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for Hours {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<u8>() {
            Ok(v @ 0..=23) => Ok(Self(v as i64)),
            Ok(_) => Err("Value must be between 0 and 23".to_string()),
            Err(_) => Err(format!("Wrong value {value}, can not parse as u8")),
        }
    }
}
#[derive(Deserialize)]
pub struct LeaseForm {
    days: Days,
    hours: Hours,
    #[serde(default)]
    hosts_ids: Vec<HostId>,
}

pub async fn lease_hosts(
    State(service): State<HostsService>,
    flash: Flash,
    Extension(user): Extension<User>,
    Form(data): Form<LeaseForm>,
) -> axum::response::Result<Redirect> {
    let res = service
        .lease(
            &user.id().into(),
            &data.hosts_ids,
            TimeDelta::hours(*data.hours + *data.days * 24),
        )
        .await;
    match res {
        Ok(_) => Ok(Redirect::to("/hosts")),
        Err(e) => Err(flash_redirect(&e.to_string(), "/hosts", flash)),
    }
}

pub async fn lease_random(
    State(service): State<HostsService>,
    flash: Flash,
    Extension(user): Extension<User>,
    Form(data): Form<LeaseForm>,
) -> axum::response::Result<Redirect> {
    let res = service
        .lease_random(
            &user.id().into(),
            TimeDelta::hours(*data.hours + *data.days * 24),
        )
        .await;
    match res {
        Ok(_) => Ok(Redirect::to("/hosts")),
        Err(e) => Err(flash_redirect(&e.to_string(), "/hosts", flash)),
    }
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

#[derive(Deserialize)]
pub struct GetHostsQuery {
    login: String,
}

pub async fn get_hosts_json(
    State(hosts_service): State<HostsService>,
    State(user_service): State<UsersService>,
    OptionalQuery(query): OptionalQuery<GetHostsQuery>,
) -> impl IntoResponse {
    match query {
        Some(GetHostsQuery { login }) => match user_service.get_user(&login).await.unwrap() {
            None => Json(vec![]),
            Some(user) => Json(
                hosts_service
                    .get_leased_hosts(&user.id)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|h| HostInfo {
                        id: h.id,
                        hostname: h.hostname,
                        ip_address: h.ip_address.ip().to_string(),
                    })
                    .collect::<Vec<_>>(),
            ),
        },
        None => Json(
            hosts_service
                .get_all_hosts()
                .await
                .unwrap()
                .into_iter()
                .map(|h| HostInfo {
                    id: h.id,
                    hostname: h.hostname,
                    ip_address: h.ip_address.ip().to_string(),
                })
                .collect::<Vec<_>>(),
        ),
    }
}
