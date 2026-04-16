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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_port_arg_with_valid_port() {
        let args = vec!["server".to_string(), "8080".to_string()];
        let result = parse_port_arg(&args);
        assert_eq!(result, Some(("8080".to_string(), 8080)));
    }

    #[test]
    fn parse_port_arg_with_port_1() {
        let args = vec!["server".to_string(), "1".to_string()];
        let result = parse_port_arg(&args);
        assert_eq!(result, Some(("1".to_string(), 1)));
    }

    #[test]
    fn parse_port_arg_with_max_port() {
        let args = vec!["server".to_string(), "65535".to_string()];
        let result = parse_port_arg(&args);
        assert_eq!(result, Some(("65535".to_string(), 65535)));
    }

    #[test]
    fn parse_port_arg_with_help_flag() {
        let args = vec!["server".to_string(), "--help".to_string()];
        let result = parse_port_arg(&args);
        assert_eq!(result, None);
    }
}
