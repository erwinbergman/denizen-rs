//! Core commands that operate on objects and output: narrate, debug, adjust, note, yaml, file, etc.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};
use crate::obj::{ElementTag, ObjectTag};
use crate::scripts::entry::ScriptEntry;
use crate::tags::{tag, tag_object, TagContext};
use std::sync::{Arc, LazyLock, RwLock};
use std::collections::HashMap;

fn arg(entry: &ScriptEntry, idx: usize) -> String {
    entry.args.get(idx).cloned().unwrap_or_default()
}

fn prefix_value(entry: &ScriptEntry, prefix: &str, ctx: &TagContext) -> Option<String> {
    entry.prefixed(prefix).map(|v| tag(v, ctx))
}

/// The text of a message command, excluding known prefixes and flags.
fn message_text(entry: &ScriptEntry, ctx: &TagContext, extra_prefixes: &[&str]) -> String {
    let mut skip = vec!["targets", "format", "level", "name", "color", "prefix"];
    skip.extend_from_slice(extra_prefixes);
    h::payload(entry, ctx, &skip, &["per_player", "raw", "no_color"])
}

fn narrate_to(target: Option<&ObjectTag>, text: &str) {
    if let Some(host) = crate::engine::get_host() {
        host.narrate(target, text);
    } else {
        crate::util::debug_log(text);
    }
}

pub fn register_all() {
    register("narrate", "narrate [<text>] (targets:<player>|...) (format:<script>)", |entry, ctx, _q| {
        let text = message_text(entry, ctx, &[]);
        let target = h::linked_player(ctx);
        narrate_to(target.as_ref(), &text);
        Ok(CommandOutcome::Continue)
    });

    register("debug", "debug (<type>) [<message>] (name:<name>) (format:<format>)", |entry, ctx, _q| {
        let mut kind = "output";
        let mut start = 0;
        if let Some(first) = entry.args.first() {
            let l = first.to_lowercase();
            if matches!(l.as_str(), "output" | "debug" | "header" | "footer" | "spacer" | "log" | "approval" | "error" | "report" | "exception" | "record") {
                kind = match l.as_str() {
                    "output" => "output",
                    "debug" => "debug",
                    "header" => "header",
                    "footer" => "footer",
                    "spacer" => "spacer",
                    "log" => "log",
                    "approval" => "approval",
                    "error" => "error",
                    "report" => "report",
                    "exception" => "exception",
                    _ => "record",
                };
                start = 1;
            }
        }
        let text: String = entry.args[start..]
            .iter()
            .filter(|a| {
                let l = a.to_lowercase();
                !l.starts_with("name:") && !l.starts_with("format:")
            })
            .map(|a| tag(a, ctx))
            .collect::<Vec<_>>()
            .join(" ");
        match kind {
            "error" => ctx.echo_error(&text),
            "log" => crate::util::debug_log(&format!("[LOG] {}", text)),
            "header" => crate::util::debug_log(&format!("===== {} =====", text)),
            "footer" => crate::util::debug_log(&format!("----- {} -----", text)),
            "spacer" => crate::util::debug_log(""),
            _ => crate::util::debug_log(&format!("[DEBUG] {}", text)),
        }
        Ok(CommandOutcome::Continue)
    });

    register("announce", "announce [<text>] (targets:<player>|...)", |entry, ctx, _q| {
        let text = message_text(entry, ctx, &[]);
        narrate_to(None, &text);
        Ok(CommandOutcome::Continue)
    });

    register("execute", "execute (as_server) (as_player) (as_op) [<command>]", |entry, ctx, _q| {
        let mut parts = Vec::new();
        for a in &entry.args {
            let lower = a.to_lowercase();
            if lower == "as_server" || lower == "as_player" || lower == "as_op" || lower == "as_npc" {
                continue;
            }
            parts.push(tag(a, ctx));
        }
        let command = parts.join(" ");
        if let Some(host) = crate::engine::get_host() {
            host.run_server_command(&command);
        } else {
            crate::util::debug_log(&format!("[execute] /{}", command));
        }
        Ok(CommandOutcome::Continue)
    });

    register("adjust", "adjust [<object>] [<mechanism>](:<value>)", |entry, ctx, _q| {
        // Multiple objects may be given as a `|`-separated list.
        let target_text = arg(entry, 0);
        let targets = h::as_list(&target_text, ctx);
        let mech_arg = arg(entry, 1);
        let (mech_name, inline_value) = match mech_arg.find(':') {
            Some(idx) => (mech_arg[..idx].to_string(), Some(mech_arg[idx + 1..].to_string())),
            None => (mech_arg.clone(), None),
        };
        let value_text = match inline_value {
            Some(v) => v,
            None => h::raw_rest(entry, 2),
        };
        let value = tag_object(&value_text, ctx);
        for t in targets.items.clone() {
            let mut mech = crate::obj::Mechanism::new(&mech_name, Some(value.clone()), ctx.clone());
            crate::obj::process_mechanism(&t, &mut mech);
        }
        Ok(CommandOutcome::Continue)
    });

    register("flag", "flag [<object>] [<action>] (<flag_name>) (<value>)", |entry, ctx, _q| {
        crate::commands::flag_cmds::handle_flag_command(entry, ctx)
    });

    register("note", "note [<object>/remove] [as:<name>]", |entry, ctx, _q| {
        let first = tag(&arg(entry, 0), ctx);
        let id = prefix_value(entry, "as", ctx)
            .or_else(|| prefix_value(entry, "id", ctx))
            .unwrap_or_else(|| "note".to_string());
        let notes = note_store();
        if first.eq_ignore_ascii_case("remove") {
            notes.write().unwrap().remove(&id);
        } else {
            notes.write().unwrap().insert(id.clone(), tag_object(&first, ctx));
        }
        Ok(CommandOutcome::Continue)
    });

    register("yaml", "yaml [<action>] (id:<id>) (key:<key>) (value:<value>)", |entry, ctx, _q| {
        crate::commands::util_cmds::handle_yaml(entry, ctx)
    });

    register("file", "file [<action>] (path:<path>) (contents:<contents>)", |entry, ctx, _q| {
        crate::commands::util_cmds::handle_file(entry, ctx)
    });

    register("say", "say [<text>] (targets:<player>|...)", |entry, ctx, _q| {
        let text = message_text(entry, ctx, &[]);
        let target = h::linked_player(ctx);
        narrate_to(target.as_ref(), &text);
        Ok(CommandOutcome::Continue)
    });

    register("log", "log [<text>] (level:<level>)", |entry, ctx, _q| {
        let text: String = entry.args.iter()
            .filter(|a| !a.to_lowercase().starts_with("level:"))
            .map(|a| tag(a, ctx))
            .collect::<Vec<_>>()
            .join(" ");
        crate::util::debug_log(&text);
        Ok(CommandOutcome::Continue)
    });

    register("showfake", "showfake [<entity>] (to:<player>)", |entry, ctx, _q| {
        h::record(&format!("[showfake] {}", h::payload(entry, ctx, &["to"], &[])));
        Ok(CommandOutcome::Continue)
    });

    register("relativeto", "relativeto [<location>|...]", |entry, ctx, _q| {
        let _ = h::payload(entry, ctx, &[], &[]);
        Ok(CommandOutcome::Continue)
    });

    register("debugblock", "debugblock [on|off] (location:<location>)", |entry, ctx, _q| {
        h::record(&format!("[debugblock] {}", h::payload(entry, ctx, &["location"], &[])));
        Ok(CommandOutcome::Continue)
    });

    // Register the remaining command families.
    crate::commands::entity_cmds::register_all();
    crate::commands::player_cmds::register_all();
    crate::commands::world_cmds::register_all();
    crate::commands::npc_cmds::register_all();
    crate::commands::item_cmds::register_all();
    crate::commands::server_cmds::register_all();
    crate::commands::data_cmds::register_all();
    crate::commands::web_cmds::register_all();
    crate::commands::interact_cmds::register_all();
}

static NOTES: LazyLock<RwLock<HashMap<String, ObjectTag>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn note_store() -> &'static RwLock<HashMap<String, ObjectTag>> {
    &NOTES
}

#[allow(dead_code)]
fn _unused(_l: &Arc<RwLock<crate::scripts::queue::ScriptQueue>>, _e: ElementTag) {}
