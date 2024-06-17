use stand_control_bot::db::run_migrations;
use stand_control_bot::{configuration::get_config, set_env, web::Application};

use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    run_migrations(&settings.database).await?;
    let server = Application::build(&settings, "bot_username".into()).await?;

    server.serve_forever().await?;
    Ok(())
}
