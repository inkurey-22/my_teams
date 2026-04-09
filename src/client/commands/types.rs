use std::collections::HashMap;
use std::io;
use std::net::TcpStream;

#[derive(Clone)]
pub enum PendingRequest {
    Login {
        user_name: String,
    },
    Logout,
    Users,
    User {
        user_uuid: String,
    },
    Send {
        user_uuid: String,
        #[allow(dead_code)]
        message_body: String,
    },
}

#[derive(Default)]
pub struct CommandContext {
    pub team_uuid: Option<String>,
    pub channel_uuid: Option<String>,
    pub thread_uuid: Option<String>,
}

#[derive(Default)]
pub struct SessionState {
    pub user_name: Option<String>,
    pub user_uuid: Option<String>,
    pub context: CommandContext,
    pub pending_request: Option<PendingRequest>,
}

pub type CommandMap = HashMap<&'static str, CommandDefinition>;

pub type CommandHandler =
    fn(&mut SessionState, &CommandMap, &mut TcpStream, &[String]) -> io::Result<()>;

pub struct CommandDefinition {
    pub usage: &'static str,
    pub description: &'static str,
    pub handler: CommandHandler,
}
