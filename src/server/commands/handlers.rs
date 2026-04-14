//! Server command handler exports.
//!
//! The actual implementations are split by concern:
//! - `auth` handles user/session operations.
//! - `context` handles subscribe and `USE` state.
//! - `resources` handles create, list, and info operations.
//! - `shared` contains validation, response formatting, and C FFI hooks.

mod auth;
mod context;
mod resources;
mod shared;

pub use auth::{
    handle_help, handle_login, handle_logout, handle_messages, handle_send, handle_user,
    handle_users,
};
pub use context::{handle_subscribe, handle_subscribed, handle_unsubscribe, handle_use};
pub use resources::{handle_create, handle_info, handle_list};
pub use shared::emit_user_logged_out;

#[cfg(test)]
mod tests;
