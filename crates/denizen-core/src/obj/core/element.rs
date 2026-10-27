//! `ElementTag`: the fundamental text/number value object.

use crate::obj::{get_processor, ObjectTag, ObjectTagImpl};
use crate::tags::{Attribute, TagContext};
use crate::util;
use std::any::Any;

/// A simple text value, optionally carrying number/bool semantics.
#[derive(Clone, Debug)]
pub struct ElementTag {
    pub value: String,
    pub prefix: String,
    pub is_plain_text: bool,
    pub is_raw_input: bool,
}

impl ElementTag {
    pub fn new(value: impl Into<String>) -> Self {
        ElementTag { value: value.into(), prefix: "element".to_string(), is_plain_text: false, is_raw_input: false }
    }

    pub fn raw(value: impl Into<String>) -> Self {
        ElementTag { value: value.into(), prefix: "element".to_string(), is_plain_text: false, is_raw_input: true }
    }

    pub fn plain(value: impl Into<String>) -> Self {
        ElementTag { value: value.into(), prefix: "element".to_string(), is_plain_text: true, is_raw_input: false }
    }

    pub fn from_bool(b: bool) -> Self {
        ElementTag { value: b.to_string(), prefix: "boolean".to_string(), is_plain_text: true, is_raw_input: false }
    }

    pub fn from_int(i: i64) -> Self {
        ElementTag { value: i.to_string(), prefix: "number".to_string(), is_plain_text: true, is_raw_input: false }
    }

    pub fn from_double(d: f64) -> Self {
        ElementTag { value: util::double_to_string(d), prefix: "decimal".to_string(), is_plain_text: true, is_raw_input: false }
    }

    pub fn with_prefix(prefix: &str, value: impl Into<String>) -> Self {
        ElementTag { value: value.into(), prefix: prefix.to_string(), is_plain_text: false, is_raw_input: false }
    }

    pub fn as_string(&self) -> String {
        self.value.clone()
    }

    pub fn as_lower_string(&self) -> String {
        self.value.to_lowercase()
    }

    pub fn to_bool(&self) -> bool {
        self.value.eq_ignore_ascii_case("true")
    }

    pub fn to_int(&self) -> Option<i64> {
        self.value.trim().parse::<i64>().ok()
    }

    pub fn to_double(&self) -> Option<f64> {
        self.value.trim().parse::<f64>().ok()
    }

    pub fn is_integer(&self) -> bool {
        self.value.trim().parse::<i64>().is_ok()
    }

    pub fn is_decimal(&self) -> bool {
        self.value.trim().parse::<f64>().is_ok()
    }

    pub fn is_boolean(&self) -> bool {
        self.value.eq_ignore_ascii_case("true") || self.value.eq_ignore_ascii_case("false")
    }
}

impl ObjectTagImpl for ElementTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        self.value.clone()
    }
    fn identify_simple(&self) -> String {
        self.value.clone()
    }
    fn object_type_name(&self) -> &'static str {
        "ElementTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_truthy(&self) -> bool {
        let v = self.value.to_lowercase();
        !(v.is_empty() || v == "false" || v == "0" || v == "null")
    }
    fn as_element(&self) -> ElementTag {
        self.clone()
    }
}

/// Parse an element from an `el@`-prefixed or plain string.
pub fn element_value_of(s: &str) -> ElementTag {
    if s.len() >= 3 && s[..3].eq_ignore_ascii_case("el@") {
        ElementTag::new(&s[3..])
    } else {
        ElementTag::new(s)
    }
}

/// Register ElementTag's tags.
pub fn register_tags() {
    let p = get_processor("ElementTag");

    p.register_tag("ElementTag", "as_element", |_a, o| Some(o.clone()), true, &[]);

    p.register_tag("ElementTag", "is_boolean", |_a, o| {
        let e = o.as_element();
        Some(ObjectTag::new(ElementTag::from_bool(e.is_boolean())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_integer", |_a, o| {
        let e = o.as_element();
        Some(ObjectTag::new(ElementTag::from_bool(e.is_integer())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_decimal", |_a, o| {
        let e = o.as_element();
        Some(ObjectTag::new(ElementTag::from_bool(e.is_decimal())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_odd", |_a, o| {
        let n = o.as_element().to_int().unwrap_or(0);
        Some(ObjectTag::new(ElementTag::from_bool(n % 2 != 0)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_even", |_a, o| {
        let n = o.as_element().to_int().unwrap_or(0);
        Some(ObjectTag::new(ElementTag::from_bool(n % 2 == 0)))
    }, true, &[]);

    p.register_tag("ElementTag", "as_boolean", |_a, o| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().to_bool())))
    }, true, &[]);

    p.register_tag("ElementTag", "as_decimal", |_a, o| {
        let e = o.as_element();
        Some(ObjectTag::new(ElementTag::from_double(e.to_double().unwrap_or(0.0))))
    }, true, &[]);

    p.register_tag("ElementTag", "as_int", |_a, o| {
        let e = o.as_element();
        Some(ObjectTag::new(ElementTag::from_int(e.to_double().unwrap_or(0.0).trunc() as i64)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().value.eq_ignore_ascii_case(&param.as_element().value))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals_case_sensitive", |_a, o, param| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().value == param.as_element().value)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "contains_text", |_a, o, param| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().value.to_lowercase().contains(&param.as_element().value.to_lowercase()))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "contains_case_sensitive_text", |_a, o, param| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().value.contains(&param.as_element().value))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "starts_with", |_a, o, param| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().value.to_lowercase().starts_with(&param.as_element().value.to_lowercase()))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "ends_with", |_a, o, param| {
        Some(ObjectTag::new(ElementTag::from_bool(o.as_element().value.to_lowercase().ends_with(&param.as_element().value.to_lowercase()))))
    }, true, &[]);

    p.register_tag("ElementTag", "length", |_a, o| {
        Some(ObjectTag::new(ElementTag::from_int(o.as_element().value.chars().count() as i64)))
    }, true, &[]);

    p.register_tag("ElementTag", "to_uppercase", |_a, o| {
        Some(ObjectTag::new(ElementTag::new(o.as_element().value.to_uppercase())))
    }, true, &[]);

    p.register_tag("ElementTag", "to_lowercase", |_a, o| {
        Some(ObjectTag::new(ElementTag::new(o.as_element().value.to_lowercase())))
    }, true, &[]);

    p.register_tag("ElementTag", "trim", |_a, o| {
        Some(ObjectTag::new(ElementTag::new(o.as_element().value.trim())))
    }, true, &[]);

    p.register_tag("ElementTag", "not", |_a, o| {
        Some(ObjectTag::new(ElementTag::from_bool(!o.as_element().to_bool())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "is_more_than", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_bool(a > b)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "is_less_than", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_bool(a < b)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "is_more_than_or_equal_to", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_bool(a >= b)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "is_less_than_or_equal_to", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_bool(a <= b)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "split", |_a, o, param| {
        let sep = param.as_element().value;
        let parts: Vec<ObjectTag> = o.as_element().value.split(&sep).map(|s| ObjectTag::new(ElementTag::new(s))).collect();
        Some(ObjectTag::new(crate::obj::ListTag::from_objects(parts)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "replace", |_a, o, param| {
        let arg = param.as_element().value;
        let mut parts = arg.splitn(2, '|');
        let find = parts.next().unwrap_or("");
        let swap = parts.next().unwrap_or("");
        Some(ObjectTag::new(ElementTag::new(util::replace(&o.as_element().value, find, swap))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "substring", |_a, o, param| {
        let arg = param.as_element().value;
        let parts: Vec<&str> = arg.split('|').collect();
        let s = o.as_element().value;
        let chars: Vec<char> = s.chars().collect();
        let start = parts.first().and_then(|x| x.trim().parse::<usize>().ok()).unwrap_or(0).min(chars.len());
        let end = parts.get(1).and_then(|x| x.trim().parse::<usize>().ok()).unwrap_or(chars.len()).min(chars.len());
        let sub: String = chars[start..end.max(start)].iter().collect();
        Some(ObjectTag::new(ElementTag::new(sub)))
    }, true, &[]);

    p.register_tag("ElementTag", "as_list", |_a, o| {
        let e = o.as_element();
        let mut l = crate::obj::ListTag::new();
        l.add_object(ObjectTag::new(e));
        Some(ObjectTag::new(l))
    }, true, &[]);

    p.register_tag("ElementTag", "as_money", |_a, o| {
        Some(ObjectTag::new(ElementTag::new(util::format_money(o.as_element().to_double().unwrap_or(0.0)))))
    }, true, &[]);

    p.register_tag("ElementTag", "truncate", |_a, o| {
        Some(ObjectTag::new(ElementTag::new(o.as_element().value.trim_end_matches(".0").to_string())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "to_number", |_a, o, _p| {
        let e = o.as_element();
        Some(ObjectTag::new(ElementTag::new(e.to_double().map(util::double_to_string).unwrap_or_default())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "padded_to", |_a, o, param| {
        let param_str = param.as_element().value;
        let parts: Vec<&str> = param_str.splitn(2, '|').collect();
        let width: usize = parts.first().and_then(|x| x.trim().parse().ok()).unwrap_or(0);
        let pad = parts.get(1).map(|s| s.chars().next().unwrap_or(' ')).unwrap_or(' ');
        let s = o.as_element().value;
        let mut out = s.clone();
        while out.chars().count() < width {
            out.push(pad);
        }
        Some(ObjectTag::new(ElementTag::new(out)))
    }, true, &[]);
}

/// Convert any object into an ElementTag.
pub fn to_element(obj: &ObjectTag) -> ElementTag {
    if let Some(e) = obj.downcast_ref::<ElementTag>() {
        return e.clone();
    }
    ElementTag::new(obj.identify())
}

/// Helper used by command execution to build an element from text.
pub fn el(s: &str) -> ObjectTag {
    ObjectTag::new(ElementTag::new(s))
}

/// Helper for booleans.
pub fn el_bool(b: bool) -> ObjectTag {
    ObjectTag::new(ElementTag::from_bool(b))
}

/// Helper for integers.
pub fn el_int(i: i64) -> ObjectTag {
    ObjectTag::new(ElementTag::from_int(i))
}

/// Helper for decimals.
pub fn el_double(d: f64) -> ObjectTag {
    ObjectTag::new(ElementTag::from_double(d))
}

#[allow(dead_code)]
fn _unused(_c: &TagContext, _a: &mut Attribute) {}
