use std::io::{self, Write};
use std::net::TcpStream;

use crate::commands::{CommandMap, ShellState};

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

pub fn write_raw_command(stream: &mut TcpStream, command: &str) -> io::Result<()> {
    let payload = format!("{}\r\n", command);
    stream.write_all(payload.as_bytes())
}

pub fn dispatch_slash_command(
    state: &mut ShellState,
    registry: &CommandMap,
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

    match registry.get(command_name) {
        Some(command) => {
            (command.handler)(state, registry, stream, args)?;
        }
        None => {
            eprintln!("unknown command: {}", command_name);
            eprintln!("type /help to list available commands");
        }
    }

    Ok(true)
}
