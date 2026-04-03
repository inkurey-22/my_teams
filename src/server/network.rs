mod helpers;

use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::Ordering;

use super::commands::{command_registry, emit_user_logged_out, SessionState};
use super::protocol::{parse_request_line, response};
use super::signal::SHOULD_STOP;
use super::storage::{default_teams_path, default_users_path, ServerStorage};
use super::users::UserStore;
use helpers::{
    build_private_message_info_payload, collect_private_message_dispatch, PrivateMessageDispatch,
    ProcessResult,
};

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

fn process_line(
    session: &mut ClientSession,
    commands: &super::commands::CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    line: &str,
) -> ProcessResult {
    let parsed = match parse_request_line(line) {
        Ok(parsed) => parsed,
        Err(_) => {
            return ProcessResult {
                reply: response(501, Some("\"bad request\"")),
                private_message: None,
            }
        }
    };

    if session.state.user_uuid.is_none() && parsed.name != "LOGIN" {
        return ProcessResult {
            reply: response(401, Some("\"unauthorized\"")),
            private_message: None,
        };
    }

    let reply = match commands.get(parsed.name.as_str()) {
        Some(definition) => {
            (definition.handler)(&mut session.state, commands, users, storage, &parsed.args)
        }
        None => response(404, Some("\"not found\"")),
    };

    let private_message = collect_private_message_dispatch(
        parsed.name.as_str(),
        &reply,
        session.state.user_uuid.as_deref(),
        &parsed.args,
    );

    ProcessResult {
        reply,
        private_message,
    }
}

fn send_response(session: &mut ClientSession, payload: &str) -> bool {
    match session.stream.write_all(payload.as_bytes()) {
        Ok(_) => true,
        Err(err) if err.kind() == ErrorKind::WouldBlock => true,
        Err(err) => {
            eprintln!("Client write error for {}: {}", session.peer, err);
            false
        }
    }
}

fn dispatch_private_messages(
    clients: &mut [ClientSession],
    pending_private_messages: Vec<PrivateMessageDispatch>,
) {
    for private_message in pending_private_messages {
        let payload = build_private_message_info_payload(
            private_message.sender_uuid.as_str(),
            private_message.message_body.as_str(),
        );

        for client in clients.iter_mut() {
            if client.state.user_uuid.as_deref() == Some(private_message.receiver_uuid.as_str()) {
                let _ = send_response(client, &payload);
            }
        }
    }
}

fn handle_client(
    session: &mut ClientSession,
    commands: &super::commands::CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
) -> (bool, Vec<PrivateMessageDispatch>) {
    let mut dispatches = Vec::new();
    let mut buf = [0u8; 1024];
    match session.stream.read(&mut buf) {
        Ok(0) => {
            if let Some(user_uuid) = session.state.user_uuid.as_deref() {
                emit_user_logged_out(user_uuid);
            }
            session.state.user_uuid = None;
            (false, dispatches)
        }
        Ok(n) => {
            session
                .input_buffer
                .push_str(String::from_utf8_lossy(&buf[..n]).as_ref());

            while let Some(newline_idx) = session.input_buffer.find('\n') {
                let line = session.input_buffer[..=newline_idx]
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                session.input_buffer.drain(..=newline_idx);
                if line.is_empty() {
                    continue;
                }

                let result = process_line(session, commands, users, storage, &line);
                if !send_response(session, &result.reply) {
                    return (false, dispatches);
                }

                if let Some(private_message) = result.private_message {
                    dispatches.push(private_message);
                }
            }

            (true, dispatches)
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => (true, dispatches),
        Err(err) => {
            eprintln!("Client read error for {}: {}", session.peer, err);
            (false, dispatches)
        }
    }
}

fn load_storage_and_users() -> (ServerStorage, UserStore) {
    let storage = match ServerStorage::load_or_default(default_users_path(), default_teams_path()) {
        Ok(storage) => storage,
        Err(err) => {
            eprintln!("Failed to initialize JSON storage: {}", err);
            std::process::exit(1);
        }
    };

    let users = UserStore::from_pairs(storage.user_pairs());
    println!(
        "Using JSON storage files: users={}, teams={}",
        storage.users_file().display(),
        storage.teams_file().display()
    );

    (storage, users)
}

fn accept_pending_clients(listener: &TcpListener, clients: &mut Vec<ClientSession>) {
    match listener.accept() {
        Ok((stream, peer)) => {
            if let Err(err) = stream.set_nonblocking(true) {
                eprintln!("Failed to set non-blocking on client socket: {}", err);
                return;
            }

            clients.push(ClientSession {
                stream,
                peer,
                input_buffer: String::new(),
                state: SessionState::default(),
            });
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => {}
        Err(err) => {
            eprintln!("Connection error: {}", err);
        }
    }
}

fn process_clients_tick(
    clients: &mut Vec<ClientSession>,
    commands: &super::commands::CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
) {
    let mut pending_private_messages = Vec::new();

    clients.retain_mut(|session| {
        let (keep_alive, dispatches) = handle_client(session, commands, users, storage);
        pending_private_messages.extend(dispatches);
        keep_alive
    });

    dispatch_private_messages(clients, pending_private_messages);
}

fn shutdown_clients(clients: Vec<ClientSession>) {
    for session in clients {
        if let Some(user_uuid) = session.state.user_uuid.as_deref() {
            emit_user_logged_out(user_uuid);
        }
        let _ = session.stream.shutdown(Shutdown::Both);
    }
}

pub fn run_accept_loop(listener: &TcpListener) {
    let mut clients: Vec<ClientSession> = Vec::new();
    let commands = command_registry();
    let (mut storage, mut users) = load_storage_and_users();

    while !SHOULD_STOP.load(Ordering::SeqCst) {
        accept_pending_clients(listener, &mut clients);
        process_clients_tick(&mut clients, &commands, &mut users, &mut storage);

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    shutdown_clients(clients);
}
