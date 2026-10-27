//! `VectorObject`: a 3D vector of doubles.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VectorObject {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub prefix: &'static str,
}

impl VectorObject {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        VectorObject { x, y, z, prefix: "vector" }
    }

    pub fn zero() -> Self {
        VectorObject::new(0.0, 0.0, 0.0)
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            return *self;
        }
        VectorObject::new(self.x / len, self.y / len, self.z / len)
    }

    pub fn add(&self, o: &VectorObject) -> Self {
        VectorObject::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    pub fn sub(&self, o: &VectorObject) -> Self {
        VectorObject::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    pub fn multiply(&self, f: f64) -> Self {
        VectorObject::new(self.x * f, self.y * f, self.z * f)
    }

    pub fn dot(&self, o: &VectorObject) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(&self, o: &VectorObject) -> Self {
        VectorObject::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 4 && s[..4].eq_ignore_ascii_case("vec@") {
            s = &s[4..];
        }
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(VectorObject::new(
            parts[0].trim().parse().ok()?,
            parts[1].trim().parse().ok()?,
            parts[2].trim().parse().ok()?,
        ))
    }
}

impl ObjectTagImpl for VectorObject {
    fn prefix(&self) -> String {
        self.prefix.to_string()
    }
    fn identify(&self) -> String {
        format!(
            "vec@{},{},{}",
            crate::util::double_to_string(self.x),
            crate::util::double_to_string(self.y),
            crate::util::double_to_string(self.z)
        )
    }
    fn identify_simple(&self) -> String {
        format!(
            "{},{},{}",
            crate::util::double_to_string(self.x),
            crate::util::double_to_string(self.y),
            crate::util::double_to_string(self.z)
        )
    }
    fn object_type_name(&self) -> &'static str {
        "VectorObject"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(*self)
    }
}

/// Register VectorObject tags.
pub fn register_tags() {
    let p = get_processor("VectorObject");
    p.register_tag("ElementTag", "x", |_a, o| o.downcast_ref::<VectorObject>().copied().map(|v| ObjectTag::new(crate::obj::ElementTag::from_double(v.x))), true, &[]);
    p.register_tag("ElementTag", "y", |_a, o| o.downcast_ref::<VectorObject>().copied().map(|v| ObjectTag::new(crate::obj::ElementTag::from_double(v.y))), true, &[]);
    p.register_tag("ElementTag", "z", |_a, o| o.downcast_ref::<VectorObject>().copied().map(|v| ObjectTag::new(crate::obj::ElementTag::from_double(v.z))), true, &[]);

    p.register_tag("ElementTag", "length", |_a, o| {
        o.downcast_ref::<VectorObject>().copied().map(|v| ObjectTag::new(crate::obj::ElementTag::from_double(v.length())))
    }, true, &[]);

    p.register_tag("VectorObject", "normalize", |_a, o| {
        o.downcast_ref::<VectorObject>().copied().map(|v| ObjectTag::new(v.normalized()))
    }, true, &[]);

    p.register_tag_with_param("VectorObject", "VectorObject", "add", |_a, o, param| {
        let a = o.downcast_ref::<VectorObject>().copied()?;
        let b = param.downcast_ref::<VectorObject>().copied()?;
        Some(ObjectTag::new(a.add(&b)))
    }, true, &[]);

    p.register_tag_with_param("VectorObject", "VectorObject", "subtract", |_a, o, param| {
        let a = o.downcast_ref::<VectorObject>().copied()?;
        let b = param.downcast_ref::<VectorObject>().copied()?;
        Some(ObjectTag::new(a.sub(&b)))
    }, true, &[]);

    p.register_tag_with_param("VectorObject", "VectorObject", "cross", |_a, o, param| {
        let a = o.downcast_ref::<VectorObject>().copied()?;
        let b = param.downcast_ref::<VectorObject>().copied()?;
        Some(ObjectTag::new(a.cross(&b)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "VectorObject", "dot", |_a, o, param| {
        let a = o.downcast_ref::<VectorObject>().copied()?;
        let b = param.downcast_ref::<VectorObject>().copied()?;
        Some(ObjectTag::new(crate::obj::ElementTag::from_double(a.dot(&b))))
    }, true, &[]);

    p.register_tag_with_param("VectorObject", "ElementTag", "multiply", |_a, o, param| {
        let a = o.downcast_ref::<VectorObject>().copied()?;
        let f = param.as_element().to_double().unwrap_or(1.0);
        Some(ObjectTag::new(a.multiply(f)))
    }, true, &[]);
}
