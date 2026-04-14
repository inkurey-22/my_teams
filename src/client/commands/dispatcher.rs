use std::io::{self, Write};
use std::net::TcpStream;

use crate::commands::{CommandMap, SessionState};

fn tokenize_command(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Write a request line to the server socket.
pub fn write_request_line(stream: &mut TcpStream, line: &str) -> io::Result<()> {
    let payload = format!("{}\r\n", line);
    stream.write_all(payload.as_bytes())
}

/// Parse one command line and route it through the client registry.
pub fn dispatch_line(
    state: &mut SessionState,
    commands: &CommandMap,
    stream: &mut TcpStream,
    line: &str,
) -> io::Result<bool> {
    if !line.starts_with('/') {
        return Ok(false);
    }

    let tokens = tokenize_command(line);
    if tokens.is_empty() {
        return Ok(true);
    }

    let command_name = tokens[0].as_str();
    let args = &tokens[1..];

    match commands.get(command_name) {
        Some(command) => {
            (command.handler)(state, commands, stream, args)?;
        }
        None => {
            eprintln!("unknown command: {}", command_name);
            eprintln!("type /help to list available commands");
        }
    }

    Ok(true)
}
