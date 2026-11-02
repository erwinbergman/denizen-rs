//! Flags: persistent per-object key/value storage.

use crate::obj::{ElementTag, ListTag, MapTag, ObjectTag};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::collections::HashMap;

/// A flag storage associated with a flaggable object (identified by a key).
#[derive(Clone, Default)]
pub struct FlagTracker {
    pub flags: Arc<RwLock<MapTag>>,
}

impl FlagTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<ObjectTag> {
        let flags = self.flags.read().unwrap();
        flags.get_deep(&key.to_lowercase())
    }

    pub fn set(&self, key: &str, value: Option<ObjectTag>) {
        let mut flags = self.flags.write().unwrap();
        flags.put_deep(&key.to_lowercase(), value);
    }

    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn remove(&self, key: &str) {
        self.set(key, None);
    }

    /// Remove every flag.
    pub fn clear(&self) {
        self.flags.write().unwrap().entries.clear();
    }

    /// Set a flag with an expiry timestamp (milliseconds since epoch).
    pub fn set_with_expiry(&self, key: &str, value: ObjectTag, expires_millis: i64) {
        let wrapped = flag_value_with_expiry(Some(expires_millis), value, &crate::tags::TagContext::default());
        self.set(key, Some(wrapped));
    }

    /// Get the expiry timestamp of a flag, if it is a duration-wrapped value.
    pub fn get_expiration_time(&self, key: &str) -> Option<i64> {
        let v = self.get(key)?;
        let m = v.to_map()?;
        let data = m.get("data")?.to_map()?;
        data.get("expire")?.as_element().value.parse::<i64>().ok()
    }

    /// List flags, optionally filtered by a path prefix.
    pub fn list(&self, prefix: Option<&str>) -> ListTag {
        let flags = self.flags.read().unwrap();
        let mut out = ListTag::new();
        for (k, _) in &flags.entries {
            match prefix {
                Some(p) if !k.starts_with(p) => continue,
                _ => { out.add_string(k); }
            }
        }
        out
    }

    /// Expire flags that have a duration wrapper `map@[...]` with an `expire` key.
    pub fn expire(&self, now_millis: i64) -> usize {
        let mut removed = 0;
        let mut flags = self.flags.write().unwrap();
        let keys: Vec<String> = flags.entries.iter().map(|(k, _)| k.clone()).collect();
        for key in keys {
            if let Some(v) = flags.get(&key).cloned() {
                if let Some(m) = v.to_map() {
                    if let Some(exp) = m.get("expire") {
                        if let Ok(expires) = exp.as_element().value.parse::<i64>() {
                            if expires <= now_millis {
                                flags.entries.retain(|(k, _)| k != &key);
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }
        removed
    }

    pub fn to_map(&self) -> MapTag {
        self.flags.read().unwrap().clone()
    }

    pub fn load_from_map(&self, map: MapTag) {
        *self.flags.write().unwrap() = map;
    }
}

/// A store of flag trackers keyed by object identifier (eg `p@uuid`, `l@x,y,z,world`).
#[derive(Default)]
pub struct FlagStore {
    trackers: RwLock<HashMap<String, FlagTracker>>,
}

impl FlagStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_tracker(&self, key: &str) -> FlagTracker {
        let key = key.to_lowercase();
        if let Some(t) = self.trackers.read().unwrap().get(&key) {
            return t.clone();
        }
        let mut w = self.trackers.write().unwrap();
        w.entry(key).or_insert_with(FlagTracker::new).clone()
    }

    pub fn has_tracker(&self, key: &str) -> bool {
        self.trackers.read().unwrap().contains_key(&key.to_lowercase())
    }

    pub fn remove_tracker(&self, key: &str) {
        self.trackers.write().unwrap().remove(&key.to_lowercase());
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.trackers.read().unwrap().keys().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.trackers.read().unwrap().len()
    }

    /// Save all trackers to a directory, one file per key.
    pub fn save_to_dir(&self, dir: &Path) -> std::io::Result<usize> {
        std::fs::create_dir_all(dir)?;
        let trackers = self.trackers.read().unwrap();
        let mut count = 0;
        for (key, tracker) in trackers.iter() {
            let safe = sanitize_filename(key);
            let path = dir.join(format!("{}.yml", safe));
            let map = tracker.to_map();
            std::fs::write(path, map.to_identify())?;
            count += 1;
        }
        Ok(count)
    }

    /// Load trackers from a directory.
    pub fn load_from_dir(&self, dir: &Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yml") {
                    if let Ok(text) = std::fs::read_to_string(&path) {
                        if let Some(map) = MapTag::from_identify(&text) {
                            let key = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                            self.get_tracker(&key).load_from_map(map);
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }
}

fn sanitize_filename(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

/// Global server-level flags.
pub static SERVER_FLAGS: LazyLock<Mutex<FlagTracker>> = LazyLock::new(|| Mutex::new(FlagTracker::new()));

/// Global player flag store.
pub static PLAYER_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Global entity flag store.
pub static ENTITY_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Global location/block flag store.
pub static LOCATION_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Global world flag store.
pub static WORLD_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Global inventory flag store.
pub static INVENTORY_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Global NPC flag store.
pub static NPC_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Global item flag store.
pub static ITEM_FLAGS: LazyLock<FlagStore> = LazyLock::new(FlagStore::new);

/// Resolve a flag tracker by object identifier.
pub fn tracker_for(object_id: &str) -> FlagTracker {
    let lower = object_id.to_lowercase();
    if lower.starts_with("p@") {
        PLAYER_FLAGS.get_tracker(&lower)
    } else if lower.starts_with("e@") {
        ENTITY_FLAGS.get_tracker(&lower)
    } else if lower.starts_with("n@") {
        NPC_FLAGS.get_tracker(&lower)
    } else if lower.starts_with("l@") {
        LOCATION_FLAGS.get_tracker(&lower)
    } else if lower.starts_with("w@") {
        WORLD_FLAGS.get_tracker(&lower)
    } else if lower.starts_with("in@") {
        INVENTORY_FLAGS.get_tracker(&lower)
    } else if lower.starts_with("i@") {
        ITEM_FLAGS.get_tracker(&lower)
    } else if lower == "server" {
        SERVER_FLAGS.lock().unwrap().clone()
    } else {
        PLAYER_FLAGS.get_tracker(&lower)
    }
}

/// Produce a flag value, honoring duration wrappers.
pub fn flag_value_with_expiry(expires_millis: Option<i64>, value: ObjectTag, context: &crate::tags::TagContext) -> ObjectTag {
    match expires_millis {
        None => value,
        Some(exp) => {
            let mut m = MapTag::new();
            m.put_string("value", value);
            let mut d = MapTag::new();
            d.put_element("expire", &exp.to_string());
            m.put_string("__fmt", ObjectTag::new(ElementTag::new("duration")));
            m.put_string("data", ObjectTag::new(d));
            let _ = context;
            ObjectTag::new(m)
        }
    }
}

/// Expire all flags across every store. Returns the total number removed.
pub fn expire_all(now_millis: i64) -> usize {
    let mut removed = 0;
    removed += SERVER_FLAGS.lock().unwrap().expire(now_millis);
    for store in [&*PLAYER_FLAGS, &*ENTITY_FLAGS, &*NPC_FLAGS, &*LOCATION_FLAGS, &*WORLD_FLAGS, &*INVENTORY_FLAGS, &*ITEM_FLAGS] {
        for key in store.all_keys() {
            removed += store.get_tracker(&key).expire(now_millis);
        }
    }
    removed
}

#[allow(dead_code)]
fn _unused(_e: ElementTag) {}
