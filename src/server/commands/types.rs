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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_outcome_response_only_has_no_events() {
        let outcome = CommandOutcome::response_only("R200 OK".to_string());
        assert_eq!(outcome.response, "R200 OK");
        assert_eq!(outcome.info_events.len(), 0);
    }

    #[test]
    fn command_outcome_with_info_events() {
        let info_event = InfoEvent {
            recipient_user_uuid: "user-123".to_string(),
            payload: "test event".to_string(),
        };
        let mut outcome = CommandOutcome::response_only("R200".to_string());
        outcome.info_events.push(info_event);

        assert_eq!(outcome.response, "R200");
        assert_eq!(outcome.info_events.len(), 1);
        assert_eq!(
            outcome.info_events[0].recipient_user_uuid,
            "user-123"
        );
    }

    #[test]
    fn command_context_default_is_empty() {
        let ctx = CommandContext::default();
        assert!(ctx.team_uuid.is_none());
        assert!(ctx.channel_uuid.is_none());
        assert!(ctx.thread_uuid.is_none());
    }

    #[test]
    fn session_state_default_is_empty() {
        let state = SessionState::default();
        assert!(state.user_uuid.is_none());
        assert!(state.context.team_uuid.is_none());
    }

    #[test]
    fn session_state_can_set_user_uuid() {
        let mut state = SessionState::default();
        state.user_uuid = Some("user-alice".to_string());
        assert_eq!(state.user_uuid.as_ref().unwrap(), "user-alice");
    }

    #[test]
    fn session_state_can_set_team_context() {
        let mut state = SessionState::default();
        state.context.team_uuid = Some("team-123".to_string());
        state.context.channel_uuid = Some("channel-456".to_string());
        assert_eq!(
            state.context.team_uuid.as_ref().unwrap(),
            "team-123"
        );
        assert_eq!(
            state.context.channel_uuid.as_ref().unwrap(),
            "channel-456"
        );
    }

    #[test]
    fn command_definition_stores_metadata() {
        let def = CommandDefinition {
            usage: "TEST [arg]",
            description: "test command",
            handler: |_, _, _, _, _| CommandOutcome::response_only("R200".to_string()),
        };

        assert_eq!(def.usage, "TEST [arg]");
        assert_eq!(def.description, "test command");
    }
}

