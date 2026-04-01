use std::env;

mod commands;
mod libcli;
mod net;
mod shell;

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

fn main() {
    let args: Vec<String> = env::args().collect();
    let (host, port) = match parse_args(&args) {
        Some(values) => values,
        None => return,
    };
    let addresses = net::resolve_addresses(&host, port);
    let mut stream = net::connect_to_server(&addresses);
    shell::run_shell(&mut stream);
}
