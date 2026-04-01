use std::env;

mod cli;
mod commands;
mod libsrv;
mod network;
mod protocol;
mod signal;
mod storage;
mod users;

use cli::parse_port_arg;
use network::{configure_listener, create_listener, run_accept_loop};
use signal::install_sigint_handler;

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
