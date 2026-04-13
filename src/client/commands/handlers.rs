use std::io::{self, Write};
use std::net::TcpStream;

use crate::commands::protocol::{
    build_create_channel_request, build_create_reply_request, build_create_team_request,
    build_create_thread_request, build_info_channel_request, build_info_team_request,
    build_info_thread_request, build_info_user_request, build_list_channels_request,
    build_list_replies_request, build_list_teams_request, build_list_threads_request,
    build_login_request, build_logout_request, build_messages_request, build_send_request,
    build_use_request, build_user_request, build_users_request,
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

enum ContextLevel {
    Root,
    Team {
        team_uuid: String,
    },
    Channel {
        team_uuid: String,
        channel_uuid: String,
    },
    Thread {
        team_uuid: String,
        channel_uuid: String,
        thread_uuid: String,
    },
}

fn current_context(state: &SessionState) -> io::Result<ContextLevel> {
    match (
        state.context.team_uuid.as_ref(),
        state.context.channel_uuid.as_ref(),
        state.context.thread_uuid.as_ref(),
    ) {
        (None, None, None) => Ok(ContextLevel::Root),
        (Some(team_uuid), None, None) => Ok(ContextLevel::Team {
            team_uuid: team_uuid.clone(),
        }),
        (Some(team_uuid), Some(channel_uuid), None) => Ok(ContextLevel::Channel {
            team_uuid: team_uuid.clone(),
            channel_uuid: channel_uuid.clone(),
        }),
        (Some(team_uuid), Some(channel_uuid), Some(thread_uuid)) => Ok(ContextLevel::Thread {
            team_uuid: team_uuid.clone(),
            channel_uuid: channel_uuid.clone(),
            thread_uuid: thread_uuid.clone(),
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid command context",
        )),
    }
}

fn send_request(stream: &mut TcpStream, request: String) -> io::Result<()> {
    stream.write_all(request.as_bytes())
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
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/use", args, 0, 3)?;

    send_request(stream, build_use_request(args))?;
    state.pending_request = Some(PendingRequest::Use {
        team_uuid: args.first().cloned(),
        channel_uuid: args.get(1).cloned(),
        thread_uuid: args.get(2).cloned(),
    });
    Ok(())
}

pub fn handle_create(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    match current_context(state)? {
        ContextLevel::Root => {
            check_arg_count("/create", args, 2, 2)?;
            send_request(stream, build_create_team_request(&args[0], &args[1]))?;
            state.pending_request = Some(PendingRequest::CreateTeam);
        }
        ContextLevel::Team { team_uuid } => {
            check_arg_count("/create", args, 2, 2)?;
            send_request(stream, build_create_channel_request(&args[0], &args[1]))?;
            state.pending_request = Some(PendingRequest::CreateChannel { team_uuid });
        }
        ContextLevel::Channel {
            team_uuid,
            channel_uuid,
        } => {
            check_arg_count("/create", args, 2, 2)?;
            send_request(stream, build_create_thread_request(&args[0], &args[1]))?;
            state.pending_request = Some(PendingRequest::CreateThread {
                team_uuid,
                channel_uuid,
            });
        }
        ContextLevel::Thread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        } => {
            check_arg_count("/create", args, 1, 1)?;
            send_request(stream, build_create_reply_request(&args[0]))?;
            state.pending_request = Some(PendingRequest::CreateReply {
                team_uuid,
                channel_uuid,
                thread_uuid,
            });
        }
    }
    Ok(())
}

pub fn handle_list(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/list", args, 0, 0)?;
    match current_context(state)? {
        ContextLevel::Root => {
            send_request(stream, build_list_teams_request())?;
            state.pending_request = Some(PendingRequest::ListTeams);
        }
        ContextLevel::Team { team_uuid } => {
            send_request(stream, build_list_channels_request())?;
            state.pending_request = Some(PendingRequest::ListChannels { team_uuid });
        }
        ContextLevel::Channel {
            team_uuid,
            channel_uuid,
        } => {
            send_request(stream, build_list_threads_request())?;
            state.pending_request = Some(PendingRequest::ListThreads {
                team_uuid,
                channel_uuid,
            });
        }
        ContextLevel::Thread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        } => {
            send_request(stream, build_list_replies_request())?;
            state.pending_request = Some(PendingRequest::ListReplies {
                team_uuid,
                channel_uuid,
                thread_uuid,
            });
        }
    }
    Ok(())
}

pub fn handle_info(
    state: &mut SessionState,
    _registry: &CommandMap,
    stream: &mut TcpStream,
    args: &[String],
) -> io::Result<()> {
    check_arg_count("/info", args, 0, 0)?;
    match current_context(state)? {
        ContextLevel::Root => {
            send_request(stream, build_info_user_request())?;
            state.pending_request = Some(PendingRequest::InfoUser);
        }
        ContextLevel::Team { team_uuid } => {
            send_request(stream, build_info_team_request())?;
            state.pending_request = Some(PendingRequest::InfoTeam { team_uuid });
        }
        ContextLevel::Channel {
            team_uuid,
            channel_uuid,
        } => {
            send_request(stream, build_info_channel_request())?;
            state.pending_request = Some(PendingRequest::InfoChannel {
                team_uuid,
                channel_uuid,
            });
        }
        ContextLevel::Thread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        } => {
            send_request(stream, build_info_thread_request())?;
            state.pending_request = Some(PendingRequest::InfoThread {
                team_uuid,
                channel_uuid,
                thread_uuid,
            });
        }
    }
    Ok(())
}
