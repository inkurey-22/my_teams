use std::ffi::CString;
use std::io;
use std::os::raw::c_int;

use crate::commands::protocol::{extract_uuid_from_body, parse_response_code, parse_response_tokens};
use crate::commands::{PendingRequest, SessionState};
use crate::libcli;

fn parse_status(status: &str) -> io::Result<c_int> {
    match status {
        "0" => Ok(0),
        "1" => Ok(1),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid user status: {}", status),
        )),
    }
}

fn parse_timestamp(timestamp: &str) -> io::Result<libcli::TimeT> {
    timestamp.parse::<libcli::TimeT>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid message timestamp: {}", timestamp),
        )
    })
}

fn cstring(value: &str, field: &str) -> io::Result<CString> {
    CString::new(value).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} contains an invalid NUL byte", field),
        )
    })
}

fn invoke_team_print(
    callback: unsafe extern "C" fn(
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
    ) -> c_int,
    team_uuid: &str,
    team_name: &str,
    team_description: &str,
) -> io::Result<()>
{
    let team_uuid_cstr = cstring(team_uuid, "team UUID")?;
    let team_name_cstr = cstring(team_name, "team name")?;
    let team_description_cstr = cstring(team_description, "team description")?;

    unsafe {
        let _ = callback(
            team_uuid_cstr.as_ptr(),
            team_name_cstr.as_ptr(),
            team_description_cstr.as_ptr(),
        );
    }

    Ok(())
}

fn invoke_channel_print(
    callback: unsafe extern "C" fn(
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
    ) -> c_int,
    channel_uuid: &str,
    channel_name: &str,
    channel_description: &str,
) -> io::Result<()>
{
    let channel_uuid_cstr = cstring(channel_uuid, "channel UUID")?;
    let channel_name_cstr = cstring(channel_name, "channel name")?;
    let channel_description_cstr = cstring(channel_description, "channel description")?;

    unsafe {
        let _ = callback(
            channel_uuid_cstr.as_ptr(),
            channel_name_cstr.as_ptr(),
            channel_description_cstr.as_ptr(),
        );
    }

    Ok(())
}

fn invoke_thread_print(
    callback: unsafe extern "C" fn(
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        libcli::TimeT,
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
    ) -> c_int,
    thread_uuid: &str,
    user_uuid: &str,
    timestamp: &str,
    thread_title: &str,
    thread_body: &str,
) -> io::Result<()>
{
    let thread_uuid_cstr = cstring(thread_uuid, "thread UUID")?;
    let user_uuid_cstr = cstring(user_uuid, "user UUID")?;
    let timestamp = parse_timestamp(timestamp)?;
    let thread_title_cstr = cstring(thread_title, "thread title")?;
    let thread_body_cstr = cstring(thread_body, "thread body")?;

    unsafe {
        let _ = callback(
            thread_uuid_cstr.as_ptr(),
            user_uuid_cstr.as_ptr(),
            timestamp,
            thread_title_cstr.as_ptr(),
            thread_body_cstr.as_ptr(),
        );
    }

    Ok(())
}

fn invoke_reply_print(
    callback: unsafe extern "C" fn(
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        libcli::TimeT,
        *const std::os::raw::c_char,
    ) -> c_int,
    thread_uuid: &str,
    user_uuid: &str,
    timestamp: &str,
    reply_body: &str,
) -> io::Result<()>
{
    let thread_uuid_cstr = cstring(thread_uuid, "thread UUID")?;
    let user_uuid_cstr = cstring(user_uuid, "user UUID")?;
    let timestamp = parse_timestamp(timestamp)?;
    let reply_body_cstr = cstring(reply_body, "reply body")?;

    unsafe {
        let _ = callback(
            thread_uuid_cstr.as_ptr(),
            user_uuid_cstr.as_ptr(),
            timestamp,
            reply_body_cstr.as_ptr(),
        );
    }

    Ok(())
}

pub fn handle_response_line(state: &mut SessionState, response: &str) -> io::Result<()> {
    let code = parse_response_code(response)?;
    let pending_request = state.pending_request.take();

    match pending_request {
        Some(PendingRequest::Login { user_name }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let user_uuid = extract_uuid_from_body(response)?;
            state.user_name = Some(user_name.clone());
            state.user_uuid = Some(user_uuid.clone());

            let user_uuid_cstr = CString::new(user_uuid).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "user UUID contains null byte")
            })?;
            let user_name_cstr = CString::new(user_name).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "user name contains an invalid NUL byte",
                )
            })?;

            unsafe {
                let _ = libcli::client_event_logged_in(
                    user_uuid_cstr.as_ptr(),
                    user_name_cstr.as_ptr(),
                );
            }
        }
        Some(PendingRequest::Logout) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let user_uuid = state.user_uuid.take();
            let user_name = state.user_name.take();

            if let (Some(user_uuid), Some(user_name)) = (user_uuid, user_name) {
                let user_uuid_cstr = CString::new(user_uuid).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "user UUID contains null byte")
                })?;
                let user_name_cstr = CString::new(user_name).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "user name contains an invalid NUL byte",
                    )
                })?;

                unsafe {
                    let _ = libcli::client_event_logged_out(
                        user_uuid_cstr.as_ptr(),
                        user_name_cstr.as_ptr(),
                    );
                }
            }
        }
        Some(PendingRequest::Users) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.first().map(|t| t.as_str()) != Some("USERS") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USERS response payload",
                ));
            }

            if (tokens.len() - 1) % 3 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USERS response entry count",
                ));
            }

            for entry in tokens[1..].chunks(3) {
                let user_uuid_cstr = CString::new(entry[0].as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "user UUID contains null byte")
                })?;
                let user_name_cstr = CString::new(entry[1].as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "user name contains null byte")
                })?;
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
        Some(PendingRequest::User { user_uuid }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let target_cstr = CString::new(user_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "target user UUID contains an invalid NUL byte",
                    )
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_user(target_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("USER") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USER response payload",
                ));
            }

            let user_uuid_cstr = CString::new(tokens[1].as_str()).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "user UUID contains null byte")
            })?;
            let user_name_cstr = CString::new(tokens[2].as_str()).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "user name contains null byte")
            })?;
            let user_status = parse_status(&tokens[3])?;

            unsafe {
                let _ = libcli::client_print_user(
                    user_uuid_cstr.as_ptr(),
                    user_name_cstr.as_ptr(),
                    user_status,
                );
            }
        }
        Some(PendingRequest::Send { user_uuid, .. }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let target_cstr = CString::new(user_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "target user UUID contains an invalid NUL byte",
                    )
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_user(target_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }
        }
        Some(PendingRequest::Messages { user_uuid }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let target_cstr = CString::new(user_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "target user UUID contains an invalid NUL byte",
                    )
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_user(target_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.first().map(|t| t.as_str()) != Some("MESSAGES") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid MESSAGES response payload",
                ));
            }

            if (tokens.len() - 1) % 3 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid MESSAGES response entry count",
                ));
            }

            for entry in tokens[1..].chunks(3) {
                let sender_uuid_cstr = CString::new(entry[0].as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "user UUID contains null byte")
                })?;
                let timestamp = parse_timestamp(&entry[1])?;
                let message_body_cstr = CString::new(entry[2].as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "message body contains null byte")
                })?;

                unsafe {
                    let _ = libcli::client_private_message_print_messages(
                        sender_uuid_cstr.as_ptr(),
                        timestamp,
                        message_body_cstr.as_ptr(),
                    );
                }
            }
        }
        Some(PendingRequest::Use {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            if code == 404 {
                if let Some(thread_uuid) = thread_uuid.as_deref() {
                    let thread_cstr = CString::new(thread_uuid).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "thread UUID contains an invalid NUL byte",
                        )
                    })?;
                    unsafe {
                        let _ = libcli::client_error_unknown_thread(thread_cstr.as_ptr());
                    }
                    return Ok(());
                }

                if let Some(channel_uuid) = channel_uuid.as_deref() {
                    let channel_cstr = CString::new(channel_uuid).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "channel UUID contains an invalid NUL byte",
                        )
                    })?;
                    unsafe {
                        let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    }
                    return Ok(());
                }

                if let Some(team_uuid) = team_uuid.as_deref() {
                    let team_cstr = CString::new(team_uuid).map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "team UUID contains an invalid NUL byte",
                        )
                    })?;
                    unsafe {
                        let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                    }
                    return Ok(());
                }

                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USE response payload",
                ));
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            state.context.team_uuid = team_uuid;
            state.context.channel_uuid = channel_uuid;
            state.context.thread_uuid = thread_uuid;
        }
        Some(PendingRequest::CreateTeam) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 409 {
                unsafe {
                    let _ = libcli::client_error_already_exist();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("TEAM") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid TEAM response payload",
                ));
            }

            invoke_team_print(
                libcli::client_print_team_created,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
            )?;
        }
        Some(PendingRequest::CreateChannel { team_uuid }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code == 409 {
                unsafe {
                    let _ = libcli::client_error_already_exist();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("CHANNEL") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid CHANNEL response payload",
                ));
            }

            invoke_channel_print(
                libcli::client_print_channel_created,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
            )?;
        }
        Some(PendingRequest::CreateThread {
            team_uuid,
            channel_uuid,
        }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let channel_cstr = CString::new(channel_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "channel UUID contains null byte",
                    )
                })?;
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code == 409 {
                unsafe {
                    let _ = libcli::client_error_already_exist();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 6 || tokens.first().map(|t| t.as_str()) != Some("THREAD") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid THREAD response payload",
                ));
            }

            invoke_thread_print(
                libcli::client_print_thread_created,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
                tokens[4].as_str(),
                tokens[5].as_str(),
            )?;
        }
        Some(PendingRequest::CreateReply {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let thread_cstr = CString::new(thread_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "thread UUID contains null byte",
                    )
                })?;
                let channel_cstr = CString::new(channel_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "channel UUID contains null byte",
                    )
                })?;
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_thread(thread_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 5 || tokens.first().map(|t| t.as_str()) != Some("REPLY") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid REPLY response payload",
                ));
            }

            invoke_reply_print(
                libcli::client_print_reply_created,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
                tokens[4].as_str(),
            )?;
        }
        Some(PendingRequest::ListTeams) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.first().map(|t| t.as_str()) != Some("TEAMS") || (tokens.len() - 1) % 3 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid TEAMS response payload",
                ));
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
        Some(PendingRequest::ListChannels { team_uuid }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.first().map(|t| t.as_str()) != Some("CHANNELS") || (tokens.len() - 1) % 3 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid CHANNELS response payload",
                ));
            }

            for entry in tokens[1..].chunks(3) {
                invoke_channel_print(
                    libcli::client_print_channel,
                    entry[0].as_str(),
                    entry[1].as_str(),
                    entry[2].as_str(),
                )?;
            }
        }
        Some(PendingRequest::ListThreads {
            team_uuid,
            channel_uuid,
        }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let channel_cstr = CString::new(channel_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "channel UUID contains null byte",
                    )
                })?;
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.first().map(|t| t.as_str()) != Some("THREADS") || (tokens.len() - 1) % 5 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid THREADS response payload",
                ));
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
        }
        Some(PendingRequest::ListReplies {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let thread_cstr = CString::new(thread_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "thread UUID contains null byte",
                    )
                })?;
                let channel_cstr = CString::new(channel_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "channel UUID contains null byte",
                    )
                })?;
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_thread(thread_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.first().map(|t| t.as_str()) != Some("REPLIES") || (tokens.len() - 1) % 3 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid REPLIES response payload",
                ));
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
        }
        Some(PendingRequest::InfoUser) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("USER") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid USER response payload",
                ));
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
        }
        Some(PendingRequest::InfoTeam { team_uuid }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("TEAM") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid TEAM response payload",
                ));
            }

            invoke_team_print(
                libcli::client_print_team,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
            )?;
        }
        Some(PendingRequest::InfoChannel {
            team_uuid,
            channel_uuid,
        }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let channel_cstr = CString::new(channel_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "channel UUID contains null byte",
                    )
                })?;
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 4 || tokens.first().map(|t| t.as_str()) != Some("CHANNEL") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid CHANNEL response payload",
                ));
            }

            invoke_channel_print(
                libcli::client_print_channel,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
            )?;
        }
        Some(PendingRequest::InfoThread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            if code == 401 {
                unsafe {
                    let _ = libcli::client_error_unauthorized();
                }
                return Ok(());
            }

            if code == 404 {
                let thread_cstr = CString::new(thread_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "thread UUID contains null byte",
                    )
                })?;
                let channel_cstr = CString::new(channel_uuid.as_str()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "channel UUID contains null byte",
                    )
                })?;
                let team_cstr = CString::new(team_uuid.as_str()).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "team UUID contains null byte")
                })?;
                unsafe {
                    let _ = libcli::client_error_unknown_thread(thread_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
                    let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
                }
                return Ok(());
            }

            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }

            let tokens = parse_response_tokens(response)?;
            if tokens.len() != 6 || tokens.first().map(|t| t.as_str()) != Some("THREAD") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid THREAD response payload",
                ));
            }

            invoke_thread_print(
                libcli::client_print_thread,
                tokens[1].as_str(),
                tokens[2].as_str(),
                tokens[3].as_str(),
                tokens[4].as_str(),
                tokens[5].as_str(),
            )?;
        }
        None => {
            if code != 200 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    response.to_string(),
                ));
            }
        }
    }

    Ok(())
}
