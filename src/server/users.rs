use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
pub struct UserStore {
    by_name: HashMap<String, String>,
    sequence: u64,
}

impl UserStore {
    pub fn from_pairs(pairs: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut by_name = HashMap::new();
        for (name, uuid) in pairs {
            by_name.insert(name, uuid);
        }

        Self {
            by_name,
            sequence: 0,
        }
    }

    pub fn login(&mut self, user_name: &str) -> (String, bool) {
        if let Some(existing) = self.by_name.get(user_name) {
            return (existing.clone(), false);
        }

        self.sequence = self.sequence.wrapping_add(1);
        let uuid = make_uuid_v4_like(user_name, self.sequence);
        self.by_name.insert(user_name.to_string(), uuid.clone());
        (uuid, true)
    }

    pub fn exists_uuid(&self, user_uuid: &str) -> bool {
        self.by_name.values().any(|uuid| uuid == user_uuid)
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
