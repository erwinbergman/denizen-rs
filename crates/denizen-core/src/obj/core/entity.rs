//! `EntityTag`: a live or hypothetical entity, identified as `e@<uuid>` or `e@<type>`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Mutable state for a spawned entity.
#[derive(Clone, Debug)]
pub struct EntityData {
    pub uuid: String,
    pub entity_type: String,
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f64,
    pub max_health: f64,
    pub alive: bool,
    pub valid: bool,
}

impl Default for EntityData {
    fn default() -> Self {
        EntityData {
            uuid: String::new(),
            entity_type: "pig".to_string(),
            name: String::new(),
            world: "world".to_string(),
            x: 0.0,
            y: 64.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            health: 20.0,
            max_health: 20.0,
            alive: true,
            valid: true,
        }
    }
}

static ENTITIES: LazyLock<RwLock<HashMap<String, EntityData>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a live entity, returning its uuid.
pub fn spawn_entity(data: EntityData) -> String {
    let uuid = if data.uuid.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        data.uuid.clone()
    };
    let mut d = data;
    d.uuid = uuid.clone();
    ENTITIES.write().unwrap().insert(uuid.clone(), d);
    uuid
}

/// Look up live entity data by uuid.
pub fn get_entity(uuid: &str) -> Option<EntityData> {
    ENTITIES.read().unwrap().get(uuid).cloned()
}

/// Mutate live entity data.
pub fn update_entity(uuid: &str, f: impl FnOnce(&mut EntityData)) -> bool {
    let mut map = ENTITIES.write().unwrap();
    match map.get_mut(uuid) {
        Some(d) => {
            f(d);
            true
        }
        None => false,
    }
}

/// Remove a live entity.
pub fn remove_entity(uuid: &str) {
    ENTITIES.write().unwrap().remove(uuid);
}

/// All entity uuids in a world.
pub fn entities_in_world(world: &str) -> ListTag {
    let w = world.to_lowercase();
    let items: Vec<ObjectTag> = ENTITIES
        .read()
        .unwrap()
        .values()
        .filter(|d| d.world.to_lowercase() == w && d.valid)
        .map(|d| ObjectTag::new(EntityTag::from_uuid(&d.uuid)))
        .collect();
    ListTag::from_objects(items)
}

/// All entities within a radius of a location.
pub fn entities_within(loc: &crate::obj::core::location::LocationTag, radius: f64) -> ListTag {
    let items: Vec<ObjectTag> = ENTITIES
        .read()
        .unwrap()
        .values()
        .filter(|d| {
            let same_world = match (&loc.world, d.world.is_empty()) {
                (Some(w), false) => w.to_lowercase() == d.world.to_lowercase(),
                _ => true,
            };
            let dist = ((d.x - loc.x).powi(2) + (d.y - loc.y).powi(2) + (d.z - loc.z).powi(2)).sqrt();
            same_world && dist <= radius && d.valid
        })
        .map(|d| ObjectTag::new(EntityTag::from_uuid(&d.uuid)))
        .collect();
    ListTag::from_objects(items)
}

/// Known entity type names (for validation and matching).
pub fn known_entity_type(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "player" | "zombie" | "skeleton" | "creeper" | "spider" | "enderman" | "cow" | "pig"
            | "sheep" | "chicken" | "horse" | "villager" | "wandering_trader" | "iron_golem"
            | "snow_golem" | "wolf" | "cat" | "ocelot" | "rabbit" | "fox" | "bee" | "bat"
            | "blaze" | "ghast" | "magma_cube" | "slime" | "witch" | "wither_skeleton" | "piglin"
            | "hoglin" | "zombified_piglin" | "drowned" | "husk" | "stray" | "phantom" | "guardian"
            | "elder_guardian" | "shulker" | "silverfish" | "endermite" | "evoker" | "vindicator"
            | "pillager" | "ravager" | "vex" | "illusioner" | "warden" | "allay" | "frog" | "tadpole"
            | "axolotl" | "goat" | "glow_squid" | "squid" | "dolphin" | "turtle" | "panda" | "polar_bear"
            | "llama" | "trader_llama" | "mule" | "donkey" | "camel" | "strider" | "zoglin"
            | "armor_stand" | "item" | "experience_orb" | "arrow" | "fireball" | "snowball" | "boat"
            | "minecart" | "tnt" | "falling_block" | "item_frame" | "painting" | "leash_knot"
            | "lightning_bolt" | "area_effect_cloud" | "marker" | "interaction" | "display"
    )
}

/// An entity object.
#[derive(Clone, Debug)]
pub struct EntityTag {
    /// The entity uuid if spawned; None for a type-only entity.
    pub uuid: Option<String>,
    /// The entity type (lowercase).
    pub entity_type: String,
    pub prefix: String,
}

impl EntityTag {
    pub fn from_uuid(uuid: &str) -> Self {
        let data = get_entity(uuid);
        EntityTag {
            uuid: Some(uuid.to_string()),
            entity_type: data.map(|d| d.entity_type).unwrap_or_else(|| "entity".to_string()),
            prefix: "Entity".to_string(),
        }
    }

    pub fn from_type(entity_type: &str) -> Self {
        let mut t = entity_type.trim().to_lowercase();
        if t.len() >= 2 && t[..2].eq_ignore_ascii_case("e@") {
            t = t[2..].to_string();
        }
        EntityTag { uuid: None, entity_type: t, prefix: "Entity".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 2 && s[..2].eq_ignore_ascii_case("e@") {
            s = &s[2..];
        }
        if s.is_empty() {
            return None;
        }
        // e@<uuid>/<type>
        if let Some(slash) = s.find('/') {
            let uuid = &s[..slash];
            if crate::util::try_parse_uuid(uuid).is_some() || get_entity(uuid).is_some() {
                return Some(EntityTag::from_uuid(uuid));
            }
        }
        if crate::util::try_parse_uuid(s).is_some() {
            return Some(EntityTag::from_uuid(s));
        }
        Some(EntityTag::from_type(s))
    }

    pub fn data(&self) -> Option<EntityData> {
        self.uuid.as_deref().and_then(get_entity)
    }

    pub fn is_spawned(&self) -> bool {
        self.data().map(|d| d.valid).unwrap_or(false)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }

    pub fn location(&self) -> Option<crate::obj::core::location::LocationTag> {
        let d = self.data()?;
        Some(crate::obj::core::location::LocationTag::new(
            d.x,
            d.y,
            d.z,
            d.yaw,
            d.pitch,
            Some(d.world),
        ))
    }
}

impl ObjectTagImpl for EntityTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        match &self.uuid {
            Some(u) => format!("e@{}/{}", u, self.entity_type),
            None => format!("e@{}", self.entity_type),
        }
    }
    fn identify_simple(&self) -> String {
        match &self.uuid {
            Some(u) => format!("e@{}", u),
            None => format!("e@{}", self.entity_type),
        }
    }
    fn object_type_name(&self) -> &'static str {
        "EntityTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_unique(&self) -> bool {
        self.uuid.is_some()
    }
    fn is_truthy(&self) -> bool {
        self.uuid.is_none() || self.is_spawned()
    }
}

/// Register EntityTag's tags.
pub fn register_tags() {
    let p = get_processor("EntityTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let name = e.data().map(|d| d.name).unwrap_or_default();
        Some(ObjectTag::new(ElementTag::new(name)))
    }, true, &[]);

    p.register_tag("ElementTag", "uuid", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::new(e.uuid.clone().unwrap_or_default())))
    }, true, &[]);

    p.register_tag("LocationTag", "location", |_a, o| {
        o.downcast_ref::<EntityTag>()?.location().map(|l| l.obj())
    }, true, &[]);

    p.register_tag("ElementTag", "health", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(e.data().map(|d| d.health).unwrap_or(0.0))))
    }, true, &[]);

    p.register_tag("ElementTag", "max_health", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(e.data().map(|d| d.max_health).unwrap_or(20.0))))
    }, true, &[]);

    p.register_tag("ElementTag", "type", |_a, o| {
        o.downcast_ref::<EntityTag>().map(|e| ObjectTag::new(ElementTag::new(e.entity_type.clone())))
    }, true, &["entity_type"]);

    p.register_tag("ElementTag", "is_alive", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(e.data().map(|d| d.alive).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_valid", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(e.is_spawned())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_spawned", |_a, o| {
        o.downcast_ref::<EntityTag>().map(|e| ObjectTag::new(ElementTag::from_bool(e.is_spawned())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_living", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let living = !matches!(e.entity_type.as_str(), "item" | "experience_orb" | "arrow" | "boat" | "minecart" | "tnt" | "falling_block" | "painting" | "item_frame" | "lightning_bolt" | "marker");
        Some(ObjectTag::new(ElementTag::from_bool(living)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_monster", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let monster = matches!(e.entity_type.as_str(), "zombie" | "skeleton" | "creeper" | "spider" | "enderman" | "witch" | "blaze" | "ghast" | "slime" | "magma_cube" | "wither_skeleton" | "piglin" | "hoglin" | "zombified_piglin" | "drowned" | "husk" | "stray" | "phantom" | "guardian" | "elder_guardian" | "shulker" | "silverfish" | "endermite" | "evoker" | "vindicator" | "pillager" | "ravager" | "vex" | "illusioner" | "warden");
        Some(ObjectTag::new(ElementTag::from_bool(monster)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_mob", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let mob = e.entity_type != "player" && !matches!(e.entity_type.as_str(), "item" | "experience_orb" | "arrow");
        Some(ObjectTag::new(ElementTag::from_bool(mob)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_player", |_a, o| {
        o.downcast_ref::<EntityTag>().map(|e| ObjectTag::new(ElementTag::from_bool(e.entity_type == "player")))
    }, true, &[]);

    p.register_tag("ElementTag", "is_projectile", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let proj = matches!(e.entity_type.as_str(), "arrow" | "snowball" | "fireball" | "small_fireball" | "egg" | "ender_pearl" | "trident" | "spectral_arrow" | "wither_skull" | "dragon_fireball");
        Some(ObjectTag::new(ElementTag::from_bool(proj)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_vehicle", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(matches!(e.entity_type.as_str(), "boat" | "minecart" | "chest_boat" | "chest_minecart" | "hopper_minecart" | "tnt_minecart"))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_glowing", |_a, o| {
        let _e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_sneaking", |_a, o| {
        let _e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_on_fire", |_a, o| {
        let _e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("WorldTag", "world", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let w = e.data().map(|d| d.world).unwrap_or_else(|| "world".to_string());
        Some(ObjectTag::new(crate::obj::core::world::WorldTag::new(&w)))
    }, true, &[]);

    p.register_tag("ListTag", "vanilla_tags", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        let mut tags = Vec::new();
        if e.entity_type == "zombie" {
            tags.push("minecraft:zombies".to_string());
        }
        Some(ObjectTag::new(ListTag::from_strings(&tags)))
    }, true, &[]);

    p.register_tag("PlayerTag", "as_player", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        if e.entity_type == "player" {
            let uuid = e.uuid.clone().unwrap_or_default();
            Some(ObjectTag::new(crate::obj::core::player::PlayerTag::from_uuid(&uuid)))
        } else {
            None
        }
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "health_percentage", |_a, o, _param| {
        let e = o.downcast_ref::<EntityTag>()?;
        let d = e.data()?;
        let pct = if d.max_health > 0.0 { d.health / d.max_health * 100.0 } else { 0.0 };
        Some(ObjectTag::new(ElementTag::from_double(pct)))
    }, true, &[]);

    p.register_tag("ElementTag", "describe", |_a, o| {
        let e = o.downcast_ref::<EntityTag>()?;
        Some(ObjectTag::new(ElementTag::new(e.identify())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let e = o.downcast_ref::<EntityTag>()?;
        let other = EntityTag::from_identify(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(match other {
            Some(x) => x.identify() == e.identify(),
            None => false,
        })))
    }, true, &[]);

    let _ = |a: &mut Attribute| {
        let _ = a;
    };

    crate::obj::core::area::register_flag_tags("EntityTag", |o| o.downcast_ref::<EntityTag>().map(|x| x.identify()));
}
