//! Client response handlers for create, list, and info resource commands.

use std::io;

use crate::commands::protocol::parse_response_tokens;
use crate::libcli;

use super::shared::{
    handle_unauthorized, handle_unknown_channel, handle_unknown_team, handle_unknown_thread,
    invalid_payload, invalid_response, invoke_channel_print, invoke_reply_print, invoke_team_print,
    invoke_thread_print, parse_status,
};

/// Handle a team creation response and print the created team.
pub(super) fn handle_create_team_response(code: u16, response: &str) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 409 {
        unsafe {
            let _ = libcli::client_error_already_exist();
        }
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("TEAM") {
        return Err(invalid_payload("invalid TEAM response payload"));
    }

    invoke_team_print(
        libcli::client_print_team_created,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
    )
}

/// Handle a channel creation response and print the created channel.
pub(super) fn handle_create_channel_response(
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
    if code == 409 {
        unsafe {
            let _ = libcli::client_error_already_exist();
        }
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("CHANNEL") {
        return Err(invalid_payload("invalid CHANNEL response payload"));
    }

    invoke_channel_print(
        libcli::client_print_channel_created,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
    )
}

/// Handle a thread creation response and print the created thread.
pub(super) fn handle_create_thread_response(
    code: u16,
    response: &str,
    team_uuid: String,
    channel_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_channel(&channel_uuid)?;
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code == 409 {
        unsafe {
            let _ = libcli::client_error_already_exist();
        }
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 6 || tokens.first().map(|t| t.as_str()) != Some("THREAD") {
        return Err(invalid_payload("invalid THREAD response payload"));
    }

    invoke_thread_print(
        libcli::client_print_thread_created,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
        tokens[4].as_str(),
        tokens[5].as_str(),
    )
}

/// Handle a reply creation response and print the created reply.
pub(super) fn handle_create_reply_response(
    code: u16,
    response: &str,
    team_uuid: String,
    channel_uuid: String,
    thread_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_thread(&thread_uuid)?;
        handle_unknown_channel(&channel_uuid)?;
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 5 || tokens.first().map(|t| t.as_str()) != Some("REPLY") {
        return Err(invalid_payload("invalid REPLY response payload"));
    }

    invoke_reply_print(
        libcli::client_print_reply_created,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
        tokens[4].as_str(),
    )
}

/// Handle a team listing response and print each team.
pub(super) fn handle_list_teams_response(code: u16, response: &str) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.first().map(|t| t.as_str()) != Some("TEAMS") || (tokens.len() - 1) % 3 != 0 {
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

    Ok(())
}

/// Handle a channel listing response and print each channel.
pub(super) fn handle_list_channels_response(
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
    if tokens.first().map(|t| t.as_str()) != Some("CHANNELS") || (tokens.len() - 1) % 3 != 0 {
        return Err(invalid_payload("invalid CHANNELS response payload"));
    }

    for entry in tokens[1..].chunks(3) {
        invoke_channel_print(
            libcli::client_print_channel,
            entry[0].as_str(),
            entry[1].as_str(),
            entry[2].as_str(),
        )?;
    }

    Ok(())
}

/// Handle a thread listing response and print each thread.
pub(super) fn handle_list_threads_response(
    code: u16,
    response: &str,
    team_uuid: String,
    channel_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_channel(&channel_uuid)?;
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.first().map(|t| t.as_str()) != Some("THREADS") || (tokens.len() - 1) % 5 != 0 {
        return Err(invalid_payload("invalid THREADS response payload"));
    }

    for entry in tokens[1..].chunks(5) {
        invoke_thread_print(
            libcli::client_print_thread,
            entry[0].as_str(),
            entry[1].as_str(),
            entry[2].as_str(),
            entry[3].as_str(),
            entry[4].as_str(),
        )?;
    }

    Ok(())
}

/// Handle a reply listing response and print each reply.
pub(super) fn handle_list_replies_response(
    code: u16,
    response: &str,
    team_uuid: String,
    channel_uuid: String,
    thread_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_thread(&thread_uuid)?;
        handle_unknown_channel(&channel_uuid)?;
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.first().map(|t| t.as_str()) != Some("REPLIES") || (tokens.len() - 1) % 3 != 0 {
        return Err(invalid_payload("invalid REPLIES response payload"));
    }

    for entry in tokens[1..].chunks(3) {
        invoke_reply_print(
            libcli::client_thread_print_replies,
            thread_uuid.as_str(),
            entry[0].as_str(),
            entry[1].as_str(),
            entry[2].as_str(),
        )?;
    }

    Ok(())
}

/// Handle an info-user response and print the current user.
pub(super) fn handle_info_user_response(code: u16, response: &str) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("USER") {
        return Err(invalid_payload("invalid USER response payload"));
    }

    let user_uuid_cstr = super::shared::cstring(tokens[1].as_str(), "user UUID")?;
    let user_name_cstr = super::shared::cstring(tokens[2].as_str(), "user name")?;
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

/// Handle an info-team response and print the selected team.
pub(super) fn handle_info_team_response(
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
    if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("TEAM") {
        return Err(invalid_payload("invalid TEAM response payload"));
    }

    invoke_team_print(
        libcli::client_print_team,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
    )
}

/// Handle an info-channel response and print the selected channel.
pub(super) fn handle_info_channel_response(
    code: u16,
    response: &str,
    team_uuid: String,
    channel_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_channel(&channel_uuid)?;
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("CHANNEL") {
        return Err(invalid_payload("invalid CHANNEL response payload"));
    }

    invoke_channel_print(
        libcli::client_print_channel,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
    )
}

/// Handle an info-thread response and print the selected thread.
pub(super) fn handle_info_thread_response(
    code: u16,
    response: &str,
    team_uuid: String,
    channel_uuid: String,
    thread_uuid: String,
) -> io::Result<()> {
    if code == 401 {
        handle_unauthorized();
        return Ok(());
    }
    if code == 404 {
        handle_unknown_thread(&thread_uuid)?;
        handle_unknown_channel(&channel_uuid)?;
        handle_unknown_team(&team_uuid)?;
        return Ok(());
    }
    if code != 200 {
        return Err(invalid_response(response));
    }

    let tokens = parse_response_tokens(response)?;
    if tokens.len() != 6 || tokens.first().map(|t| t.as_str()) != Some("THREAD") {
        return Err(invalid_payload("invalid THREAD response payload"));
    }

    invoke_thread_print(
        libcli::client_print_thread,
        tokens[1].as_str(),
        tokens[2].as_str(),
        tokens[3].as_str(),
        tokens[4].as_str(),
        tokens[5].as_str(),
    )
}
