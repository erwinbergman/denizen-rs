//! Interact commands: `interact` scripts, `click`, `zap`, and NPC interaction control.

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
    register("interact", "interact [<action>] (npc:<npc>) (player:<player>) (script:<script>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let npc = h::prefixed(entry, "npc", ctx)
            .or_else(|| h::linked_npc(ctx).map(|n| n.identify()))
            .unwrap_or_default();
        let script = h::prefixed(entry, "script", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("interact", &format!("{} {} script:{}", action, npc, script));
        Ok(CommandOutcome::Continue)
    });

    register("click", "click (npc:<npc>) (player:<player>) (location:<location>)", |entry, ctx, _q| {
        let npc = h::prefixed(entry, "npc", ctx)
            .or_else(|| h::linked_npc(ctx).map(|n| n.identify()))
            .unwrap_or_default();
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("click", &format!("{} at:{}", npc, loc));
        Ok(CommandOutcome::Continue)
    });

    register("path", "path [<action>] (npc:<npc>) (location:<location>) (path:<path>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let npc = h::prefixed(entry, "npc", ctx)
            .or_else(|| h::linked_npc(ctx).map(|n| n.identify()))
            .unwrap_or_default();
        let path = h::prefixed(entry, "path", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("path", &format!("{} {} {}", action, npc, path));
        Ok(CommandOutcome::Continue)
    });

    register("engage_npc", "engage (npc:<npc>) (target:<entity>)", |entry, ctx, _q| {
        let npc = h::prefixed(entry, "npc", ctx)
            .or_else(|| h::linked_npc(ctx).map(|n| n.identify()))
            .unwrap_or_default();
        let target = h::prefixed(entry, "target", ctx).unwrap_or_default();
        intent("engage", &format!("{} target:{}", npc, target));
        Ok(CommandOutcome::Continue)
    });

    register("walkto", "walkto (npc:<npc>) (location:<location>) (speed:<#>)", |entry, ctx, _q| {
        let npc = h::prefixed(entry, "npc", ctx)
            .or_else(|| h::linked_npc(ctx).map(|n| n.identify()))
            .unwrap_or_default();
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        intent("walkto", &format!("{} to:{}", npc, loc));
        Ok(CommandOutcome::Continue)
    });
}
