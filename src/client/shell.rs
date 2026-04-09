use crate::commands::{
    command_registry, dispatch_line, handle_response_line, parse_new_message_info,
    write_request_line, SessionState,
};
use crate::libcli;
use crate::poll::{wait as poll_wait, PollFd, POLLERR, POLLHUP, POLLIN, POLLNVAL};
use std::collections::VecDeque;
use std::ffi::CString;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::os::fd::AsRawFd;

fn print_prompt() -> io::Result<()> {
    print!("myteams > ");
    io::stdout().flush()
}

fn handle_info_message(line: &str) {
    if let Ok(Some((sender_uuid, message_body))) = parse_new_message_info(line) {
        let Ok(sender_cstr) = CString::new(sender_uuid) else {
            return;
        };
        let Ok(message_cstr) = CString::new(message_body) else {
            return;
        };

        unsafe {
            let _ = libcli::client_event_private_message_received(
                sender_cstr.as_ptr(),
                message_cstr.as_ptr(),
            );
        }
    }
}

fn read_socket_messages(stream: &mut TcpStream, buffer: &mut String) -> io::Result<Vec<String>> {
    let mut messages = Vec::new();

    loop {
        let mut chunk = [0u8; 1024];
        match stream.read(&mut chunk) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "server closed connection",
                ))
            }
            Ok(n) => {
                buffer.push_str(String::from_utf8_lossy(&chunk[..n]).as_ref());

                while let Some(newline_idx) = buffer.find('\n') {
                    let line = buffer[..=newline_idx]
                        .trim_end_matches(['\r', '\n'])
                        .to_string();
                    buffer.drain(..=newline_idx);
                    if !line.is_empty() {
                        messages.push(line);
                    }
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(err) => return Err(err),
        }
    }

    Ok(messages)
}

fn process_socket_message(state: &mut SessionState, message: &str) {
    if message.starts_with('I') {
        println!("{}", message);
        handle_info_message(message);
        return;
    }

    if message.starts_with('R') {
        println!("{}", message);
        if let Err(err) = handle_response_line(state, message) {
            eprintln!("failed to handle server response: {}", err);
        }
        return;
    }
}

fn process_pending_input(
    state: &mut SessionState,
    registry: &crate::commands::CommandMap,
    stream: &mut TcpStream,
    queued_input: &mut VecDeque<String>,
) -> io::Result<()> {
    while state.pending_request.is_none() {
        let Some(command) = queued_input.pop_front() else {
            break;
        };

        if command == "exit" || command == "quit" {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "quit"));
        }

        if command.is_empty() {
            continue;
        }

        match dispatch_line(state, registry, stream, &command) {
            Ok(true) => {
                if state.pending_request.is_none() {
                    print_prompt()?;
                }
            }
            Ok(false) => {
                if let Err(err) = write_request_line(stream, &command) {
                    return Err(err);
                }

                if state.pending_request.is_none() {
                    break;
                }
            }
            Err(err) => return Err(err),
        }
    }

    Ok(())
}

pub fn run_shell(stream: &mut TcpStream) {
    if let Err(err) = stream.set_nonblocking(true) {
        eprintln!("failed to configure client socket: {}", err);
        return;
    }

    let stdin = io::stdin();
    let stdin_fd = stdin.as_raw_fd();
    let socket_fd = stream.as_raw_fd();
    let mut state = SessionState::default();
    let registry = command_registry();
    let mut socket_buffer = String::new();
    let mut queued_input = VecDeque::new();

    println!("Type commands to send to the server. Use 'exit' or 'quit' to disconnect.");
    if let Err(err) = print_prompt() {
        eprintln!("failed to flush prompt: {}", err);
        return;
    }

    loop {
        if let Err(err) = process_pending_input(&mut state, &registry, stream, &mut queued_input) {
            if err.kind() == io::ErrorKind::Interrupted {
                break;
            }

            eprintln!("command error: {}", err);
            break;
        }

        let mut poll_fds = [
            PollFd::new(stdin_fd, POLLIN),
            PollFd::new(socket_fd, POLLIN),
        ];

        match poll_wait(&mut poll_fds, -1) {
            Ok(_) => {}
            Err(err) => {
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }

                eprintln!("poll error: {}", err);
                break;
            }
        }

        let stdin_revents = poll_fds[0].revents;
        let socket_revents = poll_fds[1].revents;

        if socket_revents & (POLLERR | POLLHUP | POLLNVAL) != 0 {
            eprintln!("server closed connection");
            break;
        }

        if socket_revents & POLLIN != 0 {
            match read_socket_messages(stream, &mut socket_buffer) {
                Ok(messages) => {
                    for message in messages {
                        process_socket_message(&mut state, &message);
                        let _ = print_prompt();
                    }
                }
                Err(err) => {
                    eprintln!("failed to read from server: {}", err);
                    break;
                }
            }
        }

        if stdin_revents & POLLIN != 0 {
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let command = line.trim_end_matches(['\r', '\n']).to_string();
                    if command == "exit" || command == "quit" {
                        break;
                    }

                    if command.is_empty() {
                        let _ = print_prompt();
                        continue;
                    }

                    if state.pending_request.is_some() {
                        queued_input.push_back(command);
                        continue;
                    }

                    match dispatch_line(&mut state, &registry, stream, command.as_str()) {
                        Ok(true) => {
                            if state.pending_request.is_none() {
                                let _ = print_prompt();
                            }
                        }
                        Ok(false) => {
                            if let Err(err) = write_request_line(stream, command.as_str()) {
                                eprintln!("failed to send command: {}", err);
                                break;
                            }
                        }
                        Err(err) => {
                            eprintln!("command error: {}", err);
                            let _ = print_prompt();
                        }
                    }
                }
                Err(err) => {
                    eprintln!("failed to read input: {}", err);
                    break;
                }
            }
        }
    }

    if let Err(err) = stream.shutdown(Shutdown::Both) {
        eprintln!("failed to disconnect cleanly: {}", err);
    } else {
        println!("disconnected");
    }
}
