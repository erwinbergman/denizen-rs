//! Data commands: flag, note, yaml, data, and persistent key/value storage helpers.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};
use crate::obj::{ElementTag, MapTag, ObjectTag};
use crate::tags::tag_object;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static DATA_STORE: LazyLock<RwLock<HashMap<String, ObjectTag>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn data_store() -> &'static RwLock<HashMap<String, ObjectTag>> {
    &DATA_STORE
}

pub fn register_all() {
    register("data", "data [<action>] (id:<id>) (key:<key>) (value:<value>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx).to_lowercase();
        let id = h::prefixed(entry, "id", ctx).unwrap_or_else(|| "data".to_string());
        let key = h::prefixed(entry, "key", ctx).unwrap_or_default();
        let value = h::prefixed(entry, "value", ctx).unwrap_or_default();
        let full = format!("{}.{}", id, key);
        match action.as_str() {
            "set" | "put" => {
                DATA_STORE.write().unwrap().insert(full, tag_object(&value, ctx));
            }
            "delete" | "remove" => {
                DATA_STORE.write().unwrap().remove(&full);
            }
            "get" => {
                let v = DATA_STORE.read().unwrap().get(&full).cloned();
                if let Some(v) = v {
                    h::set_def(_q, "value", v);
                }
            }
            "has" => {
                let has = DATA_STORE.read().unwrap().contains_key(&full);
                h::set_def(_q, "value", ObjectTag::new(ElementTag::from_bool(has)));
            }
            _ => {}
        }
        Ok(CommandOutcome::Continue)
    });

    register("define_map", "define_map [<name>] [<key>] [<value>]", |entry, ctx, q| {
        let name = h::arg_tag(entry, 0, ctx);
        let key = h::arg_tag(entry, 1, ctx);
        let value = h::tag_rest(entry, 2, ctx);
        let mut q = q.write().unwrap();
        let mut m = q.get_definition(&name).and_then(|o| o.to_map()).unwrap_or_default();
        m.put_string(&key, tag_object(&value, ctx));
        q.set_definition(&name, Some(ObjectTag::new(m)));
        Ok(CommandOutcome::Continue)
    });

    register("map_set", "map_set [<name>] [<key>] [<value>]", |entry, ctx, q| {
        let name = h::arg_tag(entry, 0, ctx);
        let key = h::arg_tag(entry, 1, ctx);
        let value = h::tag_rest(entry, 2, ctx);
        let mut q = q.write().unwrap();
        let mut m = q.get_definition(&name).and_then(|o| o.to_map()).unwrap_or_default();
        m.put_string(&key, tag_object(&value, ctx));
        q.set_definition(&name, Some(ObjectTag::new(m)));
        Ok(CommandOutcome::Continue)
    });

    register("map_get", "map_get [<name>] [<key>]", |entry, ctx, q| {
        let name = h::arg_tag(entry, 0, ctx);
        let key = h::arg_tag(entry, 1, ctx);
        let val = q.read().unwrap().get_definition(&name).and_then(|o| o.to_map()).and_then(|m| m.get(&key).cloned());
        if let Some(v) = val {
            h::set_def(q, "value", v);
        }
        Ok(CommandOutcome::Continue)
    });

    register("map_remove", "map_remove [<name>] [<key>]", |entry, ctx, q| {
        let name = h::arg_tag(entry, 0, ctx);
        let key = h::arg_tag(entry, 1, ctx);
        let mut q = q.write().unwrap();
        if let Some(mut m) = q.get_definition(&name).and_then(|o| o.to_map()) {
            m.put_deep(&key, None);
            q.set_definition(&name, Some(ObjectTag::new(m)));
        }
        Ok(CommandOutcome::Continue)
    });

    register("list_add", "list_add [<name>] [<value>]", |entry, ctx, q| {
        let name = h::arg_tag(entry, 0, ctx);
        let value = h::tag_rest(entry, 1, ctx);
        let mut q = q.write().unwrap();
        let mut list = q.get_definition(&name).map(|o| o.to_list()).unwrap_or_default();
        list.add_object(tag_object(&value, ctx));
        q.set_definition(&name, Some(ObjectTag::new(list)));
        Ok(CommandOutcome::Continue)
    });

    register("list_remove", "list_remove [<name>] [<index>]", |entry, ctx, q| {
        let name = h::arg_tag(entry, 0, ctx);
        let idx: usize = h::arg_tag(entry, 1, ctx).trim().parse().unwrap_or(0);
        let mut q = q.write().unwrap();
        if let Some(mut list) = q.get_definition(&name).map(|o| o.to_list()) {
            if idx < list.items.len() {
                list.items.remove(idx);
            }
            q.set_definition(&name, Some(ObjectTag::new(list)));
        }
        Ok(CommandOutcome::Continue)
    });

    register("datamap", "datamap [<action>] (id:<id>) (key:<key>) (value:<value>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx).to_lowercase();
        let id = h::prefixed(entry, "id", ctx).unwrap_or_else(|| "map".to_string());
        let mut q = _q.write().unwrap();
        match action.as_str() {
            "create" => {
                q.set_definition(&id, Some(ObjectTag::new(MapTag::new())));
            }
            "set" => {
                let key = h::prefixed(entry, "key", ctx).unwrap_or_default();
                let value = h::prefixed(entry, "value", ctx).unwrap_or_default();
                let mut m = q.get_definition(&id).and_then(|o| o.to_map()).unwrap_or_default();
                m.put_string(&key, tag_object(&value, ctx));
                q.set_definition(&id, Some(ObjectTag::new(m)));
            }
            _ => {}
        }
        Ok(CommandOutcome::Continue)
    });
}
