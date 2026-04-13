use crate::commands::{CommandMap, CommandOutcome, SessionState};
use crate::protocol::response;
use crate::storage::ServerStorage;
use crate::users::UserStore;

use super::shared::{bad_request, not_found, set_context, validate_arg_count, validate_use_context};

pub fn handle_subscribe(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_subscribed(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_unsubscribe(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_use(
    state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 3).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    if let Err(target) = validate_use_context(storage, args) {
        if target.is_empty() {
            return CommandOutcome::response_only(bad_request());
        }

        return CommandOutcome::response_only(response(404, Some(&target)));
    }

    set_context(state, args);
    CommandOutcome::response_only(response(200, None))
}