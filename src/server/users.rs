use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// In-memory view of users and their online state.
#[derive(Default)]
pub struct UserStore {
    by_name: HashMap<String, String>,
    by_uuid: HashMap<String, String>,
    online_sessions: HashMap<String, usize>,
    team_subscribers: HashMap<String, HashSet<String>>,
    user_subscriptions: HashMap<String, HashSet<String>>,
    sequence: u64,
}

impl UserStore {
    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut by_name = HashMap::new();
        let mut by_uuid = HashMap::new();
        for (name, uuid) in pairs {
            by_uuid.insert(uuid.clone(), name.clone());
            by_name.insert(name, uuid);
        }

        Self {
            by_name,
            by_uuid,
            online_sessions: HashMap::new(),
            team_subscribers: HashMap::new(),
            user_subscriptions: HashMap::new(),
            sequence: 0,
        }
    }

    pub fn login(&mut self, user_name: &str) -> (String, bool) {
        if let Some(existing) = self.by_name.get(user_name) {
            let existing_uuid = existing.clone();
            self.bump_online(&existing_uuid);
            return (existing_uuid, false);
        }

        self.sequence = self.sequence.wrapping_add(1);
        let uuid = make_uuid_v4_like(user_name, self.sequence);
        self.by_name.insert(user_name.to_string(), uuid.clone());
        self.by_uuid.insert(uuid.clone(), user_name.to_string());
        self.bump_online(&uuid);
        (uuid, true)
    }

    pub fn exists_uuid(&self, user_uuid: &str) -> bool {
        self.by_uuid.contains_key(user_uuid)
    }

    pub fn logout(&mut self, user_uuid: &str) {
        let mut should_remove = false;

        if let Some(count) = self.online_sessions.get_mut(user_uuid) {
            if *count <= 1 {
                should_remove = true;
            } else {
                *count -= 1;
            }
        }

        if should_remove {
            self.online_sessions.remove(user_uuid);
        }
    }

    pub fn list_users(&self) -> Vec<(String, String, bool)> {
        let mut users = self
            .by_uuid
            .iter()
            .map(|(uuid, name)| (uuid.clone(), name.clone(), self.is_online(uuid)))
            .collect::<Vec<_>>();

        users.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        users
    }

    pub fn user_details(&self, user_uuid: &str) -> Option<(String, bool)> {
        self.by_uuid
            .get(user_uuid)
            .map(|name| (name.clone(), self.is_online(user_uuid)))
    }

    pub fn subscribe_to_team(&mut self, user_uuid: &str, team_uuid: &str) {
        self.team_subscribers
            .entry(team_uuid.to_string())
            .or_default()
            .insert(user_uuid.to_string());

        self.user_subscriptions
            .entry(user_uuid.to_string())
            .or_default()
            .insert(team_uuid.to_string());
    }

    pub fn unsubscribe_from_team(&mut self, user_uuid: &str, team_uuid: &str) {
        if let Some(subscribers) = self.team_subscribers.get_mut(team_uuid) {
            subscribers.remove(user_uuid);
            if subscribers.is_empty() {
                self.team_subscribers.remove(team_uuid);
            }
        }

        if let Some(teams) = self.user_subscriptions.get_mut(user_uuid) {
            teams.remove(team_uuid);
            if teams.is_empty() {
                self.user_subscriptions.remove(user_uuid);
            }
        }
    }

    pub fn subscribed_team_ids(&self, user_uuid: &str) -> Vec<String> {
        let mut teams = self
            .user_subscriptions
            .get(user_uuid)
            .map(|values| values.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        teams.sort();
        teams
    }

    pub fn subscribed_user_ids(&self, team_uuid: &str) -> Vec<String> {
        let mut users = self
            .team_subscribers
            .get(team_uuid)
            .map(|values| values.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        users.sort();
        users
    }

    fn bump_online(&mut self, user_uuid: &str) {
        let counter = self
            .online_sessions
            .entry(user_uuid.to_string())
            .or_insert(0);
        *counter += 1;
    }

    fn is_online(&self, user_uuid: &str) -> bool {
        self.online_sessions
            .get(user_uuid)
            .is_some_and(|count| *count > 0)
    }
}

fn make_uuid_v4_like(user_name: &str, sequence: u64) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut h1 = DefaultHasher::new();
    user_name.hash(&mut h1);
    nanos.hash(&mut h1);
    sequence.hash(&mut h1);
    let p1 = h1.finish();

    let mut h2 = DefaultHasher::new();
    p1.hash(&mut h2);
    nanos.rotate_left(17).hash(&mut h2);
    sequence.rotate_left(9).hash(&mut h2);
    let p2 = h2.finish();

    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&p1.to_be_bytes());
    bytes[8..].copy_from_slice(&p2.to_be_bytes());

    // UUIDv4 layout: version=4 and RFC4122 variant bits.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_creates_new_user_and_is_marked_as_new() {
        let mut store = UserStore::default();
        let (uuid, is_new) = store.login("alice");

        assert!(is_new);
        assert!(!uuid.is_empty());
        assert!(store.exists_uuid(&uuid));
    }

    #[test]
    fn login_existing_user_returns_same_uuid_and_not_new() {
        let mut store = UserStore::default();
        let (uuid1, is_new1) = store.login("alice");
        let (uuid2, is_new2) = store.login("alice");

        assert!(is_new1);
        assert!(!is_new2);
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn login_increments_online_sessions() {
        let mut store = UserStore::default();
        let (uuid, _) = store.login("alice");
        assert!(store.is_online(&uuid));

        let (uuid2, _) = store.login("alice");
        assert_eq!(uuid, uuid2);
        assert!(store.is_online(&uuid));
    }

    #[test]
    fn exists_uuid_returns_true_for_existing_user() {
        let mut store = UserStore::default();
        let (uuid, _) = store.login("alice");
        assert!(store.exists_uuid(&uuid));
        assert!(!store.exists_uuid("nonexistent-uuid"));
    }

    #[test]
    fn logout_removes_online_status_after_last_session() {
        let mut store = UserStore::default();
        let (uuid, _) = store.login("alice");
        assert!(store.is_online(&uuid));

        store.logout(&uuid);
        assert!(!store.is_online(&uuid));
    }

    #[test]
    fn logout_decrements_but_keeps_online_with_multiple_sessions() {
        let mut store = UserStore::default();
        let (uuid, _) = store.login("alice");
        store.login("alice");
        assert!(store.is_online(&uuid));

        store.logout(&uuid);
        assert!(store.is_online(&uuid));
        store.logout(&uuid);
        assert!(!store.is_online(&uuid));
    }

    #[test]
    fn list_users_returns_all_users_sorted_by_name() {
        let mut store = UserStore::default();
        let (_uuid_charlie, _) = store.login("charlie");
        let (_uuid_alice, _) = store.login("alice");
        let (_uuid_bob, _) = store.login("bob");

        let users = store.list_users();
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].1, "alice");
        assert_eq!(users[1].1, "bob");
        assert_eq!(users[2].1, "charlie");
    }

    #[test]
    fn list_users_includes_online_status() {
        let mut store = UserStore::default();
        let (_uuid_alice, _) = store.login("alice");
        let (uuid_bob, _) = store.login("bob");
        store.logout(&uuid_bob);

        let users = store.list_users();
        let alice = users.iter().find(|u| u.1 == "alice").unwrap();
        let bob = users.iter().find(|u| u.1 == "bob").unwrap();

        assert!(alice.2);
        assert!(!bob.2);
    }

    #[test]
    fn user_details_returns_name_and_online_status() {
        let mut store = UserStore::default();
        let (uuid, _) = store.login("alice");

        let (name, online) = store.user_details(&uuid).unwrap();
        assert_eq!(name, "alice");
        assert!(online);

        store.logout(&uuid);
        let (name, online) = store.user_details(&uuid).unwrap();
        assert_eq!(name, "alice");
        assert!(!online);
    }

    #[test]
    fn subscribe_to_team_creates_bidirectional_subscription() {
        let mut store = UserStore::default();
        let (user_uuid, _) = store.login("alice");
        let team_uuid = "team-123";

        store.subscribe_to_team(&user_uuid, team_uuid);

        assert_eq!(store.subscribed_team_ids(&user_uuid), vec![team_uuid]);
        assert_eq!(store.subscribed_user_ids(team_uuid), vec![user_uuid]);
    }

    #[test]
    fn subscribe_multiple_users_to_same_team() {
        let mut store = UserStore::default();
        let (user1, _) = store.login("alice");
        let (user2, _) = store.login("bob");
        let team_uuid = "team-123";

        store.subscribe_to_team(&user1, team_uuid);
        store.subscribe_to_team(&user2, team_uuid);

        let mut subscribers = store.subscribed_user_ids(team_uuid);
        subscribers.sort();
        assert_eq!(subscribers.len(), 2);
    }

    #[test]
    fn unsubscribe_from_team_removes_subscription() {
        let mut store = UserStore::default();
        let (user_uuid, _) = store.login("alice");
        let team_uuid = "team-123";

        store.subscribe_to_team(&user_uuid, team_uuid);
        assert_eq!(store.subscribed_team_ids(&user_uuid).len(), 1);

        store.unsubscribe_from_team(&user_uuid, team_uuid);
        assert_eq!(store.subscribed_team_ids(&user_uuid).len(), 0);
        assert_eq!(store.subscribed_user_ids(team_uuid).len(), 0);
    }

    #[test]
    fn subscribed_team_ids_returns_sorted_list() {
        let mut store = UserStore::default();
        let (user_uuid, _) = store.login("alice");

        store.subscribe_to_team(&user_uuid, "team-c");
        store.subscribe_to_team(&user_uuid, "team-a");
        store.subscribe_to_team(&user_uuid, "team-b");

        let teams = store.subscribed_team_ids(&user_uuid);
        assert_eq!(teams, vec!["team-a", "team-b", "team-c"]);
    }

    #[test]
    fn from_pairs_creates_store_with_initial_users() {
        let pairs = vec![
            ("alice".to_string(), "uuid-alice".to_string()),
            ("bob".to_string(), "uuid-bob".to_string()),
        ];
        let store = UserStore::from_pairs(pairs);

        assert!(store.exists_uuid("uuid-alice"));
        assert!(store.exists_uuid("uuid-bob"));
        let users = store.list_users();
        assert_eq!(users.len(), 2);
    }
}
