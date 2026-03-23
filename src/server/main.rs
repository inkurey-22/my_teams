use std::env;
use std::io::ErrorKind;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
const SIGINT: i32 = 2;

unsafe extern "C" {
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
}

extern "C" fn handle_sigint(_signal: i32) {
    SHOULD_STOP.store(true, Ordering::SeqCst);
}

fn install_sigint_handler() {
    unsafe {
        let _ = signal(SIGINT, handle_sigint);
    }
}

fn print_usage() {
    println!("USAGE: ./myteams_server port");
    println!("port is the port number on which the server socket listens.");
}

fn parse_port_arg(args: &[String]) -> Option<(String, u16)> {
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

fn create_listener(port_arg: &str, port: u16) -> TcpListener {
    let addr = format!("0.0.0.0:{}", port);
    match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind {}: {}", port_arg, err);
            std::process::exit(1);
        }
    }
}

fn configure_listener(listener: &TcpListener) {
    if let Err(err) = listener.set_nonblocking(true) {
        eprintln!("Failed to configure server socket: {}", err);
        std::process::exit(1);
    }
}

fn handle_client(stream: TcpStream, peer: std::net::SocketAddr) {
    println!("Client connected: {}", peer);
    if let Err(err) = stream.shutdown(Shutdown::Both) {
        eprintln!("Failed to disconnect client {}: {}", peer, err);
    } else {
        println!("Client disconnected: {}", peer);
    }
}

fn run_accept_loop(listener: &TcpListener) {
    while !SHOULD_STOP.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, peer)) => handle_client(stream, peer),
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(err) => {
                eprintln!("Connection error: {}", err);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (port_arg, port) = match parse_port_arg(&args) {
        Some(values) => values,
        None => return,
    };
    let listener = create_listener(&port_arg, port);
    configure_listener(&listener);

    install_sigint_handler();

    println!("Server listening on {}", port_arg);
    run_accept_loop(&listener);

    println!("Shutting down...");
}
