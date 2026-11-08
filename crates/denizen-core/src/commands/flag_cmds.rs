//! Flag commands: read/write persistent per-object flags.

use crate::commands::registry::CommandOutcome;
use crate::obj::{ElementTag, ListTag, ObjectTag};
use crate::scripts::entry::ScriptEntry;
use crate::tags::{tag, tag_object, TagContext};

fn tag_arg(text: &str, ctx: &TagContext) -> String {
    tag(text, ctx)
}

fn resolve_target(text: &str, ctx: &TagContext) -> String {
    let t = text.trim();
    if t.is_empty() {
        if let Some(p) = crate::commands::helpers::linked_player(ctx) {
            return p.identify();
        }
        return "server".to_string();
    }
    let tagged = tag_arg(t, ctx);
    if tagged.eq_ignore_ascii_case("server") {
        return "server".to_string();
    }
    // If the text looks like a bare object reference without a prefix, tag it.
    if tagged.contains('@') {
        return tagged;
    }
    // Bare word: treat as server-level flag name in the legacy form.
    t.to_string()
}

/// Determine whether the first argument is an object reference (target) or a flag name.
fn looks_like_target(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t == "server" || t == "global" {
        return true;
    }
    if t.contains('@') {
        return true;
    }
    false
}

/// Handle the `flag` command.
pub fn handle_flag_command(entry: &ScriptEntry, ctx: &TagContext) -> Result<CommandOutcome, String> {
    let target_text = entry.args.first().cloned().unwrap_or_default();
    let (target_id, action, idx) = if looks_like_target(&target_text) {
        let id = resolve_target(&target_text, ctx);
        let action = entry.args.get(1).map(|s| s.to_lowercase()).unwrap_or_default();
        (id, action, 2)
    } else {
        // Legacy form: `flag <action> ...` operating on the linked player.
        let action = target_text.to_lowercase();
        let id = resolve_target("", ctx);
        (id, action, 1)
    };

    // Recognize the special `object` prefix to override the target.
    let tracker = crate::flags::tracker_for(&target_id);

    match action.as_str() {
        "set" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let value: String = entry.args[idx + 1..]
                .iter()
                .filter(|a| !a.to_lowercase().starts_with("expire:"))
                .map(|s| tag_arg(s, ctx))
                .collect::<Vec<_>>()
                .join(" ");
            let expire = entry
                .args
                .iter()
                .find_map(|a| a.to_lowercase().strip_prefix("expire:").map(|v| tag(v, ctx)));
            let obj = if let Some(exp) = expire {
                let millis = crate::obj::DurationTag::from_identify(&exp)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                crate::flags::flag_value_with_expiry(Some(crate::util::current_time_millis() + millis), tag_object(&value, ctx), ctx)
            } else {
                tag_object(&value, ctx)
            };
            tracker.set(&key, Some(obj));
        }
        "delete" | "remove" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            if key.is_empty() {
                tracker.clear();
            } else {
                tracker.remove(&key);
            }
        }
        "add" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let value: String = entry.args[idx + 1..]
                .iter()
                .map(|s| tag_arg(s, ctx))
                .collect::<Vec<_>>()
                .join(" ");
            let mut list = tracker.get(&key).map(|o| o.to_list()).unwrap_or_default();
            list.add_object(tag_object(&value, ctx));
            tracker.set(&key, Some(ObjectTag::new(list)));
        }
        "remove_value" | "remove_list" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let value = entry.args.get(idx + 1).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let existing = tracker.get(&key).map(|o| o.to_list()).unwrap_or_default();
            let mut list = ListTag::new();
            for item in existing.items {
                if !item.identify().eq_ignore_ascii_case(&value) {
                    list.add_object(item);
                }
            }
            tracker.set(&key, Some(ObjectTag::new(list)));
        }
        "insert" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let position = entry
                .args
                .get(idx + 1)
                .map(|s| tag_arg(s, ctx))
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0);
            let value = entry.args.get(idx + 2).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let mut list = tracker.get(&key).map(|o| o.to_list()).unwrap_or_default();
            let pos = position.min(list.items.len());
            list.items.insert(pos, tag_object(&value, ctx));
            tracker.set(&key, Some(ObjectTag::new(list)));
        }
        "list" => {
            let prefix = entry.args.get(idx).map(|s| tag_arg(s, ctx));
            let list = tracker.list(prefix.as_deref());
            if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                q.write().unwrap().set_definition("value", Some(ObjectTag::new(list)));
            }
        }
        "has" | "contains" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let has = tracker.has(&key);
            if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                q.write().unwrap().set_definition("value", Some(ObjectTag::new(ElementTag::from_bool(has))));
            }
        }
        "get" => {
            let key = entry.args.get(idx).map(|s| tag_arg(s, ctx)).unwrap_or_default();
            let value = tracker.get(&key);
            if let Some(q) = entry.queue_id.as_ref().and_then(|id| crate::scripts::queue::get_queue(id)) {
                q.write().unwrap().set_definition("value", value);
            }
        }
        _ => {
            // Treat unknown action as `set` with the action itself as the key.
            let key = action.clone();
            let value: String = entry.args[idx..].iter().map(|s| tag_arg(s, ctx)).collect::<Vec<_>>().join(" ");
            if value.is_empty() {
                tracker.set(&key, Some(ObjectTag::new(ElementTag::from_bool(true))));
            } else {
                tracker.set(&key, Some(tag_object(&value, ctx)));
            }
        }
    }
    Ok(CommandOutcome::Continue)
}

pub fn register_all() {
    // `flag` is registered in core_cmds to delegate here.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_set_and_get() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        let ctx = TagContext::default();
        let e = ScriptEntry::new("flag", vec!["server".into(), "set".into(), "myflag".into(), "hello".into()], None, 1);
        handle_flag_command(&e, &ctx).unwrap();
        let t = crate::flags::tracker_for("server");
        assert_eq!(t.get("myflag").unwrap().identify(), "hello");
    }

    #[test]
    fn flag_add_list() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        let ctx = TagContext::default();
        let e = ScriptEntry::new("flag", vec!["server".into(), "add".into(), "listflag".into(), "a".into()], None, 1);
        handle_flag_command(&e, &ctx).unwrap();
        let t = crate::flags::tracker_for("server");
        let l = t.get("listflag").unwrap().to_list();
        assert_eq!(l.size(), 1);
    }
}
