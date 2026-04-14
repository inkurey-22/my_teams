use std::io;

use crate::commands::protocol::parse_response_tokens;
use crate::commands::SessionState;
use crate::libcli;

use super::shared::{
    cstring, handle_unauthorized, handle_unknown_channel, handle_unknown_team,
    handle_unknown_thread, invalid_payload, invalid_response, invoke_team_print, parse_status,
};

pub(super) fn handle_subscribe_response(
    code: u16,
    response: &str,
    team_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 3 || tokens.first().map(|t| t.as_str()) != Some("SUBSCRIBED") {
        return Err(invalid_payload("invalid SUBSCRIBED response payload"));
    }

    let user_uuid_cstr = cstring(tokens[1].as_str(), "user UUID")?;
    let team_uuid_cstr = cstring(tokens[2].as_str(), "team UUID")?;
    unsafe {
        let _ = libcli::client_print_subscribed(user_uuid_cstr.as_ptr(), team_uuid_cstr.as_ptr());
    }

    Ok(())
}

pub(super) fn handle_subscribed_response(
    code: u16,
    response: &str,
    team_uuid: Option<String>,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        if let Some(team_uuid) = team_uuid {
            handle_unknown_team(&team_uuid)?;
            return Ok(());
        }
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    match tokens.first().map(|t| t.as_str()) {
        Some("TEAMS") => {
            if (tokens.len() - 1) % 3 != 0 {
                return Err(invalid_payload("invalid TEAMS response payload"));
            }

            for entry in tokens[1..].chunks(3) {
                invoke_team_print(
                    libcli::client_print_teams,
                    entry[0].as_str(),
                    entry[1].as_str(),
                    entry[2].as_str(),
                )?;
            }
        }
        Some("USERS") => {
            if (tokens.len() - 1) % 3 != 0 {
                return Err(invalid_payload("invalid USERS response payload"));
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
        }
        _ => {
            return Err(invalid_payload("invalid SUBSCRIBED response payload"));
        }
    }

    Ok(())
}

pub(super) fn handle_unsubscribe_response(
    code: u16,
    response: &str,
    team_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 3 || tokens.first().map(|t| t.as_str()) != Some("UNSUBSCRIBED") {
        return Err(invalid_payload("invalid UNSUBSCRIBED response payload"));
    }

    let user_uuid_cstr = cstring(tokens[1].as_str(), "user UUID")?;
    let team_uuid_cstr = cstring(tokens[2].as_str(), "team UUID")?;
    unsafe {
        let _ = libcli::client_print_unsubscribed(user_uuid_cstr.as_ptr(), team_uuid_cstr.as_ptr());
    }

    Ok(())
}

pub(super) fn handle_use_response(
    state: &mut SessionState,
    code: u16,
    response: &str,
    team_uuid: Option<String>,
    channel_uuid: Option<String>,
    thread_uuid: Option<String>,
) -> io::Result<()> {
    if code == 404 {
        if let Some(thread_uuid) = thread_uuid.as_deref() {
            handle_unknown_thread(thread_uuid)?;
            return Ok(());
        }
        if let Some(channel_uuid) = channel_uuid.as_deref() {
            handle_unknown_channel(channel_uuid)?;
            return Ok(());
        }
        if let Some(team_uuid) = team_uuid.as_deref() {
            handle_unknown_team(team_uuid)?;
            return Ok(());
        }

        return Err(invalid_payload("invalid USE response payload"));
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    state.context.team_uuid = team_uuid;
    state.context.channel_uuid = channel_uuid;
    state.context.thread_uuid = thread_uuid;
    Ok(())
}
