//! Shared helpers for command implementations: argument access, prefix parsing,
//! and common object-intent recording via the host implementation.

use crate::obj::{ElementTag, ListTag, ObjectTag};
use crate::scripts::entry::ScriptEntry;
use crate::scripts::queue::ScriptQueue;
use crate::tags::{tag, tag_object, TagContext};
use std::sync::{Arc, RwLock};

/// Get the raw (untagged) argument at an index.
pub fn arg(entry: &ScriptEntry, idx: usize) -> String {
    entry.args.get(idx).cloned().unwrap_or_default()
}

/// Get the tagged value of the argument at an index.
pub fn arg_tag(entry: &ScriptEntry, idx: usize, ctx: &TagContext) -> String {
    tag(&arg(entry, idx), ctx)
}

/// Join all arguments from `start` onward, tagging each.
pub fn tag_rest(entry: &ScriptEntry, start: usize, ctx: &TagContext) -> String {
    entry.args[start.min(entry.args.len())..]
        .iter()
        .map(|a| tag(a, ctx))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Join all arguments from `start` onward without tagging.
pub fn raw_rest(entry: &ScriptEntry, start: usize) -> String {
    entry.args[start.min(entry.args.len())..].join(" ")
}

/// Return the value of a `prefix:value` argument (tagged).
pub fn prefixed(entry: &ScriptEntry, prefix: &str, ctx: &TagContext) -> Option<String> {
    entry.prefixed(prefix).map(|v| tag(v, ctx))
}

/// Return the raw value of a `prefix:value` argument (untagged).
pub fn prefixed_raw(entry: &ScriptEntry, prefix: &str) -> Option<String> {
    entry.prefixed(prefix).map(|v| v.to_string())
}

/// Whether an argument equals any of the given values (case-insensitive).
pub fn any_match(entry: &ScriptEntry, values: &[&str]) -> bool {
    entry.args.iter().any(|a| {
        let l = a.to_lowercase();
        values.iter().any(|v| l == *v)
    })
}

/// Arguments excluding those with the given prefixes and exact flags.
pub fn positional(entry: &ScriptEntry, skip_prefixes: &[&str], skip_flags: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for a in &entry.args {
        let lower = a.to_lowercase();
        if skip_flags.iter().any(|f| lower == *f) {
            continue;
        }
        if skip_prefixes.iter().any(|p| lower.starts_with(&format!("{}:", p))) {
            continue;
        }
        if lower == "{" {
            break;
        }
        out.push(a.clone());
    }
    out
}

/// Build the full textual payload of a command (tagged), skipping prefixes/flags.
pub fn payload(entry: &ScriptEntry, ctx: &TagContext, skip_prefixes: &[&str], skip_flags: &[&str]) -> String {
    positional(entry, skip_prefixes, skip_flags)
        .iter()
        .map(|a| tag(a, ctx))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Resolve the "target" object of a command: the linked player if no explicit target is given.
pub fn target_object(_entry: &ScriptEntry, ctx: &TagContext, text: &str) -> ObjectTag {
    if text.trim().is_empty() {
        if let Some(p) = linked_player(ctx) {
            return p;
        }
    }
    tag_object(text, ctx)
}

/// The queue's linked player, if any.
pub fn linked_player(ctx: &TagContext) -> Option<ObjectTag> {
    if let Some(provider) = &ctx.definition_provider {
        if let Some(p) = provider.get_definition_object("__player") {
            return Some(p);
        }
    }
    crate::engine::special_def_get("__player", None)
}

/// The queue's linked NPC, if any.
pub fn linked_npc(ctx: &TagContext) -> Option<ObjectTag> {
    if let Some(provider) = &ctx.definition_provider {
        if let Some(n) = provider.get_definition_object("__npc") {
            return Some(n);
        }
    }
    crate::engine::special_def_get("__npc", None)
}

/// Set a queue definition named `value`.
pub fn set_value(queue: &Arc<RwLock<ScriptQueue>>, obj: ObjectTag) {
    queue.write().unwrap().set_definition("value", Some(obj));
}

/// Set an arbitrary queue definition.
pub fn set_def(queue: &Arc<RwLock<ScriptQueue>>, key: &str, obj: ObjectTag) {
    queue.write().unwrap().set_definition(key, Some(obj));
}

/// Record a command intent with the host (narrate/log) if no host is present.
pub fn record(text: &str) {
    crate::util::debug_log(text);
}

/// Parse a list argument into an ObjectTag list.
pub fn as_list(text: &str, ctx: &TagContext) -> ListTag {
    tag_object(text, ctx).to_list()
}

/// A helper to build an ElementTag object quickly.
pub fn el(text: &str) -> ObjectTag {
    ObjectTag::new(ElementTag::new(text))
}

/// A helper to build a boolean ElementTag object.
pub fn el_bool(b: bool) -> ObjectTag {
    ObjectTag::new(ElementTag::from_bool(b))
}

/// A helper to build an integer ElementTag object.
pub fn el_int(i: i64) -> ObjectTag {
    ObjectTag::new(ElementTag::from_int(i))
}

/// Resolve a duration argument to milliseconds.
pub fn duration_millis(text: &str, ctx: &TagContext) -> i64 {
    let t = tag(text, ctx);
    crate::obj::DurationTag::from_identify(&t)
        .map(|d| d.as_millis() as i64)
        .or_else(|| t.trim().parse::<i64>().ok().map(|n| n * 1000))
        .unwrap_or(0)
}
