//! Script entries and the data they carry.

use crate::obj::{ElementTag, ObjectTag};
use crate::scripts::container::ScriptContainer;
use crate::tags::{Attribute, ParseableTag, TagContext};
use std::sync::Arc;

/// Per-entry data (player/npc/extra context).
pub type ScriptEntryData = crate::scripts::container::EntryData;

/// A single script line: a command name plus parsed arguments.
#[derive(Clone)]
pub struct ScriptEntry {
    /// The command name (lowercased).
    pub command: String,
    /// Raw argument strings (already split).
    pub args: Vec<String>,
    /// The original unparsed line.
    pub original_line: String,
    /// The script container this entry belongs to.
    pub script: Option<Arc<ScriptContainer>>,
    /// Line number within the script file.
    pub line_number: usize,
    /// Data attached to this entry (player/npc).
    pub entry_data: ScriptEntryData,
    /// The queue id this entry is executing within, if any.
    pub queue_id: Option<String>,
    /// Whether debug output is enabled for this entry.
    pub should_debug: bool,
    /// Parsed arguments cache.
    pub parsed_args: Option<Vec<Argument>>,
    /// Braced sub-command body (the indented block following `- command:`).
    pub braced: Option<Vec<ScriptEntry>>,
    /// Owner entry (for loop callbacks): the entry that spawned this one.
    pub owner_line: usize,
}

/// A parsed command argument with optional prefix.
#[derive(Clone, Debug)]
pub struct Argument {
    pub prefix: Option<String>,
    pub lower_prefix: Option<String>,
    pub raw_value: String,
    pub value: String,
    pub lower_value: String,
    pub object: Option<ObjectTag>,
    pub prefix_was_dynamic: bool,
}

impl Argument {
    pub fn new(prefix: Option<&str>, value: &str) -> Self {
        let raw = match prefix {
            Some(p) if p != "no_prefix" => format!("{}:{}", p, value),
            _ => value.to_string(),
        };
        let actual_prefix = match prefix {
            Some("no_prefix") => None,
            other => other.map(|s| s.to_string()),
        };
        Argument {
            lower_prefix: actual_prefix.as_ref().map(|p| p.to_lowercase()),
            prefix: actual_prefix,
            raw_value: raw,
            value: value.to_string(),
            lower_value: value.to_lowercase(),
            object: Some(ObjectTag::new(ElementTag::raw(value))),
            prefix_was_dynamic: false,
        }
    }

    /// Parse a raw `prefix:value` string.
    pub fn from_raw(raw: &str) -> Self {
        if let Some(colon) = find_prefix_colon(raw) {
            let prefix = &raw[..colon];
            let value = &raw[colon + 1..];
            Argument::new(Some(prefix), value)
        } else {
            Argument::new(None, raw)
        }
    }

    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn get_raw_value(&self) -> &str {
        &self.raw_value
    }

    pub fn as_element(&self) -> ElementTag {
        match &self.object {
            Some(o) => o.as_element(),
            None => ElementTag::new(&self.value),
        }
    }

    pub fn as_string(&self) -> String {
        self.value.clone()
    }

    pub fn as_int(&self) -> i32 {
        self.value.trim().parse().unwrap_or(0)
    }

    pub fn as_long(&self) -> i64 {
        self.value.trim().parse().unwrap_or(0)
    }

    pub fn as_double(&self) -> f64 {
        self.value.trim().parse().unwrap_or(0.0)
    }

    pub fn as_bool(&self) -> bool {
        self.value.eq_ignore_ascii_case("true")
    }

    pub fn as_list(&self) -> crate::obj::ListTag {
        crate::obj::ListTag::from_identify(&self.value)
    }

    pub fn matches(&self, prefix: &str) -> bool {
        self.lower_prefix.as_deref() == Some(prefix)
    }

    /// Convert to an ObjectTag, parsing tags if needed.
    pub fn as_object(&self, context: &TagContext) -> ObjectTag {
        if let Some(o) = &self.object {
            return o.clone();
        }
        crate::tags::tag_object(&self.value, context)
    }
}

fn find_prefix_colon(s: &str) -> Option<usize> {
    let colon = s.find(':')?;
    let prefix = &s[..colon];
    if prefix.is_empty() {
        return None;
    }
    if prefix.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
        Some(colon)
    } else {
        None
    }
}

impl ScriptEntry {
    pub fn new(command: &str, args: Vec<String>, script: Option<Arc<ScriptContainer>>, line_number: usize) -> Self {
        ScriptEntry {
            command: command.to_lowercase(),
            args,
            original_line: String::new(),
            script,
            line_number,
            entry_data: ScriptEntryData::new(),
            queue_id: None,
            should_debug: false,
            parsed_args: None,
            braced: None,
            owner_line: line_number,
        }
    }

    /// Whether this entry has a braced sub-command body.
    pub fn has_braced(&self) -> bool {
        self.braced.is_some()
    }

    /// A clone of the braced body, if any.
    pub fn braced_body(&self) -> Option<Vec<ScriptEntry>> {
        self.braced.clone()
    }

    /// Arguments that appear before any legacy `{` brace marker.
    pub fn original_args(&self) -> Vec<String> {
        let mut out = Vec::new();
        for a in &self.args {
            if a == "{" {
                break;
            }
            out.push(a.clone());
        }
        out
    }

    /// The value of a `prefix:value` argument, if present.
    pub fn prefixed(&self, prefix: &str) -> Option<&str> {
        let p = format!("{}:", prefix.to_lowercase());
        for a in &self.args {
            let lower = a.to_lowercase();
            if lower.starts_with(&p) {
                return Some(&a[p.len()..]);
            }
        }
        None
    }

    /// Whether any argument has the given `prefix:`.
    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.prefixed(prefix).is_some()
    }

    /// Build the tag context for this entry, given a queue's definitions.
    pub fn tag_context(&self, provider: Option<Arc<dyn crate::tags::DefinitionProvider>>, context_source: Option<Arc<dyn crate::tags::ContextSource>>) -> TagContext {
        TagContext {
            debug: self.should_debug,
            show_errors: true,
            definition_provider: provider,
            context_source,
            script_name: self.script.as_ref().map(|s| s.name.clone()),
            queue_id: self.queue_id.clone(),
            entry_line: Some(self.original_line.clone()),
        }
    }

    /// Parse the arguments, caching the result.
    pub fn get_parsed_args(&mut self) -> &Vec<Argument> {
        if self.parsed_args.is_none() {
            self.parsed_args = Some(self.args.iter().map(|a| Argument::from_raw(a)).collect());
        }
        self.parsed_args.as_ref().unwrap()
    }

    /// Parse text into a ParseableTag, for later evaluation.
    pub fn parse_text(&self, text: &str, context: &TagContext) -> ParseableTag {
        crate::tags::parse_text_to_tag(text, context)
    }

    /// Evaluate a tag chain against the entry's context.
    pub fn tag_object(&self, text: &str, context: &TagContext) -> ObjectTag {
        crate::tags::tag_object(text, context)
    }

    /// Build an attribute for a tag string.
    pub fn attribute(&self, tag: &str, context: &TagContext) -> Option<Attribute> {
        Attribute::new(tag, context.clone()).ok()
    }
}

impl std::fmt::Debug for ScriptEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScriptEntry({} {:?})", self.command, self.args)
    }
}
