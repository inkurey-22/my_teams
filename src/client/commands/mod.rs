mod handlers;
mod parser;
mod protocol;
mod registry;
mod types;

pub use parser::{dispatch_slash_command, read_server_response_line, write_raw_command};
pub use registry::command_registry;
pub use types::ShellState;
pub(crate) use types::{CommandDefinition, CommandMap};
