use stand_control_bot::{db::Registry, logic::hosts::HostsService};

use super::{configure_db, gen::Generator, setup_settings};

pub async fn create_service() -> (Generator, HostsService) {
    let configuration = setup_settings();
    let pool = configure_db(&configuration.database).await;
    (
        Generator { pool },
        HostsService::new(Registry::new(&configuration.database).await.unwrap(), 9999),
    )
}

pub async fn create_service_with_limit(limit: usize) -> (Generator, HostsService) {
    let configuration = setup_settings();
    let pool = configure_db(&configuration.database).await;
    (
        Generator { pool },
        HostsService::new(Registry::new(&configuration.database).await.unwrap(), limit),
    )
}

pub async fn create_registry() -> (Generator, Registry) {
    let configuration = setup_settings();
    let pool = configure_db(&configuration.database).await;
    (
        Generator { pool },
        Registry::new(&configuration.database).await.unwrap(),
    )
}
