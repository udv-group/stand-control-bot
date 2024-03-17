use std::error::Error;

use teloxide::{
    dispatching::{
        dialogue::{GetChatId, InMemStorage},
        DpHandlerDescription, HandlerExt, UpdateFilterExt,
    },
    dptree, filter_command,
    prelude::{DependencyMap, Handler},
    types::{CallbackQuery, Message, Update},
    Bot,
};

use stated_dialogues::controller::handler::{handle_interaction, process_ctx_results};
use stated_dialogues::controller::{teloxide::HandlerResult, DialCtxActions, DialInteraction};
use std::sync::Arc;

use super::{BotContext, BotState, Command};

pub fn build_handler(
) -> Handler<'static, DependencyMap, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
{
    let commands_handler = filter_command::<Command, _>()
        .branch(dptree::case![Command::Reset].endpoint(handle_reset_command))
        .endpoint(handle_command);

    let messages_hanler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<BotState>, BotState>()
        .branch(commands_handler)
        .endpoint(main_state_handler);

    let callbacks_hanlder = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<BotState>, BotState>()
        .endpoint(default_callback_handler);

    dptree::entry()
        .branch(messages_hanler)
        .branch(callbacks_hanlder)
}

async fn main_state_handler(msg: Message, context: Arc<BotContext>) -> HandlerResult {
    log::debug!(
        "Handling message. chat_id={} from={:?}",
        msg.chat.id,
        msg.from().map(|f| f.id)
    );

    let user_id = msg.from().unwrap().id;
    handle_interaction(
        &user_id.0,
        &context.bot_adapter,
        &context.dial,
        DialInteraction::Message(msg.into()),
    )
    .await
}

async fn default_callback_handler(
    _bot: Bot,
    query: CallbackQuery,
    context: Arc<BotContext>,
) -> HandlerResult {
    log::debug!(
        "Callback: called, chat_id: {:?}; from: {:?}",
        query.chat_id(),
        query.from.id
    );

    let user_id = query.from.id;
    log::debug!("Callback ({user_id}): Handling \"{:?}\"", query.data);
    handle_interaction(
        &user_id.0,
        &context.bot_adapter,
        &context.dial,
        DialInteraction::Select(query.into()),
    )
    .await
}

async fn handle_reset_command(msg: Message, context: Arc<BotContext>) -> HandlerResult {
    log::debug!(
        "Handling reset command. chat_id={} from={:?}",
        msg.chat.id,
        msg.from().map(|f| f.id)
    );
    let user_id = msg.from().unwrap().id;
    if let Some(old_controller) = context.dial.write().await.take_controller(&user_id.0) {
        process_ctx_results(user_id.0, old_controller.shutdown()?, &context.bot_adapter).await?;
    }

    handle_interaction(
        &user_id.0,
        &context.bot_adapter,
        &context.dial,
        DialInteraction::Command(msg.clone().into()),
    )
    .await
}

async fn handle_command(msg: Message, context: Arc<BotContext>) -> HandlerResult {
    log::debug!(
        "Handling {:?} command. chat_id={} from={:?}",
        msg.text(),
        msg.chat.id,
        msg.from().map(|f| f.id)
    );
    let user_id = msg.from().unwrap().id;
    handle_interaction(
        &user_id.0,
        &context.bot_adapter,
        &context.dial,
        DialInteraction::Command(msg.clone().into()),
    )
    .await
}
