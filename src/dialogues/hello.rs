use anyhow::Result;

use stated_dialogues::dialogues::{CtxResult, DialContext, Message, MessageId, Select};

pub struct HelloDialogue {}

impl Default for HelloDialogue {
    fn default() -> Self {
        Self::new()
    }
}

impl HelloDialogue {
    pub fn new() -> Self {
        HelloDialogue {}
    }
}

impl DialContext for HelloDialogue {
    fn init(&mut self) -> Result<Vec<CtxResult>> {
        Ok(vec![CtxResult::Messages(vec!["Dialog created!".into()])])
    }

    fn shutdown(&mut self) -> Result<Vec<CtxResult>> {
        Ok(vec![CtxResult::Messages(vec!["Shutdown dialog...".into()])])
    }

    fn handle_select(&mut self, _select: Select) -> Result<Vec<CtxResult>> {
        Ok(vec![])
    }

    fn handle_message(&mut self, _input: Message) -> Result<Vec<CtxResult>> {
        Ok(vec![CtxResult::Messages(vec!["Message handled!".into()])])
    }

    fn handle_command(&mut self, _command: Message) -> Result<Vec<CtxResult>> {
        Ok(vec![CtxResult::Messages(vec!["Command handled...".into()])])
    }

    fn remember_sent_messages(&mut self, _msg_ids: Vec<MessageId>) {}
}
