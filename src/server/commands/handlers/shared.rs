//! Shared helpers used by server command handlers.

use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commands::SessionState;
use crate::libsrv;
use crate::protocol::{quoted, response};
use crate::storage::{ChannelEntry, ReplyEntry, ServerStorage, TeamEntry, TeamTree, ThreadEntry};
use crate::users::UserStore;

pub(crate) const MAX_NAME_LENGTH: usize = 32;
pub(crate) const MAX_DESCRIPTION_LENGTH: usize = 255;
pub(crate) const MAX_BODY_LENGTH: usize = 512;

/// The current resource scope selected by the session context.
pub(crate) enum ResourceContext {
    /// No team, channel, or thread is selected.
    Root,
    /// A team is selected.
    Team {
        /// Selected team UUID.
        team_uuid: String,
    },
    /// A team and channel are selected.
    Channel {
        /// Selected team UUID.
        team_uuid: String,
        /// Selected channel UUID.
        channel_uuid: String,
    },
    /// A team, channel, and thread are selected.
    Thread {
        /// Selected team UUID.
        team_uuid: String,
        /// Selected channel UUID.
        channel_uuid: String,
        /// Selected thread UUID.
        thread_uuid: String,
    },
}

/// Build the standard bad-request response line.
pub(crate) fn bad_request() -> String {
    response(501, Some("\"bad request\""))
}

/// Build a standard unknown-user response line.
pub(crate) fn unknown_user(user_uuid: &str) -> String {
    response(404, Some(&quoted(user_uuid)))
}

/// Return the current UNIX timestamp in seconds.
pub(crate) fn now_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Generate a UUID-like identifier derived from a seed and sequence number.
pub(crate) fn make_uuid_v4_like(seed: &str, sequence: u64) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut h1);
    nanos.hash(&mut h1);
    sequence.hash(&mut h1);
    let p1 = h1.finish();

    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    p1.hash(&mut h2);
    nanos.rotate_left(17).hash(&mut h2);
    sequence.rotate_left(9).hash(&mut h2);
    let p2 = h2.finish();

    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&p1.to_be_bytes());
    bytes[8..].copy_from_slice(&p2.to_be_bytes());

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

/// Convert the session context into a concrete resource scope.
pub(crate) fn current_context(state: &SessionState) -> Result<ResourceContext, ()> {
    match (
        state.context.team_uuid.as_ref(),
        state.context.channel_uuid.as_ref(),
        state.context.thread_uuid.as_ref(),
    ) {
        (None, None, None) => Ok(ResourceContext::Root),
        (Some(team_uuid), None, None) => Ok(ResourceContext::Team {
            team_uuid: team_uuid.clone(),
        }),
        (Some(team_uuid), Some(channel_uuid), None) => Ok(ResourceContext::Channel {
            team_uuid: team_uuid.clone(),
            channel_uuid: channel_uuid.clone(),
        }),
        (Some(team_uuid), Some(channel_uuid), Some(thread_uuid)) => Ok(ResourceContext::Thread {
            team_uuid: team_uuid.clone(),
            channel_uuid: channel_uuid.clone(),
            thread_uuid: thread_uuid.clone(),
        }),
        _ => Err(()),
    }
}

/// Convert a Rust string to a C string if it contains no NUL bytes.
pub(crate) fn cstring(value: &str) -> Option<CString> {
    CString::new(value).ok()
}

/// Notify the C bridge that a team was created.
pub(crate) fn call_event_team_created(team_uuid: &str, team_name: &str, user_uuid: &str) {
    let (Some(team_uuid), Some(team_name), Some(user_uuid)) =
        (cstring(team_uuid), cstring(team_name), cstring(user_uuid))
    else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_team_created(
            team_uuid.as_ptr(),
            team_name.as_ptr(),
            user_uuid.as_ptr(),
        );
    }
}

/// Notify the C bridge that a channel was created.
pub(crate) fn call_event_channel_created(team_uuid: &str, channel_uuid: &str, channel_name: &str) {
    let (Some(team_uuid), Some(channel_uuid), Some(channel_name)) = (
        cstring(team_uuid),
        cstring(channel_uuid),
        cstring(channel_name),
    ) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_channel_created(
            team_uuid.as_ptr(),
            channel_uuid.as_ptr(),
            channel_name.as_ptr(),
        );
    }
}

/// Notify the C bridge that a thread was created.
pub(crate) fn call_event_thread_created(
    channel_uuid: &str,
    thread_uuid: &str,
    user_uuid: &str,
    thread_title: &str,
    thread_body: &str,
) {
    let (
        Some(channel_uuid),
        Some(thread_uuid),
        Some(user_uuid),
        Some(thread_title),
        Some(thread_body),
    ) = (
        cstring(channel_uuid),
        cstring(thread_uuid),
        cstring(user_uuid),
        cstring(thread_title),
        cstring(thread_body),
    )
    else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_thread_created(
            channel_uuid.as_ptr(),
            thread_uuid.as_ptr(),
            user_uuid.as_ptr(),
            thread_title.as_ptr(),
            thread_body.as_ptr(),
        );
    }
}

/// Notify the C bridge that a reply was created.
pub(crate) fn call_event_reply_created(thread_uuid: &str, user_uuid: &str, reply_body: &str) {
    let (Some(thread_uuid), Some(user_uuid), Some(reply_body)) = (
        cstring(thread_uuid),
        cstring(user_uuid),
        cstring(reply_body),
    ) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_reply_created(
            thread_uuid.as_ptr(),
            user_uuid.as_ptr(),
            reply_body.as_ptr(),
        );
    }
}

/// Notify the C bridge that a user subscribed to a team.
pub(crate) fn call_event_user_subscribed(team_uuid: &str, user_uuid: &str) {
    let (Some(team_uuid), Some(user_uuid)) = (cstring(team_uuid), cstring(user_uuid)) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_subscribed(team_uuid.as_ptr(), user_uuid.as_ptr());
    }
}

/// Notify the C bridge that a user unsubscribed from a team.
pub(crate) fn call_event_user_unsubscribed(team_uuid: &str, user_uuid: &str) {
    let (Some(team_uuid), Some(user_uuid)) = (cstring(team_uuid), cstring(user_uuid)) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_unsubscribed(team_uuid.as_ptr(), user_uuid.as_ptr());
    }
}

/// Notify the C bridge that a user was created.
pub(crate) fn call_event_user_created(user_uuid: &str, user_name: &str) {
    let Ok(uuid) = CString::new(user_uuid) else {
        return;
    };
    let Ok(name) = CString::new(user_name) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_created(uuid.as_ptr(), name.as_ptr());
    }
}

/// Notify the C bridge that a user logged in.
pub(crate) fn call_event_user_logged_in(user_uuid: &str) {
    let Ok(uuid) = CString::new(user_uuid) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_logged_in(uuid.as_ptr());
    }
}

/// Notify the C bridge that a private message was sent.
pub(crate) fn call_event_private_message_sended(
    sender_uuid: &str,
    recipient_uuid: &str,
    message_body: &str,
) {
    let (Some(sender_uuid), Some(recipient_uuid), Some(message_body)) = (
        cstring(sender_uuid),
        cstring(recipient_uuid),
        cstring(message_body),
    ) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_private_message_sended(
            sender_uuid.as_ptr(),
            recipient_uuid.as_ptr(),
            message_body.as_ptr(),
        );
    }
}

/// Notify the C bridge that a user logged out.
pub fn emit_user_logged_out(user_uuid: &str) {
    let Ok(uuid) = CString::new(user_uuid) else {
        return;
    };

    unsafe {
        let _ = libsrv::server_event_user_logged_out(uuid.as_ptr());
    }
}

/// Validate that the argument count falls within the accepted range.
pub(crate) fn validate_arg_count(args: &[String], min: usize, max: usize) -> Result<(), String> {
    if args.len() < min || args.len() > max {
        return Err(bad_request());
    }
    Ok(())
}

/// Check whether a string fits within a maximum character length.
pub(crate) fn validate_max_len(value: &str, max: usize) -> bool {
    value.chars().count() <= max
}

/// Find a mutable reference to a team entry.
pub(crate) fn team_index_mut<'a>(
    tree: &'a mut TeamTree,
    team_uuid: &str,
) -> Option<&'a mut TeamEntry> {
    tree.teams.iter_mut().find(|team| team.uuid == team_uuid)
}

/// Find a mutable reference to a channel entry.
pub(crate) fn channel_index_mut<'a>(
    tree: &'a mut TeamTree,
    team_uuid: &str,
    channel_uuid: &str,
) -> Option<&'a mut ChannelEntry> {
    team_index_mut(tree, team_uuid).and_then(|team| {
        team.channels
            .iter_mut()
            .find(|channel| channel.uuid == channel_uuid)
    })
}

/// Find a mutable reference to a thread entry.
pub(crate) fn thread_index_mut<'a>(
    tree: &'a mut TeamTree,
    team_uuid: &str,
    channel_uuid: &str,
    thread_uuid: &str,
) -> Option<&'a mut ThreadEntry> {
    channel_index_mut(tree, team_uuid, channel_uuid).and_then(|channel| {
        channel
            .threads
            .iter_mut()
            .find(|thread| thread.uuid == thread_uuid)
    })
}

/// Format a team entry as a response body.
pub(crate) fn team_response(team: &TeamEntry) -> String {
    [
        quoted("TEAM"),
        quoted(&team.uuid),
        quoted(&team.name),
        quoted(&team.description),
    ]
    .join(" ")
}

/// Format a channel entry as a response body.
pub(crate) fn channel_response(channel: &ChannelEntry) -> String {
    [
        quoted("CHANNEL"),
        quoted(&channel.uuid),
        quoted(&channel.name),
        quoted(&channel.description),
    ]
    .join(" ")
}

/// Format a thread entry as a response body.
pub(crate) fn thread_response(thread: &ThreadEntry) -> String {
    [
        quoted("THREAD"),
        quoted(&thread.uuid),
        quoted(&thread.user_uuid),
        quoted(&thread.timestamp.to_string()),
        quoted(&thread.title),
        quoted(&thread.body),
    ]
    .join(" ")
}

/// Format a reply entry as a response body.
pub(crate) fn reply_response(thread_uuid: &str, reply: &ReplyEntry) -> String {
    [
        quoted("REPLY"),
        quoted(thread_uuid),
        quoted(&reply.user_uuid),
        quoted(&reply.timestamp.to_string()),
        quoted(&reply.body),
    ]
    .join(" ")
}

/// Fetch the current user details from the session and user store.
pub(crate) fn current_user_details(
    state: &SessionState,
    users: &UserStore,
) -> Option<(String, String, bool)> {
    let user_uuid = state.user_uuid.as_ref()?;
    let (user_name, is_online) = users.user_details(user_uuid)?;
    Some((user_uuid.clone(), user_name, is_online))
}

/// Update the session context from parsed arguments.
pub(crate) fn set_context(state: &mut SessionState, args: &[String]) {
    state.context.team_uuid = args.first().cloned();
    state.context.channel_uuid = args.get(1).cloned();
    state.context.thread_uuid = args.get(2).cloned();
}

/// Validate the requested `USE` context against storage.
pub(crate) fn validate_use_context(storage: &ServerStorage, args: &[String]) -> Result<(), String> {
    if args.iter().any(|arg| arg.is_empty()) {
        return Err(String::new());
    }

    match args {
        [] => Ok(()),
        [team_uuid] => storage
            .team(team_uuid)
            .map(|_| ())
            .ok_or_else(|| quoted(team_uuid)),
        [team_uuid, channel_uuid] => {
            if storage.team(team_uuid).is_none() {
                return Err(quoted(team_uuid));
            }

            if storage.channel(team_uuid, channel_uuid).is_none() {
                return Err(quoted(channel_uuid));
            }

            Ok(())
        }
        [team_uuid, channel_uuid, thread_uuid] => {
            if storage.team(team_uuid).is_none() {
                return Err(quoted(team_uuid));
            }

            if storage.channel(team_uuid, channel_uuid).is_none() {
                return Err(quoted(channel_uuid));
            }

            if storage
                .thread(team_uuid, channel_uuid, thread_uuid)
                .is_none()
            {
                return Err(quoted(thread_uuid));
            }

            Ok(())
        }
        _ => unreachable!(),
    }
}
