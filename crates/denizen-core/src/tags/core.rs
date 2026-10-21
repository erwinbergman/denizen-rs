//! Core tag bases: `<[def]>`, `<element[...]>`, `<list[...]>`, `<map[...]>`, `<util....>`, etc.

use crate::obj::{ElementTag, ListTag, MapTag, ObjectTag};
use crate::tags::{
    register_tag_handler, register_tag_handler_with_param, Attribute, TagContext,
};
use std::sync::Arc;

/// `<[name]>` / `<def[name]>` / `<definition[name]>`: definition lookup.
pub fn definition_tag(attr: &mut Attribute) -> Option<ObjectTag> {
    let provider = attr.context.definition_provider.clone()?;
    let name = attr.get_param_object()?.as_element().as_lower_string();
    let def = provider.get_definition_object(&name)?;
    if attr.attributes.len() == 1 {
        return Some(def.refresh_state());
    }
    Some(def)
}

/// Register all core base tags.
pub fn register_core_tags() {
    register_tag_handler("ObjectTag", "", Arc::new(definition_tag), false);
    register_tag_handler("ObjectTag", "def", Arc::new(definition_tag), false);
    register_tag_handler("ObjectTag", "definition", Arc::new(definition_tag), false);

    register_tag_handler_with_param(
        "ElementTag",
        "ElementTag",
        "element",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| Some(ObjectTag::new(param.as_element()))),
        true,
    );

    register_tag_handler_with_param(
        "ListTag",
        "ElementTag",
        "list",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            let sep = '|';
            let items: Vec<ObjectTag> = param
                .as_element()
                .value
                .split(sep)
                .filter(|s| !s.is_empty())
                .map(|s| ObjectTag::new(ElementTag::new(s)))
                .collect();
            Some(ObjectTag::new(ListTag::from_objects(items)))
        }),
        true,
    );

    register_tag_handler_with_param(
        "MapTag",
        "ElementTag",
        "map",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| MapTag::from_identify(&param.as_element().value).map(|m| ObjectTag::new(m))),
        true,
    );

    register_tag_handler_with_param(
        "ElementTag",
        "ElementTag",
        "ternary",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            let arg = param.as_element().value;
            let parts: Vec<&str> = arg.split('|').collect();
            let cond = parts.first().map(|s| s.trim().eq_ignore_ascii_case("true")).unwrap_or(false);
            let pick = if cond { parts.get(1) } else { parts.get(2) };
            Some(ObjectTag::new(ElementTag::new(pick.copied().unwrap_or(""))))
        }),
        true,
    );

    register_tag_handler("ElementTag", "true", Arc::new(|_a| Some(ObjectTag::new(ElementTag::from_bool(true)))), true);
    register_tag_handler("ElementTag", "false", Arc::new(|_a| Some(ObjectTag::new(ElementTag::from_bool(false)))), true);
    register_tag_handler("ElementTag", "null", Arc::new(|_a| Some(ObjectTag::new(ElementTag::new("null")))), true);

    // <util.random_uuid>
    register_tag_handler("ElementTag", "util", Arc::new(|a: &mut Attribute| {
        let sub = a.get_attribute_without_param(1);
        match sub.as_str() {
            "random_uuid" => {
                a.fulfill(1);
                Some(ObjectTag::new(ElementTag::new(uuid::Uuid::new_v4().to_string())))
            }
            "system_time" => {
                a.fulfill(1);
                Some(ObjectTag::new(ElementTag::from_int(crate::util::current_time_millis())))
            }
            _ => None,
        }
    }), false);
}

/// Helper to register a base tag returning an ElementTag string.
pub fn register_element_tag(name: &'static str, f: impl Fn(&mut Attribute) -> Option<String> + Send + Sync + 'static) {
    register_tag_handler(
        "ElementTag",
        name,
        Arc::new(move |a: &mut Attribute| f(a).map(|s| ObjectTag::new(ElementTag::new(s)))),
        true,
    );
}

/// Helper to register a base tag returning an ObjectTag.
pub fn register_object_tag(name: &'static str, f: impl Fn(&mut Attribute) -> Option<ObjectTag> + Send + Sync + 'static) {
    register_tag_handler("ObjectTag", name, Arc::new(f), false);
}

/// Re-export for convenience.
pub use crate::tags::{tag, tag_object};
#[allow(unused_imports)]

#[allow(dead_code)]
fn _unused(_c: TagContext) {}
