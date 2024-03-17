pub mod handlers;

use anyhow::Result;
use stated_dialogues::controller::{
    teloxide::TeloxideAdapter, CtxResult, DialCtxActions, DialogueController,
};
use std::{collections::HashMap, sync::Arc};
use teloxide::{macros::BotCommands, types::UserId, Bot};
use tokio::sync::RwLock;

use crate::dialogues::hello::HelloDialogue;

#[derive(Clone, Default, Debug)]
pub enum BotState {
    #[default]
    Default,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Remove and initialize dialog")]
    Reset,
}

pub struct BotContext {
    pub dial: Arc<RwLock<DialContext>>,
    pub bot_adapter: Arc<TeloxideAdapter>,
}

pub struct DialContext {
    pub dial_ctxs: HashMap<UserId, DialogueController>,
}

impl BotContext {
    pub fn new(bot: Bot) -> Self {
        BotContext {
            dial: Arc::new(RwLock::new(DialContext {
                dial_ctxs: HashMap::new(),
            })),
            bot_adapter: Arc::new(TeloxideAdapter::new(bot)),
        }
    }
}

impl DialCtxActions for DialContext {
    fn new_controller(&self, _user_id: u64) -> Result<(DialogueController, Vec<CtxResult>)> {
        let context = HelloDialogue::new();
        DialogueController::create(context)
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
