//! `LocationTag`: a point in a world, identified as `l@x,y,z,pitch,yaw,world`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl, VectorObject};
use crate::tags::Attribute;
use crate::util;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// A world position with optional rotation and world name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocationTag {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub world: Option<String>,
}

/// A registry of block material names, keyed by `world:x,y,z`.
static BLOCKS: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn block_key(world: Option<&str>, x: i64, y: i64, z: i64) -> String {
    format!("{}:{}:{}:{}", world.unwrap_or(""), x, y, z)
}

/// Read the material name at a block position (defaults to `air`).
pub fn get_block(world: Option<&str>, x: i64, y: i64, z: i64) -> String {
    BLOCKS
        .read()
        .unwrap()
        .get(&block_key(world, x, y, z))
        .cloned()
        .unwrap_or_else(|| "air".to_string())
}

/// Set the material name at a block position.
pub fn set_block(world: Option<&str>, x: i64, y: i64, z: i64, material: &str) {
    BLOCKS.write().unwrap().insert(
        block_key(world, x, y, z),
        material.to_lowercase().trim_start_matches("m@").to_string(),
    );
}

/// Remove a block override (reverting it to air).
pub fn clear_block(world: Option<&str>, x: i64, y: i64, z: i64) {
    BLOCKS.write().unwrap().remove(&block_key(world, x, y, z));
}

/// All non-air blocks in a chunk (chunk coordinates), as `(x,y,z,material)`.
pub fn blocks_in_chunk(world: Option<&str>, cx: i64, cz: i64) -> Vec<(i64, i64, i64, String)> {
    let wx = world.unwrap_or("");
    let mut out = Vec::new();
    for (k, v) in BLOCKS.read().unwrap().iter() {
        if v == "air" {
            continue;
        }
        let mut parts = k.split(':');
        let kw = parts.next().unwrap_or("");
        let x: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let y: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        let z: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        if kw == wx && x.div_euclid(16) == cx && z.div_euclid(16) == cz {
            out.push((x, y, z, v.clone()));
        }
    }
    out
}

impl LocationTag {
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32, world: Option<String>) -> Self {
        LocationTag { x, y, z, yaw, pitch, world: world.map(|w| normalize_world(&w)) }
    }

    pub fn at(x: f64, y: f64, z: f64) -> Self {
        LocationTag::new(x, y, z, 0.0, 0.0, None)
    }

    pub fn with_world_name(mut self, world: &str) -> Self {
        self.world = Some(normalize_world(world));
        self
    }

    pub fn world_name(&self) -> Option<String> {
        self.world.clone()
    }

    pub fn block_x(&self) -> i64 {
        self.x.floor() as i64
    }
    pub fn block_y(&self) -> i64 {
        self.y.floor() as i64
    }
    pub fn block_z(&self) -> i64 {
        self.z.floor() as i64
    }

    pub fn block(&self) -> Self {
        LocationTag::new(
            self.block_x() as f64,
            self.block_y() as f64,
            self.block_z() as f64,
            0.0,
            0.0,
            self.world.clone(),
        )
    }

    pub fn center(&self) -> Self {
        LocationTag::new(
            self.block_x() as f64 + 0.5,
            self.block_y() as f64 + 0.5,
            self.block_z() as f64 + 0.5,
            self.yaw,
            self.pitch,
            self.world.clone(),
        )
    }

    pub fn add(&self, x: f64, y: f64, z: f64) -> Self {
        LocationTag::new(self.x + x, self.y + y, self.z + z, self.yaw, self.pitch, self.world.clone())
    }

    pub fn subtract(&self, x: f64, y: f64, z: f64) -> Self {
        LocationTag::new(self.x - x, self.y - y, self.z - z, self.yaw, self.pitch, self.world.clone())
    }

    /// The unit direction vector implied by yaw and pitch.
    pub fn direction(&self) -> VectorObject {
        let yaw = (self.yaw as f64).to_radians();
        let pitch = (self.pitch as f64).to_radians();
        VectorObject::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        )
    }

    pub fn distance_to(&self, other: &LocationTag) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn distance_squared_to(&self, other: &LocationTag) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn chunk_x(&self) -> i64 {
        self.block_x().div_euclid(16)
    }
    pub fn chunk_z(&self) -> i64 {
        self.block_z().div_euclid(16)
    }

    pub fn material_name(&self) -> String {
        get_block(self.world.as_deref(), self.block_x(), self.block_y(), self.block_z())
    }

    pub fn with_x(mut self, x: f64) -> Self {
        self.x = x;
        self
    }
    pub fn with_y(mut self, y: f64) -> Self {
        self.y = y;
        self
    }
    pub fn with_z(mut self, z: f64) -> Self {
        self.z = z;
        self
    }
    pub fn with_yaw(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self
    }
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn round_all(&self) -> Self {
        LocationTag::new(self.x.round(), self.y.round(), self.z.round(), self.yaw.round(), self.pitch.round(), self.world.clone())
    }

    pub fn round_up(&self) -> Self {
        LocationTag::new(self.x.ceil(), self.y.ceil(), self.z.ceil(), self.yaw.ceil(), self.pitch.ceil(), self.world.clone())
    }

    pub fn round_down(&self) -> Self {
        LocationTag::new(self.x.floor(), self.y.floor(), self.z.floor(), self.yaw.floor(), self.pitch.floor(), self.world.clone())
    }

    pub fn round_to(&self, places: i32) -> Self {
        let ten = 10f64.powi(places);
        let r = |v: f64| (v * ten).round() / ten;
        LocationTag::new(r(self.x), r(self.y), r(self.z), r(self.yaw as f64) as f32, r(self.pitch as f64) as f32, self.world.clone())
    }

    /// Parse a location identity string (with or without the `l@` prefix).
    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 2 && s[..2].eq_ignore_ascii_case("l@") {
            s = &s[2..];
        }
        if s.is_empty() {
            return None;
        }
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() < 2 || parts.len() > 6 {
            return None;
        }
        let x: f64 = parts[0].trim().parse().ok()?;
        let y: f64 = parts[1].trim().parse().ok()?;
        let mut z = 0.0;
        let mut yaw = 0.0f32;
        let mut pitch = 0.0f32;
        let mut world: Option<String> = None;
        if parts.len() > 2 {
            z = parts[2].trim().parse().ok()?;
        }
        let is_num = |t: &str| t.trim().parse::<f64>().is_ok();
        match parts.len() {
            4 => {
                // x,y,z,world (world at index 3)
                world = Some(normalize_world(parts[3].trim()));
            }
            5 => {
                if !is_num(parts[3]) {
                    // task form: x,y,z,world,pitch
                    world = Some(normalize_world(parts[3].trim()));
                    pitch = parts[4].trim().parse().ok()?;
                } else {
                    // java form: x,y,z,pitch,yaw
                    pitch = parts[3].trim().parse().ok()?;
                    yaw = parts[4].trim().parse().ok()?;
                }
            }
            6 => {
                if !is_num(parts[3]) {
                    // task form: x,y,z,world,pitch,yaw
                    world = Some(normalize_world(parts[3].trim()));
                    pitch = parts[4].trim().parse().ok()?;
                    yaw = parts[5].trim().parse().ok()?;
                } else {
                    // java form: x,y,z,pitch,yaw,world
                    pitch = parts[3].trim().parse().ok()?;
                    yaw = parts[4].trim().parse().ok()?;
                    world = Some(normalize_world(parts[5].trim()));
                }
            }
            _ => {}
        }
        Some(LocationTag { x, y, z, yaw, pitch, world })
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

fn normalize_world(w: &str) -> String {
    let w = w.trim();
    if w.len() >= 2 && w[..2].eq_ignore_ascii_case("w@") {
        w[2..].to_string()
    } else {
        w.to_string()
    }
}

impl ObjectTagImpl for LocationTag {
    fn prefix(&self) -> String {
        "l".to_string()
    }
    fn identify(&self) -> String {
        if self.yaw != 0.0 || self.pitch != 0.0 {
            format!(
                "l@{},{},{},{},{}{}",
                util::double_to_string(self.x),
                util::double_to_string(self.y),
                util::double_to_string(self.z),
                util::float_to_clean_string(self.pitch),
                util::float_to_clean_string(self.yaw),
                self.world.as_ref().map(|w| format!(",{}", w)).unwrap_or_default()
            )
        } else {
            format!(
                "l@{},{},{}{}",
                util::double_to_string(self.x),
                util::double_to_string(self.y),
                util::double_to_string(self.z),
                self.world.as_ref().map(|w| format!(",{}", w)).unwrap_or_default()
            )
        }
    }
    fn identify_simple(&self) -> String {
        match &self.world {
            Some(w) => format!("l@{},{},{},{}", self.block_x(), self.block_y(), self.block_z(), w),
            None => format!("l@{},{},{}", self.block_x(), self.block_y(), self.block_z()),
        }
    }
    fn object_type_name(&self) -> &'static str {
        "LocationTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_unique(&self) -> bool {
        false
    }
}

/// Parse a location-or-vector parameter into coordinates.
fn parse_coords(s: &str) -> Option<(f64, f64, f64)> {
    let s = s.trim();
    let s = if s.len() >= 2 && s[..2].eq_ignore_ascii_case("l@") {
        &s[2..]
    } else if s.len() >= 4 && s[..4].eq_ignore_ascii_case("vec@") {
        &s[4..]
    } else {
        s
    };
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() < 3 {
        return None;
    }
    Some((
        parts[0].trim().parse().ok()?,
        parts[1].trim().parse().ok()?,
        parts[2].trim().parse().ok()?,
    ))
}

fn param_coords(attr: &mut Attribute) -> Option<(f64, f64, f64)> {
    let p = attr.get_param_object()?;
    parse_coords(&p.as_element().value)
}

fn param_location(attr: &mut Attribute) -> Option<LocationTag> {
    let p = attr.get_param_object()?;
    let s = p.as_element().value;
    LocationTag::from_identify(&s).or_else(|| {
        parse_coords(&s).map(|(x, y, z)| LocationTag::at(x, y, z))
    })
}

fn flag_id(o: &ObjectTag) -> Option<String> {
    o.downcast_ref::<LocationTag>().map(|l| l.identify())
}

fn optional_double(a: &mut Attribute) -> Option<f64> {
    if !a.has_param() {
        return None;
    }
    a.get_param_object().map(|p| p.as_element().to_double().unwrap_or(1.0))
}

/// Register LocationTag's tags.
pub fn register_tags() {
    let p = get_processor("LocationTag");

    p.register_tag("ElementTag", "x", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_double(l.x)))
    }, true, &[]);
    p.register_tag("ElementTag", "y", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_double(l.y)))
    }, true, &[]);
    p.register_tag("ElementTag", "z", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_double(l.z)))
    }, true, &[]);
    p.register_tag("ElementTag", "yaw", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_double(l.yaw as f64)))
    }, true, &[]);
    p.register_tag("ElementTag", "pitch", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_double(l.pitch as f64)))
    }, true, &[]);

    p.register_tag("ElementTag", "block_x", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_int(l.block_x())))
    }, true, &[]);
    p.register_tag("ElementTag", "block_y", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_int(l.block_y())))
    }, true, &[]);
    p.register_tag("ElementTag", "block_z", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::from_int(l.block_z())))
    }, true, &[]);

    p.register_tag("WorldTag", "world", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        l.world.as_ref().map(|w| ObjectTag::new(crate::obj::core::world::WorldTag::new(w)))
    }, true, &[]);

    p.register_tag("LocationTag", "block", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| l.block().obj())
    }, true, &[]);

    p.register_tag("LocationTag", "center", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| l.center().obj())
    }, true, &[]);

    p.register_tag("ElementTag", "simple", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| ObjectTag::new(ElementTag::new(l.identify_simple())))
    }, true, &[]);

    p.register_tag("ElementTag", "formatted", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(ObjectTag::new(ElementTag::new(format!(
            "X '{}', Y '{}', Z '{}', in world '{}'",
            l.block_x(),
            l.block_y(),
            l.block_z(),
            l.world.clone().unwrap_or_default()
        ))))
    }, true, &[]);

    p.register_tag_with_param("LocationTag", "ElementTag", "add", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?.clone();
        let s = param.as_element().value;
        let (x, y, z) = parse_coords(&s).or_else(|| {
            let loc = LocationTag::from_identify(&s)?;
            Some((loc.x, loc.y, loc.z))
        })?;
        let _ = a;
        Some(l.add(x, y, z).obj())
    }, true, &[]);

    p.register_tag_with_param("LocationTag", "ElementTag", "subtract", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?.clone();
        let s = param.as_element().value;
        let (x, y, z) = parse_coords(&s).or_else(|| {
            let loc = LocationTag::from_identify(&s)?;
            Some((loc.x, loc.y, loc.z))
        })?;
        let _ = a;
        Some(l.subtract(x, y, z).obj())
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "distance", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        let (x, y, z) = parse_coords(&param.as_element().value)?;
        let _ = a;
        Some(ObjectTag::new(ElementTag::from_double(
            ((l.x - x).powi(2) + (l.y - y).powi(2) + (l.z - z).powi(2)).sqrt(),
        )))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "distance_squared", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        let (x, y, z) = parse_coords(&param.as_element().value)?;
        let _ = a;
        Some(ObjectTag::new(ElementTag::from_double(
            (l.x - x).powi(2) + (l.y - y).powi(2) + (l.z - z).powi(2),
        )))
    }, true, &[]);

    p.register_tag("LocationTag", "round", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| l.round_all().obj())
    }, true, &[]);
    p.register_tag("LocationTag", "round_up", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| l.round_up().obj())
    }, true, &[]);
    p.register_tag("LocationTag", "round_down", |_a, o| {
        o.downcast_ref::<LocationTag>().map(|l| l.round_down().obj())
    }, true, &["floor"]);
    p.register_tag_with_param("LocationTag", "ElementTag", "round_to", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        let places = param.as_element().to_int().unwrap_or(0) as i32;
        let _ = a;
        Some(l.round_to(places).obj())
    }, true, &[]);

    p.register_tag("LocationTag", "above", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.add(0.0, n, 0.0).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "below", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.subtract(0.0, n, 0.0).obj())
    }, true, &[]);

    p.register_tag("LocationTag", "north", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.add(0.0, 0.0, -n).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "south", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.add(0.0, 0.0, n).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "east", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.add(n, 0.0, 0.0).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "west", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.add(-n, 0.0, 0.0).obj())
    }, true, &[]);

    p.register_tag("LocationTag", "forward", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        let d = l.direction();
        Some(l.add(d.x * n, d.y * n, d.z * n).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "backward", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        let d = l.direction();
        Some(l.subtract(d.x * n, d.y * n, d.z * n).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "forward_flat", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        let mut flat = l.clone();
        flat.pitch = 0.0;
        let d = flat.direction();
        Some(l.add(d.x * n, 0.0, d.z * n).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "backward_flat", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        let mut flat = l.clone();
        flat.pitch = 0.0;
        let d = flat.direction();
        Some(l.subtract(d.x * n, 0.0, d.z * n).obj())
    }, true, &[]);

    p.register_tag("LocationTag", "up", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.add(0.0, n, 0.0).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "down", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        Some(l.subtract(0.0, n, 0.0).obj())
    }, true, &[]);

    p.register_tag("LocationTag", "left", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        let mut flat = l.clone();
        flat.pitch = 0.0;
        let d = flat.direction();
        Some(l.add(d.z * n, 0.0, -d.x * n).obj())
    }, true, &[]);
    p.register_tag("LocationTag", "right", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let n = optional_double(a).unwrap_or(1.0);
        let mut flat = l.clone();
        flat.pitch = 0.0;
        let d = flat.direction();
        Some(l.add(-d.z * n, 0.0, d.x * n).obj())
    }, true, &[]);

    p.register_tag_with_param("LocationTag", "ElementTag", "with_x", |_a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(l.clone().with_x(param.as_element().to_double().unwrap_or(0.0)).obj())
    }, true, &[]);
    p.register_tag_with_param("LocationTag", "ElementTag", "with_y", |_a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(l.clone().with_y(param.as_element().to_double().unwrap_or(0.0)).obj())
    }, true, &[]);
    p.register_tag_with_param("LocationTag", "ElementTag", "with_z", |_a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(l.clone().with_z(param.as_element().to_double().unwrap_or(0.0)).obj())
    }, true, &[]);
    p.register_tag_with_param("LocationTag", "ElementTag", "with_yaw", |_a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(l.clone().with_yaw(param.as_element().to_double().unwrap_or(0.0) as f32).obj())
    }, true, &[]);
    p.register_tag_with_param("LocationTag", "ElementTag", "with_pitch", |_a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(l.clone().with_pitch(param.as_element().to_double().unwrap_or(0.0) as f32).obj())
    }, true, &[]);
    p.register_tag_with_param("LocationTag", "ElementTag", "with_world", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        let _ = a;
        Some(l.clone().with_world_name(&param.as_element().value).obj())
    }, true, &[]);

    p.register_tag("MaterialTag", "material", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(ObjectTag::new(crate::obj::core::material::MaterialTag::new(&l.material_name())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_liquid", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let m = l.material_name();
        let liquid = matches!(m.as_str(), "water" | "lava" | "flowing_water" | "flowing_lava" | "bubble_column");
        Some(ObjectTag::new(ElementTag::from_bool(liquid)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_passable", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let m = l.material_name();
        let passable = matches!(m.as_str(), "air" | "cave_air" | "void_air" | "water" | "lava" | "short_grass" | "grass" | "tall_grass" | "fern" | "dead_bush" | "fire" | "soul_fire" | "torch" | "wall_torch" | "redstone_wire" | "vine" | "light");
        Some(ObjectTag::new(ElementTag::from_bool(passable)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_within", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let area = a.get_param_object()?.as_element().value;
        Some(ObjectTag::new(ElementTag::from_bool(crate::obj::core::area::contains(&area, l))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_within_border", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let inside = l.x.abs() < 29_999_984.0 && l.z.abs() < 29_999_984.0;
        Some(ObjectTag::new(ElementTag::from_bool(inside)))
    }, true, &[]);

    p.register_tag("ChunkTag", "chunk", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(ObjectTag::new(crate::obj::core::chunk::ChunkTag::new(
            l.chunk_x(),
            l.chunk_z(),
            l.world.clone().unwrap_or_default(),
        )))
    }, true, &[]);

    p.register_tag("ElementTag", "is_chunk_loaded", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(
            crate::obj::core::chunk::is_loaded(l.chunk_x(), l.chunk_z(), l.world.as_deref()),
        )))
    }, true, &[]);

    p.register_tag("BiomeTag", "biome", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let biome = crate::obj::core::biome::biome_at(l.block_x(), l.block_y(), l.block_z(), l.world.as_deref());
        Some(ObjectTag::new(crate::obj::core::biome::BiomeTag::new(&biome)))
    }, true, &[]);

    p.register_tag("CuboidTag", "to_cuboid", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        Some(ObjectTag::new(crate::obj::core::cuboid::CuboidTag::from_corners(l.clone(), l.clone())))
    }, true, &[]);

    p.register_tag("ListTag", "find_players_within", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let r = a.get_param_object().and_then(|p| p.as_element().to_double()).unwrap_or(10.0);
        let list = crate::obj::core::player::players_within(l, r);
        Some(ObjectTag::new(list))
    }, true, &[]);

    p.register_tag("ListTag", "find_entities_within", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let r = a.get_param_object().and_then(|p| p.as_element().to_double()).unwrap_or(10.0);
        let list = crate::obj::core::entity::entities_within(l, r);
        Some(ObjectTag::new(list))
    }, true, &[]);

    p.register_tag("ListTag", "find_npcs_within", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let r = a.get_param_object().and_then(|p| p.as_element().to_double()).unwrap_or(10.0);
        let list = crate::obj::core::npc::npcs_within(l, r);
        Some(ObjectTag::new(list))
    }, true, &[]);

    p.register_tag("ElementTag", "vector_to_face", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let yaw = (l.yaw as f64).rem_euclid(360.0);
        let face = if yaw >= 315.0 || yaw < 45.0 {
            "SOUTH"
        } else if yaw < 135.0 {
            "WEST"
        } else if yaw < 225.0 {
            "NORTH"
        } else {
            "EAST"
        };
        Some(ObjectTag::new(ElementTag::new(face)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "format", |a, o, param| {
        let l = o.downcast_ref::<LocationTag>()?;
        let fmt = param.as_element().value;
        let _ = a;
        let out = fmt
            .replace("<x>", &util::double_to_string(l.x))
            .replace("<y>", &util::double_to_string(l.y))
            .replace("<z>", &util::double_to_string(l.z))
            .replace("<world>", l.world.as_deref().unwrap_or(""))
            .replace("<yaw>", &util::float_to_clean_string(l.yaw))
            .replace("<pitch>", &util::float_to_clean_string(l.pitch));
        Some(ObjectTag::new(ElementTag::new(out)))
    }, true, &[]);


    p.register_tag("ElementTag", "block_facing", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let yaw = (l.yaw as f64).rem_euclid(360.0);
        let face = if yaw >= 315.0 || yaw < 45.0 {
            "SOUTH"
        } else if yaw < 135.0 {
            "WEST"
        } else if yaw < 225.0 {
            "NORTH"
        } else {
            "EAST"
        };
        Some(ObjectTag::new(ElementTag::new(face)))
    }, true, &[]);

    p.register_tag("LocationTag", "rotate_yaw", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let delta = a.get_param_object().and_then(|p| p.as_element().to_double()).unwrap_or(0.0);
        let mut out = l.clone();
        out.yaw = (out.yaw as f64 + delta).rem_euclid(360.0) as f32;
        Some(out.obj())
    }, true, &[]);

    p.register_tag("LocationTag", "rotate_pitch", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let delta = a.get_param_object().and_then(|p| p.as_element().to_double()).unwrap_or(0.0);
        let mut out = l.clone();
        out.pitch = ((out.pitch as f64 + delta).clamp(-90.0, 90.0)) as f32;
        Some(out.obj())
    }, true, &[]);

    p.register_tag("LocationTag", "face", |a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let target = a.get_param_object().and_then(|p| LocationTag::from_identify(&p.as_element().value))?;
        let dx = target.x - l.x;
        let dz = target.z - l.z;
        let dy = target.y - l.y;
        let yaw = (-dx).atan2(dz).to_degrees().rem_euclid(360.0) as f32;
        let pitch = (-dy).atan2((dx * dx + dz * dz).sqrt()).to_degrees() as f32;
        let mut out = l.clone();
        out.yaw = yaw;
        out.pitch = pitch;
        Some(out.obj())
    }, true, &[]);

    p.register_tag("ListTag", "areas", |_a, o| {
        let _l = o.downcast_ref::<LocationTag>()?;
        Some(ObjectTag::new(ListTag::new()))
    }, true, &[]);

    p.register_tag("ElementTag", "has_inventory", |_a, o| {
        let l = o.downcast_ref::<LocationTag>()?;
        let m = l.material_name();
        let has = matches!(m.as_str(), "chest" | "trapped_chest" | "barrel" | "furnace" | "hopper" | "dispenser" | "dropper" | "shulker_box" | "ender_chest");
        Some(ObjectTag::new(ElementTag::from_bool(has)))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("LocationTag", flag_id);

    let _ = param_coords;
    let _ = param_location;
    let _ = ListTag::new;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{Attribute, TagContext};

    fn eval(obj: &ObjectTag, path: &str) -> Option<String> {
        crate::obj::register_core_objects();
        let ctx = TagContext::no_debug();
        let mut attr = Attribute::new(path, ctx).ok()?;
        obj.get_object_attribute(&mut attr).map(|o| o.identify())
    }

    #[test]
    fn parses_identify_forms() {
        let l = LocationTag::from_identify("l@1,2.5,3,world").unwrap();
        assert_eq!(l.x, 1.0);
        assert_eq!(l.y, 2.5);
        assert_eq!(l.world.as_deref(), Some("world"));

        let l = LocationTag::from_identify("l@1,2,3,45,90,space").unwrap();
        assert_eq!(l.pitch, 45.0);
        assert_eq!(l.yaw, 90.0);
        assert_eq!(l.world.as_deref(), Some("space"));

        let l = LocationTag::from_identify("l@1,2,3,space,45,90").unwrap();
        assert_eq!(l.pitch, 45.0);
        assert_eq!(l.yaw, 90.0);
    }

    #[test]
    fn basic_tags() {
        let loc = LocationTag::from_identify("l@10,64,20,world").unwrap().obj();
        assert_eq!(eval(&loc, "x").as_deref(), Some("10"));
        assert_eq!(eval(&loc, "y").as_deref(), Some("64"));
        assert_eq!(eval(&loc, "z").as_deref(), Some("20"));
        assert_eq!(eval(&loc, "block_x").as_deref(), Some("10"));
        assert_eq!(eval(&loc, "world.name").as_deref(), Some("world"));
        assert_eq!(eval(&loc, "above").map(|s| s.contains("65")), Some(true));
        assert_eq!(eval(&loc, "below").map(|s| s.contains("63")), Some(true));
        assert_eq!(eval(&loc, "north").map(|s| s.contains("19")), Some(true));
    }
}

#[cfg(test)]
mod registration_tests {
    #[test]
    fn all_new_object_types_registered() {
        crate::obj::register_core_objects();
        for name in [
            "LocationTag", "EntityTag", "PlayerTag", "NPCTag", "WorldTag", "ItemTag",
            "MaterialTag", "InventoryTag", "EnchantmentTag", "CuboidTag", "BiomeTag",
            "ChunkTag", "PluginTag", "TradeTag", "EllipsoidTag", "PolygonTag",
        ] {
            assert!(crate::obj::get_object_type(name).is_some(), "missing object type {}", name);
        }
    }

    #[test]
    fn base_tags_construct_objects() {
        use crate::tags::{tag_object, TagContext};
        crate::obj::register_core_objects();
        let ctx = TagContext::no_debug();
        let obj = tag_object("<item[stick]>", &ctx);
        assert_eq!(obj.object_type_name(), "ItemTag");
        let obj = tag_object("<material[stone]>", &ctx);
        assert_eq!(obj.object_type_name(), "MaterialTag");
        let obj = tag_object("<location[1,2,3,world]>", &ctx);
        assert_eq!(obj.object_type_name(), "LocationTag");
        let obj = tag_object("<cuboid[1,2,3,world,4,5,6]>", &ctx);
        assert_eq!(obj.object_type_name(), "CuboidTag");
        let obj = tag_object("<biome[plains]>", &ctx);
        assert_eq!(obj.object_type_name(), "BiomeTag");
    }

    #[test]
    fn item_and_cuboid_tags_work() {
        use crate::tags::{tag, TagContext};
        crate::obj::register_core_objects();
        let ctx = TagContext::no_debug();
        assert_eq!(tag("<item[stick].material.name>", &ctx), "stick");
        assert_eq!(tag("<item[diamond].is_empty>", &ctx), "false");
        assert_eq!(tag("<cuboid[0,0,0,world,1,1,1].volume>", &ctx), "8");
        assert_eq!(tag("<material[stone].is_block>", &ctx), "true");
        assert_eq!(tag("<enchantment[sharpness].max_level>", &ctx), "5");
    }
}
