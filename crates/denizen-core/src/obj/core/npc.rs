//! `NPCTag`: a Citizens-style NPC, identified as `n@<id>`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Mutable state for an NPC.
#[derive(Clone, Debug)]
pub struct NpcData {
    pub id: i64,
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub spawned: bool,
    pub protected: bool,
    pub invulnerable: bool,
    pub sitting: bool,
    pub sneaking: bool,
    pub sleeping: bool,
    pub pushable: bool,
    pub lookclose: bool,
    pub traits: Vec<String>,
    pub skin: Option<String>,
    pub owner: Option<String>,
    pub entity_type: String,
}

impl Default for NpcData {
    fn default() -> Self {
        NpcData {
            id: 0,
            name: String::new(),
            world: "world".to_string(),
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            spawned: false,
            protected: false,
            invulnerable: false,
            sitting: false,
            sneaking: false,
            sleeping: false,
            pushable: true,
            lookclose: true,
            traits: Vec::new(),
            skin: None,
            owner: None,
            entity_type: "player".to_string(),
        }
    }
}

static NPCS: LazyLock<RwLock<HashMap<i64, NpcData>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Create or replace an NPC, returning its id.
pub fn create_npc(mut data: NpcData) -> i64 {
    if data.id == 0 {
        let max = NPCS.read().unwrap().keys().copied().max().unwrap_or(0);
        data.id = max + 1;
    }
    let id = data.id;
    NPCS.write().unwrap().insert(id, data);
    id
}

/// Look up NPC data.
pub fn get_npc(id: i64) -> Option<NpcData> {
    NPCS.read().unwrap().get(&id).cloned()
}

/// Mutate NPC data.
pub fn update_npc(id: i64, f: impl FnOnce(&mut NpcData)) -> bool {
    let mut map = NPCS.write().unwrap();
    match map.get_mut(&id) {
        Some(d) => {
            f(d);
            true
        }
        None => false,
    }
}

/// Remove an NPC.
pub fn remove_npc(id: i64) {
    NPCS.write().unwrap().remove(&id);
}

/// All NPCs in a world.
pub fn npcs_in_world(world: &str) -> ListTag {
    let w = world.to_lowercase();
    let items: Vec<ObjectTag> = NPCS
        .read()
        .unwrap()
        .values()
        .filter(|d| d.world.to_lowercase() == w && d.spawned)
        .map(|d| ObjectTag::new(NPCTag::new(d.id)))
        .collect();
    ListTag::from_objects(items)
}

/// NPCs within a radius of a location.
pub fn npcs_within(loc: &crate::obj::core::location::LocationTag, radius: f64) -> ListTag {
    let items: Vec<ObjectTag> = NPCS
        .read()
        .unwrap()
        .values()
        .filter(|d| {
            let same_world = match &loc.world {
                Some(w) => w.to_lowercase() == d.world.to_lowercase(),
                None => true,
            };
            let dist = ((d.x - loc.x).powi(2) + (d.y - loc.y).powi(2) + (d.z - loc.z).powi(2)).sqrt();
            same_world && dist <= radius && d.spawned
        })
        .map(|d| ObjectTag::new(NPCTag::new(d.id)))
        .collect();
    ListTag::from_objects(items)
}

/// Find an NPC by name (case-insensitive).
pub fn npc_by_name(name: &str) -> Option<i64> {
    let n = name.to_lowercase();
    NPCS.read().unwrap().values().find(|d| d.name.to_lowercase() == n).map(|d| d.id)
}

/// An NPC object.
#[derive(Clone, Debug)]
pub struct NPCTag {
    pub id: i64,
    pub registry: Option<String>,
    pub prefix: String,
}

impl NPCTag {
    pub fn new(id: i64) -> Self {
        NPCTag { id, registry: None, prefix: "npc".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 2 && s[..2].eq_ignore_ascii_case("n@") {
            s = &s[2..];
        }
        if s.is_empty() {
            return None;
        }
        let mut parts = s.splitn(2, ',');
        let id_part = parts.next()?.trim();
        let registry = parts.next().map(|r| r.trim().to_string());
        let id: i64 = match id_part.parse() {
            Ok(v) => v,
            Err(_) => npc_by_name(id_part)?,
        };
        Some(NPCTag { id, registry, prefix: "npc".to_string() })
    }

    pub fn data(&self) -> Option<NpcData> {
        get_npc(self.id)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for NPCTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        match &self.registry {
            Some(r) => format!("n@{},{}", self.id, r),
            None => format!("n@{}", self.id),
        }
    }
    fn identify_simple(&self) -> String {
        format!("n@{}", self.id)
    }
    fn object_type_name(&self) -> &'static str {
        "NPCTag"
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
        self.data().is_some()
    }
}

/// Register NPCTag's tags.
pub fn register_tags() {
    let p = get_processor("NPCTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::new(n.data().map(|d| d.name).unwrap_or_default())))
    }, true, &["nickname"]);

    p.register_tag("ElementTag", "id", |_a, o| {
        o.downcast_ref::<NPCTag>().map(|n| ObjectTag::new(ElementTag::from_int(n.id)))
    }, true, &[]);

    p.register_tag("LocationTag", "location", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        let d = n.data()?;
        Some(ObjectTag::new(crate::obj::core::location::LocationTag::new(
            d.x,
            d.y,
            d.z,
            d.yaw,
            d.pitch,
            Some(d.world),
        )))
    }, true, &[]);

    p.register_tag("ElementTag", "is_spawned", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.spawned).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_protected", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.protected).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_invulnerable", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.invulnerable).unwrap_or(false))))
    }, true, &["invulnerable"]);

    p.register_tag("ElementTag", "is_sitting", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.sitting).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_sneaking", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.sneaking).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_sleeping", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.sleeping).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_pushable", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.pushable).unwrap_or(true))))
    }, true, &["pushable"]);

    p.register_tag("ElementTag", "is_lookclose", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().map(|d| d.lookclose).unwrap_or(false))))
    }, true, &["lookclose"]);

    p.register_tag("ListTag", "traits", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&n.data().map(|d| d.traits).unwrap_or_default())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "has_trait", |_a, o, param| {
        let n = o.downcast_ref::<NPCTag>()?;
        let trait_name = param.as_element().value.to_lowercase();
        let has = n.data().map(|d| d.traits.iter().any(|t| t.to_lowercase() == trait_name)).unwrap_or(false);
        Some(ObjectTag::new(ElementTag::from_bool(has)))
    }, true, &[]);

    p.register_tag("ElementTag", "skin", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::new(n.data().and_then(|d| d.skin).unwrap_or_default())))
    }, true, &["skull_skin"]);

    p.register_tag("ElementTag", "has_skin", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(n.data().and_then(|d| d.skin).is_some())))
    }, true, &[]);

    p.register_tag("PlayerTag", "owner", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        let owner = n.data().and_then(|d| d.owner)?;
        Some(ObjectTag::new(crate::obj::core::player::PlayerTag::from_name(&owner)))
    }, true, &[]);

    p.register_tag("ElementTag", "entity_type", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::new(n.data().map(|d| d.entity_type).unwrap_or_else(|| "player".to_string()))))
    }, true, &["type"]);

    p.register_tag("WorldTag", "world", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        let w = n.data().map(|d| d.world).unwrap_or_else(|| "world".to_string());
        Some(ObjectTag::new(crate::obj::core::world::WorldTag::new(&w)))
    }, true, &[]);

    p.register_tag("InventoryTag", "inventory", |_a, o| {
        let _n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(crate::obj::core::inventory::InventoryTag::new("npc", 36)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_npc", |_a, o| {
        o.downcast_ref::<NPCTag>().map(|_| ObjectTag::new(ElementTag::from_bool(true)))
    }, true, &[]);

    p.register_tag("ElementTag", "name_hologram", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        let name = n.data().map(|d| d.name).unwrap_or_default();
        Some(ObjectTag::new(ElementTag::new(name)))
    }, true, &[]);

    p.register_tag("ElementTag", "debuggable", |_a, o| {
        let n = o.downcast_ref::<NPCTag>()?;
        Some(ObjectTag::new(ElementTag::new(n.identify())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let n = o.downcast_ref::<NPCTag>()?;
        let other = NPCTag::from_identify(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(match other {
            Some(x) => x.id == n.id,
            None => false,
        })))
    }, true, &[]);

    p.register_tag_with_param("NPCTag", "ElementTag", "with_name", |_a, o, param| {
        let n = o.downcast_ref::<NPCTag>()?.clone();
        update_npc(n.id, |d| d.name = param.as_element().value);
        Some(n.obj())
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("NPCTag", |o| o.downcast_ref::<NPCTag>().map(|x| x.identify()));
}
