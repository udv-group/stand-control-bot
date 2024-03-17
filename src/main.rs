use stand_control_bot::bot::{handlers::build_handler, BotContext, BotState};
use std::{path::Path, sync::Arc};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

#[tokio::main]
async fn main() {
    {
        let env_file = Path::new(".env");
        if env_file.exists() {
            dotenv::from_filename(".env").unwrap();
        }
    }
    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or("DEBUG".to_string()))
        .init();

    log::info!("Starting bot...");
    let bot = Bot::from_env();
    let context = Arc::new(BotContext::new(bot.clone()));

    Dispatcher::builder(bot, build_handler())
        .dependencies(dptree::deps![InMemStorage::<BotState>::new(), context])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
