//! `ChunkTag`: a 16x16 column region, identified as `ch@x,z,world`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use std::any::Any;
use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};

/// Explicitly loaded chunk keys (`world:x:z`).
static LOADED: LazyLock<RwLock<HashSet<String>>> = LazyLock::new(|| RwLock::new(HashSet::new()));
/// Force-loaded chunk keys.
static FORCE_LOADED: LazyLock<RwLock<HashSet<String>>> = LazyLock::new(|| RwLock::new(HashSet::new()));
/// Generated chunk keys.
static GENERATED: LazyLock<RwLock<HashSet<String>>> = LazyLock::new(|| RwLock::new(HashSet::new()));

fn chunk_key(world: &str, x: i64, z: i64) -> String {
    format!("{}:{}:{}", world.to_lowercase(), x, z)
}

/// Mark a chunk as loaded (and generated).
pub fn mark_loaded(x: i64, z: i64, world: &str) {
    let key = chunk_key(world, x, z);
    LOADED.write().unwrap().insert(key.clone());
    GENERATED.write().unwrap().insert(key);
}

/// Unload a chunk.
pub fn unload(x: i64, z: i64, world: &str) {
    LOADED.write().unwrap().remove(&chunk_key(world, x, z));
}

/// Whether a chunk is loaded.
pub fn is_loaded(x: i64, z: i64, world: Option<&str>) -> bool {
    LOADED.read().unwrap().contains(&chunk_key(world.unwrap_or(""), x, z))
}

/// Whether a chunk has been generated.
pub fn is_generated(x: i64, z: i64, world: &str) -> bool {
    GENERATED.read().unwrap().contains(&chunk_key(world, x, z))
}

/// Set force-loaded state.
pub fn set_force_loaded(x: i64, z: i64, world: &str, forced: bool) {
    let key = chunk_key(world, x, z);
    if forced {
        FORCE_LOADED.write().unwrap().insert(key);
    } else {
        FORCE_LOADED.write().unwrap().remove(&key);
    }
}

/// Whether a chunk is force-loaded.
pub fn is_force_loaded(x: i64, z: i64, world: &str) -> bool {
    FORCE_LOADED.read().unwrap().contains(&chunk_key(world, x, z))
}

/// All loaded chunks in a world.
pub fn loaded_chunks(world: &str) -> ListTag {
    let prefix = format!("{}:", world.to_lowercase());
    let items: Vec<ObjectTag> = LOADED
        .read()
        .unwrap()
        .iter()
        .filter(|k| k.starts_with(&prefix))
        .filter_map(|k| {
            let mut parts = k.splitn(3, ':');
            let _w = parts.next()?;
            let x: i64 = parts.next()?.parse().ok()?;
            let z: i64 = parts.next()?.parse().ok()?;
            Some(ObjectTag::new(ChunkTag::new(x, z, world.to_string())))
        })
        .collect();
    ListTag::from_objects(items)
}

/// A chunk object.
#[derive(Clone, Debug)]
pub struct ChunkTag {
    pub x: i64,
    pub z: i64,
    pub world: String,
    pub prefix: String,
}

impl ChunkTag {
    pub fn new(x: i64, z: i64, world: impl Into<String>) -> Self {
        let w = world.into();
        let w = w.trim_start_matches("w@").to_string();
        ChunkTag { x, z, world: w, prefix: "Chunk".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 3 && s[..3].eq_ignore_ascii_case("ch@") {
            s = &s[3..];
        }
        let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
        if parts.len() < 3 {
            return None;
        }
        let x: i64 = parts[0].parse().ok()?;
        let z: i64 = parts[1].parse().ok()?;
        let world = parts[2].trim_start_matches("w@").to_string();
        Some(ChunkTag::new(x, z, world))
    }

    pub fn block_min_x(&self) -> i64 {
        self.x * 16
    }
    pub fn block_min_z(&self) -> i64 {
        self.z * 16
    }

    pub fn entities(&self) -> ListTag {
        let items: Vec<ObjectTag> = crate::obj::core::entity::entities_in_world(&self.world)
            .items
            .into_iter()
            .filter(|e| {
                if let Some(et) = e.downcast_ref::<crate::obj::core::entity::EntityTag>() {
                    if let Some(loc) = et.location() {
                        return loc.chunk_x() == self.x && loc.chunk_z() == self.z;
                    }
                }
                false
            })
            .collect();
        ListTag::from_objects(items)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for ChunkTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("ch@{},{},{}", self.x, self.z, self.world)
    }
    fn identify_simple(&self) -> String {
        self.identify()
    }
    fn object_type_name(&self) -> &'static str {
        "ChunkTag"
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

/// Register ChunkTag's tags.
pub fn register_tags() {
    let p = get_processor("ChunkTag");

    p.register_tag("ElementTag", "x", |_a, o| {
        o.downcast_ref::<ChunkTag>().map(|c| ObjectTag::new(ElementTag::from_int(c.x)))
    }, true, &[]);

    p.register_tag("ElementTag", "z", |_a, o| {
        o.downcast_ref::<ChunkTag>().map(|c| ObjectTag::new(ElementTag::from_int(c.z)))
    }, true, &[]);

    p.register_tag("ElementTag", "world", |_a, o| {
        o.downcast_ref::<ChunkTag>().map(|c| ObjectTag::new(ElementTag::new(c.world.clone())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_loaded", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(is_loaded(c.x, c.z, Some(&c.world)))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_generated", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(is_generated(c.x, c.z, &c.world))))
    }, true, &[]);

    p.register_tag("ElementTag", "force_loaded", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(is_force_loaded(c.x, c.z, &c.world))))
    }, true, &[]);

    p.register_tag("ListTag", "entities", |_a, o| {
        o.downcast_ref::<ChunkTag>().map(|c| ObjectTag::new(c.entities()))
    }, true, &[]);

    p.register_tag("ListTag", "tile_entities", |_a, o| {
        let _c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ListTag::new()))
    }, true, &[]);

    p.register_tag("ElementTag", "xz", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::new(format!("{},{}", c.x, c.z))))
    }, true, &[]);

    p.register_tag("ElementTag", "simple", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::new(format!("{},{},{}", c.x, c.z, c.world))))
    }, true, &[]);

    p.register_tag("CuboidTag", "cuboid", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        let low = crate::obj::core::location::LocationTag::new(
            (c.x * 16) as f64,
            -64.0,
            (c.z * 16) as f64,
            0.0,
            0.0,
            Some(c.world.clone()),
        );
        let high = crate::obj::core::location::LocationTag::new(
            (c.x * 16 + 15) as f64,
            319.0,
            (c.z * 16 + 15) as f64,
            0.0,
            0.0,
            Some(c.world.clone()),
        );
        Some(ObjectTag::new(crate::obj::core::cuboid::CuboidTag::from_corners(low, high)))
    }, true, &[]);

    p.register_tag("ElementTag", "inhabited_time", |_a, o| {
        let _c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(0)))
    }, true, &[]);

    p.register_tag("ElementTag", "spawn_slimes", |_a, o| {
        let _c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_flat", |_a, o| {
        let _c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(false)))
    }, true, &[]);

    p.register_tag("ElementTag", "average_height", |_a, o| {
        let _c = o.downcast_ref::<ChunkTag>()?;
        Some(ObjectTag::new(ElementTag::from_double(64.0)))
    }, true, &[]);

    p.register_tag("ListTag", "players", |_a, o| {
        let c = o.downcast_ref::<ChunkTag>()?;
        let items: Vec<ObjectTag> = crate::obj::core::player::players_in_world(&c.world)
            .items
            .into_iter()
            .filter(|p| {
                if let Some(pt) = p.downcast_ref::<crate::obj::core::player::PlayerTag>() {
                    if let Some(d) = pt.data() {
                        return (d.x.floor() as i64).div_euclid(16) == c.x
                            && (d.z.floor() as i64).div_euclid(16) == c.z;
                    }
                }
                false
            })
            .collect();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("ChunkTag", |o| o.downcast_ref::<ChunkTag>().map(|x| x.identify()));
}
