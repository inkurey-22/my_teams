mod helpers;

use crate::commands::{
    command_registry, dispatch_slash_command, read_server_response_line, write_raw_command,
    CommandMap, ShellState,
};
use std::io;
use std::net::{Shutdown, TcpStream};

fn handle_input_line(
    state: &mut ShellState,
    registry: &CommandMap,
    stream: &mut TcpStream,
    line: &str,
) -> io::Result<bool> {
    let command = line.trim_end_matches(['\r', '\n']);
    if command == "exit" || command == "quit" {
        return Ok(false);
    }

    if command.is_empty() {
        return Ok(true);
    }

    match dispatch_slash_command(state, registry, stream, command) {
        Ok(true) => {}
        Ok(false) => {
            write_raw_command(stream, command)?;
            println!("{}", read_server_response_line(stream)?);
        }
        Err(err) => {
            eprintln!("command error: {}", err);
        }
    }

    Ok(true)
}

pub fn run_shell(stream: &mut TcpStream) {
    let stdin = io::stdin();
    let mut state = ShellState::default();
    let registry = command_registry();

    println!("Type commands to send to the server. Use 'exit' or 'quit' to disconnect.");
    if let Err(err) = helpers::print_prompt() {
        eprintln!("failed to flush prompt: {}", err);
        return;
    }

    loop {
        let (stdin_ready, socket_ready) = match helpers::wait_for_input_events(stream) {
            Ok(ready) => ready,
            Err(err) => {
                eprintln!("failed while waiting for input: {}", err);
                break;
            }
        };

        if socket_ready {
            if let Err(err) = helpers::drain_pending_server_infos(stream) {
                eprintln!("failed to receive server info: {}", err);
                break;
            }
            if let Err(err) = helpers::print_prompt() {
                eprintln!("failed to flush prompt: {}", err);
                break;
            }
        }

        if !stdin_ready {
            continue;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                match handle_input_line(&mut state, &registry, stream, &line) {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(err) => {
                        eprintln!("failed to process input: {}", err);
                        break;
                    }
                }

                if let Err(err) = helpers::print_prompt() {
                    eprintln!("failed to flush prompt: {}", err);
                    break;
                }
            }
            Err(err) => {
                eprintln!("failed to read input: {}", err);
                break;
            }
        }
    }

    if let Err(err) = stream.shutdown(Shutdown::Both) {
        eprintln!("failed to disconnect cleanly: {}", err);
    } else {
        println!("disconnected");
    }
}
