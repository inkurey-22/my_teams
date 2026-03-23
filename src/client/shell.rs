use std::io::{self, Write};
use std::net::{Shutdown, TcpStream};

pub fn run_shell(stream: &mut TcpStream) {
    let stdin = io::stdin();

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

                let payload = format!("{}\n", command);
                if let Err(err) = stream.write_all(payload.as_bytes()) {
                    eprintln!("failed to send command: {}", err);
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
