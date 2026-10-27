//! `ScriptTag`: a reference to a script container.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Debug, PartialEq)]
pub struct ScriptTag {
    pub name: String,
    pub prefix: String,
}

impl ScriptTag {
    pub fn new(name: &str) -> Self {
        ScriptTag { name: name.to_string(), prefix: "script".to_string() }
    }

    pub fn from_identify(s: &str) -> Self {
        let mut s = s.trim();
        if s.len() >= 7 && s[..7].eq_ignore_ascii_case("script@") {
            s = &s[7..];
        }
        ScriptTag::new(s)
    }
}

impl ObjectTagImpl for ScriptTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("script@{}", self.name)
    }
    fn identify_simple(&self) -> String {
        self.name.clone()
    }
    fn object_type_name(&self) -> &'static str {
        "ScriptTag"
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
        crate::scripts::registry::get_container(&self.name).is_some()
    }
}

pub fn register_tags() {
    let p = get_processor("ScriptTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<ScriptTag>().map(|s| ObjectTag::new(crate::obj::ElementTag::new(&s.name)))
    }, true, &[]);

    p.register_tag("ElementTag", "exists", |_a, o| {
        let exists = o.downcast_ref::<ScriptTag>().map(|s| crate::scripts::registry::get_container(&s.name).is_some()).unwrap_or(false);
        Some(ObjectTag::new(crate::obj::ElementTag::from_bool(exists)))
    }, true, &[]);

    p.register_tag("ElementTag", "type", |_a, o| {
        let s = o.downcast_ref::<ScriptTag>()?;
        let c = crate::scripts::registry::get_container(&s.name)?;
        Some(ObjectTag::new(crate::obj::ElementTag::new(c.container_type())))
    }, true, &[]);
}
