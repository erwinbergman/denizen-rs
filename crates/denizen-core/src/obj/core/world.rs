//! `WorldTag`: a server world, identified as `w@<name>`.

use crate::obj::{get_processor, ElementTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Mutable state for a world.
#[derive(Clone, Debug)]
pub struct WorldData {
    pub name: String,
    pub time: i64,
    pub storm: bool,
    pub thundering: bool,
    pub difficulty: String,
    pub seed: i64,
    pub environment: String,
    pub spawn: (f64, f64, f64),
    pub border_size: f64,
    pub keep_spawn: bool,
    pub pvp: bool,
    pub hardcore: bool,
    pub auto_save: bool,
    pub gamerules: HashMap<String, String>,
}

impl Default for WorldData {
    fn default() -> Self {
        WorldData {
            name: String::new(),
            time: 0,
            storm: false,
            thundering: false,
            difficulty: "normal".to_string(),
            seed: 0,
            environment: "normal".to_string(),
            spawn: (0.0, 64.0, 0.0),
            border_size: 29_999_984.0,
            keep_spawn: true,
            pvp: true,
            hardcore: false,
            auto_save: true,
            gamerules: HashMap::new(),
        }
    }
}

/// Global registry of worlds.
static WORLDS: LazyLock<RwLock<HashMap<String, WorldData>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn normalize(name: &str) -> String {
    let n = name.trim();
    let n = if n.len() >= 2 && n[..2].eq_ignore_ascii_case("w@") { &n[2..] } else { n };
    n.to_lowercase()
}

/// Whether a world is registered.
pub fn world_exists(name: &str) -> bool {
    WORLDS.read().unwrap().contains_key(&normalize(name))
}

/// Register (or replace) a world.
pub fn register_world(name: &str) {
    let key = normalize(name);
    WORLDS.write().unwrap().entry(key.clone()).or_insert_with(|| WorldData { name: key, ..Default::default() });
}

/// Fetch world data, creating it on demand.
pub fn get_world_data(name: &str) -> WorldData {
    let key = normalize(name);
    if let Some(w) = WORLDS.read().unwrap().get(&key) {
        return w.clone();
    }
    let mut w = WORLDS.write().unwrap();
    w.entry(key.clone()).or_insert_with(|| WorldData { name: key.clone(), ..Default::default() }).clone()
}

/// Mutate world data in place.
pub fn update_world(name: &str, f: impl FnOnce(&mut WorldData)) {
    let key = normalize(name);
    let mut w = WORLDS.write().unwrap();
    let data = w.entry(key.clone()).or_insert_with(|| WorldData { name: key.clone(), ..Default::default() });
    f(data);
}

/// All registered world names.
pub fn all_world_names() -> Vec<String> {
    let mut names: Vec<String> = WORLDS.read().unwrap().keys().cloned().collect();
    names.sort();
    names
}

/// A world object.
#[derive(Clone, Debug)]
pub struct WorldTag {
    pub name: String,
    pub prefix: String,
}

impl WorldTag {
    pub fn new(name: &str) -> Self {
        WorldTag { name: normalize(name), prefix: "World".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let n = normalize(s);
        if n.is_empty() {
            return None;
        }
        Some(WorldTag::new(&n))
    }

    pub fn data(&self) -> WorldData {
        get_world_data(&self.name)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for WorldTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("w@{}", self.name)
    }
    fn identify_simple(&self) -> String {
        format!("w@{}", self.name)
    }
    fn object_type_name(&self) -> &'static str {
        "WorldTag"
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
}

fn param_world(attr: &mut Attribute) -> Option<WorldTag> {
    let p = attr.get_param_object()?;
    WorldTag::from_identify(&p.as_element().value)
}

/// Register WorldTag's tags.
pub fn register_tags() {
    let p = get_processor("WorldTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<WorldTag>().map(|w| ObjectTag::new(ElementTag::new(w.name.clone())))
    }, true, &[]);

    p.register_tag("ElementTag", "time", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(w.data().time)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_day", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let t = w.data().time.rem_euclid(24000);
        Some(ObjectTag::new(ElementTag::from_bool(t < 12300)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_night", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let t = w.data().time.rem_euclid(24000);
        Some(ObjectTag::new(ElementTag::from_bool(t >= 12300)))
    }, true, &[]);

    p.register_tag("ElementTag", "time_period", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let t = w.data().time.rem_euclid(24000);
        let period = if t < 1000 {
            "day"
        } else if t < 12000 {
            "day"
        } else if t < 13800 {
            "sunset"
        } else if t < 22200 {
            "night"
        } else {
            "sunrise"
        };
        Some(ObjectTag::new(ElementTag::new(period)))
    }, true, &[]);

    p.register_tag("ElementTag", "moon_phase", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let days = w.data().time.div_euclid(24000);
        Some(ObjectTag::new(ElementTag::from_int(days.rem_euclid(8))))
    }, true, &[]);

    p.register_tag("ElementTag", "has_storm", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(w.data().storm)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_thundering", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(w.data().thundering)))
    }, true, &["thundering"]);

    p.register_tag("ElementTag", "weather", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let d = w.data();
        let weather = if d.thundering { "thunder" } else if d.storm { "rain" } else { "clear" };
        Some(ObjectTag::new(ElementTag::new(weather)))
    }, true, &[]);

    p.register_tag("ElementTag", "difficulty", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::new(w.data().difficulty)))
    }, true, &[]);

    p.register_tag("ElementTag", "seed", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(w.data().seed)))
    }, true, &[]);

    p.register_tag("ElementTag", "environment", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::new(w.data().environment)))
    }, true, &["world_type"]);

    p.register_tag("LocationTag", "spawn_location", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let d = w.data();
        Some(ObjectTag::new(crate::obj::core::location::LocationTag::new(
            d.spawn.0,
            d.spawn.1,
            d.spawn.2,
            0.0,
            0.0,
            Some(w.name.clone()),
        )))
    }, true, &["spawn"]);

    p.register_tag("ElementTag", "border_size", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(w.data().border_size)))
    }, true, &[]);

    p.register_tag("ElementTag", "allows_pvp", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(w.data().pvp)))
    }, true, &["pvp"]);

    p.register_tag("ElementTag", "is_hardcore", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(w.data().hardcore)))
    }, true, &["hardcore"]);

    p.register_tag("ElementTag", "keep_spawn", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(w.data().keep_spawn)))
    }, true, &[]);

    p.register_tag("ElementTag", "auto_save", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(w.data().auto_save)))
    }, true, &[]);

    p.register_tag("ElementTag", "sea_level", |_a, o| {
        let _w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(63)))
    }, true, &[]);

    p.register_tag("ElementTag", "max_height", |_a, o| {
        let _w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(320)))
    }, true, &[]);

    p.register_tag("ElementTag", "min_height", |_a, o| {
        let _w = o.downcast_ref::<WorldTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(-64)))
    }, true, &[]);

    p.register_tag("ListTag", "players", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let list = crate::obj::core::player::players_in_world(&w.name);
        Some(ObjectTag::new(list))
    }, true, &[]);

    p.register_tag("ListTag", "entities", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let list = crate::obj::core::entity::entities_in_world(&w.name);
        Some(ObjectTag::new(list))
    }, true, &[]);

    p.register_tag("ListTag", "npcs", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let list = crate::obj::core::npc::npcs_in_world(&w.name);
        Some(ObjectTag::new(list))
    }, true, &[]);

    p.register_tag("ListTag", "loaded_chunks", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let chunks = crate::obj::core::chunk::loaded_chunks(&w.name);
        Some(ObjectTag::new(chunks))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "gamerule", |a, o, param| {
        let w = o.downcast_ref::<WorldTag>()?;
        let key = param.as_element().value.to_lowercase();
        let _ = a;
        Some(ObjectTag::new(ElementTag::new(w.data().gamerules.get(&key).cloned().unwrap_or_default())))
    }, true, &[]);

    p.register_tag("MapTag", "gamerule_map", |_a, o| {
        let w = o.downcast_ref::<WorldTag>()?;
        let mut m = crate::obj::MapTag::new();
        for (k, v) in &w.data().gamerules {
            m.put_element(k, v);
        }
        Some(ObjectTag::new(m))
    }, true, &[]);

    p.register_tag_with_param("WorldTag", "ElementTag", "with_time", |_a, o, param| {
        let w = o.downcast_ref::<WorldTag>()?;
        let _ = param;
        Some(w.obj())
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let w = o.downcast_ref::<WorldTag>()?;
        let other = WorldTag::from_identify(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(
            other.map(|x| x.name == w.name).unwrap_or(false),
        )))
    }, true, &[]);

    let _ = param_world;

    crate::obj::core::area::register_flag_tags("WorldTag", |o| o.downcast_ref::<WorldTag>().map(|x| x.identify()));
}
