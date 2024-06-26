use anyhow::Context;
use stand_control_bot::{
    bot::build_tg_bot,
    configuration::get_config,
    db::{run_migrations, Registry},
    logic::{
        notifications::{BotAdapter, Notifier},
        release::hosts_release_timer,
        users::UsersService,
    },
    set_env,
    web::Application,
};

use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio::select;
use tracing::info;

use stand_control_bot::telemetry::init_tracing;

struct TgBotAdapter {
    bot: Bot,
}

impl TgBotAdapter {
    fn new(bot: Bot) -> Self {
        TgBotAdapter { bot }
    }
}

impl BotAdapter for TgBotAdapter {
    async fn send_message(&self, user_id: i64, msg: String) -> anyhow::Result<()> {
        self.bot.send_message(ChatId(user_id), msg).await?;
        Ok(())
    }
}

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
    let notifier = Notifier::new(registry.clone(), TgBotAdapter::new(bot.clone()));
    let server = Application::build(&settings, format!("https://t.me/{bot_username}")).await?;

    let mut dispatcher = build_tg_bot(bot, UsersService::new(registry.clone()));

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
    };
    info!("stand-control shut down");
    Ok(())
}
