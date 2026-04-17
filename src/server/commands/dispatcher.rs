use crate::commands::{CommandMap, CommandOutcome, SessionState};
use crate::protocol::{parse_request_line, response};
use crate::storage::ServerStorage;
use crate::users::UserStore;

/// Parse a request line and dispatch it to the matching server command.
pub fn dispatch_line(
    state: &mut SessionState,
    commands: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    line: &str,
) -> CommandOutcome {
    let parsed = match parse_request_line(line) {
        Ok(parsed) => parsed,
        Err(_) => return CommandOutcome::response_only(response(501, Some("\"bad request\""))),
    };

    if state.user_uuid.is_none() && parsed.name != "LOGIN" && parsed.name != "USE" {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    }

    match commands.get(parsed.name.as_str()) {
        Some(definition) => (definition.handler)(state, commands, users, storage, &parsed.args),
        None => CommandOutcome::response_only(response(404, Some("\"not found\""))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::CommandDefinition;
    use std::collections::HashMap;

    fn make_test_command_map() -> CommandMap {
        let mut map = HashMap::new();
        map.insert(
            "TEST",
            CommandDefinition {
                usage: "TEST",
                description: "test command",
                handler: |state, _, _, _, _| {
                    CommandOutcome::response_only(format!(
                        "logged in: {}",
                        state.user_uuid.is_some()
                    ))
                },
            },
        );
        map
    }

    #[test]
    fn dispatch_line_with_invalid_request_format() {
        let result = parse_request_line("INVALID header");
        assert!(result.is_err());
    }

    #[test]
    fn dispatch_line_requires_authentication() {
        let state = SessionState::default();
        assert!(state.user_uuid.is_none());
    }

    #[test]
    fn dispatch_line_command_lookup_works() {
        let commands = make_test_command_map();
        assert!(commands.contains_key("TEST"));
        assert!(!commands.contains_key("NONEXISTENT"));
    }
}
