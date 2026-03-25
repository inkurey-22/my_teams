use crate::commands::{command_registry, dispatch_slash_command, write_raw_command, ShellState};
use std::io::{self, Write};
use std::net::{Shutdown, TcpStream};

pub fn run_shell(stream: &mut TcpStream) {
    let stdin = io::stdin();
    let mut state = ShellState::default();
    let registry = command_registry();

    println!("Type commands to send to the server. Use 'exit' or 'quit' to disconnect.");

    loop {
        print!("myteams > ");
        if let Err(err) = io::stdout().flush() {
            eprintln!("failed to flush prompt: {}", err);
            break;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let command = line.trim_end_matches(['\r', '\n']);
                if command == "exit" || command == "quit" {
                    break;
                }

                if command.is_empty() {
                    continue;
                }

                match dispatch_slash_command(&mut state, &registry, stream, command) {
                    Ok(true) => {}
                    Ok(false) => {
                        if let Err(err) = write_raw_command(stream, command) {
                            eprintln!("failed to send command: {}", err);
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("command error: {}", err);
                    }
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
