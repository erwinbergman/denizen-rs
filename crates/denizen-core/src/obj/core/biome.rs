//! `BiomeTag`: a world biome type, identified as `b@<world>,<key>`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Static properties for a biome.
#[derive(Clone, Debug)]
pub struct BiomeInfo {
    pub temperature: f64,
    pub humidity: f64,
    pub downfall: bool,
    pub foliage_color: String,
    pub fog_color: String,
    pub water_fog_color: String,
    pub spawnable: Vec<String>,
}

fn default_info(name: &str) -> BiomeInfo {
    let n = name.to_lowercase();
    let n = n.trim_start_matches("minecraft:");
    let (temp, hum, foliage, fog) = match n {
        "desert" | "badlands" | "eroded_badlands" | "wooded_badlands" => (2.0, 0.0, "#aea42a", "#c2b280"),
        "snowy_plains" | "snowy_tundra" | "ice_spikes" | "snowy_taiga" | "frozen_river" | "frozen_ocean" => (0.0, 0.5, "#80b497", "#a0c4e0"),
        "jungle" | "sparse_jungle" | "bamboo_jungle" => (0.95, 0.9, "#30bb0b", "#a0c4b0"),
        "swamp" | "mangrove_swamp" => (0.8, 0.9, "#6a7039", "#8a9a8a"),
        "plains" | "sunflower_plains" => (0.8, 0.4, "#91bd59", "#a0c0c0"),
        "forest" | "flower_forest" | "birch_forest" | "dark_forest" => (0.7, 0.8, "#79c05a", "#a0b8a0"),
        "taiga" | "old_growth_taiga" => (0.25, 0.8, "#86b783", "#a0b8b0"),
        "ocean" | "deep_ocean" | "warm_ocean" | "cold_ocean" => (0.5, 0.5, "#8eb971", "#a0c0e0"),
        "nether_wastes" | "soul_sand_valley" | "crimson_forest" | "warped_forest" | "basalt_deltas" => (2.0, 0.0, "#ae5b00", "#a05050"),
        "the_end" | "end_highlands" | "end_midlands" => (0.5, 0.5, "#8eb971", "#a080c0"),
        _ => (0.5, 0.5, "#8eb971", "#a0c0c0"),
    };
    BiomeInfo {
        temperature: temp,
        humidity: hum,
        downfall: hum > 0.0,
        foliage_color: foliage.to_string(),
        fog_color: fog.to_string(),
        water_fog_color: "#050533".to_string(),
        spawnable: vec!["zombie".to_string(), "skeleton".to_string(), "cow".to_string(), "pig".to_string(), "sheep".to_string(), "chicken".to_string()],
    }
}

static BIOMES: LazyLock<RwLock<HashMap<String, BiomeInfo>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a biome with default properties.
pub fn register_biome(name: &str) {
    let key = normalize_biome(name);
    BIOMES.write().unwrap().entry(key.clone()).or_insert_with(|| default_info(&key));
}

/// Look up biome info, creating defaults on demand.
pub fn get_biome_info(name: &str) -> BiomeInfo {
    let key = normalize_biome(name);
    if let Some(i) = BIOMES.read().unwrap().get(&key) {
        return i.clone();
    }
    let info = default_info(&key);
    BIOMES.write().unwrap().insert(key, info.clone());
    info
}

/// A lightweight per-column biome map keyed by `world:x:z`.
static BIOME_MAP: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Set the biome at a column.
pub fn set_biome_at(x: i64, z: i64, world: Option<&str>, biome: &str) {
    let key = format!("{}:{}:{}", world.unwrap_or(""), x, z);
    BIOME_MAP.write().unwrap().insert(key, normalize_biome(biome));
}

/// Get the biome at a block position (defaults to plains).
pub fn biome_at(x: i64, _y: i64, z: i64, world: Option<&str>) -> String {
    let key = format!("{}:{}:{}", world.unwrap_or(""), x, z);
    BIOME_MAP
        .read()
        .unwrap()
        .get(&key)
        .cloned()
        .unwrap_or_else(|| "plains".to_string())
}

fn normalize_biome(name: &str) -> String {
    let n = name.trim().to_lowercase();
    let n = if n.len() >= 2 && n[..2].eq_ignore_ascii_case("b@") { n[2..].to_string() } else { n };
    let n = n.trim_start_matches("minecraft:").to_string();
    n
}

/// A biome object.
#[derive(Clone, Debug)]
pub struct BiomeTag {
    pub world: Option<String>,
    pub key: String,
    pub prefix: String,
}

impl BiomeTag {
    pub fn new(name: &str) -> Self {
        let mut n = name.trim();
        if n.len() >= 2 && n[..2].eq_ignore_ascii_case("b@") {
            n = &n[2..];
        }
        // Format may be `world,key`.
        if let Some((world, key)) = n.split_once(',') {
            BiomeTag { world: Some(world.trim_start_matches("w@").to_string()), key: normalize_biome(key), prefix: "biome".to_string() }
        } else {
            BiomeTag { world: None, key: normalize_biome(n), prefix: "biome".to_string() }
        }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        if s.trim().is_empty() {
            return None;
        }
        Some(BiomeTag::new(s))
    }

    pub fn info(&self) -> BiomeInfo {
        get_biome_info(&self.key)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for BiomeTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        match &self.world {
            Some(w) => format!("b@{},{}", w, self.key),
            None => format!("b@{}", self.key),
        }
    }
    fn identify_simple(&self) -> String {
        format!("b@{}", self.key)
    }
    fn object_type_name(&self) -> &'static str {
        "BiomeTag"
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

/// Register BiomeTag's tags.
pub fn register_tags() {
    let p = get_processor("BiomeTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<BiomeTag>().map(|b| ObjectTag::new(ElementTag::new(b.key.clone())))
    }, true, &[]);

    p.register_tag("ElementTag", "key", |_a, o| {
        o.downcast_ref::<BiomeTag>().map(|b| ObjectTag::new(ElementTag::new(format!("minecraft:{}", b.key))))
    }, true, &[]);

    p.register_tag("ElementTag", "temperature", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(b.info().temperature)))
    }, true, &["base_temperature"]);

    p.register_tag("ElementTag", "humidity", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(b.info().humidity)))
    }, true, &[]);

    p.register_tag("ElementTag", "has_downfall", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(b.info().downfall)))
    }, true, &[]);

    p.register_tag("ElementTag", "downfall_type", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        let info = b.info();
        let t = if !info.downfall {
            "none"
        } else if info.temperature <= 0.15 {
            "snow"
        } else {
            "rain"
        };
        Some(ObjectTag::new(ElementTag::new(t)))
    }, true, &[]);

    p.register_tag("ElementTag", "foliage_color", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::new(b.info().foliage_color)))
    }, true, &[]);

    p.register_tag("ElementTag", "fog_color", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::new(b.info().fog_color)))
    }, true, &[]);

    p.register_tag("ElementTag", "water_fog_color", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::new(b.info().water_fog_color)))
    }, true, &[]);

    p.register_tag("ListTag", "spawnable_entities", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&b.info().spawnable)))
    }, true, &[]);

    p.register_tag("WorldTag", "world", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        b.world.as_ref().map(|w| ObjectTag::new(crate::obj::core::world::WorldTag::new(w)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let b = o.downcast_ref::<BiomeTag>()?;
        let other = BiomeTag::new(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(b.key.eq_ignore_ascii_case(&other.key))))
    }, true, &[]);


    p.register_tag("ElementTag", "is_hot", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(b.info().temperature >= 1.0)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_cold", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(b.info().temperature <= 0.15)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_dry", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(!b.info().downfall)))
    }, true, &[]);

    p.register_tag("ListTag", "all_biomes", |_a, _o| {
        let names = vec![
            "plains", "forest", "desert", "taiga", "swamp", "jungle", "ocean", "snowy_plains",
            "badlands", "savanna", "nether_wastes", "the_end",
        ];
        Some(ObjectTag::new(ListTag::from_strings(&names)))
    }, true, &[]);

    p.register_tag("ElementTag", "debuggable", |_a, o| {
        let b = o.downcast_ref::<BiomeTag>()?;
        Some(ObjectTag::new(ElementTag::new(b.identify())))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("BiomeTag", |o| o.downcast_ref::<BiomeTag>().map(|x| x.identify()));
}
