use super::*;
use crate::commands::types::CommandContext;
use crate::commands::{CommandMap, SessionState};
use crate::storage::ServerStorage;
use crate::users::UserStore;

use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestStorage {
    storage: ServerStorage,
    root: PathBuf,
}

impl TestStorage {
    fn new() -> Self {
        let unique = format!(
            "my_teams_handlers_{}_{}",
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );

        let root = std::env::temp_dir().join(unique);
        let users_path = root.join("users.json");
        let teams_path = root.join("teams.json");
        let messages_path = root.join("messages.json");
        let storage = ServerStorage::load_or_default(users_path, teams_path, messages_path)
            .expect("test storage should be created");

        Self { storage, root }
    }
}

impl Drop for TestStorage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn users_command_lists_all_users_with_online_flags() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![
        ("alice".to_string(), "uuid-alice".to_string()),
        ("bob".to_string(), "uuid-bob".to_string()),
    ]);
    let mut test_storage = TestStorage::new();

    let _ = users.login("alice");

    let outcome = handle_users(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[],
    );

    assert_eq!(
        outcome.response,
        "R200 \"USERS\" \"uuid-alice\" \"alice\" \"1\" \"uuid-bob\" \"bob\" \"0\"\r\n"
    );
    assert!(outcome.info_events.is_empty());
}

#[test]
fn users_command_rejects_unexpected_arguments() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let outcome = handle_users(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["extra".to_string()],
    );

    assert_eq!(outcome.response, "R501 \"bad request\"\r\n");
}

#[test]
fn user_command_returns_user_details() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![("alice".to_string(), "uuid-alice".to_string())]);
    let mut test_storage = TestStorage::new();

    let _ = users.login("alice");

    let outcome = handle_user(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["uuid-alice".to_string()],
    );

    assert_eq!(
        outcome.response,
        "R200 \"USER\" \"uuid-alice\" \"alice\" \"1\"\r\n"
    );
    assert!(outcome.info_events.is_empty());
}

#[test]
fn user_command_returns_not_found_for_unknown_uuid() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let outcome = handle_user(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["missing-uuid".to_string()],
    );

    assert_eq!(outcome.response, "R404 \"missing-uuid\"\r\n");
}

#[test]
fn user_command_rejects_wrong_argument_count() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let outcome = handle_user(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[],
    );

    assert_eq!(outcome.response, "R501 \"bad request\"\r\n");
}

#[test]
fn send_command_persists_message_and_emits_info_event() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![
        ("alice".to_string(), "uuid-alice".to_string()),
        ("bob".to_string(), "uuid-bob".to_string()),
    ]);
    let mut test_storage = TestStorage::new();

    let outcome = handle_send(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["uuid-bob".to_string(), "hello bob".to_string()],
    );

    assert_eq!(outcome.response, "R200\r\n");
    assert_eq!(outcome.info_events.len(), 1);
    assert_eq!(outcome.info_events[0].recipient_user_uuid, "uuid-bob");
    assert_eq!(
        outcome.info_events[0].payload,
        "I100 NEW_MESSAGE \"uuid-alice\" \"hello bob\"\r\n"
    );

    let conversation = test_storage
        .storage
        .conversation_messages("uuid-alice", "uuid-bob");
    assert_eq!(conversation.len(), 1);
    assert_eq!(conversation[0].sender_uuid, "uuid-alice");
    assert_eq!(conversation[0].recipient_uuid, "uuid-bob");
    assert_eq!(conversation[0].body, "hello bob");
}

#[test]
fn send_command_rejects_unauthorized_user() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![("bob".to_string(), "uuid-bob".to_string())]);
    let mut test_storage = TestStorage::new();

    let outcome = handle_send(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["uuid-bob".to_string(), "hello".to_string()],
    );

    assert_eq!(outcome.response, "R401 \"unauthorized\"\r\n");
    assert!(outcome.info_events.is_empty());
    assert!(test_storage
        .storage
        .conversation_messages("uuid-alice", "uuid-bob")
        .is_empty());
}

#[test]
fn messages_command_returns_bidirectional_history() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![
        ("alice".to_string(), "uuid-alice".to_string()),
        ("bob".to_string(), "uuid-bob".to_string()),
        ("carol".to_string(), "uuid-carol".to_string()),
    ]);
    let mut test_storage = TestStorage::new();

    test_storage
        .storage
        .append_private_message("uuid-alice", "uuid-bob", 10, "a->b")
        .expect("message should be stored");
    test_storage
        .storage
        .append_private_message("uuid-bob", "uuid-alice", 20, "b->a")
        .expect("message should be stored");
    test_storage
        .storage
        .append_private_message("uuid-carol", "uuid-bob", 30, "ignored")
        .expect("message should be stored");

    let outcome = handle_messages(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["uuid-bob".to_string()],
    );

    assert_eq!(
        outcome.response,
        "R200 \"MESSAGES\" \"uuid-alice\" \"10\" \"a->b\" \"uuid-bob\" \"20\" \"b->a\"\r\n"
    );
    assert!(outcome.info_events.is_empty());
}

#[test]
fn messages_command_rejects_unauthorized_user() {
    let mut state = SessionState::default();
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![("bob".to_string(), "uuid-bob".to_string())]);
    let mut test_storage = TestStorage::new();

    let outcome = handle_messages(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["uuid-bob".to_string()],
    );

    assert_eq!(outcome.response, "R401 \"unauthorized\"\r\n");
    assert!(outcome.info_events.is_empty());
}

#[test]
fn use_command_updates_context_fields() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();
    test_storage
        .storage
        .replace_team_tree(crate::storage::TeamTree {
            teams: vec![crate::storage::TeamEntry {
                uuid: "team-1".to_string(),
                name: "Team 1".to_string(),
                description: "Desc 1".to_string(),
                channels: vec![crate::storage::ChannelEntry {
                    uuid: "chan-1".to_string(),
                    name: "Channel 1".to_string(),
                    description: "Desc 2".to_string(),
                    threads: vec![crate::storage::ThreadEntry {
                        uuid: "thread-1".to_string(),
                        user_uuid: "uuid-alice".to_string(),
                        timestamp: 1,
                        title: "Thread 1".to_string(),
                        body: "Body 1".to_string(),
                        replies: Vec::new(),
                    }],
                }],
            }],
        })
        .expect("seed tree should persist");

    let outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[
            "team-1".to_string(),
            "chan-1".to_string(),
            "thread-1".to_string(),
        ],
    );

    assert_eq!(outcome.response, "R200\r\n");
    assert_eq!(state.context.team_uuid.as_deref(), Some("team-1"));
    assert_eq!(state.context.channel_uuid.as_deref(), Some("chan-1"));
    assert_eq!(state.context.thread_uuid.as_deref(), Some("thread-1"));
}

#[test]
fn use_command_rejects_unknown_channel_without_mutating_context() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: CommandContext {
            team_uuid: Some("team-1".to_string()),
            channel_uuid: None,
            thread_uuid: None,
        },
    };
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();
    test_storage
        .storage
        .replace_team_tree(crate::storage::TeamTree {
            teams: vec![crate::storage::TeamEntry {
                uuid: "team-1".to_string(),
                name: "Team 1".to_string(),
                description: "Desc 1".to_string(),
                channels: Vec::new(),
            }],
        })
        .expect("seed tree should persist");

    let outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-1".to_string(), "chan-1".to_string()],
    );

    assert_eq!(outcome.response, "R404 \"chan-1\"\r\n");
    assert_eq!(state.context.team_uuid.as_deref(), Some("team-1"));
    assert_eq!(state.context.channel_uuid.as_deref(), None);
    assert_eq!(state.context.thread_uuid.as_deref(), None);
}

#[test]
fn subscribe_and_subscribed_commands_track_team_membership() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![("alice".to_string(), "uuid-alice".to_string())]);
    let mut test_storage = TestStorage::new();
    test_storage
        .storage
        .replace_team_tree(crate::storage::TeamTree {
            teams: vec![crate::storage::TeamEntry {
                uuid: "team-1".to_string(),
                name: "Team 1".to_string(),
                description: "Desc 1".to_string(),
                channels: Vec::new(),
            }],
        })
        .expect("seed tree should persist");

    let subscribe_outcome = handle_subscribe(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-1".to_string()],
    );
    assert_eq!(
        subscribe_outcome.response,
        "R200 \"SUBSCRIBED\" \"uuid-alice\" \"team-1\"\r\n"
    );

    let subscribed_teams_outcome = handle_subscribed(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[],
    );
    assert_eq!(
        subscribed_teams_outcome.response,
        "R200 \"TEAMS\" \"team-1\" \"Team 1\" \"Desc 1\"\r\n"
    );

    let subscribed_users_outcome = handle_subscribed(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-1".to_string()],
    );
    assert_eq!(
        subscribed_users_outcome.response,
        "R200 \"USERS\" \"uuid-alice\" \"alice\" \"0\"\r\n"
    );
}

#[test]
fn unsubscribe_removes_membership_from_listings() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![("alice".to_string(), "uuid-alice".to_string())]);
    let mut test_storage = TestStorage::new();
    test_storage
        .storage
        .replace_team_tree(crate::storage::TeamTree {
            teams: vec![crate::storage::TeamEntry {
                uuid: "team-1".to_string(),
                name: "Team 1".to_string(),
                description: "Desc 1".to_string(),
                channels: Vec::new(),
            }],
        })
        .expect("seed tree should persist");

    let _ = handle_subscribe(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-1".to_string()],
    );

    let unsubscribe_outcome = handle_unsubscribe(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-1".to_string()],
    );
    assert_eq!(
        unsubscribe_outcome.response,
        "R200 \"UNSUBSCRIBED\" \"uuid-alice\" \"team-1\"\r\n"
    );

    let subscribed_teams_outcome = handle_subscribed(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[],
    );
    assert_eq!(subscribed_teams_outcome.response, "R200 \"TEAMS\"\r\n");
}

#[test]
fn subscribe_returns_not_found_for_unknown_team() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![("alice".to_string(), "uuid-alice".to_string())]);
    let mut test_storage = TestStorage::new();

    let outcome = handle_subscribe(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["unknown-team".to_string()],
    );

    assert_eq!(outcome.response, "R404 \"unknown-team\"\r\n");
}

#[test]
fn create_team_in_root_context_persists_team() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["Core Team".to_string(), "Core Description".to_string()],
    );

    assert!(outcome.response.starts_with("R200 \"TEAM\" "));
    let tree = test_storage.storage.team_tree();
    assert_eq!(tree.teams.len(), 1);
    assert_eq!(tree.teams[0].name, "Core Team");
    assert_eq!(tree.teams[0].description, "Core Description");
}

#[test]
fn create_channel_in_team_context_persists_channel() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let seed_tree = crate::storage::TeamTree {
        teams: vec![crate::storage::TeamEntry {
            uuid: "team-a".to_string(),
            name: "Team A".to_string(),
            description: "Desc A".to_string(),
            channels: Vec::new(),
        }],
    };
    test_storage
        .storage
        .replace_team_tree(seed_tree)
        .expect("seed tree should persist");

    let use_outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-a".to_string()],
    );
    assert_eq!(use_outcome.response, "R200\r\n");

    let outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["general".to_string(), "General channel".to_string()],
    );

    assert!(outcome.response.starts_with("R200 \"CHANNEL\" "));
    let team = test_storage
        .storage
        .team("team-a")
        .expect("team should exist after create channel");
    assert_eq!(team.channels.len(), 1);
    assert_eq!(team.channels[0].name, "general");
    assert_eq!(team.channels[0].description, "General channel");

    assert_eq!(outcome.info_events.len(), 0);
}

#[test]
fn create_channel_notifies_all_team_subscribers() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![
        ("alice".to_string(), "uuid-alice".to_string()),
        ("bob".to_string(), "uuid-bob".to_string()),
    ]);
    let mut test_storage = TestStorage::new();

    test_storage
        .storage
        .replace_team_tree(crate::storage::TeamTree {
            teams: vec![crate::storage::TeamEntry {
                uuid: "team-a".to_string(),
                name: "Team A".to_string(),
                description: "Desc A".to_string(),
                channels: Vec::new(),
            }],
        })
        .expect("seed tree should persist");

    users.subscribe_to_team("uuid-alice", "team-a");
    users.subscribe_to_team("uuid-bob", "team-a");

    let use_outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-a".to_string()],
    );
    assert_eq!(use_outcome.response, "R200\r\n");

    let outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["general".to_string(), "General channel".to_string()],
    );

    assert_eq!(outcome.info_events.len(), 2);
    assert_eq!(outcome.info_events[0].recipient_user_uuid, "uuid-alice");
    assert_eq!(outcome.info_events[1].recipient_user_uuid, "uuid-bob");
    assert!(outcome.info_events[0].payload.starts_with("I100 NEW_CHANNEL \"team-a\" "));
    assert_eq!(outcome.info_events[0].payload, outcome.info_events[1].payload);
}

#[test]
fn create_reply_in_thread_context_persists_reply() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let seed_tree = crate::storage::TeamTree {
        teams: vec![crate::storage::TeamEntry {
            uuid: "team-a".to_string(),
            name: "Team A".to_string(),
            description: "Desc A".to_string(),
            channels: vec![crate::storage::ChannelEntry {
                uuid: "chan-a".to_string(),
                name: "general".to_string(),
                description: "General".to_string(),
                threads: vec![crate::storage::ThreadEntry {
                    uuid: "thread-a".to_string(),
                    user_uuid: "uuid-bob".to_string(),
                    timestamp: 42,
                    title: "Topic".to_string(),
                    body: "Body".to_string(),
                    replies: Vec::new(),
                }],
            }],
        }],
    };
    test_storage
        .storage
        .replace_team_tree(seed_tree)
        .expect("seed tree should persist");

    let use_outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[
            "team-a".to_string(),
            "chan-a".to_string(),
            "thread-a".to_string(),
        ],
    );
    assert_eq!(use_outcome.response, "R200\r\n");

    let outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["reply body".to_string()],
    );

    assert!(outcome.response.starts_with("R200 \"REPLY\" "));
    let thread = test_storage
        .storage
        .thread("team-a", "chan-a", "thread-a")
        .expect("thread should exist");
    assert_eq!(thread.replies.len(), 1);
    assert_eq!(thread.replies[0].user_uuid, "uuid-alice");
    assert_eq!(thread.replies[0].body, "reply body");

    assert_eq!(outcome.info_events.len(), 0);
}

#[test]
fn create_thread_and_reply_notify_all_team_subscribers() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: Default::default(),
    };
    let registry = CommandMap::new();
    let mut users = UserStore::from_pairs(vec![
        ("alice".to_string(), "uuid-alice".to_string()),
        ("bob".to_string(), "uuid-bob".to_string()),
    ]);
    let mut test_storage = TestStorage::new();

    test_storage
        .storage
        .replace_team_tree(crate::storage::TeamTree {
            teams: vec![crate::storage::TeamEntry {
                uuid: "team-a".to_string(),
                name: "Team A".to_string(),
                description: "Desc A".to_string(),
                channels: vec![crate::storage::ChannelEntry {
                    uuid: "chan-a".to_string(),
                    name: "general".to_string(),
                    description: "General".to_string(),
                    threads: Vec::new(),
                }],
            }],
        })
        .expect("seed tree should persist");

    users.subscribe_to_team("uuid-alice", "team-a");
    users.subscribe_to_team("uuid-bob", "team-a");

    let use_channel_outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["team-a".to_string(), "chan-a".to_string()],
    );
    assert_eq!(use_channel_outcome.response, "R200\r\n");

    let thread_outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["Topic".to_string(), "Body".to_string()],
    );
    assert_eq!(thread_outcome.info_events.len(), 2);
    assert!(thread_outcome.info_events[0]
        .payload
        .starts_with("I100 NEW_THREAD \"team-a\" \"chan-a\" "));

    let created_thread_uuid = test_storage
        .storage
        .channel("team-a", "chan-a")
        .expect("channel should exist")
        .threads
        .first()
        .expect("thread should have been created")
        .uuid
        .clone();

    let use_thread_outcome = handle_use(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &[
            "team-a".to_string(),
            "chan-a".to_string(),
            created_thread_uuid,
        ],
    );
    assert_eq!(use_thread_outcome.response, "R200\r\n");

    let reply_outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["reply body".to_string()],
    );
    assert_eq!(reply_outcome.info_events.len(), 2);
    assert!(reply_outcome.info_events[0]
        .payload
        .starts_with("I100 NEW_REPLY \"team-a\" "));
}

#[test]
fn create_rejects_malformed_context() {
    let mut state = SessionState {
        user_uuid: Some("uuid-alice".to_string()),
        context: CommandContext {
            team_uuid: None,
            channel_uuid: Some("chan-a".to_string()),
            thread_uuid: None,
        },
    };
    let registry = CommandMap::new();
    let mut users = UserStore::default();
    let mut test_storage = TestStorage::new();

    let outcome = handle_create(
        &mut state,
        &registry,
        &mut users,
        &mut test_storage.storage,
        &["ignored".to_string(), "ignored".to_string()],
    );

    assert_eq!(outcome.response, "R501 \"bad request\"\r\n");
}
