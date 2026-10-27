//! `CustomObjectTag`: a user-defined object composed of arbitrary key/value data.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Debug, Default)]
pub struct CustomObjectTag {
    pub map: crate::obj::MapTag,
    pub prefix: String,
}

impl CustomObjectTag {
    pub fn new(map: crate::obj::MapTag) -> Self {
        CustomObjectTag { map, prefix: "custom".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 7 && s[..7].eq_ignore_ascii_case("custom@") {
            s = &s[7..];
        }
        crate::obj::MapTag::from_identify(s).map(CustomObjectTag::new)
    }
}

impl ObjectTagImpl for CustomObjectTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("custom@{}", self.map.identify().trim_start_matches("map@"))
    }
    fn object_type_name(&self) -> &'static str {
        "CustomObjectTag"
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

pub fn register_tags() {
    let p = get_processor("CustomObjectTag");

    p.register_tag("ObjectTag", "get", |a, o| {
        let c = o.downcast_ref::<CustomObjectTag>()?;
        let key = a.get_param_object()?.as_element().value;
        c.map.get(&key).cloned()
    }, false, &[]);

    p.register_tag("ListTag", "keys", |_a, o| {
        let c = o.downcast_ref::<CustomObjectTag>()?;
        Some(ObjectTag::new(crate::obj::ListTag::from_strings(&c.map.keys())))
    }, true, &[]);
}
