use crate::commands::{CommandMap, CommandOutcome, SessionState};
use crate::protocol::{quoted, response};
use crate::storage::ServerStorage;
use crate::users::UserStore;

use super::shared::{
    bad_request, call_event_user_subscribed, call_event_user_unsubscribed, set_context,
    validate_arg_count, validate_use_context,
};

/// Subscribe the current user to a team.
pub fn handle_subscribe(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(user_uuid) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    let team_uuid = &args[0];
    if storage.team(team_uuid).is_none() {
        return CommandOutcome::response_only(response(404, Some(&quoted(team_uuid))));
    }

    users.subscribe_to_team(user_uuid, team_uuid);
    call_event_user_subscribed(team_uuid, user_uuid);

    let body = [quoted("SUBSCRIBED"), quoted(user_uuid), quoted(team_uuid)].join(" ");
    CommandOutcome::response_only(response(200, Some(&body)))
}

/// List subscribed users for a team or teams for the current user.
pub fn handle_subscribed(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(requester_uuid) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    if let Some(team_uuid) = args.first() {
        if storage.team(team_uuid).is_none() {
            return CommandOutcome::response_only(response(404, Some(&quoted(team_uuid))));
        }

        let mut chunks = vec![quoted("USERS")];
        for user_uuid in users.subscribed_user_ids(team_uuid) {
            let Some((user_name, is_online)) = users.user_details(&user_uuid) else {
                continue;
            };

            chunks.push(quoted(&user_uuid));
            chunks.push(quoted(&user_name));
            chunks.push(quoted(if is_online { "1" } else { "0" }));
        }

        return CommandOutcome::response_only(response(200, Some(&chunks.join(" "))));
    }

    let mut chunks = vec![quoted("TEAMS")];
    for team_uuid in users.subscribed_team_ids(requester_uuid) {
        let Some(team) = storage.team(&team_uuid) else {
            continue;
        };

        chunks.push(quoted(&team.uuid));
        chunks.push(quoted(&team.name));
        chunks.push(quoted(&team.description));
    }

    CommandOutcome::response_only(response(200, Some(&chunks.join(" "))))
}

/// Unsubscribe the current user from a team.
pub fn handle_unsubscribe(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(user_uuid) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    let team_uuid = &args[0];
    if storage.team(team_uuid).is_none() {
        return CommandOutcome::response_only(response(404, Some(&quoted(team_uuid))));
    }

    users.unsubscribe_from_team(user_uuid, team_uuid);
    call_event_user_unsubscribed(team_uuid, user_uuid);

    let body = [quoted("UNSUBSCRIBED"), quoted(user_uuid), quoted(team_uuid)].join(" ");
    CommandOutcome::response_only(response(200, Some(&body)))
}

/// Update the current team/channel/thread context.
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
