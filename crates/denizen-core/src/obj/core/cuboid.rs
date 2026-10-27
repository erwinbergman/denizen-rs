//! `CuboidTag`: a cuboidal region, identified as `cu@x1,y1,z1,world,x2,y2,z2`.

use crate::obj::core::location::LocationTag;
use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;

/// A low/high corner pair defining one cuboid member.
#[derive(Clone, Debug)]
pub struct CuboidPair {
    pub low: LocationTag,
    pub high: LocationTag,
}

impl CuboidPair {
    pub fn new(a: LocationTag, b: LocationTag) -> Self {
        let (low, high) = normalize_pair(a, b);
        CuboidPair { low, high }
    }

    pub fn contains(&self, loc: &LocationTag) -> bool {
        let x = loc.block_x();
        let y = loc.block_y();
        let z = loc.block_z();
        x >= self.low.block_x()
            && x <= self.high.block_x()
            && y >= self.low.block_y()
            && y <= self.high.block_y()
            && z >= self.low.block_z()
            && z <= self.high.block_z()
    }

    pub fn volume(&self) -> f64 {
        let sx = (self.high.block_x() - self.low.block_x() + 1) as f64;
        let sy = (self.high.block_y() - self.low.block_y() + 1) as f64;
        let sz = (self.high.block_z() - self.low.block_z() + 1) as f64;
        sx * sy * sz
    }

    pub fn size(&self) -> (f64, f64, f64) {
        (
            (self.high.block_x() - self.low.block_x() + 1) as f64,
            (self.high.block_y() - self.low.block_y() + 1) as f64,
            (self.high.block_z() - self.low.block_z() + 1) as f64,
        )
    }

    pub fn center(&self) -> LocationTag {
        LocationTag::new(
            (self.low.x + self.high.x + 1.0) / 2.0,
            (self.low.y + self.high.y + 1.0) / 2.0,
            (self.low.z + self.high.z + 1.0) / 2.0,
            0.0,
            0.0,
            self.low.world.clone(),
        )
    }
}

fn normalize_pair(a: LocationTag, b: LocationTag) -> (LocationTag, LocationTag) {
    let mut low = a;
    let mut high = b;
    if low.world.is_none() {
        low.world = high.world.clone();
    }
    if high.world.is_none() {
        high.world = low.world.clone();
    }
    let lx = low.block_x().min(high.block_x());
    let ly = low.block_y().min(high.block_y());
    let lz = low.block_z().min(high.block_z());
    let hx = low.block_x().max(high.block_x());
    let hy = low.block_y().max(high.block_y());
    let hz = low.block_z().max(high.block_z());
    let world = low.world.clone();
    low = LocationTag::new(lx as f64, ly as f64, lz as f64, 0.0, 0.0, world.clone());
    high = LocationTag::new(hx as f64, hy as f64, hz as f64, 0.0, 0.0, world);
    (low, high)
}

/// A cuboid region (possibly multi-member).
#[derive(Clone, Debug)]
pub struct CuboidTag {
    pub pairs: Vec<CuboidPair>,
    pub prefix: String,
}

impl CuboidTag {
    pub fn from_corners(a: LocationTag, b: LocationTag) -> Self {
        CuboidTag { pairs: vec![CuboidPair::new(a, b)], prefix: "Cuboid".to_string() }
    }

    /// Parse the task format `cu@x1,y1,z1,world,x2,y2,z2` and the Java format
    /// `cu@world,x1,y1,z1,x2,y2,z2`, plus multi-member variants.
    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 3 && s[..3].eq_ignore_ascii_case("cu@") {
            s = &s[3..];
        }
        if s.is_empty() {
            return None;
        }
        let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
        // Determine format by whether the first token is numeric.
        let first_is_num = parts.first().and_then(|p| p.parse::<f64>().ok()).is_some();
        let (world, nums): (Option<String>, Vec<f64>) = if first_is_num {
            // task format: x,y,z,world,...
            let mut world = None;
            let mut nums = Vec::new();
            for (i, p) in parts.iter().enumerate() {
                if i == 3 && p.parse::<f64>().is_err() {
                    world = Some(p.trim_start_matches("w@").to_string());
                    continue;
                }
                if let Ok(v) = p.parse::<f64>() {
                    nums.push(v);
                }
            }
            (world, nums)
        } else {
            // java format: world,x,y,z,...
            let world = Some(parts[0].trim_start_matches("w@").to_string());
            let nums: Vec<f64> = parts[1..].iter().filter_map(|p| p.parse::<f64>().ok()).collect();
            (world, nums)
        };
        if nums.len() < 6 {
            return None;
        }
        let mut pairs = Vec::new();
        let mut i = 0;
        while i + 5 < nums.len() {
            let low = LocationTag::new(nums[i], nums[i + 1], nums[i + 2], 0.0, 0.0, world.clone());
            let high = LocationTag::new(nums[i + 3], nums[i + 4], nums[i + 5], 0.0, 0.0, world.clone());
            pairs.push(CuboidPair::new(low, high));
            i += 6;
        }
        if pairs.is_empty() {
            return None;
        }
        Some(CuboidTag { pairs, prefix: "Cuboid".to_string() })
    }

    pub fn contains(&self, loc: &LocationTag) -> bool {
        self.pairs.iter().any(|p| p.contains(loc))
    }

    pub fn volume(&self) -> f64 {
        self.pairs.iter().map(|p| p.volume()).sum()
    }

    pub fn first(&self) -> Option<&CuboidPair> {
        self.pairs.first()
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }

    pub fn identify_full(&self) -> String {
        let world = self
            .pairs
            .first()
            .and_then(|p| p.low.world.clone())
            .unwrap_or_default();
        let mut out = format!("cu@{}", world);
        for p in &self.pairs {
            out.push_str(&format!(
                ",{},{},{},{},{},{}",
                p.low.block_x(),
                p.low.block_y(),
                p.low.block_z(),
                p.high.block_x(),
                p.high.block_y(),
                p.high.block_z()
            ));
        }
        out
    }

    pub fn blocks(&self) -> Vec<LocationTag> {
        let mut out = Vec::new();
        for p in &self.pairs {
            for x in p.low.block_x()..=p.high.block_x() {
                for y in p.low.block_y()..=p.high.block_y() {
                    for z in p.low.block_z()..=p.high.block_z() {
                        out.push(LocationTag::new(x as f64, y as f64, z as f64, 0.0, 0.0, p.low.world.clone()));
                    }
                }
            }
        }
        out
    }
}

impl ObjectTagImpl for CuboidTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        self.identify_full()
    }
    fn identify_simple(&self) -> String {
        self.identify_full()
    }
    fn object_type_name(&self) -> &'static str {
        "CuboidTag"
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

fn param_location(attr: &mut Attribute) -> Option<LocationTag> {
    let p = attr.get_param_object()?;
    let s = p.as_element().value;
    LocationTag::from_identify(&s).or_else(|| {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() >= 3 {
            Some(LocationTag::at(
                parts[0].trim().parse().ok()?,
                parts[1].trim().parse().ok()?,
                parts[2].trim().parse().ok()?,
            ))
        } else {
            None
        }
    })
}

/// Register CuboidTag's tags.
pub fn register_tags() {
    let p = get_processor("CuboidTag");

    p.register_tag("ElementTag", "volume", |_a, o| {
        o.downcast_ref::<CuboidTag>().map(|c| ObjectTag::new(ElementTag::from_double(c.volume())))
    }, true, &[]);

    p.register_tag("LocationTag", "size", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        let pair = c.first()?;
        let (sx, sy, sz) = pair.size();
        Some(ObjectTag::new(LocationTag::new(sx, sy, sz, 0.0, 0.0, pair.low.world.clone())))
    }, true, &[]);

    p.register_tag("LocationTag", "min", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        Some(c.first()?.low.obj())
    }, true, &[]);

    p.register_tag("LocationTag", "max", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        Some(c.first()?.high.obj())
    }, true, &[]);

    p.register_tag("LocationTag", "center", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        Some(c.first()?.center().obj())
    }, true, &[]);

    p.register_tag("ListTag", "corners", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        let mut items = Vec::new();
        for pair in &c.pairs {
            let lx = pair.low.block_x();
            let ly = pair.low.block_y();
            let lz = pair.low.block_z();
            let hx = pair.high.block_x();
            let hy = pair.high.block_y();
            let hz = pair.high.block_z();
            let w = pair.low.world.clone();
            for (x, y, z) in [
                (lx, ly, lz),
                (hx, ly, lz),
                (lx, ly, hz),
                (hx, ly, hz),
                (lx, hy, lz),
                (hx, hy, lz),
                (lx, hy, hz),
                (hx, hy, hz),
            ] {
                items.push(ObjectTag::new(LocationTag::new(x as f64, y as f64, z as f64, 0.0, 0.0, w.clone())));
            }
        }
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "contains", |a, o, param| {
        let c = o.downcast_ref::<CuboidTag>()?;
        let s = param.as_element().value;
        let contains = LocationTag::from_identify(&s)
            .map(|l| c.contains(&l))
            .unwrap_or(false);
        let _ = a;
        Some(ObjectTag::new(ElementTag::from_bool(contains)))
    }, true, &[]);

    p.register_tag_with_param("CuboidTag", "ElementTag", "expand", |a, o, param| {
        let c = o.downcast_ref::<CuboidTag>()?.clone();
        let n = param.as_element().to_int().unwrap_or(1);
        let _ = a;
        let mut pairs = c.pairs;
        for pair in pairs.iter_mut() {
            *pair = CuboidPair::new(
                pair.low.clone().add(-(n as f64), -(n as f64), -(n as f64)),
                pair.high.clone().add(n as f64, n as f64, n as f64),
            );
        }
        Some(ObjectTag::new(CuboidTag { pairs, ..c }))
    }, true, &[]);

    p.register_tag_with_param("CuboidTag", "ElementTag", "contract", |a, o, param| {
        let c = o.downcast_ref::<CuboidTag>()?.clone();
        let n = param.as_element().to_int().unwrap_or(1);
        let _ = a;
        let mut pairs = c.pairs;
        for pair in pairs.iter_mut() {
            *pair = CuboidPair::new(
                pair.low.clone().add(n as f64, n as f64, n as f64),
                pair.high.clone().add(-(n as f64), -(n as f64), -(n as f64)),
            );
        }
        Some(ObjectTag::new(CuboidTag { pairs, ..c }))
    }, true, &["shrink_one_side"]);

    p.register_tag_with_param("CuboidTag", "ElementTag", "shift", |a, o, param| {
        let c = o.downcast_ref::<CuboidTag>()?.clone();
        let s = param.as_element().value;
        let (dx, dy, dz) = parse_vector(&s)?;
        let _ = a;
        let mut pairs = c.pairs;
        for pair in pairs.iter_mut() {
            *pair = CuboidPair::new(pair.low.clone().add(dx, dy, dz), pair.high.clone().add(dx, dy, dz));
        }
        Some(ObjectTag::new(CuboidTag { pairs, ..c }))
    }, true, &[]);

    p.register_tag_with_param("CuboidTag", "ElementTag", "include", |a, o, _param| {
        let c = o.downcast_ref::<CuboidTag>()?.clone();
        let loc = param_location(a)?;
        let mut pairs = c.pairs;
        if let Some(first) = pairs.first_mut() {
            *first = CuboidPair::new(
                LocationTag::new(
                    first.low.block_x().min(loc.block_x()) as f64,
                    first.low.block_y().min(loc.block_y()) as f64,
                    first.low.block_z().min(loc.block_z()) as f64,
                    0.0,
                    0.0,
                    first.low.world.clone(),
                ),
                LocationTag::new(
                    first.high.block_x().max(loc.block_x()) as f64,
                    first.high.block_y().max(loc.block_y()) as f64,
                    first.high.block_z().max(loc.block_z()) as f64,
                    0.0,
                    0.0,
                    first.high.world.clone(),
                ),
            );
        }
        Some(ObjectTag::new(CuboidTag { pairs, ..c }))
    }, true, &[]);

    p.register_tag("ListTag", "blocks", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        let items: Vec<ObjectTag> = c.blocks().iter().map(|l| l.obj()).collect();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &["get_blocks"]);

    p.register_tag("ElementTag", "members_size", |_a, o| {
        o.downcast_ref::<CuboidTag>().map(|c| ObjectTag::new(ElementTag::from_int(c.pairs.len() as i64)))
    }, true, &[]);

    p.register_tag("ChunkTag", "chunks", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        let pair = c.first()?;
        let world = pair.low.world.clone().unwrap_or_default();
        let mut chunks = Vec::new();
        for cx in (pair.low.block_x().div_euclid(16))..=(pair.high.block_x().div_euclid(16)) {
            for cz in (pair.low.block_z().div_euclid(16))..=(pair.high.block_z().div_euclid(16)) {
                chunks.push(ObjectTag::new(crate::obj::core::chunk::ChunkTag::new(cx, cz, world.clone())));
            }
        }
        Some(ObjectTag::new(ListTag::from_objects(chunks)))
    }, true, &[]);

    p.register_tag("ElementTag", "note_name", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        Some(ObjectTag::new(ElementTag::new(c.identify_simple())))
    }, true, &[]);

    p.register_tag("ElementTag", "world", |_a, o| {
        let c = o.downcast_ref::<CuboidTag>()?;
        let w = c.first().and_then(|p| p.low.world.clone()).unwrap_or_default();
        Some(ObjectTag::new(ElementTag::new(w)))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("CuboidTag", |o| o.downcast_ref::<CuboidTag>().map(|x| x.identify()));
}

fn parse_vector(s: &str) -> Option<(f64, f64, f64)> {
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
