//! Client response handlers for login, user, and message commands.

use std::io;

use crate::commands::protocol::{extract_uuid_from_body, parse_response_tokens};
use crate::commands::SessionState;
use crate::libcli;

use super::shared::{
    cstring, handle_unauthorized, handle_unknown_user, invalid_payload, invalid_response,
    parse_status, parse_timestamp,
};

/// Handle a login response and update the cached session state.
pub(super) fn handle_login_response(
    state: &mut SessionState,
    code: u16,
    response: &str,
    user_name: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let user_uuid = extract_uuid_from_body(response)?;
    state.user_name = Some(user_name.clone());
    state.user_uuid = Some(user_uuid.clone());

    Ok(())
}

/// Handle a logout response and clear the cached session state.
pub(super) fn handle_logout_response(
    state: &mut SessionState,
    code: u16,
    response: &str,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let user_uuid = state.user_uuid.take();
    let user_name = state.user_name.take();
    let _ = (user_uuid, user_name);

    Ok(())
}

/// Handle a users listing response and print each entry.
pub(super) fn handle_users_response(code: u16, response: &str) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.first().map(|t| t.as_str()) != Some("USERS") {
        return Err(invalid_payload("invalid USERS response payload"));
    }
    if (tokens.len() - 1) % 3 != 0 {
        return Err(invalid_payload("invalid USERS response entry count"));
    }

    for entry in tokens[1..].chunks(3) {
        let user_uuid_cstr = cstring(entry[0].as_str(), "user UUID")?;
        let user_name_cstr = cstring(entry[1].as_str(), "user name")?;
        let user_status = parse_status(&entry[2])?;

        unsafe {
            let _ = libcli::client_print_users(
                user_uuid_cstr.as_ptr(),
                user_name_cstr.as_ptr(),
                user_status,
            );
        }
    }

    Ok(())
}

/// Handle a user detail response and print the selected user.
pub(super) fn handle_user_response(code: u16, response: &str, user_uuid: String) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_user(&user_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("USER") {
        return Err(invalid_payload("invalid USER response payload"));
    }

    let user_uuid_cstr = cstring(tokens[1].as_str(), "user UUID")?;
    let user_name_cstr = cstring(tokens[2].as_str(), "user name")?;
    let user_status = parse_status(&tokens[3])?;

    unsafe {
        let _ = libcli::client_print_user(
            user_uuid_cstr.as_ptr(),
            user_name_cstr.as_ptr(),
            user_status,
        );
    }

    Ok(())
}

/// Handle a send response and surface any server-side errors.
pub(super) fn handle_send_response(code: u16, response: &str, user_uuid: String) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_user(&user_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    Ok(())
}

/// Handle a message history response and print each message.
pub(super) fn handle_messages_response(
    code: u16,
    response: &str,
    user_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_user(&user_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.first().map(|t| t.as_str()) != Some("MESSAGES") {
        return Err(invalid_payload("invalid MESSAGES response payload"));
    }
    if (tokens.len() - 1) % 3 != 0 {
        return Err(invalid_payload("invalid MESSAGES response entry count"));
    }

    for entry in tokens[1..].chunks(3) {
        let sender_uuid_cstr = cstring(entry[0].as_str(), "user UUID")?;
        let timestamp = parse_timestamp(&entry[1])?;
        let message_body_cstr = cstring(entry[2].as_str(), "message body")?;

        unsafe {
            let _ = libcli::client_private_message_print_messages(
                sender_uuid_cstr.as_ptr(),
                timestamp,
                message_body_cstr.as_ptr(),
            );
        }
    }

    Ok(())
}
