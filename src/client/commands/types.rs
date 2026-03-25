use std::collections::HashMap;
use std::io;
use std::net::TcpStream;

#[derive(Default)]
pub struct CommandContext {
    pub team_uuid: Option<String>,
    pub channel_uuid: Option<String>,
    pub thread_uuid: Option<String>,
}

#[derive(Default)]
pub struct ShellState {
    pub user_name: Option<String>,
    pub context: CommandContext,
}

pub type CommandMap = HashMap<&'static str, CommandDefinition>;

pub type CommandHandler = fn(
    &mut ShellState,
    &CommandMap,
    &mut TcpStream,
    &[String],
) -> io::Result<()>;

pub struct CommandDefinition {
    pub usage: &'static str,
    pub description: &'static str,
    pub handler: CommandHandler,
}
