//! Client response handlers.
//!
//! The client keeps one outstanding request at a time. This module owns the
//! response dispatch step that inspects the pending request, parses the server
//! reply, and forwards the data to the appropriate FFI callback.
//!
//! Each submodule handles one part of the protocol:
//! - `auth` for login, logout, users, and private messages.
//! - `context` for subscribe, subscribed, unsubscribe, and use.
//! - `resources` for create, list, and info commands.

use std::io;

use crate::commands::protocol::parse_response_code;
use crate::commands::{PendingRequest, SessionState};

mod auth;
mod context;
mod resources;
mod shared;

fn handle_no_pending_response(code: u16, response: &str) -> io::Result<()> {
    if code != 200 {
        return Err(shared::invalid_response(response));
    }

    Ok(())
}

/// Handle a server response for the current pending request.
pub fn handle_response_line(state: &mut SessionState, response: &str) -> io::Result<()> {
    let code = parse_response_code(response)?;
    let pending_request = state.pending_request.take();

    match pending_request {
        Some(PendingRequest::Login { user_name }) => {
            auth::handle_login_response(state, code, response, user_name)
        }
        Some(PendingRequest::Logout) => auth::handle_logout_response(state, code, response),
        Some(PendingRequest::Users) => auth::handle_users_response(code, response),
        Some(PendingRequest::User { user_uuid }) => {
            auth::handle_user_response(code, response, user_uuid)
        }
        Some(PendingRequest::Send { user_uuid, .. }) => {
            auth::handle_send_response(code, response, user_uuid)
        }
        Some(PendingRequest::Messages { user_uuid }) => {
            auth::handle_messages_response(code, response, user_uuid)
        }
        Some(PendingRequest::Subscribe { team_uuid }) => {
            context::handle_subscribe_response(code, response, team_uuid)
        }
        Some(PendingRequest::Subscribed { team_uuid }) => {
            context::handle_subscribed_response(code, response, team_uuid)
        }
        Some(PendingRequest::Unsubscribe { team_uuid }) => {
            context::handle_unsubscribe_response(code, response, team_uuid)
        }
        Some(PendingRequest::Use {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => context::handle_use_response(
            state,
            code,
            response,
            team_uuid,
            channel_uuid,
            thread_uuid,
        ),
        Some(PendingRequest::CreateTeam) => resources::handle_create_team_response(code, response),
        Some(PendingRequest::CreateChannel { team_uuid }) => {
            resources::handle_create_channel_response(code, response, team_uuid)
        }
        Some(PendingRequest::CreateThread {
            team_uuid,
            channel_uuid,
        }) => resources::handle_create_thread_response(code, response, team_uuid, channel_uuid),
        Some(PendingRequest::CreateReply {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => resources::handle_create_reply_response(
            code,
            response,
            team_uuid,
            channel_uuid,
            thread_uuid,
        ),
        Some(PendingRequest::ListTeams) => resources::handle_list_teams_response(code, response),
        Some(PendingRequest::ListChannels { team_uuid }) => {
            resources::handle_list_channels_response(code, response, team_uuid)
        }
        Some(PendingRequest::ListThreads {
            team_uuid,
            channel_uuid,
        }) => resources::handle_list_threads_response(code, response, team_uuid, channel_uuid),
        Some(PendingRequest::ListReplies {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => resources::handle_list_replies_response(
            code,
            response,
            team_uuid,
            channel_uuid,
            thread_uuid,
        ),
        Some(PendingRequest::InfoUser) => resources::handle_info_user_response(code, response),
        Some(PendingRequest::InfoTeam { team_uuid }) => {
            resources::handle_info_team_response(code, response, team_uuid)
        }
        Some(PendingRequest::InfoChannel {
            team_uuid,
            channel_uuid,
        }) => resources::handle_info_channel_response(code, response, team_uuid, channel_uuid),
        Some(PendingRequest::InfoThread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => resources::handle_info_thread_response(
            code,
            response,
            team_uuid,
            channel_uuid,
            thread_uuid,
        ),
        None => handle_no_pending_response(code, response),
    }
}
