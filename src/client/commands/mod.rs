//! Client command pipeline.
//!
//! This module groups the pieces that turn shell input into wire requests,
//! track the pending request state, and render the server response back into
//! the client callbacks.
//!
//! The flow is:
//! - `registry` exposes the command table used by the shell.
//! - `handlers` converts a typed command into a request line.
//! - `responses` matches the next server response to the pending request.
//! - `protocol` defines the request and response text format.

mod dispatcher;
mod handlers;
mod protocol;
mod registry;
mod responses;
mod types;

pub use dispatcher::{dispatch_line, write_request_line};
pub use protocol::{parse_info_message, InfoMessage};
pub use registry::command_registry;
pub use responses::handle_response_line;
pub use types::SessionState;
pub(crate) use types::{CommandDefinition, CommandMap, PendingRequest};
