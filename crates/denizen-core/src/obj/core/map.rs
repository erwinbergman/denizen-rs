//! `MapTag`: a key/value mapping, identified as `map@[k=v;k2=v2]`.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use std::any::Any;
use std::collections::HashMap;

/// An insertion-ordered key/value map.
#[derive(Clone, Default, Debug)]
pub struct MapTag {
    pub entries: Vec<(String, ObjectTag)>,
    pub prefix: String,
}

impl MapTag {
    pub fn new() -> Self {
        MapTag { entries: Vec::new(), prefix: "Map".to_string() }
    }

    pub fn put_string(&mut self, key: &str, value: ObjectTag) -> &mut Self {
        if let Some(e) = self.entries.iter_mut().find(|(k, _)| k == key) {
            e.1 = value;
        } else {
            self.entries.push((key.to_string(), value));
        }
        self
    }

    pub fn put_element(&mut self, key: &str, value: &str) -> &mut Self {
        self.put_string(key, ObjectTag::new(crate::obj::ElementTag::new(value)))
    }

    pub fn get(&self, key: &str) -> Option<&ObjectTag> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Deep lookup using `.` separated keys.
    pub fn get_deep(&self, path: &str) -> Option<ObjectTag> {
        let mut parts = path.split('.');
        let first = parts.next()?;
        let mut cur = self.get(first)?.clone();
        for part in parts {
            let m = cur.to_map()?;
            cur = m.get(part)?.clone();
        }
        Some(cur)
    }

    pub fn put_deep(&mut self, path: &str, value: Option<ObjectTag>) {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() == 1 {
            match value {
                Some(v) => {
                    self.put_string(parts[0], v);
                }
                None => {
                    self.entries.retain(|(k, _)| k != parts[0]);
                }
            }
            return;
        }
        // Build nested maps along the path.
        let mut maps: Vec<MapTag> = Vec::new();
        let mut cursor = self.clone();
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                break;
            }
            let next = cursor.get(part).and_then(|v| v.to_map()).unwrap_or_default();
            maps.push(cursor.clone());
            cursor = next;
        }
        let mut nested = cursor;
        let last = parts[parts.len() - 1];
        match value {
            Some(v) => { nested.put_string(last, v); }
            None => { nested.entries.retain(|(k, _)| k != last); }
        }
        // Rebuild from innermost to outermost.
        for (i, mut parent) in maps.into_iter().enumerate().rev() {
            parent.put_string(parts[i], ObjectTag::new(nested.clone()));
            nested = parent;
        }
        *self = nested;
    }

    pub fn to_identify(&self) -> String {
        ObjectTagImpl::identify(self)
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.iter().map(|(k, _)| k.clone()).collect()
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 4 && s[..4].eq_ignore_ascii_case("map@") {
            s = &s[4..];
        }
        let has_brackets = s.starts_with('[') && s.ends_with(']');
        if has_brackets {
            s = &s[1..s.len() - 1];
        }
        let mut map = MapTag::new();
        if s.is_empty() {
            return Some(map);
        }
        // Legacy `k/v|` format.
        if s.ends_with('|') && !s.contains('=') {
            for entry in s.split('|') {
                if entry.is_empty() {
                    continue;
                }
                if let Some(slash) = entry.find('/') {
                    let k = unescape_legacy(&entry[..slash]);
                    let v = unescape_legacy(&entry[slash + 1..]);
                    map.put_string(&k, ObjectTag::new(crate::obj::ElementTag::new(v)));
                } else {
                    return None;
                }
            }
            return Some(map);
        }
        for entry in split_unescaped(s, ';') {
            if entry.is_empty() {
                continue;
            }
            let eq = find_unescaped(&entry, '=')?;
            let k = unescape_map_key(&entry[..eq]);
            let v = unescape_map_value(&entry[eq + 1..]);
            map.put_string(&k, ObjectTag::new(crate::obj::ElementTag::new(v)));
        }
        Some(map)
    }
}

fn unescape_legacy(v: &str) -> String {
    v.replace("&fs", "/").replace("&pipe", "|").replace("&amp", "&")
}

fn split_unescaped(s: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&n) = chars.peek() {
                cur.push('\\');
                cur.push(n);
                chars.next();
                continue;
            }
            cur.push(c);
        } else if c == sep {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

fn find_unescaped(s: &str, target: char) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == target {
            return Some(i);
        }
    }
    None
}

fn unescape_map_key(s: &str) -> String {
    s.replace("\\=", "=").replace("\\;", ";").replace("\\", "")
}

fn unescape_map_value(s: &str) -> String {
    s.replace("\\;", ";").replace("\\]", "]").replace("\\=", "=").replace("\\", "")
}

fn escape_map_key(s: &str) -> String {
    s.replace('=', "\\=").replace(';', "\\;")
}

fn escape_map_value(s: &str) -> String {
    s.replace(';', "\\;").replace(']', "\\]")
}

impl ObjectTagImpl for MapTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        if self.entries.is_empty() {
            return "map@[]".to_string();
        }
        let mut out = String::from("map@[");
        for (k, v) in &self.entries {
            out.push_str(&escape_map_key(k));
            out.push('=');
            out.push_str(&escape_map_value(&v.savable()));
            out.push(';');
        }
        out.pop();
        out.push(']');
        out
    }
    fn object_type_name(&self) -> &'static str {
        "MapTag"
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
        !self.entries.is_empty()
    }
    fn debuggable(&self) -> String {
        if self.entries.is_empty() {
            return "map@".to_string();
        }
        let parts: Vec<String> = self.entries.iter().map(|(k, v)| format!("{} <LG>=<Y> {}", k, v.debuggable())).collect();
        format!("<LG>map@[<Y>{}<LG>]", parts.join("<LG>;<Y> "))
    }
}

impl MapTag {
    /// Convert to a plain `HashMap<String,ObjectTag>` (keys exact).
    pub fn to_hash_map(&self) -> HashMap<String, ObjectTag> {
        self.entries.iter().cloned().collect()
    }
}

/// Register MapTag tags.
pub fn register_tags() {
    let p = get_processor("MapTag");

    p.register_tag_with_param("ObjectTag", "ElementTag", "get", |a, o, param| {
        let m = o.to_map()?;
        let key = param.as_element().value;
        match m.get_deep(&key) {
            Some(v) => Some(v),
            None => {
                a.echo_error(&format!("Invalid map key '{}'.", key));
                None
            }
        }
    }, true, &[]);

    p.register_tag("ListTag", "keys", |_a, o| {
        let m = o.to_map()?;
        Some(ObjectTag::new(crate::obj::ListTag::from_strings(&m.keys())))
    }, true, &[]);

    p.register_tag("ListTag", "values", |_a, o| {
        let m = o.to_map()?;
        Some(ObjectTag::new(crate::obj::ListTag::from_objects(m.entries.iter().map(|(_, v)| v.clone()).collect())))
    }, true, &[]);

    p.register_tag("ElementTag", "size", |_a, o| {
        let m = o.to_map()?;
        Some(ObjectTag::new(crate::obj::ElementTag::from_int(m.size() as i64)))
    }, true, &["length"]);

    p.register_tag("ElementTag", "is_empty", |_a, o| {
        let m = o.to_map()?;
        Some(ObjectTag::new(crate::obj::ElementTag::from_bool(m.is_empty())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "contains_key", |_a, o, param| {
        let m = o.to_map()?;
        Some(ObjectTag::new(crate::obj::ElementTag::from_bool(m.get(&param.as_element().value).is_some())))
    }, true, &[]);

    p.register_tag_with_param("MapTag", "ElementTag", "with_key", |_a, o, param| {
        let mut m = o.to_map()?;
        let arg = param.as_element().value;
        let mut parts = arg.splitn(2, '|');
        let k = parts.next().unwrap_or("");
        let v = parts.next().unwrap_or("");
        m.put_element(k, v);
        Some(ObjectTag::new(m))
    }, true, &[]);

    p.register_tag_with_param("MapTag", "ElementTag", "without_key", |_a, o, param| {
        let mut m = o.to_map()?;
        let key = param.as_element().value;
        m.entries.retain(|(k, _)| k != &key);
        Some(ObjectTag::new(m))
    }, true, &[]);
}
