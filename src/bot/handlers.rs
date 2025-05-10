use std::error::Error;

use anyhow::Context;
use teloxide::{
    Bot,
    dispatching::{DpHandlerDescription, HandlerExt, UpdateFilterExt, dialogue::InMemStorage},
    dptree, filter_command,
    prelude::{DependencyMap, Handler},
    requests::Requester,
    types::{Message, Update},
};

use crate::logic::users::UsersService;

use super::{BotState, Command};

pub type AnyResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type HandlerResult = AnyResult<()>;

pub fn build_handler()
-> Handler<'static, DependencyMap, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription> {
    let commands_handler = filter_command::<Command, _>()
        .branch(dptree::case![Command::Start].endpoint(handle_start_command));

    let messages_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<BotState>, BotState>()
        .branch(commands_handler)
        .endpoint(main_state_handler);

    dptree::entry().branch(messages_handler)
}

async fn main_state_handler(bot: Bot, msg: Message, users_service: UsersService) -> HandlerResult {
    tracing::debug!(
        "Handling message. chat_id={} from={:?}",
        msg.chat.id,
        msg.from.as_ref().map(|f| f.id)
    );

    let user_id = msg
        .from
        .as_ref()
        .map(|m_from| m_from.id)
        .with_context(|| "Message without user_id")?;
    let link = msg
        .text()
        .unwrap_or("")
        .trim_matches(|c| c == ' ' || c == '"');

    match users_service
        .link_user(link, user_id.0.to_string().as_ref())
        .await
        .with_context(|| "Failed user link")?
    {
        Some(user) => {
            bot.send_message(
                msg.chat.id,
                format!("Telegram successfully linked for user '{}'", user.email),
            )
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "User for this link code not found")
                .await?;
        }
    };
    bot.delete_message(msg.chat.id, msg.id).await?;

    Ok(())
}

async fn handle_start_command(bot: Bot, msg: Message) -> HandlerResult {
    tracing::debug!(
        "Handling {:?} command. chat_id={} from={:?}",
        msg.text(),
        msg.chat.id,
        msg.from.as_ref().map(|f| f.id)
    );
    bot.send_message(msg.chat.id, "Hello, send your link code")
        .await?;

    Ok(())
}
