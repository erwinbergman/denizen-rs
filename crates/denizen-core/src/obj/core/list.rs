//! `ListTag`: an ordered list of ObjectTags, identified as `li@a|b|c`.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;

/// An ordered list of objects.
#[derive(Clone, Default)]
pub struct ListTag {
    pub items: Vec<ObjectTag>,
    pub prefix: String,
}

impl ListTag {
    pub fn new() -> Self {
        ListTag { items: Vec::new(), prefix: "List".to_string() }
    }

    pub fn from_objects(items: Vec<ObjectTag>) -> Self {
        ListTag { items, prefix: "List".to_string() }
    }

    pub fn from_strings<S: AsRef<str>>(items: &[S]) -> Self {
        ListTag {
            items: items.iter().map(|s| ObjectTag::new(crate::obj::ElementTag::new(s.as_ref()))).collect(),
            prefix: "List".to_string(),
        }
    }

    pub fn add_string(&mut self, s: &str) -> &mut Self {
        self.items.push(ObjectTag::new(crate::obj::ElementTag::new(s)));
        self
    }

    pub fn add_object(&mut self, o: ObjectTag) -> &mut Self {
        self.items.push(o);
        self
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&ObjectTag> {
        self.items.get(index)
    }

    pub fn strings(&self) -> Vec<String> {
        self.items.iter().map(|o| o.as_element().value).collect()
    }

    /// Parse `li@a|b|c` into a list.
    pub fn from_identify(s: &str) -> Self {
        let mut s = s;
        if s.len() >= 3 && s[..3].eq_ignore_ascii_case("li@") {
            s = &s[3..];
        }
        if s.is_empty() {
            return ListTag::new();
        }
        let mut items = Vec::new();
        let mut current = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    if next == '|' || next == '\\' || next == '&' {
                        current.push(next);
                        chars.next();
                        continue;
                    }
                }
                current.push(c);
            } else if c == '|' {
                items.push(parse_list_entry(&current));
                current.clear();
            } else {
                current.push(c);
            }
        }
        items.push(parse_list_entry(&current));
        ListTag { items, prefix: "List".to_string() }
    }

    pub fn escape_entry(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            if c == '|' || c == '\\' {
                out.push('\\');
            }
            out.push(c);
        }
        out
    }
}

fn parse_list_entry(s: &str) -> ObjectTag {
    let unescaped = s.replace("&pipe", "|").replace("&amp", "&");
    ObjectTag::new(crate::obj::ElementTag::new(unescaped))
}

impl ObjectTagImpl for ListTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        if self.items.is_empty() {
            return "li@".to_string();
        }
        let mut out = String::from("li@");
        for o in &self.items {
            out.push_str(&ListTag::escape_entry(&o.savable()));
            out.push('|');
        }
        out
    }
    fn object_type_name(&self) -> &'static str {
        "ListTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_truthy(&self) -> bool {
        !self.items.is_empty()
    }
    fn debuggable(&self) -> String {
        if self.items.is_empty() {
            return "<LG>li@ (Size <GR>0<LG>)".to_string();
        }
        let parts: Vec<String> = self.items.iter().map(|i| i.debuggable()).collect();
        format!("<LG>li@ (Size <GR>{}<LG>): <Y>{}", self.items.len(), parts.join(" <LG>|<Y> "))
    }
}

/// Register ListTag tags.
pub fn register_tags() {
    let p = get_processor("ListTag");

    p.register_tag("ElementTag", "size", |_a, o| {
        let l = o.to_list();
        Some(ObjectTag::new(crate::obj::ElementTag::from_int(l.size() as i64)))
    }, true, &["length"]);

    p.register_tag("ElementTag", "is_empty", |_a, o| {
        let l = o.to_list();
        Some(ObjectTag::new(crate::obj::ElementTag::from_bool(l.is_empty())))
    }, true, &[]);

    p.register_tag_with_param("ObjectTag", "ElementTag", "get", |a, o, param| {
        let l = o.to_list();
        let arg = param.as_element().value;
        let idx = arg.trim().parse::<i64>().unwrap_or_else(|_| {
            a.echo_error(&format!("Invalid integer input '{}' to list.get.", arg));
            0
        });
        let size = l.size() as i64;
        let real = if idx < 0 { size + idx } else { idx };
        if real < 0 || real >= size {
            a.echo_error(&format!("List index out of bounds: {}", idx));
            return None;
        }
        Some(l.items[real as usize].clone())
    }, true, &["at"]);

    p.register_tag_with_param("ListTag", "ElementTag", "get_sub_list", |a, o, param| {
        let l = o.to_list();
        let arg = param.as_element().value;
        let parts: Vec<&str> = arg.split('|').collect();
        let size = l.size() as i64;
        let start = parts.first().and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0);
        let end = parts.get(1).and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(size - 1);
        let mut s = if start < 0 { size + start } else { start };
        let mut e = if end < 0 { size + end } else { end };
        if s < 0 || e >= size || s > e {
            a.echo_error(&format!("Invalid sub-list range: {}|{}", start, end));
            return None;
        }
        s = s.max(0);
        e = e.min(size - 1);
        let items: Vec<ObjectTag> = l.items[s as usize..=e as usize].to_vec();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &[]);

    p.register_tag("ElementTag", "comma_separated", |_a, o| {
        Some(ObjectTag::new(crate::obj::ElementTag::new(o.to_list().strings().join(", "))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "join", |_a, o, param| {
        let sep = param.as_element().value;
        Some(ObjectTag::new(crate::obj::ElementTag::new(o.to_list().strings().join(&sep))))
    }, true, &["separated_by"]);

    p.register_tag_with_param("ElementTag", "ElementTag", "contains", |_a, o, param| {
        let needle = param.as_element().value.to_lowercase();
        let found = o.to_list().items.iter().any(|i| i.as_element().value.to_lowercase() == needle);
        Some(ObjectTag::new(crate::obj::ElementTag::from_bool(found)))
    }, true, &[]);

    p.register_tag_with_param("ListTag", "ElementTag", "exclude", |_a, o, param| {
        let needle = param.as_element().value.to_lowercase();
        let items: Vec<ObjectTag> = o.to_list().items.into_iter()
            .filter(|i| i.as_element().value.to_lowercase() != needle).collect();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &["remove"]);

    p.register_tag_with_param("ListTag", "ObjectTag", "include", |_a, o, param| {
        let mut l = o.to_list();
        l.add_object(param);
        Some(ObjectTag::new(l))
    }, true, &["add"]);

    p.register_tag("ListTag", "reverse", |_a, o| {
        let mut items = o.to_list().items;
        items.reverse();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &[]);

    p.register_tag("ListTag", "deduplicate", |_a, o| {
        let mut seen = std::collections::HashSet::new();
        let items: Vec<ObjectTag> = o.to_list().items.into_iter()
            .filter(|i| seen.insert(i.as_element().value.to_lowercase())).collect();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &["unique"]);

    p.register_tag("ElementTag", "first", |_a, o| {
        o.to_list().items.first().cloned()
    }, true, &[]);

    p.register_tag("ElementTag", "last", |_a, o| {
        o.to_list().items.last().cloned()
    }, true, &[]);

    p.register_tag("ListTag", "as_list", |_a, o| Some(o.clone()), true, &[]);

    p.register_tag_with_param("ListTag", "ElementTag", "sort", |_a, o, param| {
        let mut items = o.to_list().items;
        let desc = param.as_element().value.eq_ignore_ascii_case("desc") || param.as_element().value.eq_ignore_ascii_case("descending");
        items.sort_by(|a, b| {
            let sa = a.as_element().value;
            let sb = b.as_element().value;
            if let (Ok(na), Ok(nb)) = (sa.parse::<f64>(), sb.parse::<f64>()) {
                na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                sa.to_lowercase().cmp(&sb.to_lowercase())
            }
        });
        if desc {
            items.reverse();
        }
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &[]);
}
