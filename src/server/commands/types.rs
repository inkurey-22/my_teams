use std::collections::HashMap;

use crate::storage::ServerStorage;
use crate::users::UserStore;

#[derive(Default)]
pub struct CommandContext {
    pub team_uuid: Option<String>,
    pub channel_uuid: Option<String>,
    pub thread_uuid: Option<String>,
}

#[derive(Default)]
pub struct SessionState {
    pub user_uuid: Option<String>,
    pub context: CommandContext,
}

pub struct InfoEvent {
    pub recipient_user_uuid: String,
    pub payload: String,
}

pub struct CommandOutcome {
    pub response: String,
    pub info_events: Vec<InfoEvent>,
}

impl CommandOutcome {
    pub fn response_only(response: String) -> Self {
        Self {
            response,
            info_events: Vec::new(),
        }
    }
}

pub type CommandMap = HashMap<&'static str, CommandDefinition>;

pub type CommandHandler = fn(
    &mut SessionState,
    &CommandMap,
    &mut UserStore,
    &mut ServerStorage,
    &[String],
) -> CommandOutcome;

pub struct CommandDefinition {
    pub usage: &'static str,
    pub description: &'static str,
    pub handler: CommandHandler,
}
