/// Print usage for the server CLI.
fn print_usage() {
    println!("USAGE: ./myteams_server port");
    println!("port is the port number on which the server socket listens.");
}

/// Parse the server command-line arguments and extract the listening port.
pub fn parse_port_arg(args: &[String]) -> Option<(String, u16)> {
    if args.len() == 2 && args[1] == "--help" {
        print_usage();
        return None;
    }

    if args.len() != 2 {
        print_usage();
        std::process::exit(1);
    }

    let port_arg = args[1].clone();
    let port = match port_arg.parse::<u16>() {
        Ok(port) if port > 0 => port,
        _ => {
            print_usage();
            std::process::exit(1);
        }
    };

    Some((port_arg, port))
}
