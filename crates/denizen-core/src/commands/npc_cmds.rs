//! NPC commands: create, despawn, engage, disengage, sit, stand, action, trigger,
//! assignment, pose, anchor, fish, sleep, vulnerable, pushable, and more.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};

fn intent(command: &str, detail: &str) {
    if detail.is_empty() {
        crate::util::debug_log(&format!("[{}]", command));
    } else {
        crate::util::debug_log(&format!("[{}] {}", command, detail));
    }
}

fn npc_target(entry: &crate::scripts::entry::ScriptEntry, ctx: &crate::tags::TagContext) -> String {
    if let Some(n) = h::prefixed(entry, "npc", ctx) {
        return n;
    }
    if let Some(n) = h::linked_npc(ctx) {
        return n.identify();
    }
    h::arg_tag(entry, 0, ctx)
}

pub fn register_all() {
    register("create", "create [<name>] (location:<location>) (type:<entity>)", |entry, ctx, _q| {
        let name = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("create", &format!("{} at:{}", name, loc));
        Ok(CommandOutcome::Continue)
    });

    register("despawn", "despawn (npc:<npc>) (id:<id>)", |entry, ctx, _q| {
        let target = npc_target(entry, ctx);
        intent("despawn", &target);
        Ok(CommandOutcome::Continue)
    });

    register("engage", "engage (npc:<npc>) (target:<entity>) (delay:<duration>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let target = h::prefixed(entry, "target", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("engage", &format!("{} target:{}", npc, target));
        Ok(CommandOutcome::Continue)
    });

    register("disengage", "disengage (npc:<npc>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        intent("disengage", &npc);
        Ok(CommandOutcome::Continue)
    });

    register("sit", "sit (npc:<npc>) (location:<location>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("sit", &format!("{} at:{}", npc, loc));
        Ok(CommandOutcome::Continue)
    });

    register("stand", "stand (npc:<npc>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        intent("stand", &npc);
        Ok(CommandOutcome::Continue)
    });

    register("sleep", "sleep (npc:<npc>) (location:<location>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("sleep", &format!("{} at:{}", npc, loc));
        Ok(CommandOutcome::Continue)
    });

    register("action", "action [<action>] (npc:<npc>) (to:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let npc = npc_target(entry, ctx);
        intent("npc_action", &format!("{} {}", npc, action));
        Ok(CommandOutcome::Continue)
    });

    register("trigger", "trigger (npc:<npc>) (targets:<player>|...) (radius:<#>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        intent("trigger", &npc);
        Ok(CommandOutcome::Continue)
    });

    register("assignment", "assignment [set/remove/clear] (npc:<npc>) (script:<script>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let npc = npc_target(entry, ctx);
        let script = h::prefixed(entry, "script", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("assignment", &format!("{} {} {}", action, npc, script));
        Ok(CommandOutcome::Continue)
    });

    register("pose", "pose [<pose>] (npc:<npc>)", |entry, ctx, _q| {
        let pose = h::arg_tag(entry, 0, ctx);
        let npc = npc_target(entry, ctx);
        intent("pose", &format!("{} {}", npc, pose));
        Ok(CommandOutcome::Continue)
    });

    register("anchor", "anchor [<anchor>] (npc:<npc>) (location:<location>)", |entry, ctx, _q| {
        let anchor = h::arg_tag(entry, 0, ctx);
        let npc = npc_target(entry, ctx);
        intent("anchor", &format!("{} {}", npc, anchor));
        Ok(CommandOutcome::Continue)
    });

    register("fish", "fish [<action>] (npc:<npc>) (location:<location>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let npc = npc_target(entry, ctx);
        intent("fish", &format!("{} {}", npc, action));
        Ok(CommandOutcome::Continue)
    });

    register("vulnerable", "vulnerable (npc:<npc>) (state:<true/false>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "true".to_string());
        intent("vulnerable", &format!("{} {}", npc, state));
        Ok(CommandOutcome::Continue)
    });

    register("pushable", "pushable (npc:<npc>) (state:<true/false>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "true".to_string());
        intent("pushable", &format!("{} {}", npc, state));
        Ok(CommandOutcome::Continue)
    });

    register("lookclose", "lookclose (npc:<npc>) (state:<true/false>) (range:<#>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "true".to_string());
        intent("lookclose", &format!("{} {}", npc, state));
        Ok(CommandOutcome::Continue)
    });

    register("trait", "trait [<action>] [<trait>] (npc:<npc>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let trait_name = h::arg_tag(entry, 1, ctx);
        let npc = npc_target(entry, ctx);
        intent("trait", &format!("{} {} {}", action, trait_name, npc));
        Ok(CommandOutcome::Continue)
    });

    register("pause", "pause (npc:<npc>) (duration:<duration>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let dur = h::prefixed(entry, "duration", ctx).unwrap_or_default();
        intent("pause", &format!("{} {}", npc, dur));
        Ok(CommandOutcome::Continue)
    });

    register("resume", "resume (npc:<npc>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        intent("resume", &npc);
        Ok(CommandOutcome::Continue)
    });

    register("break", "break (npc:<npc>) (location:<location>)", |entry, ctx, _q| {
        let npc = npc_target(entry, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("npc_break", &format!("{} at:{}", npc, loc));
        Ok(CommandOutcome::Continue)
    });

    register("npcbossbar", "npcbossbar [<action>] (npc:<npc>) (title:<title>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let npc = npc_target(entry, ctx);
        intent("npcbossbar", &format!("{} {}", npc, action));
        Ok(CommandOutcome::Continue)
    });
}
