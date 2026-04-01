use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::Ordering;

use super::commands::{command_registry, emit_user_logged_out, SessionState};
use super::protocol::{parse_request_line, response};
use super::signal::SHOULD_STOP;
use super::storage::{default_teams_path, default_users_path, ServerStorage};
use super::users::UserStore;

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
) -> String {
    let parsed = match parse_request_line(line) {
        Ok(parsed) => parsed,
        Err(_) => return response(501, Some("\"bad request\"")),
    };

    if session.state.user_uuid.is_none() && parsed.name != "LOGIN" {
        return response(401, Some("\"unauthorized\""));
    }

    match commands.get(parsed.name.as_str()) {
        Some(definition) => {
            (definition.handler)(&mut session.state, commands, users, storage, &parsed.args)
        }
        None => response(404, Some("\"not found\"")),
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

fn handle_client(
    session: &mut ClientSession,
    commands: &super::commands::CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
) -> bool {
    let mut buf = [0u8; 1024];
    match session.stream.read(&mut buf) {
        Ok(0) => {
            if let Some(user_uuid) = session.state.user_uuid.as_deref() {
                emit_user_logged_out(user_uuid);
            }
            session.state.user_uuid = None;
            false
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

                let reply = process_line(session, commands, users, storage, &line);
                if !send_response(session, &reply) {
                    return false;
                }
            }

            true
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => true,
        Err(err) => {
            eprintln!("Client read error for {}: {}", session.peer, err);
            false
        }
    }
}

pub fn run_accept_loop(listener: &TcpListener) {
    let mut clients: Vec<ClientSession> = Vec::new();
    let commands = command_registry();
    let mut storage =
        match ServerStorage::load_or_default(default_users_path(), default_teams_path()) {
            Ok(storage) => storage,
            Err(err) => {
                eprintln!("Failed to initialize JSON storage: {}", err);
                std::process::exit(1);
            }
        };

    let mut users = UserStore::from_pairs(storage.user_pairs());
    println!(
        "Using JSON storage files: users={}, teams={}",
        storage.users_file().display(),
        storage.teams_file().display()
    );

    while !SHOULD_STOP.load(Ordering::SeqCst) {
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
            Err(err) if err.kind() == ErrorKind::WouldBlock => {}
            Err(err) => {
                eprintln!("Connection error: {}", err);
            }
        }

        clients.retain_mut(|session| handle_client(session, &commands, &mut users, &mut storage));

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    for session in clients {
        if let Some(user_uuid) = session.state.user_uuid.as_deref() {
            emit_user_logged_out(user_uuid);
        }
        let _ = session.stream.shutdown(Shutdown::Both);
    }
}
