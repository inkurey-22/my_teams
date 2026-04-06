use std::ffi::CString;

use crate::commands::{CommandMap, CommandOutcome, InfoEvent, SessionState};
use crate::libsrv;
use crate::protocol::{quoted, response};
use crate::storage::ServerStorage;
use crate::users::UserStore;

fn bad_request() -> String {
    response(501, Some("\"bad request\""))
}

fn not_found() -> String {
    response(404, Some("\"not found\""))
}

fn unknown_user(user_uuid: &str) -> String {
    response(404, Some(&quoted(user_uuid)))
}

fn validate_arg_count(args: &[String], min: usize, max: usize) -> Result<(), String> {
    if args.len() < min || args.len() > max {
        return Err(bad_request());
    }
    Ok(())
}

fn call_event_user_created(user_uuid: &str, user_name: &str) {
    let Ok(uuid) = CString::new(user_uuid) else {
        return;
    };
    let Ok(name) = CString::new(user_name) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_created(uuid.as_ptr(), name.as_ptr());
    }
}

fn call_event_user_logged_in(user_uuid: &str) {
    let Ok(uuid) = CString::new(user_uuid) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_logged_in(uuid.as_ptr());
    }
}

pub fn emit_user_logged_out(user_uuid: &str) {
    let Ok(uuid) = CString::new(user_uuid) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_logged_out(uuid.as_ptr());
    }
}

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
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    if let Some(user_uuid) = state.user_uuid.take() {
        emit_user_logged_out(&user_uuid);
    }

    CommandOutcome::response_only(response(200, None))
}

pub fn handle_users(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_user(
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

pub fn handle_send(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    _storage: &mut ServerStorage,
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

    if !users.exists_uuid(recipient_uuid) {
        return CommandOutcome::response_only(unknown_user(recipient_uuid));
    }

    let Ok(sender_cstr) = CString::new(sender_uuid) else {
        return CommandOutcome::response_only(response(500, Some("\"internal server error\"")));
    };
    let Ok(receiver_cstr) = CString::new(recipient_uuid.as_str()) else {
        return CommandOutcome::response_only(response(500, Some("\"internal server error\"")));
    };
    let Ok(message_cstr) = CString::new(message_body.as_str()) else {
        return CommandOutcome::response_only(response(500, Some("\"internal server error\"")));
    };

    unsafe {
        let _ = libsrv::server_event_private_message_sended(
            sender_cstr.as_ptr(),
            receiver_cstr.as_ptr(),
            message_cstr.as_ptr(),
        );
    }

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
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 3).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_create(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_list(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}

pub fn handle_info(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }
    CommandOutcome::response_only(not_found())
}
