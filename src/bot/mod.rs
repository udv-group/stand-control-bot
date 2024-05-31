pub mod handlers;

use self::handlers::build_handler;
use crate::{dialogues::link_account::LinkAccountDialogue, logic::users::UsersService};
use anyhow::Result;
use async_trait::async_trait;
use stated_dialogues::controller::{
    teloxide::TeloxideAdapter, CtxResult, DialCtxActions, DialogueController,
};
use std::{collections::HashMap, error::Error, sync::Arc};
use teloxide::{
    dispatching::{dialogue::InMemStorage, DefaultKey, Dispatcher},
    macros::BotCommands,
    prelude::*,
    types::UserId,
    Bot,
};
use tokio::sync::RwLock;

#[derive(Clone, Default, Debug)]
pub enum BotState {
    #[default]
    Default,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Remove and initialize dialogue")]
    Reset,
    #[command(description = "Start dialogue")]
    Start,
}

pub struct BotContext {
    pub dial: Arc<RwLock<DialContext>>,
    pub bot_adapter: Arc<TeloxideAdapter>,
}

pub struct DialContext {
    pub dial_ctxs: HashMap<UserId, DialogueController>,
    pub users_service: UsersService,
}

impl BotContext {
    pub fn new(bot: Bot, users_service: UsersService) -> Self {
        BotContext {
            dial: Arc::new(RwLock::new(DialContext {
                dial_ctxs: HashMap::new(),
                users_service,
            })),
            bot_adapter: Arc::new(TeloxideAdapter::new(bot)),
        }
    }
}

#[async_trait]
impl DialCtxActions for DialContext {
    async fn new_controller(&self, _user_id: u64) -> Result<(DialogueController, Vec<CtxResult>)> {
        let context = LinkAccountDialogue::new(self.users_service.clone());
        DialogueController::create(context).await
    }

    fn take_controller(&mut self, user_id: &u64) -> Option<DialogueController> {
        self.dial_ctxs.remove(&UserId(*user_id))
    }

    fn put_controller(&mut self, user_id: u64, controller: DialogueController) {
        self.dial_ctxs.insert(UserId(user_id), controller);
    }

    fn dialogues_list(&self) -> Vec<(&u64, &DialogueController)> {
        self.dial_ctxs
            .iter()
            .map(|(user_id, controller)| (&user_id.0, controller))
            .collect()
    }
}

pub fn build_tg_bot(
    bot: Bot,
    context: BotContext,
) -> Dispatcher<Bot, Box<dyn Error + Send + Sync>, DefaultKey> {
    tracing::info!("Starting stand-control");

    Dispatcher::builder(bot, build_handler())
        .dependencies(dptree::deps![
            InMemStorage::<BotState>::new(),
            Arc::new(context)
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
