//! `PlayerTag`: a server player, identified as `p@<uuid>` or `p@<name>`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Mutable state for a player.
#[derive(Clone, Debug)]
pub struct PlayerData {
    pub uuid: String,
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f64,
    pub max_health: f64,
    pub food_level: i64,
    pub gamemode: String,
    pub online: bool,
    pub op: bool,
    pub banned: bool,
    pub whitelisted: bool,
    pub permissions: Vec<String>,
    pub groups: Vec<String>,
    pub chat_prefix: String,
    pub chat_suffix: String,
    pub first_played: i64,
    pub last_played: i64,
    pub inventory: Vec<Option<crate::obj::core::item::ItemTag>>,
}

impl Default for PlayerData {
    fn default() -> Self {
        PlayerData {
            uuid: String::new(),
            name: String::new(),
            world: "world".to_string(),
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            health: 20.0,
            max_health: 20.0,
            food_level: 20,
            gamemode: "survival".to_string(),
            online: false,
            op: false,
            banned: false,
            whitelisted: true,
            permissions: Vec::new(),
            groups: Vec::new(),
            chat_prefix: String::new(),
            chat_suffix: String::new(),
            first_played: 0,
            last_played: 0,
            inventory: vec![None; 36],
        }
    }
}

static PLAYERS: LazyLock<RwLock<HashMap<String, PlayerData>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register or update a player (keyed by uuid; name lookup supported).
pub fn register_player(mut data: PlayerData) -> String {
    if data.uuid.is_empty() {
        data.uuid = uuid::Uuid::new_v4().to_string();
    }
    let uuid = data.uuid.clone();
    PLAYERS.write().unwrap().insert(uuid.clone(), data);
    uuid
}

/// Look up a player by uuid.
pub fn get_player(uuid: &str) -> Option<PlayerData> {
    PLAYERS.read().unwrap().get(uuid).cloned()
}

/// Look up a player by uuid or name.
pub fn get_player_by_name_or_uuid(key: &str) -> Option<PlayerData> {
    let map = PLAYERS.read().unwrap();
    if let Some(p) = map.get(key) {
        return Some(p.clone());
    }
    let lower = key.to_lowercase();
    map.values().find(|p| p.name.to_lowercase() == lower).cloned()
}

/// Mutate player data.
pub fn update_player(uuid: &str, f: impl FnOnce(&mut PlayerData)) -> bool {
    let mut map = PLAYERS.write().unwrap();
    match map.get_mut(uuid) {
        Some(d) => {
            f(d);
            true
        }
        None => false,
    }
}

/// Players in a world.
pub fn players_in_world(world: &str) -> ListTag {
    let w = world.to_lowercase();
    let items: Vec<ObjectTag> = PLAYERS
        .read()
        .unwrap()
        .values()
        .filter(|p| p.world.to_lowercase() == w)
        .map(|p| ObjectTag::new(PlayerTag::from_uuid(&p.uuid)))
        .collect();
    ListTag::from_objects(items)
}

/// Players within a radius of a location.
pub fn players_within(loc: &crate::obj::core::location::LocationTag, radius: f64) -> ListTag {
    let items: Vec<ObjectTag> = PLAYERS
        .read()
        .unwrap()
        .values()
        .filter(|p| {
            let same_world = match &loc.world {
                Some(w) => w.to_lowercase() == p.world.to_lowercase(),
                None => true,
            };
            let dist = ((p.x - loc.x).powi(2) + (p.y - loc.y).powi(2) + (p.z - loc.z).powi(2)).sqrt();
            same_world && dist <= radius
        })
        .map(|p| ObjectTag::new(PlayerTag::from_uuid(&p.uuid)))
        .collect();
    ListTag::from_objects(items)
}

/// All known player names.
pub fn all_player_names() -> Vec<String> {
    PLAYERS.read().unwrap().values().map(|p| p.name.clone()).collect()
}

/// A player object.
#[derive(Clone, Debug)]
pub struct PlayerTag {
    pub uuid: String,
    pub name: Option<String>,
    pub prefix: String,
}

impl PlayerTag {
    pub fn from_uuid(uuid: &str) -> Self {
        let name = get_player(uuid).map(|p| p.name);
        PlayerTag { uuid: uuid.to_string(), name, prefix: "Player".to_string() }
    }

    pub fn from_name(name: &str) -> Self {
        let mut n = name.trim();
        if n.len() >= 2 && n[..2].eq_ignore_ascii_case("p@") {
            n = &n[2..];
        }
        if let Some(p) = get_player_by_name_or_uuid(n) {
            return PlayerTag { uuid: p.uuid, name: Some(p.name), prefix: "Player".to_string() };
        }
        PlayerTag { uuid: n.to_string(), name: Some(n.to_string()), prefix: "Player".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 2 && s[..2].eq_ignore_ascii_case("p@") {
            s = &s[2..];
        }
        if s.is_empty() {
            return None;
        }
        Some(PlayerTag::from_name(s))
    }

    pub fn data(&self) -> Option<PlayerData> {
        get_player_by_name_or_uuid(&self.uuid)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for PlayerTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("p@{}", self.uuid)
    }
    fn identify_simple(&self) -> String {
        match &self.name {
            Some(n) => format!("p@{}", n),
            None => format!("p@{}", self.uuid),
        }
    }
    fn object_type_name(&self) -> &'static str {
        "PlayerTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_unique(&self) -> bool {
        true
    }
    fn is_truthy(&self) -> bool {
        self.data().map(|p| p.online).unwrap_or(false)
    }
}

/// Register PlayerTag's tags.
pub fn register_tags() {
    let p = get_processor("PlayerTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let name = pl.data().map(|d| d.name).or_else(|| pl.name.clone()).unwrap_or_default();
        Some(ObjectTag::new(ElementTag::new(name)))
    }, true, &[]);

    p.register_tag("ElementTag", "uuid", |_a, o| {
        o.downcast_ref::<PlayerTag>().map(|pl| ObjectTag::new(ElementTag::new(pl.uuid.clone())))
    }, true, &[]);

    p.register_tag("LocationTag", "location", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let d = pl.data()?;
        Some(ObjectTag::new(crate::obj::core::location::LocationTag::new(
            d.x,
            d.y,
            d.z,
            d.yaw,
            d.pitch,
            Some(d.world),
        )))
    }, true, &[]);

    p.register_tag("ElementTag", "health", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(pl.data().map(|d| d.health).unwrap_or(0.0))))
    }, true, &[]);

    p.register_tag("ElementTag", "max_health", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(pl.data().map(|d| d.max_health).unwrap_or(20.0))))
    }, true, &[]);

    p.register_tag("ElementTag", "food_level", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(pl.data().map(|d| d.food_level).unwrap_or(20))))
    }, true, &[]);

    p.register_tag("ElementTag", "gamemode", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::new(pl.data().map(|d| d.gamemode).unwrap_or_else(|| "survival".to_string()))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_online", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.data().map(|d| d.online).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_op", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.data().map(|d| d.op).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_banned", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.data().map(|d| d.banned).unwrap_or(false))))
    }, true, &["banned"]);

    p.register_tag("ElementTag", "is_whitelisted", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.data().map(|d| d.whitelisted).unwrap_or(false))))
    }, true, &["whitelisted"]);

    p.register_tag("ElementTag", "is_player", |_a, o| {
        o.downcast_ref::<PlayerTag>().map(|_| ObjectTag::new(ElementTag::from_bool(true)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "has_permission", |_a, o, param| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let perm = param.as_element().value.to_lowercase();
        let has = pl.data().map(|d| d.op || d.permissions.iter().any(|p| p.to_lowercase() == perm || p == "*")).unwrap_or(false);
        Some(ObjectTag::new(ElementTag::from_bool(has)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "in_group", |_a, o, param| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let group = param.as_element().value.to_lowercase();
        let has = pl.data().map(|d| d.groups.iter().any(|g| g.to_lowercase() == group)).unwrap_or(false);
        Some(ObjectTag::new(ElementTag::from_bool(has)))
    }, true, &[]);

    p.register_tag("ListTag", "groups", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&pl.data().map(|d| d.groups).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ElementTag", "chat_prefix", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::new(pl.data().map(|d| d.chat_prefix).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ElementTag", "chat_suffix", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::new(pl.data().map(|d| d.chat_suffix).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ElementTag", "first_played_time", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(crate::obj::TimeTag::from_millis(pl.data().map(|d| d.first_played).unwrap_or(0))))
    }, true, &["first_played"]);

    p.register_tag("ElementTag", "last_played_time", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(crate::obj::TimeTag::from_millis(pl.data().map(|d| d.last_played).unwrap_or(0))))
    }, true, &["last_played"]);

    p.register_tag("ElementTag", "has_played_before", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.data().map(|d| d.last_played > 0).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("WorldTag", "world", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let w = pl.data().map(|d| d.world).unwrap_or_else(|| "world".to_string());
        Some(ObjectTag::new(crate::obj::core::world::WorldTag::new(&w)))
    }, true, &[]);

    p.register_tag("InventoryTag", "inventory", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let d = pl.data()?;
        let mut inv = crate::obj::core::inventory::InventoryTag::new("player", d.inventory.len());
        inv.items = d.inventory.iter().cloned().collect();
        Some(ObjectTag::new(inv))
    }, true, &[]);

    p.register_tag("ElementTag", "is_sneaking", |_a, o| {
        let _pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_flying", |_a, o| {
        let _pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_sprinting", |_a, o| {
        let _pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("EntityTag", "as_entity", |_a, o| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        Some(ObjectTag::new(crate::obj::core::entity::EntityTag::from_uuid(&pl.uuid)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let pl = o.downcast_ref::<PlayerTag>()?;
        let other = PlayerTag::from_identify(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(match other {
            Some(x) => x.uuid.eq_ignore_ascii_case(&pl.uuid) || (x.name.is_some() && x.name == pl.name),
            None => false,
        })))
    }, true, &[]);

    let _ = |a: &mut Attribute| {
        let _ = a;
    };

    crate::obj::core::area::register_flag_tags("PlayerTag", |o| o.downcast_ref::<PlayerTag>().map(|x| x.identify()));
}
