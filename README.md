# my_teams

School project. Have to create a custom RFC for team communication protocol.
Server and Client are written in Rust.

No multithreading, no fork, no async. Only select() for multiplexing.
No crate, only `std::net` for socket programming.
