use askama::Template;
use axum::{extract::State, response::Html};

use crate::db::models::Host;
use crate::db::Registry;
use crate::logic::hosts::get_available_hosts;

#[derive(Template)]
#[template(path = "available_hosts.html", escape = "none")]
struct AvailableHostsPage {
    hosts: Vec<Host>,
}

pub async fn get_hosts(State(registry): State<Registry>) -> Html<String> {
    let hosts = get_available_hosts(&registry).await.unwrap();
    let page = AvailableHostsPage { hosts };
    Html(page.render().unwrap())
}
