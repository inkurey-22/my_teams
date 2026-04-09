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
