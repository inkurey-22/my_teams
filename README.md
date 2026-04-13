# my_teams

Rust client/server implementation of a Team-like messaging service for an academic networking project.

The project includes:
- A TCP server: `myteams_server`
- A CLI client: `myteams_cli`
- A custom text protocol inspired by command/response patterns
- JSON-backed persistence for users, teams, and direct messages

## Subject compliance summary

This repository is designed to match the project subject requirements:
- Binaries are named `myteams_server` and `myteams_cli`
- Build is available through the Makefile (`all`, `clean`, `fclean`, `re`)
- Networking uses TCP from the Rust standard library (`std::net`)
- No external crates are used
- Server/client event and data display rely on the provided C logging library via Rust FFI
- Server is single-threaded and event-driven
- Persisted state is restored on startup and saved on shutdown (including Ctrl+C)

## Project constraints

The implementation follows strict project rules:
- No multithreading
- No `fork`
- No async runtime
- Multiplexing via `poll`/`select` style APIs
- Socket handling through Rust standard library networking
- Handle multiple simultaneous clients
- Correctly handle disconnections
- Correctly handle partial reads/writes and command buffering
- Keep server logic in a single-threaded event loop

## Repository layout

- `src/server/`: server runtime, command handling, protocol, persistence
- `src/client/`: interactive CLI shell, command dispatch, network I/O
- `src/json/`: in-house JSON read/write module used by the server
- `data/`: runtime JSON storage files (`users.json`, `teams.json`, `messages.json`)
- `libs/myteams/`: provided shared library (`libmyteams.so`) and C headers
- `STANDARDS.md`: protocol and command specification document

## Prerequisites

You need a Rust toolchain and a runtime path to `libmyteams.so`.

Recommended setup (Nix):

```bash
nix develop
```

The dev shell exports `LD_LIBRARY_PATH` to include `libs/myteams`.

If you do not use Nix, make sure these tools are available:
- `cargo`
- `rustc`

And export runtime library path before launching binaries:

```bash
export LD_LIBRARY_PATH="$PWD/libs/myteams:${LD_LIBRARY_PATH:-}"
```

## Build

Build release artifacts via Makefile:

```bash
make
```

This compiles in release mode and copies executables to project root:
- `./myteams_server`
- `./myteams_cli`

Debug build:

```bash
make debug
```

Rebuild from scratch:

```bash
make re
```

Or use Cargo directly:

```bash
cargo build --release
```

## Run

Start server:

```bash
./myteams_server 4242
```

Start client:

```bash
./myteams_cli 127.0.0.1 4242
```

Usage helpers:

```bash
./myteams_server --help
./myteams_cli --help
```

Inside the client shell, type commands or use `exit`/`quit` to disconnect.

## Supported CLI commands

The client understands the following commands:
- `/help`
- `/login "user_name"`
- `/logout`
- `/users`
- `/user "user_uuid"`
- `/send "user_uuid" "message_body"`
- `/messages "user_uuid"`
- `/subscribe "team_uuid"`
- `/subscribed ?"team_uuid"`
- `/unsubscribe "team_uuid"`
- `/use ?"team_uuid" ?"channel_uuid" ?"thread_uuid"`
- `/create`
- `/list`
- `/info`

Arguments must be wrapped in double quotes when required by command syntax. A missing quote is considered an error.

Subject-defined limits:
- `MAX_NAME_LENGTH`: 32
- `MAX_DESCRIPTION_LENGTH`: 255
- `MAX_BODY_LENGTH`: 512

`/create`, `/list`, and `/info` are context-dependent and mapped to specific server operations based on current `/use` context.

## Data and persistence

Server state is persisted in:
- `data/users.json`
- `data/teams.json`
- `data/messages.json`

Files are created automatically on first run if missing.

## Protocol reference

The detailed command/message specification is documented in `STANDARDS.md`.

High-level framing:
- Client requests are line-based and end with `\r\n`
- Server answers with response lines (`R...`)
- Server events/notifications are sent as info lines (`I...`)

## Logging and output rules

Per subject expectations:
- Event/data output should go through the provided logging library callbacks
- Do not print custom error logs to stderr for project event display
- Prefer stdout if textual diagnostics are needed during development

## Tests

Run tests with Cargo:

```bash
cargo test
```

If `cargo` is unavailable in your default shell, run inside Nix:

```bash
nix develop -c cargo test
```
