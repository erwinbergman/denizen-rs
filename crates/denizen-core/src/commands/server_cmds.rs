//! Server commands: announce, ban, execute, scoreboard, bossbar, cooldown, zap, reset, etc.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};

fn intent(command: &str, detail: &str) {
    if detail.is_empty() {
        crate::util::debug_log(&format!("[{}]", command));
    } else {
        crate::util::debug_log(&format!("[{}] {}", command, detail));
    }
}

pub fn register_all() {
    register("ban", "ban [<player>] (reason:<reason>)", |entry, ctx, _q| {
        let target = h::arg_tag(entry, 0, ctx);
        let reason = h::prefixed(entry, "reason", ctx).unwrap_or_default();
        intent("ban", &format!("{} {}", target, reason));
        Ok(CommandOutcome::Continue)
    });

    register("cooldown", "cooldown [<duration>] (global) (script:<script>)", |entry, ctx, q| {
        let dur_text = h::arg_tag(entry, 0, ctx);
        let millis = h::duration_millis(&dur_text, ctx);
        let global = h::any_match(entry, &["global"]);
        let script = h::prefixed(entry, "script", ctx).unwrap_or_else(|| "default".to_string());
        let key = if global {
            format!("__interact_cooldown.{}", script)
        } else {
            format!("__interact_cooldown.{}.{}", h::linked_player(ctx).map(|p| p.identify()).unwrap_or_default(), script)
        };
        let expires = crate::util::current_time_millis() + millis;
        if global {
            crate::flags::SERVER_FLAGS.lock().unwrap().set(&key, Some(h::el_int(expires)));
        } else if let Some(p) = h::linked_player(ctx) {
            crate::flags::tracker_for(&p.identify()).set(&key, Some(h::el_int(expires)));
        }
        h::set_def(q, "value", h::el_int(millis));
        Ok(CommandOutcome::Continue)
    });

    register("zap", "zap [<steps>] (player:<player>)", |entry, ctx, _q| {
        let steps = h::arg_tag(entry, 0, ctx);
        intent("zap", &steps);
        Ok(CommandOutcome::Continue)
    });

    register("reset", "reset (player:<player>)", |entry, ctx, _q| {
        let target = h::prefixed(entry, "player", ctx)
            .or_else(|| h::linked_player(ctx).map(|p| p.identify()))
            .unwrap_or_default();
        intent("reset", &target);
        Ok(CommandOutcome::Continue)
    });

    register("reload", "reload (scripts)", |_entry, _ctx, _q| {
        crate::engine::reload_scripts();
        Ok(CommandOutcome::Continue)
    });

    register("op", "op [<player>]", |entry, ctx, _q| {
        let target = h::arg_tag(entry, 0, ctx);
        intent("op", &target);
        Ok(CommandOutcome::Continue)
    });

    register("deop", "deop [<player>]", |entry, ctx, _q| {
        let target = h::arg_tag(entry, 0, ctx);
        intent("deop", &target);
        Ok(CommandOutcome::Continue)
    });

    register("whitelist", "whitelist [<action>] (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("whitelist", &action);
        Ok(CommandOutcome::Continue)
    });

    register("save", "save (world:<world>)", |entry, ctx, _q| {
        let world = h::prefixed(entry, "world", ctx).unwrap_or_default();
        intent("save", &world);
        Ok(CommandOutcome::Continue)
    });
}
