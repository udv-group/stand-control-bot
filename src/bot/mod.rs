pub mod handlers;

use self::handlers::build_handler;
use crate::logic::users::UsersService;

use std::error::Error;
use teloxide::{
    dispatching::{dialogue::InMemStorage, DefaultKey, Dispatcher},
    macros::BotCommands,
    prelude::*,
    Bot,
};

#[derive(Clone, Default, Debug)]
pub enum BotState {
    #[default]
    Default,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Start dialogue")]
    Start,
}

pub fn build_tg_bot(
    bot: Bot,
    users_service: UsersService,
) -> Dispatcher<Bot, Box<dyn Error + Send + Sync>, DefaultKey> {
    tracing::info!("Starting stand-control");

    Dispatcher::builder(bot, build_handler())
        .dependencies(dptree::deps![
            InMemStorage::<BotState>::new(),
            users_service
        ])
        .default_handler(|upd| async move {
            tracing::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
}
