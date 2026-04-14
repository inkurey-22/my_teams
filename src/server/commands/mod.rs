//! Server command pipeline.
//!
//! This module groups the request parser, dispatch layer, command registry,
//! and the command handlers that mutate storage and session state.
//!
//! The public surface is intentionally small:
//! - `dispatch_line` parses and routes one request.
//! - `command_registry` provides the built-in command table.
//! - `SessionState` carries per-client context across requests.

mod dispatcher;
mod handlers;
mod registry;
mod types;

pub use dispatcher::dispatch_line;
pub use handlers::emit_user_logged_out;
pub use registry::command_registry;
pub use types::SessionState;
pub(crate) use types::{CommandDefinition, CommandMap, CommandOutcome, InfoEvent};
