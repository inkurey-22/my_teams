use crate::commands::{CommandMap, CommandOutcome, SessionState};
use crate::protocol::{parse_request_line, response};
use crate::storage::ServerStorage;
use crate::users::UserStore;

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

    if state.user_uuid.is_none() && parsed.name != "LOGIN" {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    }

    match commands.get(parsed.name.as_str()) {
        Some(definition) => (definition.handler)(state, commands, users, storage, &parsed.args),
        None => CommandOutcome::response_only(response(404, Some("\"not found\""))),
    }
}
