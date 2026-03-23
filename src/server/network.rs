use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::Ordering;

use super::signal::SHOULD_STOP;

pub fn create_listener(port_arg: &str, port: u16) -> TcpListener {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind {}: {}", port_arg, err);
            std::process::exit(1);
        }
    }
}

pub fn configure_listener(listener: &TcpListener) {
    if let Err(err) = listener.set_nonblocking(true) {
        eprintln!("Failed to configure server socket: {}", err);
        std::process::exit(1);
    }
}

fn handle_client(stream: &mut TcpStream, peer: std::net::SocketAddr) -> bool {
    let mut buf = [0; 1024];
    match stream.peek(&mut buf) {
        Ok(0) => {
            println!("Client disconnected: {}", peer);
            return false;
        }
        Ok(n) => {
            let mut read_buf = vec![0; n];
            match std::io::Read::read(stream, &mut read_buf) {
                Ok(0) => {
                    println!("Client disconnected: {}", peer);
                    return false;
                }
                Ok(_) => {
                    if let Ok(text) = String::from_utf8(read_buf) {
                        for line in text.lines() {
                            if !line.is_empty() {
                                println!("{} > {}", peer, line);
                            }
                        }
                    }
                    return true;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    return true;
                }
                Err(err) => {
                    eprintln!("Client read error for {}: {}", peer, err);
                    return false;
                }
            }
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => {
            return true;
        }
        Err(err) => {
            eprintln!("Client peek error for {}: {}", peer, err);
            return false;
        }
    }
}

pub fn run_accept_loop(listener: &TcpListener) {
    let mut clients: Vec<(TcpStream, std::net::SocketAddr)> = Vec::new();

    while !SHOULD_STOP.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, peer)) => {
                if let Err(err) = stream.set_nonblocking(true) {
                    eprintln!("Failed to set non-blocking on client socket: {}", err);
                } else {
                    println!("Client connected: {}", peer);
                    clients.push((stream, peer));
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {}
            Err(err) => {
                eprintln!("Connection error: {}", err);
            }
        }

        clients.retain_mut(|(stream, peer)| handle_client(stream, *peer));

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    for (stream, peer) in clients {
        let _ = stream.shutdown(Shutdown::Both);
        println!("Closed connection: {}", peer);
    }
}
