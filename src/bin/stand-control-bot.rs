use stand_control_bot::{
    configuration::get_config,
    db::Registry,
    logic::{notifications::Notifier, release::hosts_release_timer},
    set_env,
    web::Application,
};

use stated_dialogues::controller::teloxide::TeloxideAdapter;
use teloxide::Bot;
use tokio::select;
use tracing::info;

use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    tracing::info!("Starting stand-control");

    let bot = Bot::from_env();
    let registry = Registry::new(&settings.database).await?;

    let (sender, notifier) = Notifier::create(registry.clone(), TeloxideAdapter::new(bot.clone()));

    let server = Application::build(&settings).await?;
    select! {
        _ = server.serve_forever() => {
            info!("Server exited")
        }
        _ = notifier.work() => {
            info!("Notifier exited")
        }
        _ = hosts_release_timer(registry, sender) => {
            info!("Hosts release timer exited")
        }
    };
    info!("stand-control shut down");
    Ok(())
}
