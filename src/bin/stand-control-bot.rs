use anyhow::Context;
use stand_control_bot::{
    bot::{build_tg_bot, BotContext},
    configuration::get_config,
    db::{run_migrations, Registry},
    logic::{notifications::Notifier, release::hosts_release_timer, users::UsersService},
    set_env,
    web::Application,
};

use stated_dialogues::controller::{teloxide::TeloxideAdapter, ttl::track_dialog_ttl};
use teloxide::{requests::Requester, Bot};
use tokio::select;
use tracing::info;

use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = get_config()?;
    init_tracing();
    set_env();

    tracing::info!("Starting stand-control");
    run_migrations(&settings.database).await?;

    let bot = Bot::from_env();
    let bot_username = bot
        .get_me()
        .await?
        .user
        .username
        .with_context(|| "Bot hasn't username?!")?;
    let registry = Registry::new(&settings.database).await?;
    let notifier = Notifier::new(registry.clone(), TeloxideAdapter::new(bot.clone()));
    let server = Application::build(&settings, format!("https://t.me/{bot_username}")).await?;

    let bot_context = BotContext::new(bot.clone(), UsersService::new(registry.clone()));
    let dialogs_ttl_track = track_dialog_ttl(
        bot_context.dial.clone(),
        bot_context.bot_adapter.clone(),
        10,
        None,
    );

    let mut dispatcher = build_tg_bot(bot, bot_context);

    select! {
        _ = server.serve_forever() => {
            info!("Server exited")
        }
        _ = hosts_release_timer(registry, notifier) => {
            info!("Hosts release timer exited")
        }
        _ = dispatcher.dispatch() => {
            info!("Bot exited")
        }
        _ = dialogs_ttl_track => {
            info!("Dialogs ttl tracking exited")
        }
    };
    info!("stand-control shut down");
    Ok(())
}
