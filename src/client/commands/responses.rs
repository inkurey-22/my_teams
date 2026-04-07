use std::ffi::CString;
use std::io;

use crate::commands::protocol::{extract_uuid_from_body, parse_response_code};
use crate::commands::{PendingRequest, SessionState};
use crate::libcli;

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
