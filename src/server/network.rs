use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::atomic::Ordering;
use std::ffi::CString;

use super::commands::{
    command_registry, dispatch_line, emit_user_logged_out, InfoEvent, SessionState,
};
use super::libsrv;
use super::notifier;
use super::signal::SHOULD_STOP;
use super::storage::{
    default_messages_path, default_teams_path, default_users_path, ServerStorage,
};
use super::transport::{read_lines_nonblocking, write_nonblocking, ReadLinesResult};
use super::users::UserStore;
use crate::poll::{wait as poll_wait, PollFd, POLLERR, POLLHUP, POLLIN, POLLNVAL};

pub fn create_listener(port_arg: &str, port: u16) -> TcpListener {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind {}: {}", port_arg, err);
            std::process::exit(1);
        }
    }
}

pub fn configure_listener(listener: &TcpListener) {
    if let Err(err) = listener.set_nonblocking(true) {
        eprintln!("Failed to configure server socket: {}", err);
        std::process::exit(1);
    }
}

struct ClientSession {
    stream: TcpStream,
    peer: std::net::SocketAddr,
    input_buffer: String,
    state: SessionState,
}

fn send_response(session: &mut ClientSession, payload: &str) -> bool {
    match write_nonblocking(&mut session.stream, payload) {
        Ok(_) => true,
        Err(err) => {
            eprintln!("Client write error for {}: {}", session.peer, err);
            false
        }
    }
}

fn emit_loaded_users(users: &UserStore) {
    for (user_uuid, user_name, _is_online) in users.list_users() {
        let Ok(uuid_cstr) = CString::new(user_uuid) else {
            continue;
        };
        let Ok(name_cstr) = CString::new(user_name) else {
            continue;
        };

        unsafe {
            let _ = libsrv::server_event_user_loaded(uuid_cstr.as_ptr(), name_cstr.as_ptr());
        }
    }
}

fn accept_pending_clients(listener: &TcpListener, clients: &mut Vec<ClientSession>) {
    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                if let Err(err) = stream.set_nonblocking(true) {
                    eprintln!("Failed to set non-blocking on client socket: {}", err);
                } else {
                    clients.push(ClientSession {
                        stream,
                        peer,
                        input_buffer: String::new(),
                        state: SessionState::default(),
                    });
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => break,
            Err(err) => {
                eprintln!("Connection error: {}", err);
                break;
            }
        }
    }
}

fn handle_client(
    session: &mut ClientSession,
    commands: &super::commands::CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
) -> (bool, Vec<InfoEvent>) {
    let mut pending_info_events = Vec::new();
    match read_lines_nonblocking(&mut session.stream, &mut session.input_buffer) {
        Ok(ReadLinesResult::Disconnected) => {
            if let Some(user_uuid) = session.state.user_uuid.as_deref() {
                users.logout(user_uuid);
                emit_user_logged_out(user_uuid);
            }
            session.state.user_uuid = None;
            (false, pending_info_events)
        }
        Ok(ReadLinesResult::Lines(lines)) => {
            for line in lines {
                let outcome = dispatch_line(&mut session.state, commands, users, storage, &line);
                if !send_response(session, &outcome.response) {
                    return (false, pending_info_events);
                }
                pending_info_events.extend(outcome.info_events);
            }

            (true, pending_info_events)
        }
        Ok(ReadLinesResult::WouldBlock) => (true, pending_info_events),
        Err(err) => {
            eprintln!("Client read error for {}: {}", session.peer, err);
            (false, pending_info_events)
        }
    }
}

pub fn run_accept_loop(listener: &TcpListener) {
    let mut clients: Vec<ClientSession> = Vec::new();
    let commands = command_registry();
    let mut storage = match ServerStorage::load_or_default(
        default_users_path(),
        default_teams_path(),
        default_messages_path(),
    ) {
            Ok(storage) => storage,
            Err(err) => {
                eprintln!("Failed to initialize JSON storage: {}", err);
                std::process::exit(1);
            }
    };

    let mut users = UserStore::from_pairs(storage.user_pairs());
    emit_loaded_users(&users);
    println!(
        "Using JSON storage files: users={}, teams={}, messages={}",
        storage.users_file().display(),
        storage.teams_file().display(),
        storage.messages_file().display()
    );

    while !SHOULD_STOP.load(Ordering::SeqCst) {
        let active_clients = std::mem::take(&mut clients);
        let mut poll_fds = Vec::with_capacity(active_clients.len() + 1);
        poll_fds.push(PollFd::new(listener.as_raw_fd(), POLLIN));
        for session in &active_clients {
            poll_fds.push(PollFd::new(session.stream.as_raw_fd(), POLLIN));
        }

        match poll_wait(&mut poll_fds, -1) {
            Ok(_) => {}
            Err(err) => {
                if err.kind() == ErrorKind::Interrupted {
                    continue;
                }

                eprintln!("poll error: {}", err);
                break;
            }
        }

        if poll_fds[0].revents & POLLIN != 0 {
            accept_pending_clients(listener, &mut clients);
        }

        let mut pending_info_events = Vec::new();
        let mut next_clients = Vec::with_capacity(active_clients.len() + clients.len());

        for (index, mut session) in active_clients.into_iter().enumerate() {
            let revents = poll_fds[index + 1].revents;
            if revents & (POLLERR | POLLHUP | POLLNVAL) != 0 {
                if let Some(user_uuid) = session.state.user_uuid.as_deref() {
                    users.logout(user_uuid);
                    emit_user_logged_out(user_uuid);
                }
                let _ = session.stream.shutdown(Shutdown::Both);
                continue;
            }

            let mut keep = true;
            if revents & POLLIN != 0 {
                let (still_keep, events) =
                    handle_client(&mut session, &commands, &mut users, &mut storage);
                pending_info_events.extend(events);
                keep = still_keep;
            }

            if keep {
                next_clients.push(session);
            }
        }

        next_clients.append(&mut clients);
        clients = next_clients;

        notifier::dispatch_info_events(
            &mut clients,
            &pending_info_events,
            |session| session.state.user_uuid.as_deref(),
            |session, payload| send_response(session, payload),
        );
    }

    for session in clients {
        if let Some(user_uuid) = session.state.user_uuid.as_deref() {
            users.logout(user_uuid);
            emit_user_logged_out(user_uuid);
        }
        let _ = session.stream.shutdown(Shutdown::Both);
    }
}
