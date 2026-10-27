//! `PluginTag`: a server plugin, identified as `pl@<name>`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Static metadata for a plugin.
#[derive(Clone, Debug, Default)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub depends: Vec<String>,
    pub soft_depends: Vec<String>,
    pub commands: Vec<String>,
    pub enabled: bool,
}

static PLUGINS: LazyLock<RwLock<HashMap<String, PluginInfo>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a plugin's metadata.
pub fn register_plugin(info: PluginInfo) {
    PLUGINS.write().unwrap().insert(info.name.to_lowercase(), info);
}

/// Look up plugin metadata by name.
pub fn get_plugin_info(name: &str) -> Option<PluginInfo> {
    PLUGINS.read().unwrap().get(&name.to_lowercase()).cloned()
}

/// All registered plugin names.
pub fn all_plugins() -> Vec<String> {
    PLUGINS.read().unwrap().values().map(|p| p.name.clone()).collect()
}

/// A plugin object.
#[derive(Clone, Debug)]
pub struct PluginTag {
    pub name: String,
    pub prefix: String,
}

impl PluginTag {
    pub fn new(name: &str) -> Self {
        let mut n = name.trim();
        if n.len() >= 3 && n[..3].eq_ignore_ascii_case("pl@") {
            n = &n[3..];
        }
        PluginTag { name: n.to_string(), prefix: "Plugin".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let n = s.trim();
        if n.is_empty() {
            return None;
        }
        Some(PluginTag::new(n))
    }

    pub fn info(&self) -> Option<PluginInfo> {
        get_plugin_info(&self.name)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for PluginTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("pl@{}", self.name)
    }
    fn identify_simple(&self) -> String {
        self.identify()
    }
    fn object_type_name(&self) -> &'static str {
        "PluginTag"
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

/// Register PluginTag's tags.
pub fn register_tags() {
    let p = get_processor("PluginTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<PluginTag>().map(|pl| ObjectTag::new(ElementTag::new(pl.name.clone())))
    }, true, &[]);

    p.register_tag("ElementTag", "version", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ElementTag::new(pl.info().map(|i| i.version).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ElementTag", "description", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ElementTag::new(pl.info().map(|i| i.description).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ListTag", "authors", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&pl.info().map(|i| i.authors).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ListTag", "depends", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&pl.info().map(|i| i.depends).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ListTag", "soft_depends", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&pl.info().map(|i| i.soft_depends).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ListTag", "commands", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&pl.info().map(|i| i.commands).unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_enabled", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.info().map(|i| i.enabled).unwrap_or(false))))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let pl = o.downcast_ref::<PluginTag>()?;
        let other = PluginTag::new(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(pl.name.eq_ignore_ascii_case(&other.name))))
    }, true, &[]);


    p.register_tag("ElementTag", "is_builtin", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(pl.info().is_none())))
    }, true, &[]);

    p.register_tag("ListTag", "all_plugins", |_a, _o| {
        Some(ObjectTag::new(ListTag::from_strings(&all_plugins())))
    }, true, &[]);

    p.register_tag("ElementTag", "author", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        let authors = pl.info().map(|i| i.authors).unwrap_or_default();
        Some(ObjectTag::new(ElementTag::new(authors.first().cloned().unwrap_or_default())))
    }, true, &[]);

    p.register_tag("ElementTag", "main_class", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ElementTag::new(format!("{}Plugin", pl.name))))
    }, true, &[]);

    p.register_tag("ElementTag", "debuggable", |_a, o| {
        let pl = o.downcast_ref::<PluginTag>()?;
        Some(ObjectTag::new(ElementTag::new(pl.identify())))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "has_dependency", |_a, o, param| {
        let pl = o.downcast_ref::<PluginTag>()?;
        let dep = param.as_element().value.to_lowercase();
        let has = pl.info().map(|i| i.depends.iter().any(|d| d.to_lowercase() == dep) || i.soft_depends.iter().any(|d| d.to_lowercase() == dep)).unwrap_or(false);
        Some(ObjectTag::new(ElementTag::from_bool(has)))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("PluginTag", |o| o.downcast_ref::<PluginTag>().map(|x| x.identify()));
}
