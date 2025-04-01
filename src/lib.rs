use std::path::Path;

pub mod bot;
pub mod configuration;
pub mod db;
pub mod ldap;
pub mod logic;
pub mod telemetry;
pub mod web;

pub fn set_env() {
    let env_file = Path::new(".env");
    if env_file.exists() {
        dotenv::from_filename(".env").unwrap();
    }
}
