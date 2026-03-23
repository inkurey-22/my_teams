use std::env;
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};

fn print_usage() {
    println!("USAGE: ./myteams_cli ip port");
    println!("ip is the server ip address on which the server socket listens");
    println!("port is the port number on which the server socket listens");
}

fn parse_port(port_str: &str) -> Option<u16> {
    match port_str.parse::<u16>() {
        Ok(port) if port > 0 => Some(port),
        _ => None,
    }
}

fn parse_args(args: &[String]) -> Option<(String, u16)> {
    if args.len() == 2 && args[1] == "--help" {
        print_usage();
        return None;
    }

    if args.len() != 3 {
        print_usage();
        std::process::exit(1);
    }

    let host = args[1].clone();
    if host.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let port = match parse_port(&args[2]) {
        Some(port) => port,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    Some((host, port))
}

fn resolve_addresses(host: &str, port: u16) -> Vec<SocketAddr> {
    let resolved = match (host, port).to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(err) => {
            eprintln!("failed to resolve {}:{}: {}", host, port, err);
            std::process::exit(1);
        }
    };

    let addresses: Vec<SocketAddr> = resolved.collect();
    if addresses.is_empty() {
        eprintln!("failed to resolve {}:{}", host, port);
        std::process::exit(1);
    }

    addresses
}

fn connect_and_disconnect(addrs: &[SocketAddr]) {
    let mut last_error = None;

    for addr in addrs {
        match TcpStream::connect(addr) {
            Ok(stream) => {
                println!("connected to {}", addr);

                if let Err(err) = stream.shutdown(Shutdown::Both) {
                    eprintln!("failed to disconnect cleanly from {}: {}", addr, err);
                    std::process::exit(1);
                }

                println!("disconnected from {}", addr);
                return;
            }
            Err(err) => {
                last_error = Some((addr, err));
            }
        }
    }

    match last_error {
        Some((addr, err)) => {
            eprintln!("failed to connect to {}: {}", addr, err);
        }
        None => {
            eprintln!("failed to connect");
        }
    }

    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (host, port) = match parse_args(&args) {
        Some(values) => values,
        None => return,
    };
    let addresses = resolve_addresses(&host, port);
    connect_and_disconnect(&addresses);
}
