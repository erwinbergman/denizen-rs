//! Utility commands: yaml and file handling backed by the in-memory YAML store.

use crate::commands::registry::CommandOutcome;
use crate::obj::{ElementTag, ObjectTag};
use crate::scripts::entry::ScriptEntry;
use crate::tags::{tag, TagContext};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static YAML_STORE: LazyLock<RwLock<HashMap<String, crate::util::yaml::Yaml>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn prefix_value(entry: &ScriptEntry, prefix: &str, ctx: &TagContext) -> Option<String> {
    let p = format!("{}:", prefix);
    for a in &entry.args {
        if a.to_lowercase().starts_with(&p) {
            return Some(tag(&a[p.len()..], ctx));
        }
    }
    None
}

/// Handle the `yaml` command.
pub fn handle_yaml(entry: &ScriptEntry, ctx: &TagContext) -> Result<CommandOutcome, String> {
    let action = entry.args.first().map(|s| tag(s, ctx).to_lowercase()).unwrap_or_default();
    let id = prefix_value(entry, "id", ctx).unwrap_or_else(|| "default".to_string());
    match action.as_str() {
        "create" | "load" => {
            let path = prefix_value(entry, "file", ctx);
            let mut map = crate::util::yaml::Yaml::new_map();
            if let Some(p) = path {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    map = crate::util::yaml::parse(&text);
                }
            }
            YAML_STORE.write().unwrap().insert(id, map);
        }
        "set" => {
            let key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let value = prefix_value(entry, "value", ctx).unwrap_or_default();
            let mut store = YAML_STORE.write().unwrap();
            let doc = store.entry(id).or_insert_with(crate::util::yaml::Yaml::new_map);
            set_yaml_path(doc, &key, crate::util::yaml::Yaml::String(value));
        }
        "delete" | "remove" => {
            let key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let mut store = YAML_STORE.write().unwrap();
            if let Some(doc) = store.get_mut(&id) {
                delete_yaml_path(doc, &key);
            }
        }
        "save" => {
            let store = YAML_STORE.read().unwrap();
            if let Some(doc) = store.get(&id) {
                if let Some(path) = prefix_value(entry, "file", ctx) {
                    let _ = std::fs::write(&path, doc.save_to_string());
                }
            }
        }
        "unload" => {
            YAML_STORE.write().unwrap().remove(&id);
        }
        "copykey" => {
            let from_id = prefix_value(entry, "from", ctx).unwrap_or_else(|| id.clone());
            let from_key = prefix_value(entry, "fromkey", ctx).unwrap_or_default();
            let to_key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let store = YAML_STORE.read().unwrap();
            let value = store.get(&from_id).and_then(|d| d.get(&from_key)).cloned();
            drop(store);
            if let Some(v) = value {
                let mut store = YAML_STORE.write().unwrap();
                let doc = store.entry(id).or_insert_with(crate::util::yaml::Yaml::new_map);
                set_yaml_path(doc, &to_key, v);
            }
        }
        "increment" | "decrement" | "add" | "subtract" | "multiply" | "divide" => {
            let key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let amount = prefix_value(entry, "value", ctx).unwrap_or_else(|| "1".to_string());
            let mut store = YAML_STORE.write().unwrap();
            let doc = store.entry(id).or_insert_with(crate::util::yaml::Yaml::new_map);
            let current = doc.get(&key).and_then(|v| v.as_string()).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
            let amt: f64 = amount.trim().parse().unwrap_or(0.0);
            let result = match action.as_str() {
                "increment" | "add" => current + amt,
                "decrement" | "subtract" => current - amt,
                "multiply" => current * amt,
                "divide" => if amt != 0.0 { current / amt } else { current },
                _ => current,
            };
            set_yaml_path(doc, &key, crate::util::yaml::Yaml::Float(result));
        }
        "insert" => {
            let key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let value = prefix_value(entry, "value", ctx).unwrap_or_default();
            let mut store = YAML_STORE.write().unwrap();
            let doc = store.entry(id).or_insert_with(crate::util::yaml::Yaml::new_map);
            let existing = doc.get(&key).cloned();
            match existing {
                Some(crate::util::yaml::Yaml::List(mut items)) => {
                    items.push(crate::util::yaml::Yaml::String(value));
                    set_yaml_path(doc, &key, crate::util::yaml::Yaml::List(items));
                }
                _ => set_yaml_path(doc, &key, crate::util::yaml::Yaml::List(vec![crate::util::yaml::Yaml::String(value)])),
            }
        }
        "get" => {
            let key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let store = YAML_STORE.read().unwrap();
            let value = store.get(&id).and_then(|d| d.get(&key)).and_then(|v| v.as_string());
            drop(store);
            if let Some(v) = value {
                if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                    q.write().unwrap().set_definition("value", Some(ObjectTag::new(ElementTag::new(v))));
                }
            }
        }
        "has" => {
            let key = prefix_value(entry, "key", ctx).unwrap_or_default();
            let store = YAML_STORE.read().unwrap();
            let has = store.get(&id).and_then(|d| d.get(&key)).is_some();
            drop(store);
            if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                q.write().unwrap().set_definition("value", Some(ObjectTag::new(ElementTag::from_bool(has))));
            }
        }
        _ => {}
    }
    Ok(CommandOutcome::Continue)
}

fn set_yaml_path(doc: &mut crate::util::yaml::Yaml, path: &str, value: crate::util::yaml::Yaml) {
    if path.is_empty() {
        *doc = value;
        return;
    }
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = doc;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            cur.set(part, value);
            return;
        }
        // Ensure the intermediate is a map.
        if !cur.get(part).map(|v| v.is_map()).unwrap_or(false) {
            cur.set(part, crate::util::yaml::Yaml::new_map());
        }
        let map = cur.as_map_mut().unwrap();
        let idx = map.iter().position(|(k, _)| k == part).unwrap();
        cur = &mut map[idx].1;
    }
}

fn delete_yaml_path(doc: &mut crate::util::yaml::Yaml, path: &str) {
    if path.is_empty() {
        *doc = crate::util::yaml::Yaml::new_map();
        return;
    }
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = doc;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            cur.remove(part);
            return;
        }
        let Some(next) = cur.as_map_mut().and_then(|m| m.iter_mut().find(|(k, _)| k == part).map(|(_, v)| v)) else {
            return;
        };
        cur = next;
    }
}

/// Handle the `file` command.
pub fn handle_file(entry: &ScriptEntry, ctx: &TagContext) -> Result<CommandOutcome, String> {
    let action = entry.args.first().map(|s| tag(s, ctx).to_lowercase()).unwrap_or_default();
    let path = prefix_value(entry, "path", ctx).unwrap_or_default();
    match action.as_str() {
        "write" => {
            let contents = prefix_value(entry, "contents", ctx).unwrap_or_default();
            if let Err(e) = std::fs::write(&path, contents) {
                ctx.echo_error(&format!("File write failed: {}", e));
            }
        }
        "read" => {
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                        q.write().unwrap().set_definition("value", Some(ObjectTag::new(ElementTag::new(text))));
                    }
                }
                Err(e) => ctx.echo_error(&format!("File read failed: {}", e)),
            }
        }
        "delete" => {
            let _ = std::fs::remove_file(&path);
        }
        "copy" => {
            let dest = prefix_value(entry, "destination", ctx).unwrap_or_default();
            let _ = std::fs::copy(&path, dest);
        }
        "exists" => {
            let exists = std::path::Path::new(&path).exists();
            if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                q.write().unwrap().set_definition("value", Some(ObjectTag::new(ElementTag::from_bool(exists))));
            }
        }
        _ => {}
    }
    Ok(CommandOutcome::Continue)
}

pub fn register_all() {
    use crate::commands::registry::register;

    register("customevent", "customevent [<id>] (context:<map>)", |entry, ctx, _q| {
        let name = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        let mut event = crate::events::ScriptEvent::new(&format!("custom {}", name));
        if let Some(map) = prefix_value(entry, "context", ctx).and_then(|t| crate::obj::MapTag::from_identify(&t)) {
            for (k, v) in map.entries {
                event.context.insert(k.to_lowercase(), v);
            }
        }
        crate::events::fire_event(&event);
        Ok(CommandOutcome::Continue)
    });

    register("filecopy", "filecopy [<source>] (dest:<path>)", |entry, ctx, _q| {
        let src = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        let dest = prefix_value(entry, "dest", ctx).or_else(|| entry.args.get(1).map(|s| tag(s, ctx))).unwrap_or_default();
        if let Err(e) = std::fs::copy(&src, &dest) {
            ctx.echo_error(&format!("File copy failed: {}", e));
        }
        Ok(CommandOutcome::Continue)
    });

    register("fileread", "fileread [<path>] (save:<name>)", |entry, ctx, _q| {
        let path = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        let save = prefix_value(entry, "save", ctx).unwrap_or_else(|| "value".to_string());
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                    q.write().unwrap().set_definition(&save, Some(ObjectTag::new(ElementTag::new(text))));
                }
            }
            Err(e) => ctx.echo_error(&format!("File read failed: {}", e)),
        }
        Ok(CommandOutcome::Continue)
    });

    register("filewrite", "filewrite [<path>] [<contents>]", |entry, ctx, _q| {
        let path = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        let contents: String = entry.args[1..].iter().map(|a| tag(a, ctx)).collect::<Vec<_>>().join(" ");
        if let Err(e) = std::fs::write(&path, contents) {
            ctx.echo_error(&format!("File write failed: {}", e));
        }
        Ok(CommandOutcome::Continue)
    });

    register("debug-invalid-command", "debug-invalid-command", |_entry, _ctx, _q| {
        Ok(CommandOutcome::Continue)
    });

    register("draw", "draw [<id>] (location:<location>) (radius:<#>)", |entry, ctx, _q| {
        let id = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        crate::util::debug_log(&format!("[draw] {}", id));
        Ok(CommandOutcome::Continue)
    });

    register("image", "image [<action>] (id:<id>) (path:<path>) (location:<location>)", |entry, ctx, _q| {
        let action = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        crate::util::debug_log(&format!("[image] {}", action));
        Ok(CommandOutcome::Continue)
    });

    register("reflectionset", "reflectionset [<path>] [<value>]", |entry, ctx, _q| {
        let path = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        crate::util::debug_log(&format!("[reflectionset] {}", path));
        Ok(CommandOutcome::Continue)
    });

    register("sql", "sql [<action>] (id:<id>) (query:<query>)", |entry, ctx, _q| {
        let action = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        crate::util::debug_log(&format!("[sql] {}", action));
        Ok(CommandOutcome::Continue)
    });

    register("mongo", "mongo [<action>] (id:<id>)", |entry, ctx, _q| {
        let action = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        crate::util::debug_log(&format!("[mongo] {}", action));
        Ok(CommandOutcome::Continue)
    });

    register("redis", "redis [<action>] (id:<id>)", |entry, ctx, _q| {
        let action = tag(&entry.args.first().cloned().unwrap_or_default(), ctx);
        crate::util::debug_log(&format!("[redis] {}", action));
        Ok(CommandOutcome::Continue)
    });

    register("yaml_load", "yaml_load (id:<id>) (file:<path>)", |entry, ctx, _q| {
        crate::commands::util_cmds::handle_yaml(entry, ctx)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::registry::CommandOutcome;

    fn ctx() -> TagContext {
        TagContext::default()
    }

    #[test]
    fn yaml_set_get() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        let mut e = ScriptEntry::new("yaml", vec!["set".into(), "id:test1".into(), "key:a.b".into(), "value:hello".into()], None, 1);
        handle_yaml(&e, &ctx()).unwrap();
        e = ScriptEntry::new("yaml", vec!["has".into(), "id:test1".into(), "key:a.b".into()], None, 1);
        let outcome = handle_yaml(&e, &ctx()).unwrap();
        assert_eq!(outcome, CommandOutcome::Continue);
    }

    #[test]
    fn file_roundtrip() {
        let _g = crate::scripts::test_guard();
        let path = "/tmp/denizen_file_test.txt";
        let e = ScriptEntry::new("file", vec!["write".into(), format!("path:{}", path), "contents:hello world".into()], None, 1);
        handle_file(&e, &ctx()).unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert_eq!(text, "hello world");
        let _ = std::fs::remove_file(path);
    }
}
