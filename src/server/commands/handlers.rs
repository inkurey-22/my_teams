use std::ffi::CString;

use crate::commands::{CommandMap, SessionState};
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
) -> String {
    if validate_arg_count(args, 0, 0).is_err() {
        return bad_request();
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

    response(200, Some(&quoted(&body)))
}

pub fn handle_login(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 1, 1).is_err() || args[0].is_empty() {
        return bad_request();
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

    response(200, Some(&quoted(&user_uuid)))
}

pub fn handle_logout(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 0).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_users(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 0).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_user(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 1, 1).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_send(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 2, 2).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_messages(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 1, 1).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_subscribe(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 1, 1).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_subscribed(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 1).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_unsubscribe(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 1, 1).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_use(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 3).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_create(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 0).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_list(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 0).is_err() {
        return bad_request();
    }
    not_found()
}

pub fn handle_info(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    _storage: &mut ServerStorage,
    args: &[String],
) -> String {
    if validate_arg_count(args, 0, 0).is_err() {
        return bad_request();
    }
    not_found()
}
