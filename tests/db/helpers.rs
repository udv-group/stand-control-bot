use std::net::{IpAddr, Ipv4Addr};

use fake::{Fake, Faker};
use ipnetwork::IpNetwork;
use sqlx::{Executor, PgPool};
use stand_control_bot::configuration::{get_config, DatabaseSettings};
use stand_control_bot::db::models::{HostId, UserId};
use stand_control_bot::db::Registry;
use uuid::Uuid;

pub struct TestRegistry {
    pool: PgPool,
}
impl TestRegistry {
    pub async fn generate_host(&mut self) -> HostId {
        let hostname = Uuid::new_v4().to_string();
        let ip: Ipv4Addr = Faker.fake();
        let net = IpNetwork::new(IpAddr::V4(ip), 32).unwrap();
        let row = sqlx::query!(
            "INSERT INTO hosts (hostname, ip_address) VALUES ($1, $2) RETURNING id",
            hostname,
            net
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();
        row.id.into()
    }
    pub async fn generate_user(&mut self) -> UserId {
        let login = Uuid::new_v4().to_string();
        let row = sqlx::query!("INSERT INTO users (login) VALUES ($1) RETURNING id", login)
            .fetch_one(&self.pool)
            .await
            .unwrap();
        row.id.into()
    }
}

pub async fn create_registry() -> (TestRegistry, Registry) {
    let configuration = {
        let mut c = get_config().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.app.port = 0;
        c
    };
    let pool = configure_db(&configuration.database).await;
    (
        TestRegistry { pool },
        Registry::new(configuration.database).await.unwrap(),
    )
}

pub async fn configure_db(settings: &DatabaseSettings) -> PgPool {
    let conn = PgPool::connect_with(settings.without_db())
        .await
        .expect("Unable to connect to DB");
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, settings.database_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect_with(settings.with_db())
        .await
        .expect("Failed to connect to DB");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
