pub mod app;
pub mod gen;
pub mod registry;

use sqlx::{Executor, PgPool};
use stand_control_bot::configuration::{get_config, DatabaseSettings, Settings};

use uuid::Uuid;

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

pub fn setup_settings() -> Settings {
    let mut c = get_config().expect("Failed to read configuration.");
    // Use a different database for each test case
    c.database.database_name = Uuid::new_v4().to_string();
    // Use a random OS port
    c.app.port = 0;
    c
}
