//! `AreaContainmentObject` helpers: ellipsoid, cuboid and polygon containment math,
//! plus the shared flag tags registered on flaggable object types.

use crate::obj::core::cuboid::CuboidTag;
use crate::obj::core::location::LocationTag;
use crate::obj::{get_processor, ElementTag, ObjectTag, ObjectTagImpl};
use std::any::Any;
use crate::tags::Attribute;

/// An axis-aligned or rotated ellipsoid region.
#[derive(Clone, Debug)]
pub struct Ellipsoid {
    pub center: LocationTag,
    pub size_x: f64,
    pub size_y: f64,
    pub size_z: f64,
}

impl Ellipsoid {
    pub fn new(center: LocationTag, size_x: f64, size_y: f64, size_z: f64) -> Self {
        Ellipsoid { center, size_x, size_y, size_z }
    }

    /// Parse `ellipsoid@x,y,z,world,size_x,size_y,size_z`.
    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 10 && s[..10].eq_ignore_ascii_case("ellipsoid@") {
            s = &s[10..];
        }
        let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
        if parts.len() < 7 {
            return None;
        }
        let x: f64 = parts[0].parse().ok()?;
        let y: f64 = parts[1].parse().ok()?;
        let z: f64 = parts[2].parse().ok()?;
        let world = parts[3].trim_start_matches("w@").to_string();
        let sx: f64 = parts[4].parse().ok()?;
        let sy: f64 = parts[5].parse().ok()?;
        let sz: f64 = parts[6].parse().ok()?;
        Some(Ellipsoid::new(LocationTag::new(x, y, z, 0.0, 0.0, Some(world)), sx, sy, sz))
    }

    pub fn contains(&self, loc: &LocationTag) -> bool {
        if let (Some(a), Some(b)) = (&self.center.world, &loc.world) {
            if !a.eq_ignore_ascii_case(b) {
                return false;
            }
        }
        if self.size_x <= 0.0 || self.size_y <= 0.0 || self.size_z <= 0.0 {
            return false;
        }
        let dx = (loc.x - self.center.x) / self.size_x;
        let dy = (loc.y - self.center.y) / self.size_y;
        let dz = (loc.z - self.center.z) / self.size_z;
        dx * dx + dy * dy + dz * dz <= 1.0
    }

    pub fn identify(&self) -> String {
        format!(
            "ellipsoid@{},{},{},{},{},{},{}",
            self.center.x, self.center.y, self.center.z,
            self.center.world.clone().unwrap_or_default(),
            self.size_x, self.size_y, self.size_z
        )
    }
}

/// A 2D polygon region at a fixed Y range.
#[derive(Clone, Debug)]
pub struct Polygon {
    pub points: Vec<(f64, f64)>,
    pub world: Option<String>,
    pub min_y: f64,
    pub max_y: f64,
}

impl Polygon {
    pub fn new(points: Vec<(f64, f64)>, world: Option<String>, min_y: f64, max_y: f64) -> Self {
        Polygon { points, world, min_y, max_y }
    }

    /// Parse `polygon@x1,y1;x2,y2;...;world,min_y,max_y` (loosely).
    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 8 && s[..8].eq_ignore_ascii_case("polygon@") {
            s = &s[8..];
        }
        let mut points = Vec::new();
        let mut world = None;
        let mut min_y = -64.0;
        let mut max_y = 320.0;
        for chunk in s.split(';') {
            let chunk = chunk.trim();
            if chunk.is_empty() {
                continue;
            }
            let parts: Vec<&str> = chunk.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(x), Ok(y)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    points.push((x, y));
                }
            } else if parts.len() == 3 {
                // world,min_y,max_y
                world = Some(parts[0].trim_start_matches("w@").to_string());
                min_y = parts[1].trim().parse().unwrap_or(min_y);
                max_y = parts[2].trim().parse().unwrap_or(max_y);
            }
        }
        if points.len() < 3 {
            return None;
        }
        Some(Polygon::new(points, world, min_y, max_y))
    }

    /// Ray-casting point-in-polygon test.
    pub fn contains_2d(&self, x: f64, z: f64) -> bool {
        let mut inside = false;
        let n = self.points.len();
        let mut j = n - 1;
        for i in 0..n {
            let (xi, zi) = self.points[i];
            let (xj, zj) = self.points[j];
            if ((zi > z) != (zj > z)) && (x < (xj - xi) * (z - zi) / (zj - zi) + xi) {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    pub fn contains(&self, loc: &LocationTag) -> bool {
        if let (Some(a), Some(b)) = (&self.world, &loc.world) {
            if !a.eq_ignore_ascii_case(b) {
                return false;
            }
        }
        if loc.y < self.min_y || loc.y > self.max_y {
            return false;
        }
        self.contains_2d(loc.x, loc.z)
    }

    pub fn area(&self) -> f64 {
        let mut sum = 0.0;
        let n = self.points.len();
        for i in 0..n {
            let (x1, z1) = self.points[i];
            let (x2, z2) = self.points[(i + 1) % n];
            sum += x1 * z2 - x2 * z1;
        }
        (sum / 2.0).abs()
    }
}

/// Test whether a location lies within an area identity string.
pub fn contains(area: &str, loc: &LocationTag) -> bool {
    let a = area.trim();
    if a.len() >= 3 && a[..3].eq_ignore_ascii_case("cu@") {
        return CuboidTag::from_identify(a).map(|c| c.contains(loc)).unwrap_or(false);
    }
    if a.len() >= 10 && a[..10].eq_ignore_ascii_case("ellipsoid@") {
        return Ellipsoid::from_identify(a).map(|e| e.contains(loc)).unwrap_or(false);
    }
    if a.len() >= 8 && a[..8].eq_ignore_ascii_case("polygon@") {
        return Polygon::from_identify(a).map(|p| p.contains(loc)).unwrap_or(false);
    }
    false
}


/// Object wrapper for an ellipsoid region.
#[derive(Clone, Debug)]
pub struct EllipsoidObject(pub Ellipsoid);

impl ObjectTagImpl for EllipsoidObject {
    fn prefix(&self) -> String {
        "ellipsoid".to_string()
    }
    fn identify(&self) -> String {
        self.0.identify()
    }
    fn object_type_name(&self) -> &'static str {
        "EllipsoidTag"
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

/// Object wrapper for a polygon region.
#[derive(Clone, Debug)]
pub struct PolygonObject(pub Polygon);

impl ObjectTagImpl for PolygonObject {
    fn prefix(&self) -> String {
        "polygon".to_string()
    }
    fn identify(&self) -> String {
        let mut out = String::from("polygon@");
        for (i, (x, z)) in self.0.points.iter().enumerate() {
            if i > 0 {
                out.push(';');
            }
            out.push_str(&format!("{},{}", x, z));
        }
        out.push_str(&format!(";{},{}", self.0.world.clone().unwrap_or_default(), self.0.min_y));
        out.push_str(&format!(",{}", self.0.max_y));
        out
    }
    fn object_type_name(&self) -> &'static str {
        "PolygonTag"
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

/// Register the shared `flag[...]` tags on a flaggable object type.
pub fn register_flag_tags(type_name: &'static str, id_fn: fn(&ObjectTag) -> Option<String>) {
    let p = get_processor(type_name);

    let id_for = move |o: &ObjectTag| id_fn(o);

    p.register_tag("ElementTag", "flag", move |a: &mut Attribute, o: &ObjectTag| {
        let id = id_for(o)?;
        let key = a.get_param_object()?.as_element().value.to_lowercase();
        let tracker = crate::flags::tracker_for(&id);
        match tracker.get(&key) {
            Some(v) => Some(v),
            None => Some(ObjectTag::new(ElementTag::new("null"))),
        }
    }, false, &[]);

    p.register_tag("ElementTag", "has_flag", move |a: &mut Attribute, o: &ObjectTag| {
        let id = id_for(o)?;
        let key = a.get_param_object()?.as_element().value.to_lowercase();
        let tracker = crate::flags::tracker_for(&id);
        Some(ObjectTag::new(ElementTag::from_bool(tracker.has(&key))))
    }, false, &[]);

    p.register_tag("ListTag", "flags", move |a: &mut Attribute, o: &ObjectTag| {
        let id = id_for(o)?;
        let tracker = crate::flags::tracker_for(&id);
        let prefix = a.get_param_object().map(|p| p.as_element().value);
        Some(ObjectTag::new(tracker.list(prefix.as_deref())))
    }, false, &[]);
}

/// Register `EllipsoidTag` and `PolygonTag` object types and their tags.
pub fn register_object_types() {
    use crate::obj::{register_object_type, ObjectTagProcessor, ObjectType};
    use crate::obj::core::location::LocationTag;

    let mk = |name: &'static str, short: &'static str, prefix: &'static str| ObjectType {
        name,
        short_name: short,
        prefix,
        processor: std::sync::Arc::new(ObjectTagProcessor::new()),
        converter: None,
        should_be_checker: None,
    };
    register_object_type(mk("EllipsoidTag", "Ellipsoid", "ellipsoid"));
    register_object_type(mk("PolygonTag", "Polygon", "polygon"));

    let p = get_processor("EllipsoidTag");
    p.register_tag("ElementTag", "volume", |_a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(
            (4.0 / 3.0) * std::f64::consts::PI * e.0.size_x * e.0.size_y * e.0.size_z,
        )))
    }, true, &[]);
    p.register_tag("LocationTag", "center", |_a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        Some(e.0.center.obj())
    }, true, &[]);
    p.register_tag("ElementTag", "size_x", |_a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(e.0.size_x)))
    }, true, &[]);
    p.register_tag("ElementTag", "size_y", |_a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(e.0.size_y)))
    }, true, &[]);
    p.register_tag("ElementTag", "size_z", |_a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(e.0.size_z)))
    }, true, &[]);
    p.register_tag("ElementTag", "contains", |a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        let loc = a.get_param_object().and_then(|p| LocationTag::from_identify(&p.as_element().value))?;
        Some(ObjectTag::new(ElementTag::from_bool(e.0.contains(&loc))))
    }, true, &[]);
    p.register_tag("ElementTag", "world", |_a, o| {
        let e = o.downcast_ref::<crate::obj::core::area::EllipsoidObject>()?;
        Some(ObjectTag::new(ElementTag::new(e.0.center.world.clone().unwrap_or_default())))
    }, true, &[]);

    let p = get_processor("PolygonTag");
    p.register_tag("ElementTag", "area", |_a, o| {
        let p = o.downcast_ref::<crate::obj::core::area::PolygonObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(p.0.area())))
    }, true, &[]);
    p.register_tag("ElementTag", "contains", |a, o| {
        let p = o.downcast_ref::<crate::obj::core::area::PolygonObject>()?;
        let loc = a.get_param_object().and_then(|x| LocationTag::from_identify(&x.as_element().value))?;
        Some(ObjectTag::new(ElementTag::from_bool(p.0.contains(&loc))))
    }, true, &[]);
    p.register_tag("ElementTag", "point_count", |_a, o| {
        let p = o.downcast_ref::<crate::obj::core::area::PolygonObject>()?;
        Some(ObjectTag::new(ElementTag::from_int(p.0.points.len() as i64)))
    }, true, &[]);
    p.register_tag("ElementTag", "min_y", |_a, o| {
        let p = o.downcast_ref::<crate::obj::core::area::PolygonObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(p.0.min_y)))
    }, true, &[]);
    p.register_tag("ElementTag", "max_y", |_a, o| {
        let p = o.downcast_ref::<crate::obj::core::area::PolygonObject>()?;
        Some(ObjectTag::new(ElementTag::from_double(p.0.max_y)))
    }, true, &[]);
}
