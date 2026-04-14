use std::ffi::CString;
use std::io;
use std::os::raw::c_int;

use crate::libcli;

pub(super) fn parse_status(status: &str) -> io::Result<c_int> {
    match status {
        "0" => Ok(0),
        "1" => Ok(1),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid user status: {}", status),
        )),
    }
}

pub(super) fn parse_timestamp(timestamp: &str) -> io::Result<libcli::TimeT> {
    timestamp.parse::<libcli::TimeT>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid message timestamp: {}", timestamp),
        )
    })
}

pub(super) fn cstring(value: &str, field: &str) -> io::Result<CString> {
    CString::new(value).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} contains an invalid NUL byte", field),
        )
    })
}

pub(super) fn invoke_team_print(
    callback: unsafe extern "C" fn(
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
    ) -> c_int,
    team_uuid: &str,
    team_name: &str,
    team_description: &str,
) -> io::Result<()> {
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

pub(super) fn invoke_channel_print(
    callback: unsafe extern "C" fn(
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
        *const std::os::raw::c_char,
    ) -> c_int,
    channel_uuid: &str,
    channel_name: &str,
    channel_description: &str,
) -> io::Result<()> {
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

pub(super) fn invoke_thread_print(
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
) -> io::Result<()> {
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

pub(super) fn invoke_reply_print(
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
) -> io::Result<()> {
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

pub(super) fn invalid_response(response: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, response.to_string())
}

pub(super) fn invalid_payload(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn handle_unauthorized() {
    unsafe {
        let _ = libcli::client_error_unauthorized();
    }
}

pub(super) fn handle_unknown_user(user_uuid: &str) -> io::Result<()> {
    let target_cstr = CString::new(user_uuid).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "target user UUID contains an invalid NUL byte",
        )
    })?;
    unsafe {
        let _ = libcli::client_error_unknown_user(target_cstr.as_ptr());
    }
    Ok(())
}

pub(super) fn handle_unknown_team(team_uuid: &str) -> io::Result<()> {
    let team_cstr = CString::new(team_uuid).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "team UUID contains an invalid NUL byte",
        )
    })?;
    unsafe {
        let _ = libcli::client_error_unknown_team(team_cstr.as_ptr());
    }
    Ok(())
}

pub(super) fn handle_unknown_channel(channel_uuid: &str) -> io::Result<()> {
    let channel_cstr = CString::new(channel_uuid).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "channel UUID contains an invalid NUL byte",
        )
    })?;
    unsafe {
        let _ = libcli::client_error_unknown_channel(channel_cstr.as_ptr());
    }
    Ok(())
}

pub(super) fn handle_unknown_thread(thread_uuid: &str) -> io::Result<()> {
    let thread_cstr = CString::new(thread_uuid).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "thread UUID contains an invalid NUL byte",
        )
    })?;
    unsafe {
        let _ = libcli::client_error_unknown_thread(thread_cstr.as_ptr());
    }
    Ok(())
}
