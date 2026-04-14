use std::collections::HashMap;

use crate::storage::ServerStorage;
use crate::users::UserStore;

/// Team/channel/thread selection maintained for a session.
#[derive(Default)]
pub struct CommandContext {
    /// Active team UUID, if any.
    pub team_uuid: Option<String>,
    /// Active channel UUID, if any.
    pub channel_uuid: Option<String>,
    /// Active thread UUID, if any.
    pub thread_uuid: Option<String>,
}

/// Mutable server-side session state for a connected client.
#[derive(Default)]
pub struct SessionState {
    /// Logged-in user UUID, if any.
    pub user_uuid: Option<String>,
    /// Selected team/channel/thread context.
    pub context: CommandContext,
}

/// Asynchronous info message to deliver to a connected client.
pub struct InfoEvent {
    /// Target user UUID.
    pub recipient_user_uuid: String,
    /// Wire payload to send.
    pub payload: String,
}

/// Outcome produced by a server command handler.
pub struct CommandOutcome {
    /// Response line sent back to the client.
    pub response: String,
    /// Deferred info events that should be broadcast afterward.
    pub info_events: Vec<InfoEvent>,
}

impl CommandOutcome {
    /// Create an outcome with only a direct response.
    pub fn response_only(response: String) -> Self {
        Self {
            response,
            info_events: Vec::new(),
        }
    }
}

/// Registry of server command definitions keyed by command name.
pub type CommandMap = HashMap<&'static str, CommandDefinition>;

/// Signature for a server command handler.
pub type CommandHandler = fn(
    &mut SessionState,
    &CommandMap,
    &mut UserStore,
    &mut ServerStorage,
    &[String],
) -> CommandOutcome;

/// Metadata and handler for a single server command.
pub struct CommandDefinition {
    /// Usage string shown in help output.
    pub usage: &'static str,
    /// Short description of the command.
    pub description: &'static str,
    /// Function that executes the command.
    pub handler: CommandHandler,
}
