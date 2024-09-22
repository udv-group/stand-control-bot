use askama::Template;
use chrono::{DateTime, TimeDelta, Utc};

use serde::{Deserialize, Serialize};

use crate::db::models::{Group, GroupId, Host, HostId, LeasedHost};

use super::auth::middleware::User;

#[derive(Template, Debug)]
#[template(path = "hosts_page_base.html", escape = "none")]
pub struct HostsPage<T: Template> {
    pub page: T,
    pub user: UserInfo,
    pub auth_link: String,
}

#[derive(Template, Debug)]
#[template(path = "hosts_lease.html", escape = "none")]
pub struct HostsLeasePage {
    pub groups: Vec<GroupInfo>,
    pub selected_group: GroupInfo,
    pub hosts: Vec<HostInfo>,
    pub leased: Vec<LeaseInfo>,
    pub error: Option<String>,
}

#[derive(Template, Debug)]
#[template(path = "all_hosts.html", escape = "none")]
pub struct AllHostsPage {
    pub hosts: Vec<LeaseInfo>,
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    pub login: String,
    pub tg_linked: bool,
    pub link: String,
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
pub struct GroupInfo {
    pub id: GroupId,
    pub name: String,
}
impl From<Group> for GroupInfo {
    fn from(value: Group) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HostInfo {
    pub id: HostId,
    pub hostname: String,
    pub ip_address: String,
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
pub struct LeaseInfo {
    pub id: HostId,
    pub hostname: String,
    pub ip_address: String,
    pub leased_until: DateTime<Utc>,
    pub valid_for: String,
    pub leased_by: String,
}

impl From<LeasedHost> for LeaseInfo {
    fn from(value: LeasedHost) -> Self {
        Self {
            id: value.id,
            hostname: value.hostname,
            ip_address: value.ip_address.ip().to_string(),
            leased_until: value.leased_until,
            valid_for: format_duration(value.leased_until - Utc::now()),
            leased_by: value.user.login,
        }
    }
}

pub fn format_duration(duration: TimeDelta) -> String {
    let days = duration.num_days();
    let hours = (duration - TimeDelta::days(days)).num_hours();
    let minutes = (duration - TimeDelta::days(days) - TimeDelta::hours(hours)).num_minutes();
    format!("{days} days, {hours} hours, {minutes} minutes")
}
