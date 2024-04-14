use stand_control_bot::db::Registry;

use super::{configure_db, gen::Generator, setup_settings};

pub async fn create_registry() -> (Generator, Registry) {
    let configuration = setup_settings();
    let pool = configure_db(&configuration.database).await;
    (
        Generator { pool },
        Registry::new(&configuration.database).await.unwrap(),
    )
}
