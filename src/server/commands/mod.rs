mod dispatcher;
mod handlers;
mod registry;
mod types;

pub use dispatcher::dispatch_line;
pub use handlers::emit_user_logged_out;
pub use registry::command_registry;
pub use types::SessionState;
pub(crate) use types::{CommandDefinition, CommandMap, CommandOutcome, InfoEvent};
