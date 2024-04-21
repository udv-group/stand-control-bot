use std::mem;
use std::net::SocketAddr;

use super::gen::Generator;
use super::{configure_db, setup_settings};

use reqwest::redirect::Policy;
use stand_control_bot::db::Registry;
use stand_control_bot::web::Application;

pub struct TestApp {
    pub gen: Generator,
    pub registry: Registry,
    pub addr: SocketAddr,
    pub api_client: reqwest::Client,
}
impl TestApp {
    pub async fn new() -> TestApp {
        let settings = setup_settings();
        let pool = configure_db(&settings.database).await;
        let app = Application::build(&settings).await.unwrap();
        let addr = app.listening_addr();
        mem::forget(tokio::spawn(app.serve_forever()));
        TestApp {
            gen: Generator { pool },
            addr,
            api_client: reqwest::Client::builder()
                .cookie_store(true)
                .redirect(Policy::none())
                .build()
                .unwrap(),
            registry: Registry::new(&settings.database).await.unwrap(),
        }
    }

    pub async fn login(&self) -> reqwest::Response {
        // Mock LDAP server does not allow AD-style login((
        let login_body = serde_json::json!({
            "username": "",
            "password": "",
        });
        self.api_client
            .post(&format!("http://{}/login", self.addr))
            .form(&login_body)
            .send()
            .await
            .expect("Failed to send request")
    }
}
