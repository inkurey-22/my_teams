use std::ffi::CString;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

use crate::commands::protocol::{build_login_request, extract_uuid_from_body, parse_response_code};
use crate::commands::{CommandMap, ShellState};
use crate::libcli;

fn check_arg_count(command: &str, args: &[String], min: usize, max: usize) -> io::Result<()> {
    if args.len() < min || args.len() > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid argument count for {}", command),
        ));
    }
    Ok(())
}

pub fn handle_help(
    _state: &mut ShellState,
    registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/help", args, 0, 0)?;

    let mut commands: Vec<_> = registry.iter().collect();
    commands.sort_by_key(|(name, _)| *name);

    for (_name, definition) in commands {
        println!("{:<62} : {}", definition.usage, definition.description);
    }

    Ok(())
}

pub fn handle_login(
    state: &mut ShellState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/login", args, 1, 1)?;

    let user_name = &args[0];
    let request = build_login_request(user_name);
    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    let mut reader = BufReader::new(stream.try_clone()?);
    let bytes_read = reader.read_line(&mut response)?;
    if bytes_read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "server closed connection while waiting for login response",
        ));
    }

    let response = response.trim_end_matches(['\r', '\n']);
    let code = parse_response_code(response)?;

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

    let user_uuid_cstr = CString::new(user_uuid)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "user UUID contains null byte"))?;
    let user_name_cstr = CString::new(user_name.as_str()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "user name contains an invalid NUL byte",
        )
    })?;

    unsafe {
        let _ = libcli::client_event_logged_in(user_uuid_cstr.as_ptr(), user_name_cstr.as_ptr());
    }

    Ok(())
}

pub fn handle_logout(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/logout", args, 0, 0)?;
    // TODO: disconnect client session from server.
    Ok(())
}

pub fn handle_users(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/users", args, 0, 0)?;
    // TODO: request users list from server.
    Ok(())
}

pub fn handle_user(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/user", args, 1, 1)?;
    let _user_uuid = &args[0];
    // TODO: request details for the target user.
    Ok(())
}

pub fn handle_send(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/send", args, 2, 2)?;
    let _user_uuid = &args[0];
    let _message_body = &args[1];
    // TODO: send private message to user.
    Ok(())
}

pub fn handle_messages(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/messages", args, 1, 1)?;
    let _user_uuid = &args[0];
    // TODO: request message history with user.
    Ok(())
}

pub fn handle_subscribe(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/subscribe", args, 1, 1)?;
    let _team_uuid = &args[0];
    // TODO: subscribe to a team.
    Ok(())
}

pub fn handle_subscribed(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/subscribed", args, 0, 1)?;
    let _team_uuid = args.first();
    // TODO: list subscriptions or subscribers for a team.
    Ok(())
}

pub fn handle_unsubscribe(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/unsubscribe", args, 1, 1)?;
    let _team_uuid = &args[0];
    // TODO: unsubscribe from a team.
    Ok(())
}

pub fn handle_use(
    state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/use", args, 0, 3)?;

    state.context.team_uuid = args.first().cloned();
    state.context.channel_uuid = args.get(1).cloned();
    state.context.thread_uuid = args.get(2).cloned();

    // TODO: propagate selected context to server-side command execution.
    Ok(())
}

pub fn handle_create(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/create", args, 0, 0)?;
    // TODO: create resource based on current context.
    Ok(())
}

pub fn handle_list(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/list", args, 0, 0)?;
    // TODO: list resources based on current context.
    Ok(())
}

pub fn handle_info(
    _state: &mut ShellState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/info", args, 0, 0)?;
    // TODO: show current resource info from context.
    Ok(())
}
