use crate::commands::{CommandMap, CommandOutcome, InfoEvent, SessionState};
use crate::protocol::{quoted, response};
use crate::storage::{ChannelEntry, ReplyEntry, ServerStorage, TeamEntry};
use crate::users::UserStore;

use super::shared::{
    bad_request, call_event_channel_created, call_event_reply_created, call_event_team_created,
    call_event_thread_created, channel_index_mut, channel_response, current_context,
    current_user_details, make_uuid_v4_like, now_unix_timestamp, reply_response, team_index_mut,
    team_response, thread_index_mut, thread_response, validate_arg_count, validate_max_len,
    ResourceContext, MAX_BODY_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH,
};

fn team_subscriber_info_events(users: &UserStore, team_uuid: &str, payload: String) -> Vec<InfoEvent> {
    users
        .subscribed_user_ids(team_uuid)
        .into_iter()
        .map(|recipient_user_uuid| InfoEvent {
            recipient_user_uuid,
            payload: payload.clone(),
        })
        .collect()
}

/// Create a team, channel, thread, or reply in the current context.
pub fn handle_create(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    let Some(user_uuid) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    match current_context(state) {
        Ok(ResourceContext::Root) => {
            if validate_arg_count(args, 2, 2).is_err()
                || args.iter().any(|arg| arg.is_empty())
                || !validate_max_len(&args[0], MAX_NAME_LENGTH)
                || !validate_max_len(&args[1], MAX_DESCRIPTION_LENGTH)
            {
                return CommandOutcome::response_only(bad_request());
            }

            let team_uuid = make_uuid_v4_like(&args[0], storage.team_tree().teams.len() as u64 + 1);
            let team = TeamEntry {
                uuid: team_uuid.clone(),
                name: args[0].clone(),
                description: args[1].clone(),
                channels: Vec::new(),
            };

            let mut tree = storage.team_tree().clone();
            tree.teams.push(team.clone());
            if let Err(err) = storage.replace_team_tree(tree) {
                eprintln!("Failed to persist team tree: {}", err);
                return CommandOutcome::response_only(response(
                    500,
                    Some("\"internal server error\""),
                ));
            }

            call_event_team_created(&team.uuid, &team.name, user_uuid);
            CommandOutcome::response_only(response(200, Some(&team_response(&team))))
        }
        Ok(ResourceContext::Team { team_uuid }) => {
            if validate_arg_count(args, 2, 2).is_err()
                || args.iter().any(|arg| arg.is_empty())
                || !validate_max_len(&args[0], MAX_NAME_LENGTH)
                || !validate_max_len(&args[1], MAX_DESCRIPTION_LENGTH)
            {
                return CommandOutcome::response_only(bad_request());
            }

            let Some(team) = storage.team(&team_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&team_uuid))));
            };

            let channel_uuid = make_uuid_v4_like(&args[0], team.channels.len() as u64 + 1);
            let channel = ChannelEntry {
                uuid: channel_uuid.clone(),
                name: args[0].clone(),
                description: args[1].clone(),
                threads: Vec::new(),
            };

            let mut tree = storage.team_tree().clone();
            let Some(team) = team_index_mut(&mut tree, &team_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&team_uuid))));
            };
            team.channels.push(channel.clone());
            if let Err(err) = storage.replace_team_tree(tree) {
                eprintln!("Failed to persist team tree: {}", err);
                return CommandOutcome::response_only(response(
                    500,
                    Some("\"internal server error\""),
                ));
            }

            call_event_channel_created(&team_uuid, &channel.uuid, &channel.name);
            let info_payload = format!(
                "I100 NEW_CHANNEL {} {} {} {}\r\n",
                quoted(&team_uuid),
                quoted(&channel.uuid),
                quoted(&channel.name),
                quoted(&channel.description)
            );

            CommandOutcome {
                response: response(200, Some(&channel_response(&channel))),
                info_events: team_subscriber_info_events(users, &team_uuid, info_payload),
            }
        }
        Ok(ResourceContext::Channel {
            team_uuid,
            channel_uuid,
        }) => {
            if validate_arg_count(args, 2, 2).is_err()
                || args.iter().any(|arg| arg.is_empty())
                || !validate_max_len(&args[0], MAX_NAME_LENGTH)
                || !validate_max_len(&args[1], MAX_BODY_LENGTH)
            {
                return CommandOutcome::response_only(bad_request());
            }

            let Some(channel) = storage.channel(&team_uuid, &channel_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&channel_uuid))));
            };

            let timestamp = now_unix_timestamp();
            let thread_uuid = make_uuid_v4_like(&args[0], channel.threads.len() as u64 + 1);
            let thread = crate::storage::ThreadEntry {
                uuid: thread_uuid.clone(),
                user_uuid: user_uuid.to_string(),
                timestamp,
                title: args[0].clone(),
                body: args[1].clone(),
                replies: Vec::new(),
            };

            let mut tree = storage.team_tree().clone();
            let Some(thread_parent) = channel_index_mut(&mut tree, &team_uuid, &channel_uuid)
            else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&channel_uuid))));
            };
            thread_parent.threads.push(thread.clone());
            if let Err(err) = storage.replace_team_tree(tree) {
                eprintln!("Failed to persist team tree: {}", err);
                return CommandOutcome::response_only(response(
                    500,
                    Some("\"internal server error\""),
                ));
            }

            call_event_thread_created(
                &channel_uuid,
                &thread.uuid,
                user_uuid,
                &thread.title,
                &thread.body,
            );
            let info_payload = format!(
                "I100 NEW_THREAD {} {} {} {} {} {} {}\r\n",
                quoted(&team_uuid),
                quoted(&channel_uuid),
                quoted(&thread.uuid),
                quoted(user_uuid),
                quoted(&thread.timestamp.to_string()),
                quoted(&thread.title),
                quoted(&thread.body)
            );

            CommandOutcome {
                response: response(200, Some(&thread_response(&thread))),
                info_events: team_subscriber_info_events(users, &team_uuid, info_payload),
            }
        }
        Ok(ResourceContext::Thread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            if validate_arg_count(args, 1, 1).is_err()
                || args[0].is_empty()
                || !validate_max_len(&args[0], MAX_BODY_LENGTH)
            {
                return CommandOutcome::response_only(bad_request());
            }

            let Some(_thread) = storage.thread(&team_uuid, &channel_uuid, &thread_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&thread_uuid))));
            };

            let timestamp = now_unix_timestamp();
            let reply = ReplyEntry {
                user_uuid: user_uuid.to_string(),
                timestamp,
                body: args[0].clone(),
            };

            let mut tree = storage.team_tree().clone();
            let Some(thread_parent) =
                thread_index_mut(&mut tree, &team_uuid, &channel_uuid, &thread_uuid)
            else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&thread_uuid))));
            };
            thread_parent.replies.push(reply.clone());
            if let Err(err) = storage.replace_team_tree(tree) {
                eprintln!("Failed to persist team tree: {}", err);
                return CommandOutcome::response_only(response(
                    500,
                    Some("\"internal server error\""),
                ));
            }

            call_event_reply_created(&thread_uuid, user_uuid, &reply.body);
            let info_payload = format!(
                "I100 NEW_REPLY {} {} {} {}\r\n",
                quoted(&team_uuid),
                quoted(&thread_uuid),
                quoted(user_uuid),
                quoted(&reply.body)
            );

            CommandOutcome {
                response: response(200, Some(&reply_response(&thread_uuid, &reply))),
                info_events: team_subscriber_info_events(users, &team_uuid, info_payload),
            }
        }
        Err(()) => CommandOutcome::response_only(bad_request()),
    }
}

/// List teams, channels, threads, or replies for the current context.
pub fn handle_list(
    state: &mut SessionState,
    _registry: &CommandMap,
    _users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(_) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    let body = match current_context(state) {
        Ok(ResourceContext::Root) => {
            let mut chunks = vec![quoted("TEAMS")];
            for team in storage.team_tree().teams.iter() {
                chunks.push(quoted(&team.uuid));
                chunks.push(quoted(&team.name));
                chunks.push(quoted(&team.description));
            }
            chunks.join(" ")
        }
        Ok(ResourceContext::Team { team_uuid }) => {
            let Some(team) = storage.team(&team_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&team_uuid))));
            };

            let mut chunks = vec![quoted("CHANNELS")];
            for channel in team.channels.iter() {
                chunks.push(quoted(&channel.uuid));
                chunks.push(quoted(&channel.name));
                chunks.push(quoted(&channel.description));
            }
            chunks.join(" ")
        }
        Ok(ResourceContext::Channel {
            team_uuid,
            channel_uuid,
        }) => {
            let Some(channel) = storage.channel(&team_uuid, &channel_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&channel_uuid))));
            };

            let mut chunks = vec![quoted("THREADS")];
            for thread in channel.threads.iter() {
                chunks.push(quoted(&thread.uuid));
                chunks.push(quoted(&thread.user_uuid));
                chunks.push(quoted(&thread.timestamp.to_string()));
                chunks.push(quoted(&thread.title));
                chunks.push(quoted(&thread.body));
            }
            chunks.join(" ")
        }
        Ok(ResourceContext::Thread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            let Some(thread) = storage.thread(&team_uuid, &channel_uuid, &thread_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&thread_uuid))));
            };

            let mut chunks = vec![quoted("REPLIES")];
            for reply in thread.replies.iter() {
                chunks.push(quoted(&reply.user_uuid));
                chunks.push(quoted(&reply.timestamp.to_string()));
                chunks.push(quoted(&reply.body));
            }
            chunks.join(" ")
        }
        Err(()) => return CommandOutcome::response_only(bad_request()),
    };

    CommandOutcome::response_only(response(200, Some(&body)))
}

/// Show details for the current user or selected resource.
pub fn handle_info(
    state: &mut SessionState,
    _registry: &CommandMap,
    users: &mut UserStore,
    storage: &mut ServerStorage,
    args: &[String],
) -> CommandOutcome {
    if validate_arg_count(args, 0, 0).is_err() {
        return CommandOutcome::response_only(bad_request());
    }

    let Some(_) = state.user_uuid.as_deref() else {
        return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
    };

    let body = match current_context(state) {
        Ok(ResourceContext::Root) => {
            let Some((user_uuid, user_name, is_online)) = current_user_details(state, users) else {
                return CommandOutcome::response_only(response(401, Some("\"unauthorized\"")));
            };

            [
                quoted("USER"),
                quoted(&user_uuid),
                quoted(&user_name),
                quoted(if is_online { "1" } else { "0" }),
            ]
            .join(" ")
        }
        Ok(ResourceContext::Team { team_uuid }) => {
            let Some(team) = storage.team(&team_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&team_uuid))));
            };

            team_response(team)
        }
        Ok(ResourceContext::Channel {
            team_uuid,
            channel_uuid,
        }) => {
            let Some(channel) = storage.channel(&team_uuid, &channel_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&channel_uuid))));
            };

            channel_response(channel)
        }
        Ok(ResourceContext::Thread {
            team_uuid,
            channel_uuid,
            thread_uuid,
        }) => {
            let Some(thread) = storage.thread(&team_uuid, &channel_uuid, &thread_uuid) else {
                return CommandOutcome::response_only(response(404, Some(&quoted(&thread_uuid))));
            };

            thread_response(thread)
        }
        Err(()) => return CommandOutcome::response_only(bad_request()),
    };

    CommandOutcome::response_only(response(200, Some(&body)))
}
