use anyhow::{Context, Result};

use async_trait::async_trait;
use stated_dialogues::dialogues::{CtxResult, DialContext, Message, MessageId, Select};

use crate::logic::users::UsersService;

pub struct LinkAccountDialogue {
    users_service: UsersService,
}

impl LinkAccountDialogue {
    pub fn new(users_service: UsersService) -> Self {
        LinkAccountDialogue { users_service }
    }
}

#[async_trait]
impl DialContext for LinkAccountDialogue {
    async fn init(&mut self) -> Result<Vec<CtxResult>> {
        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<Vec<CtxResult>> {
        Ok(vec![])
    }

    async fn handle_select(&mut self, _select: Select) -> Result<Vec<CtxResult>> {
        Ok(vec![])
    }

    async fn handle_message(&mut self, input: Message) -> Result<Vec<CtxResult>> {
        let user_id = input
            .user_id
            .clone()
            .with_context(|| "Message without user_id")?;
        let link = input
            .text()
            .unwrap_or("")
            .trim_matches(|c| c == ' ' || c == '"');

        match self
            .users_service
            .link_user(link, &user_id.0)
            .await
            .with_context(|| "Failed user link")?
        {
            Some(user) => Ok(vec![
                CtxResult::RemoveMessages(vec![input.id]),
                CtxResult::Messages(vec![format!(
                    "Telegram successfully linked for user '{}'",
                    user.login
                )
                .into()]),
            ]),
            None => Ok(vec![CtxResult::RemoveMessages(vec![input.id])]),
        }
    }

    async fn handle_command(&mut self, _command: Message) -> Result<Vec<CtxResult>> {
        Ok(vec![])
    }

    fn remember_sent_messages(&mut self, _msg_ids: Vec<MessageId>) {}
}
