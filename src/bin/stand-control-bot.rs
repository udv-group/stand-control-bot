use stand_control_bot::{bot::build_tg_bot, configuration::get_config, set_env, web::Application};

use tokio::select;
use tracing::info;

use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    tracing::info!("Starting stand-control");
    let server = Application::build(&settings).await?;
    let bot_handle = tokio::spawn(async { build_tg_bot().dispatch().await });

    select! {
        _ = bot_handle => {
            info!("Bot exited")
        }
        _ = server.serve_forever() => {
            info!("Server exited")
        }
    };
    info!("stand-control shut down");
    Ok(())
}
