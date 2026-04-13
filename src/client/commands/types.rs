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
    Messages {
        user_uuid: String,
    },
    Use {
        team_uuid: Option<String>,
        channel_uuid: Option<String>,
        thread_uuid: Option<String>,
    },
    CreateTeam,
    CreateChannel {
        team_uuid: String,
    },
    CreateThread {
        team_uuid: String,
        channel_uuid: String,
    },
    CreateReply {
        team_uuid: String,
        channel_uuid: String,
        thread_uuid: String,
    },
    ListTeams,
    ListChannels {
        team_uuid: String,
    },
    ListThreads {
        team_uuid: String,
        channel_uuid: String,
    },
    ListReplies {
        team_uuid: String,
        channel_uuid: String,
        thread_uuid: String,
    },
    InfoUser,
    InfoTeam {
        team_uuid: String,
    },
    InfoChannel {
        team_uuid: String,
        channel_uuid: String,
    },
    InfoThread {
        team_uuid: String,
        channel_uuid: String,
        thread_uuid: String,
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
