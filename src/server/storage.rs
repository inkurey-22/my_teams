#![allow(dead_code)]

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use myteams_json::{
    read_json_value, stringify_json_value, write_json_value, JsonIoError, JsonObject, JsonValue,
};

#[derive(Debug, Clone)]
pub struct UserEntry {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Clone)]
pub struct MessageEntry {
    pub sender_uuid: String,
    pub recipient_uuid: String,
    pub timestamp: i64,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ReplyEntry {
    pub user_uuid: String,
    pub timestamp: i64,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ThreadEntry {
    pub uuid: String,
    pub user_uuid: String,
    pub timestamp: i64,
    pub title: String,
    pub body: String,
    pub replies: Vec<ReplyEntry>,
}

#[derive(Debug, Clone)]
pub struct ChannelEntry {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub threads: Vec<ThreadEntry>,
}

#[derive(Debug, Clone)]
pub struct TeamEntry {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub channels: Vec<ChannelEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct TeamTree {
    pub teams: Vec<TeamEntry>,
}

#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Json(JsonIoError),
    Schema(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(err) => write!(f, "storage I/O error: {err}"),
            StorageError::Json(err) => write!(f, "storage JSON error: {err}"),
            StorageError::Schema(msg) => write!(f, "storage schema error: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<io::Error> for StorageError {
    fn from(value: io::Error) -> Self {
        StorageError::Io(value)
    }
}

impl From<JsonIoError> for StorageError {
    fn from(value: JsonIoError) -> Self {
        StorageError::Json(value)
    }
}

pub struct ServerStorage {
    users_path: PathBuf,
    teams_path: PathBuf,
    messages_path: PathBuf,
    users: Vec<UserEntry>,
    team_tree: TeamTree,
    messages: Vec<MessageEntry>,
}

impl ServerStorage {
    pub fn load_or_default<P1, P2, P3>(
        users_path: P1,
        teams_path: P2,
        messages_path: P3,
    ) -> Result<Self, StorageError>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
    {
        let users_path = users_path.as_ref().to_path_buf();
        let teams_path = teams_path.as_ref().to_path_buf();
        let messages_path = messages_path.as_ref().to_path_buf();

        ensure_parent_dir(&users_path)?;
        ensure_parent_dir(&teams_path)?;
        ensure_parent_dir(&messages_path)?;

        let users = if users_path.exists() {
            parse_users_file(&users_path)?
        } else {
            Vec::new()
        };

        let team_tree = if teams_path.exists() {
            parse_teams_file(&teams_path)?
        } else {
            TeamTree::default()
        };

        let messages = if messages_path.exists() {
            parse_messages_file(&messages_path)?
        } else {
            Vec::new()
        };

        let storage = Self {
            users_path,
            teams_path,
            messages_path,
            users,
            team_tree,
            messages,
        };

        storage.flush_users()?;
        storage.flush_teams()?;
        storage.flush_messages()?;

        Ok(storage)
    }

    pub fn user_pairs(&self) -> Vec<(String, String)> {
        self.users
            .iter()
            .map(|u| (u.name.clone(), u.uuid.clone()))
            .collect()
    }

    pub fn upsert_user(&mut self, name: &str, uuid: &str) -> Result<(), StorageError> {
        if let Some(existing) = self.users.iter_mut().find(|u| u.name == name) {
            existing.uuid = uuid.to_string();
        } else {
            self.users.push(UserEntry {
                name: name.to_string(),
                uuid: uuid.to_string(),
            });
        }
        self.flush_users()
    }

    pub fn team_tree(&self) -> &TeamTree {
        &self.team_tree
    }

    pub fn team(&self, team_uuid: &str) -> Option<&TeamEntry> {
        self.team_tree.teams.iter().find(|team| team.uuid == team_uuid)
    }

    pub fn channel(&self, team_uuid: &str, channel_uuid: &str) -> Option<&ChannelEntry> {
        self.team(team_uuid)
            .and_then(|team| team.channels.iter().find(|channel| channel.uuid == channel_uuid))
    }

    pub fn thread(
        &self,
        team_uuid: &str,
        channel_uuid: &str,
        thread_uuid: &str,
    ) -> Option<&ThreadEntry> {
        self.channel(team_uuid, channel_uuid)
            .and_then(|channel| channel.threads.iter().find(|thread| thread.uuid == thread_uuid))
    }

    pub fn replace_team_tree(&mut self, team_tree: TeamTree) -> Result<(), StorageError> {
        self.team_tree = team_tree;
        self.flush_teams()
    }

    pub fn users_file(&self) -> &Path {
        &self.users_path
    }

    pub fn teams_file(&self) -> &Path {
        &self.teams_path
    }

    pub fn messages_file(&self) -> &Path {
        &self.messages_path
    }

    pub fn append_private_message(
        &mut self,
        sender_uuid: &str,
        recipient_uuid: &str,
        timestamp: i64,
        body: &str,
    ) -> Result<(), StorageError> {
        self.messages.push(MessageEntry {
            sender_uuid: sender_uuid.to_string(),
            recipient_uuid: recipient_uuid.to_string(),
            timestamp,
            body: body.to_string(),
        });

        self.flush_messages()
    }

    pub fn conversation_messages(&self, user_a: &str, user_b: &str) -> Vec<MessageEntry> {
        self.messages
            .iter()
            .filter(|message| {
                (message.sender_uuid == user_a && message.recipient_uuid == user_b)
                    || (message.sender_uuid == user_b && message.recipient_uuid == user_a)
            })
            .cloned()
            .collect()
    }

    fn flush_users(&self) -> Result<(), StorageError> {
        let users_json = users_to_json_value(&self.users);
        write_json_value(&self.users_path, &users_json).map_err(StorageError::from)
    }

    fn flush_teams(&self) -> Result<(), StorageError> {
        let teams_json = teams_to_json_value(&self.team_tree);
        write_json_value(&self.teams_path, &teams_json).map_err(StorageError::from)
    }

    fn flush_messages(&self) -> Result<(), StorageError> {
        let messages_json = messages_to_json_value(&self.messages);
        write_json_value(&self.messages_path, &messages_json).map_err(StorageError::from)
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn parse_users_file(path: &Path) -> Result<Vec<UserEntry>, StorageError> {
    let value = read_json_value(path).map_err(StorageError::from)?;
    users_from_json_value(&value)
}

fn parse_teams_file(path: &Path) -> Result<TeamTree, StorageError> {
    let value = read_json_value(path).map_err(StorageError::from)?;
    teams_from_json_value(&value)
}

fn parse_messages_file(path: &Path) -> Result<Vec<MessageEntry>, StorageError> {
    let value = read_json_value(path).map_err(StorageError::from)?;
    messages_from_json_value(&value)
}

fn users_to_json_value(users: &[UserEntry]) -> JsonValue {
    let mut root = JsonObject::new();
    let mut arr = Vec::with_capacity(users.len());
    for user in users {
        let mut item = JsonObject::new();
        item.insert("name".to_string(), JsonValue::String(user.name.clone()));
        item.insert("uuid".to_string(), JsonValue::String(user.uuid.clone()));
        arr.push(JsonValue::Object(item));
    }
    root.insert("users".to_string(), JsonValue::Array(arr));
    JsonValue::Object(root)
}

fn users_from_json_value(value: &JsonValue) -> Result<Vec<UserEntry>, StorageError> {
    let root = expect_object(value, "users file root")?;
    let users_value = root
        .get("users")
        .ok_or_else(|| StorageError::Schema("missing 'users' field".to_string()))?;
    let users_arr = expect_array(users_value, "users")?;

    let mut out = Vec::with_capacity(users_arr.len());
    for item in users_arr {
        let obj = expect_object(item, "users[] item")?;
        let name = expect_string_field(obj, "name")?.to_string();
        let uuid = expect_string_field(obj, "uuid")?.to_string();
        out.push(UserEntry { name, uuid });
    }
    Ok(out)
}

fn teams_to_json_value(tree: &TeamTree) -> JsonValue {
    let mut root = JsonObject::new();
    let teams = tree
        .teams
        .iter()
        .map(|team| {
            let mut team_obj = JsonObject::new();
            team_obj.insert("uuid".to_string(), JsonValue::String(team.uuid.clone()));
            team_obj.insert("name".to_string(), JsonValue::String(team.name.clone()));
            team_obj.insert(
                "description".to_string(),
                JsonValue::String(team.description.clone()),
            );

            let channels = team
                .channels
                .iter()
                .map(|channel| {
                    let mut channel_obj = JsonObject::new();
                    channel_obj.insert("uuid".to_string(), JsonValue::String(channel.uuid.clone()));
                    channel_obj.insert("name".to_string(), JsonValue::String(channel.name.clone()));
                    channel_obj.insert(
                        "description".to_string(),
                        JsonValue::String(channel.description.clone()),
                    );

                    let threads = channel
                        .threads
                        .iter()
                        .map(|thread| {
                            let mut thread_obj = JsonObject::new();
                            thread_obj
                                .insert("uuid".to_string(), JsonValue::String(thread.uuid.clone()));
                            thread_obj.insert(
                                "user_uuid".to_string(),
                                JsonValue::String(thread.user_uuid.clone()),
                            );
                            thread_obj.insert(
                                "timestamp".to_string(),
                                JsonValue::Number(thread.timestamp as f64),
                            );
                            thread_obj.insert(
                                "title".to_string(),
                                JsonValue::String(thread.title.clone()),
                            );
                            thread_obj
                                .insert("body".to_string(), JsonValue::String(thread.body.clone()));

                            let replies = thread
                                .replies
                                .iter()
                                .map(|reply| {
                                    let mut reply_obj = JsonObject::new();
                                    reply_obj.insert(
                                        "user_uuid".to_string(),
                                        JsonValue::String(reply.user_uuid.clone()),
                                    );
                                    reply_obj.insert(
                                        "timestamp".to_string(),
                                        JsonValue::Number(reply.timestamp as f64),
                                    );
                                    reply_obj.insert(
                                        "body".to_string(),
                                        JsonValue::String(reply.body.clone()),
                                    );
                                    JsonValue::Object(reply_obj)
                                })
                                .collect();

                            thread_obj.insert("replies".to_string(), JsonValue::Array(replies));
                            JsonValue::Object(thread_obj)
                        })
                        .collect();

                    channel_obj.insert("threads".to_string(), JsonValue::Array(threads));
                    JsonValue::Object(channel_obj)
                })
                .collect();

            team_obj.insert("channels".to_string(), JsonValue::Array(channels));
            JsonValue::Object(team_obj)
        })
        .collect();

    root.insert("teams".to_string(), JsonValue::Array(teams));
    JsonValue::Object(root)
}

fn teams_from_json_value(value: &JsonValue) -> Result<TeamTree, StorageError> {
    let root = expect_object(value, "teams file root")?;
    let teams_value = root
        .get("teams")
        .ok_or_else(|| StorageError::Schema("missing 'teams' field".to_string()))?;
    let teams_arr = expect_array(teams_value, "teams")?;

    let mut teams = Vec::with_capacity(teams_arr.len());
    for team_value in teams_arr {
        let team_obj = expect_object(team_value, "teams[] item")?;
        let team_uuid = expect_string_field(team_obj, "uuid")?.to_string();
        let team_name = expect_string_field(team_obj, "name")?.to_string();
        let team_description = expect_string_field_or_default(team_obj, "description")?.to_string();

        let channels_value = team_obj
            .get("channels")
            .ok_or_else(|| StorageError::Schema("missing 'channels' field".to_string()))?;
        let channels_arr = expect_array(channels_value, "channels")?;

        let mut channels = Vec::with_capacity(channels_arr.len());
        for channel_value in channels_arr {
            let channel_obj = expect_object(channel_value, "channels[] item")?;
            let channel_uuid = expect_string_field(channel_obj, "uuid")?.to_string();
            let channel_name = expect_string_field(channel_obj, "name")?.to_string();
            let channel_description =
                expect_string_field_or_default(channel_obj, "description")?.to_string();

            let threads_value = channel_obj
                .get("threads")
                .ok_or_else(|| StorageError::Schema("missing 'threads' field".to_string()))?;
            let threads_arr = expect_array(threads_value, "threads")?;

            let mut threads = Vec::with_capacity(threads_arr.len());
            for thread_value in threads_arr {
                let thread_obj = expect_object(thread_value, "threads[] item")?;
                let uuid = expect_string_field(thread_obj, "uuid")?.to_string();
                let user_uuid = expect_string_field_or_default(thread_obj, "user_uuid")?.to_string();
                let timestamp = expect_i64_field_or_default(thread_obj, "timestamp")?;
                let title = expect_string_field(thread_obj, "title")?.to_string();
                let body = expect_string_field(thread_obj, "body")?.to_string();

                let replies = match thread_obj.get("replies") {
                    Some(value) => {
                        let replies_arr = expect_array(value, "replies")?;
                        let mut replies = Vec::with_capacity(replies_arr.len());

                        for reply_value in replies_arr {
                            let reply_obj = expect_object(reply_value, "replies[] item")?;
                            let user_uuid =
                                expect_string_field(reply_obj, "user_uuid")?.to_string();
                            let timestamp = expect_i64_field_or_default(reply_obj, "timestamp")?;
                            let body = expect_string_field(reply_obj, "body")?.to_string();
                            replies.push(ReplyEntry {
                                user_uuid,
                                timestamp,
                                body,
                            });
                        }

                        replies
                    }
                    None => Vec::new(),
                };

                threads.push(ThreadEntry {
                    uuid,
                    user_uuid,
                    timestamp,
                    title,
                    body,
                    replies,
                });
            }

            channels.push(ChannelEntry {
                uuid: channel_uuid,
                name: channel_name,
                description: channel_description,
                threads,
            });
        }

        teams.push(TeamEntry {
            uuid: team_uuid,
            name: team_name,
            description: team_description,
            channels,
        });
    }

    Ok(TeamTree { teams })
}

fn messages_to_json_value(messages: &[MessageEntry]) -> JsonValue {
    let mut root = JsonObject::new();
    let mut arr = Vec::with_capacity(messages.len());

    for message in messages {
        let mut item = JsonObject::new();
        item.insert(
            "sender_uuid".to_string(),
            JsonValue::String(message.sender_uuid.clone()),
        );
        item.insert(
            "recipient_uuid".to_string(),
            JsonValue::String(message.recipient_uuid.clone()),
        );
        item.insert(
            "timestamp".to_string(),
            JsonValue::Number(message.timestamp as f64),
        );
        item.insert("body".to_string(), JsonValue::String(message.body.clone()));
        arr.push(JsonValue::Object(item));
    }

    root.insert("messages".to_string(), JsonValue::Array(arr));
    JsonValue::Object(root)
}

fn messages_from_json_value(value: &JsonValue) -> Result<Vec<MessageEntry>, StorageError> {
    let root = expect_object(value, "messages file root")?;
    let messages_value = root
        .get("messages")
        .ok_or_else(|| StorageError::Schema("missing 'messages' field".to_string()))?;
    let messages_arr = expect_array(messages_value, "messages")?;

    let mut out = Vec::with_capacity(messages_arr.len());
    for item in messages_arr {
        let obj = expect_object(item, "messages[] item")?;
        let sender_uuid = expect_string_field(obj, "sender_uuid")?.to_string();
        let recipient_uuid = expect_string_field(obj, "recipient_uuid")?.to_string();
        let timestamp = expect_i64_field(obj, "timestamp")?;
        let body = expect_string_field(obj, "body")?.to_string();

        out.push(MessageEntry {
            sender_uuid,
            recipient_uuid,
            timestamp,
            body,
        });
    }

    Ok(out)
}

fn expect_object<'a>(value: &'a JsonValue, ctx: &str) -> Result<&'a JsonObject, StorageError> {
    match value {
        JsonValue::Object(obj) => Ok(obj),
        _ => Err(StorageError::Schema(format!("{ctx} is not an object"))),
    }
}

fn expect_array<'a>(value: &'a JsonValue, ctx: &str) -> Result<&'a [JsonValue], StorageError> {
    match value {
        JsonValue::Array(arr) => Ok(arr.as_slice()),
        _ => Err(StorageError::Schema(format!("{ctx} is not an array"))),
    }
}

fn expect_string_field<'a>(obj: &'a JsonObject, field: &str) -> Result<&'a str, StorageError> {
    let value = obj
        .get(field)
        .ok_or_else(|| StorageError::Schema(format!("missing '{field}' field")))?;

    match value {
        JsonValue::String(s) => Ok(s),
        _ => Err(StorageError::Schema(format!(
            "field '{field}' is not a string"
        ))),
    }
}

fn expect_string_field_or_default<'a>(
    obj: &'a JsonObject,
    field: &str,
) -> Result<&'a str, StorageError> {
    match obj.get(field) {
        Some(JsonValue::String(value)) => Ok(value),
        Some(_) => Err(StorageError::Schema(format!(
            "field '{field}' is not a string"
        ))),
        None => Ok(""),
    }
}

fn expect_i64_field(obj: &JsonObject, field: &str) -> Result<i64, StorageError> {
    let value = obj
        .get(field)
        .ok_or_else(|| StorageError::Schema(format!("missing '{field}' field")))?;

    match value {
        JsonValue::Number(n) if n.is_finite() && n.fract() == 0.0 => {
            if *n < i64::MIN as f64 || *n > i64::MAX as f64 {
                return Err(StorageError::Schema(format!(
                    "field '{field}' is out of i64 range"
                )));
            }
            Ok(*n as i64)
        }
        _ => Err(StorageError::Schema(format!(
            "field '{field}' is not an integer number"
        ))),
    }
}

fn expect_i64_field_or_default(obj: &JsonObject, field: &str) -> Result<i64, StorageError> {
    match obj.get(field) {
        Some(JsonValue::Number(n)) if n.is_finite() && n.fract() == 0.0 => {
            if *n < i64::MIN as f64 || *n > i64::MAX as f64 {
                return Err(StorageError::Schema(format!(
                    "field '{field}' is out of i64 range"
                )));
            }
            Ok(*n as i64)
        }
        Some(_) => Err(StorageError::Schema(format!(
            "field '{field}' is not an integer number"
        ))),
        None => Ok(0),
    }
}

pub fn default_users_path() -> &'static str {
    "data/users.json"
}

pub fn default_teams_path() -> &'static str {
    "data/teams.json"
}

pub fn default_messages_path() -> &'static str {
    "data/messages.json"
}

pub fn dump_team_tree(tree: &TeamTree) -> String {
    stringify_json_value(&teams_to_json_value(tree))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestPaths {
        root: PathBuf,
        users: PathBuf,
        teams: PathBuf,
        messages: PathBuf,
    }

    impl TestPaths {
        fn new() -> Self {
            let unique = format!(
                "my_teams_storage_{}_{}",
                process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            );

            let root = std::env::temp_dir().join(unique);
            Self {
                users: root.join("users.json"),
                teams: root.join("teams.json"),
                messages: root.join("messages.json"),
                root,
            }
        }
    }

    impl Drop for TestPaths {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn append_private_message_is_persisted_on_disk() {
        let paths = TestPaths::new();
        let mut storage = ServerStorage::load_or_default(&paths.users, &paths.teams, &paths.messages)
            .expect("storage should initialize");

        storage
            .append_private_message("uuid-alice", "uuid-bob", 1234, "hello")
            .expect("message should be persisted");

        let reloaded = ServerStorage::load_or_default(&paths.users, &paths.teams, &paths.messages)
            .expect("storage should reload");
        let messages = reloaded.conversation_messages("uuid-alice", "uuid-bob");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].sender_uuid, "uuid-alice");
        assert_eq!(messages[0].recipient_uuid, "uuid-bob");
        assert_eq!(messages[0].timestamp, 1234);
        assert_eq!(messages[0].body, "hello");
    }

    #[test]
    fn conversation_messages_only_returns_matching_pair() {
        let paths = TestPaths::new();
        let mut storage = ServerStorage::load_or_default(&paths.users, &paths.teams, &paths.messages)
            .expect("storage should initialize");

        storage
            .append_private_message("uuid-a", "uuid-b", 1, "a->b")
            .expect("message should be persisted");
        storage
            .append_private_message("uuid-b", "uuid-a", 2, "b->a")
            .expect("message should be persisted");
        storage
            .append_private_message("uuid-a", "uuid-c", 3, "a->c")
            .expect("message should be persisted");

        let conversation = storage.conversation_messages("uuid-a", "uuid-b");
        assert_eq!(conversation.len(), 2);
        assert_eq!(conversation[0].body, "a->b");
        assert_eq!(conversation[1].body, "b->a");
    }
}
