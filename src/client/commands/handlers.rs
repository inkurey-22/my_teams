use std::io::{self, Write};
use std::net::TcpStream;

use crate::commands::protocol::{
    build_login_request, build_logout_request, build_messages_request, build_send_request,
    build_user_request, build_users_request,
};
use crate::commands::{CommandMap, PendingRequest, SessionState};

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
    _state: &mut SessionState,
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
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/login", args, 1, 1)?;

    let user_name = &args[0];
    let request = build_login_request(user_name);
    stream.write_all(request.as_bytes())?;

    state.pending_request = Some(PendingRequest::Login {
        user_name: user_name.clone(),
    });

    Ok(())
}

pub fn handle_logout(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/logout", args, 0, 0)?;

    let request = build_logout_request();
    stream.write_all(request.as_bytes())?;

    state.pending_request = Some(PendingRequest::Logout);

    Ok(())
}

pub fn handle_users(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/users", args, 0, 0)?;

    let request = build_users_request();
    stream.write_all(request.as_bytes())?;

    state.pending_request = Some(PendingRequest::Users);
    Ok(())
}

pub fn handle_user(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/user", args, 1, 1)?;

    let user_uuid = &args[0];
    let request = build_user_request(user_uuid);
    stream.write_all(request.as_bytes())?;

    state.pending_request = Some(PendingRequest::User {
        user_uuid: user_uuid.clone(),
    });
    Ok(())
}

pub fn handle_send(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/send", args, 2, 2)?;

    let user_uuid = &args[0];
    let message_body = &args[1];
    let request = build_send_request(user_uuid, message_body);
    stream.write_all(request.as_bytes())?;

    state.pending_request = Some(PendingRequest::Send {
        user_uuid: user_uuid.clone(),
        message_body: message_body.clone(),
    });

    Ok(())
}

pub fn handle_messages(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/messages", args, 1, 1)?;

    let user_uuid = &args[0];
    let request = build_messages_request(user_uuid);
    stream.write_all(request.as_bytes())?;

    state.pending_request = Some(PendingRequest::Messages {
        user_uuid: user_uuid.clone(),
    });

    Ok(())
}

pub fn handle_subscribe(
    _state: &mut SessionState,
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
    _state: &mut SessionState,
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
    _state: &mut SessionState,
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
    state: &mut SessionState,
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
    _state: &mut SessionState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/create", args, 0, 0)?;
    // TODO: create resource based on current context.
    Ok(())
}

pub fn handle_list(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/list", args, 0, 0)?;
    // TODO: list resources based on current context.
    Ok(())
}

pub fn handle_info(
    _state: &mut SessionState,
    _registry: &CommandMap,
    _stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/info", args, 0, 0)?;
    // TODO: show current resource info from context.
    Ok(())
}
