use std::collections::HashMap;

use crate::storage::ServerStorage;
use crate::users::UserStore;

#[derive(Default)]
pub struct SessionState {
    pub user_uuid: Option<String>,
}

pub type CommandMap = HashMap<&'static str, CommandDefinition>;

pub type CommandHandler =
    fn(&mut SessionState, &CommandMap, &mut UserStore, &mut ServerStorage, &[String]) -> String;

pub struct CommandDefinition {
    pub usage: &'static str,
    pub description: &'static str,
    pub handler: CommandHandler,
}
