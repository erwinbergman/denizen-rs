//! Script containers: the top-level units defined in `.dsc`/`.yml` script files.

use crate::obj::ObjectTag;
use crate::scripts::entry::ScriptEntry;
use crate::tags::{DefinitionProvider, SimpleDefinitionProvider};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The kind of a script container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptContainerKind {
    Task,
    World,
    Procedure,
    Format,
    Data,
    Custom,
    Command,
    Economy,
    Item,
    Inventory,
    Book,
    Map,
    Unknown(String),
}

impl ScriptContainerKind {
    pub fn from_type_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "task" => ScriptContainerKind::Task,
            "world" => ScriptContainerKind::World,
            "procedure" => ScriptContainerKind::Procedure,
            "format" => ScriptContainerKind::Format,
            "data" => ScriptContainerKind::Data,
            "custom" => ScriptContainerKind::Custom,
            "command" => ScriptContainerKind::Command,
            "economy" => ScriptContainerKind::Economy,
            "item" => ScriptContainerKind::Item,
            "inventory" => ScriptContainerKind::Inventory,
            "book" => ScriptContainerKind::Book,
            "map" => ScriptContainerKind::Map,
            other => ScriptContainerKind::Unknown(other.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ScriptContainerKind::Task => "task",
            ScriptContainerKind::World => "world",
            ScriptContainerKind::Procedure => "procedure",
            ScriptContainerKind::Format => "format",
            ScriptContainerKind::Data => "data",
            ScriptContainerKind::Custom => "custom",
            ScriptContainerKind::Command => "command",
            ScriptContainerKind::Economy => "economy",
            ScriptContainerKind::Item => "item",
            ScriptContainerKind::Inventory => "inventory",
            ScriptContainerKind::Book => "book",
            ScriptContainerKind::Map => "map",
            ScriptContainerKind::Unknown(s) => s,
        }
    }
}

/// A parsed script container.
pub struct ScriptContainer {
    pub name: String,
    pub original_name: String,
    pub kind: ScriptContainerKind,
    /// The raw container body (the parsed YAML map for this script).
    pub contents: crate::util::yaml::Yaml,
    /// Cached script entries for event/queue sections.
    entries: RwLock<HashMap<String, Arc<Vec<ScriptEntry>>>>,
    /// Definition provider local to this container.
    pub definitions: Arc<SimpleDefinitionProvider>,
}

impl ScriptContainer {
    pub fn new(name: &str, kind: ScriptContainerKind, contents: crate::util::yaml::Yaml) -> Self {
        ScriptContainer {
            name: name.to_string(),
            original_name: name.to_string(),
            kind,
            contents,
            entries: RwLock::new(HashMap::new()),
            definitions: Arc::new(SimpleDefinitionProvider::new()),
        }
    }

    pub fn container_type(&self) -> &str {
        self.kind.name()
    }

    /// Read a top-level string value from the container body.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.contents.get_string(key)
    }

    /// Read a top-level list value as string entries.
    pub fn get_string_list(&self, key: &str) -> Vec<String> {
        self.contents.get_string_list(key)
    }

    /// Read a sub-section as a YAML value.
    pub fn get_section(&self, key: &str) -> Option<&crate::util::yaml::Yaml> {
        self.contents.get_section(key)
    }

    /// Build (and cache) the script entries for a named section (eg "script", "events").
    pub fn get_entries(&self, section: &str) -> Option<Arc<Vec<ScriptEntry>>> {
        if let Some(cached) = self.entries.read().unwrap().get(section) {
            return Some(cached.clone());
        }
        let section_val = self.contents.get(section)?;
        let built = crate::scripts::builder::build_script_entries(section_val, self);
        let arc = Arc::new(built);
        self.entries.write().unwrap().insert(section.to_string(), arc.clone());
        Some(arc)
    }

    /// For event containers, return the list of event names.
    pub fn event_names(&self) -> Vec<String> {
        self.contents.get_string_list("events")
    }

    /// Whether this container is a procedure script.
    pub fn is_procedure(&self) -> bool {
        self.kind == ScriptContainerKind::Procedure
    }

    /// The procedure's `determine` default value, if any.
    pub fn determine_default(&self) -> Option<String> {
        self.contents.get_string("determine")
    }

    /// Run this container's procedure body, returning the determined value.
    ///
    /// A procedure's `script:` section is executed with `procedural` set, and the value
    /// returned by the first `determine` command is the result.
    pub fn run_procedure(&self, definitions: Option<crate::obj::MapTag>) -> Option<ObjectTag> {
        let entries = self.get_entries("script")?;
        let mut queue = crate::scripts::queue::ScriptQueue::new("");
        queue.procedural = true;
        if let Some(d) = definitions {
            queue.definitions = d;
        }
        for e in entries.iter() {
            queue.add_entry(e.clone());
        }
        let arc = crate::scripts::queue::register_queue(queue);
        crate::scripts::queue::run_queue(&arc);
        let determined = {
            let q = arc.read().unwrap();
            q.determined.clone()
        };
        determined.or_else(|| self.determine_default().map(|s| ObjectTag::new(crate::obj::ElementTag::new(s))))
    }

    /// Build the formatting context for a `format` script (returns the format string body).
    pub fn format_body(&self) -> Option<String> {
        self.contents
            .get_string("format")
            .or_else(|| self.contents.get_string("text"))
    }
}

/// Data carried between script entries (player/npc references, etc.).
#[derive(Clone, Default)]
pub struct EntryData {
    pub player: Option<ObjectTag>,
    pub npc: Option<ObjectTag>,
    pub extra: HashMap<String, ObjectTag>,
}

impl EntryData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transfer_from(&mut self, other: &EntryData) {
        if other.player.is_some() {
            self.player = other.player.clone();
        }
        if other.npc.is_some() {
            self.npc = other.npc.clone();
        }
        for (k, v) in &other.extra {
            self.extra.insert(k.clone(), v.clone());
        }
    }
}

impl DefinitionProvider for ScriptContainer {
    fn get_definition_object(&self, definition: &str) -> Option<ObjectTag> {
        self.definitions.get_definition_object(definition)
    }
    fn add_definition(&self, definition: &str, value: Option<ObjectTag>) {
        self.definitions.add_definition(definition, value)
    }
    fn get_all_definitions(&self) -> Option<crate::obj::MapTag> {
        self.definitions.get_all_definitions()
    }
}

#[allow(dead_code)]
fn _unused(_c: &TagContext) {}

use crate::tags::TagContext;

#[cfg(test)]
mod tests {
    #[test]
    fn procedure_returns_determine() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        let parsed = crate::util::yaml::parse(
            "my_proc:\n  type: procedure\n  script:\n  - if <[a]> == 1:\n    - determine yes\n  - determine no\n",
        );
        let mut container = None;
        if let crate::util::yaml::Yaml::Map(entries) = parsed {
            for (name, body) in entries {
                let c = crate::scripts::container::ScriptContainer::new(&name, crate::scripts::container::ScriptContainerKind::from_type_name("procedure"), body);
                container = Some(c);
            }
        }
        let c = container.unwrap();
        let mut defs = crate::obj::MapTag::new();
        defs.put_string("a", crate::obj::ObjectTag::new(crate::obj::ElementTag::from_int(1)));
        let result = c.run_procedure(Some(defs));
        assert_eq!(result.map(|o| o.identify()), Some("yes".to_string()));
    }
}
