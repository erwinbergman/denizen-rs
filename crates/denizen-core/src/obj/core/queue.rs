//! `QueueTag`: a reference to a running script queue.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

#[derive(Clone, Debug, PartialEq)]
pub struct QueueTag {
    pub id: String,
    pub prefix: String,
}

impl QueueTag {
    pub fn new(id: &str) -> Self {
        QueueTag { id: id.to_string(), prefix: "queue".to_string() }
    }

    pub fn from_identify(s: &str) -> Self {
        let mut s = s.trim();
        if s.len() >= 6 && s[..6].eq_ignore_ascii_case("queue@") {
            s = &s[6..];
        }
        QueueTag::new(s)
    }
}

impl ObjectTagImpl for QueueTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("queue@{}", self.id)
    }
    fn identify_simple(&self) -> String {
        self.id.clone()
    }
    fn object_type_name(&self) -> &'static str {
        "QueueTag"
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
    let p = get_processor("QueueTag");

    p.register_tag("ElementTag", "id", |_a, o| {
        o.downcast_ref::<QueueTag>().map(|q| ObjectTag::new(crate::obj::ElementTag::new(&q.id)))
    }, true, &[]);

    p.register_tag("ElementTag", "exists", |_a, o| {
        let exists = o.downcast_ref::<QueueTag>().map(|q| crate::scripts::queue::queue_exists(&q.id)).unwrap_or(false);
        Some(ObjectTag::new(crate::obj::ElementTag::from_bool(exists)))
    }, true, &[]);
}
