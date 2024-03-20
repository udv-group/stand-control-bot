use stand_control_bot::bot::{handlers::build_handler, BotContext, BotState};
use std::{path::Path, sync::Arc};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

use stand_control_bot::telemetry::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing();
    set_env();

    tracing::info!("Starting bot...");
    let bot = Bot::from_env();
    let context = Arc::new(BotContext::new(bot.clone()));

    Dispatcher::builder(bot, build_handler())
        .dependencies(dptree::deps![InMemStorage::<BotState>::new(), context])
        .default_handler(|upd| async move {
            tracing::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn set_env() {
    let env_file = Path::new(".env");
    if env_file.exists() {
        dotenv::from_filename(".env").unwrap();
    }
}
