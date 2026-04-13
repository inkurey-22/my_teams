use crate::commands::{CommandMap, CommandOutcome, InfoEvent, SessionState};
use crate::protocol::{quoted, response};
use crate::storage::ServerStorage;
use crate::users::UserStore;

use super::shared::{
    bad_request, call_event_private_message_sended, call_event_user_created,
    call_event_user_logged_in, emit_user_logged_out, now_unix_timestamp, unknown_user,
    validate_arg_count,
};

pub fn handle_help(
    _state: &mut SessionState,
    registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let mut commands: Vec<_> = registry.iter().collect();
    commands.sort_by_key(|(name, _)| *name);

    let body = commands
        .iter()
        .map(|(name, definition)| {
            format!("{} {} : {}", name, definition.usage, definition.description)
        })
        .collect::<Vec<_>>()
        .join("\\n");

    CommandOutcome::response_only(response(200, Some(&quoted(&body))))
}

pub fn handle_login(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() || args[0].is_empty() {
        return CommandOutcome::response_only(bad_request());
    }

    let user_name = &args[0];
    let (user_uuid, created) = users.login(user_name);
    if created {
        call_event_user_created(&user_uuid, user_name);
        if let Err(err) = storage.upsert_user(user_name, &user_uuid) {
            eprintln!("Failed to persist users JSON: {}", err);
        }
    }

    state.user_uuid = Some(user_uuid.clone());
    call_event_user_logged_in(&user_uuid);

    CommandOutcome::response_only(response(200, Some(&quoted(&user_uuid))))
}

pub fn handle_logout(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    if let Some(user_uuid) = state.user_uuid.take() {
        users.logout(&user_uuid);
        emit_user_logged_out(&user_uuid);
    }

    CommandOutcome::response_only(response(200, None))
}

pub fn handle_users(
    _state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let mut chunks = vec![quoted("USERS")];
    for (user_uuid, user_name, is_online) in users.list_users() {
        chunks.push(quoted(&user_uuid));
        chunks.push(quoted(&user_name));
        chunks.push(quoted(if is_online { "1" } else { "0" }));
    }

    CommandOutcome::response_only(response(200, Some(&chunks.join(" "))))
}

pub fn handle_user(
    _state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let user_uuid = &args[0];
    let Some((user_name, is_online)) = users.user_details(user_uuid) else {
        return CommandOutcome::response_only(unknown_user(user_uuid));
    };

    let body = [
        quoted("USER"),
        quoted(user_uuid),
        quoted(&user_name),
        quoted(if is_online { "1" } else { "0" }),
    ]
    .join(" ");

    CommandOutcome::response_only(response(200, Some(&body)))
}

pub fn handle_send(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 2, 2).is_err() || args[0].is_empty() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(sender_uuid) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    let recipient_uuid = &args[0];
    let message_body = &args[1];
    let timestamp = now_unix_timestamp();

    if !users.exists_uuid(recipient_uuid) {
        return CommandOutcome::response_only(unknown_user(recipient_uuid));
    }

    if let Err(err) =
        storage.append_private_message(sender_uuid, recipient_uuid, timestamp, message_body)
    {
        eprintln!("Failed to persist private message: {}", err);
        return CommandOutcome::response_only(response(500, Some("\"internal server error\"")));
    }

    call_event_private_message_sended(sender_uuid, recipient_uuid, message_body);

    let info_payload = format!(
        "I100 NEW_MESSAGE {} {}\r\n",
        quoted(sender_uuid),
        quoted(message_body)
    );

    CommandOutcome {
        response: response(200, None),
        info_events: vec![InfoEvent {
            recipient_user_uuid: recipient_uuid.clone(),
            payload: info_payload,
        }],
    }
}

pub fn handle_messages(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 1, 1).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(requester_uuid) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    let other_user_uuid = &args[0];
    if !users.exists_uuid(other_user_uuid) {
        return CommandOutcome::response_only(unknown_user(other_user_uuid));
    }

    let mut chunks = vec![quoted("MESSAGES")];
    for message in storage.conversation_messages(requester_uuid, other_user_uuid) {
        chunks.push(quoted(&message.sender_uuid));
        chunks.push(quoted(&message.timestamp.to_string()));
        chunks.push(quoted(&message.body));
    }

    CommandOutcome::response_only(response(200, Some(&chunks.join(" "))))
}