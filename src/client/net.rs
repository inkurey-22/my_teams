use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

/// Resolve a host and port to one or more socket addresses.
pub fn resolve_addresses(host: &str, port: u16) -> Vec<SocketAddr> {
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

/// Connect to the first reachable address in the supplied list.
pub fn connect_to_server(addrs: &[SocketAddr]) -> TcpStream {
    let mut last_error = None;

    for addr in addrs {
        match TcpStream::connect(addr) {
            Ok(stream) => {
                println!("connected to {}", addr);
                return stream;
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
