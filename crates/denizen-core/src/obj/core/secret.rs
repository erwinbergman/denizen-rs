//! `SecretTag`: a named secret value, identified as `secret@<name>`.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SecretTag {
    pub name: String,
    pub prefix: String,
}

impl SecretTag {
    pub fn new(name: &str) -> Self {
        SecretTag { name: name.to_string(), prefix: "secret".to_string() }
    }

    pub fn from_identify(s: &str) -> Self {
        let mut s = s.trim();
        if s.len() >= 7 && s[..7].eq_ignore_ascii_case("secret@") {
            s = &s[7..];
        }
        SecretTag::new(s)
    }
}

impl ObjectTagImpl for SecretTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("secret@{}", self.name)
    }
    fn identify_simple(&self) -> String {
        "secret@".to_string()
    }
    fn object_type_name(&self) -> &'static str {
        "SecretTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn debuggable(&self) -> String {
        "secret@<hidden>".to_string()
    }
}

pub fn register_tags() {
    let p = get_processor("SecretTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<SecretTag>().map(|s| ObjectTag::new(crate::obj::ElementTag::new(&s.name)))
    }, true, &[]);

    p.register_tag("ElementTag", "value", |_a, o| {
        o.downcast_ref::<SecretTag>().map(|s| ObjectTag::new(crate::obj::ElementTag::new(&s.name)))
    }, true, &[]);
}
