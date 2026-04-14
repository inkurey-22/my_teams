use std::collections::HashMap;
use std::io;
use std::net::TcpStream;

/// Commands that are waiting for a server response.
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
    Subscribe {
        team_uuid: String,
    },
    Subscribed {
        team_uuid: Option<String>,
    },
    Unsubscribe {
        team_uuid: String,
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

/// The current command path selected by `USE`.
#[derive(Default)]
pub struct CommandContext {
    /// Active team UUID, if any.
    pub team_uuid: Option<String>,
    /// Active channel UUID, if any.
    pub channel_uuid: Option<String>,
    /// Active thread UUID, if any.
    pub thread_uuid: Option<String>,
}

/// Mutable client session state shared across command handlers.
#[derive(Default)]
pub struct SessionState {
    /// Logged-in user name, if known.
    pub user_name: Option<String>,
    /// Logged-in user UUID, if known.
    pub user_uuid: Option<String>,
    /// Selected team/channel/thread context.
    pub context: CommandContext,
    /// Request that is awaiting a response.
    pub pending_request: Option<PendingRequest>,
}

/// Registry of client command definitions keyed by command name.
pub type CommandMap = HashMap<&'static str, CommandDefinition>;

/// Signature for a client command handler.
pub type CommandHandler =
    fn(&mut SessionState, &CommandMap, &mut TcpStream, &[String]) -> io::Result<()>;

/// Metadata and handler for a single client command.
pub struct CommandDefinition {
    /// Usage string shown in help output.
    pub usage: &'static str,
    /// Short description of the command.
    pub description: &'static str,
    /// Function that executes the command.
    pub handler: CommandHandler,
}
