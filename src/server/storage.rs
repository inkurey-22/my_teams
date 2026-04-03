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
pub struct ThreadEntry {
    pub uuid: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ChannelEntry {
    pub uuid: String,
    pub name: String,
    pub threads: Vec<ThreadEntry>,
}

#[derive(Debug, Clone)]
pub struct TeamEntry {
    pub uuid: String,
    pub name: String,
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
    users: Vec<UserEntry>,
    team_tree: TeamTree,
}

impl ServerStorage {
    pub fn load_or_default<P1, P2>(users_path: P1, teams_path: P2) -> Result<Self, StorageError>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let users_path = users_path.as_ref().to_path_buf();
        let teams_path = teams_path.as_ref().to_path_buf();

        ensure_parent_dir(&users_path)?;
        ensure_parent_dir(&teams_path)?;

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

        let storage = Self {
            users_path,
            teams_path,
            users,
            team_tree,
        };

        storage.flush_users()?;
        storage.flush_teams()?;

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

    fn flush_users(&self) -> Result<(), StorageError> {
        let users_json = users_to_json_value(&self.users);
        write_json_value(&self.users_path, &users_json).map_err(StorageError::from)
    }

    fn flush_teams(&self) -> Result<(), StorageError> {
        let teams_json = teams_to_json_value(&self.team_tree);
        write_json_value(&self.teams_path, &teams_json).map_err(StorageError::from)
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

            let channels = team
                .channels
                .iter()
                .map(|channel| {
                    let mut channel_obj = JsonObject::new();
                    channel_obj.insert("uuid".to_string(), JsonValue::String(channel.uuid.clone()));
                    channel_obj.insert("name".to_string(), JsonValue::String(channel.name.clone()));

                    let threads = channel
                        .threads
                        .iter()
                        .map(|thread| {
                            let mut thread_obj = JsonObject::new();
                            thread_obj
                                .insert("uuid".to_string(), JsonValue::String(thread.uuid.clone()));
                            thread_obj.insert(
                                "title".to_string(),
                                JsonValue::String(thread.title.clone()),
                            );
                            thread_obj
                                .insert("body".to_string(), JsonValue::String(thread.body.clone()));
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

        let channels_value = team_obj
            .get("channels")
            .ok_or_else(|| StorageError::Schema("missing 'channels' field".to_string()))?;
        let channels_arr = expect_array(channels_value, "channels")?;

        let mut channels = Vec::with_capacity(channels_arr.len());
        for channel_value in channels_arr {
            let channel_obj = expect_object(channel_value, "channels[] item")?;
            let channel_uuid = expect_string_field(channel_obj, "uuid")?.to_string();
            let channel_name = expect_string_field(channel_obj, "name")?.to_string();

            let threads_value = channel_obj
                .get("threads")
                .ok_or_else(|| StorageError::Schema("missing 'threads' field".to_string()))?;
            let threads_arr = expect_array(threads_value, "threads")?;

            let mut threads = Vec::with_capacity(threads_arr.len());
            for thread_value in threads_arr {
                let thread_obj = expect_object(thread_value, "threads[] item")?;
                let uuid = expect_string_field(thread_obj, "uuid")?.to_string();
                let title = expect_string_field(thread_obj, "title")?.to_string();
                let body = expect_string_field(thread_obj, "body")?.to_string();
                threads.push(ThreadEntry { uuid, title, body });
            }

            channels.push(ChannelEntry {
                uuid: channel_uuid,
                name: channel_name,
                threads,
            });
        }

        teams.push(TeamEntry {
            uuid: team_uuid,
            name: team_name,
            channels,
        });
    }

    Ok(TeamTree { teams })
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

pub fn default_users_path() -> &'static str {
    "data/users.json"
}

pub fn default_teams_path() -> &'static str {
    "data/teams.json"
}

pub fn dump_team_tree(tree: &TeamTree) -> String {
    stringify_json_value(&teams_to_json_value(tree))
}
