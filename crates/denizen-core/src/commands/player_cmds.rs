//! Player commands: chat, actionbar, title, kick, gamemode, experience, permission,
//! sidebar, tablist, money, group, resourcepack, and more.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};

fn intent(command: &str, detail: &str) {
    if detail.is_empty() {
        crate::util::debug_log(&format!("[{}]", command));
    } else {
        crate::util::debug_log(&format!("[{}] {}", command, detail));
    }
}

fn player_target(entry: &crate::scripts::entry::ScriptEntry, ctx: &crate::tags::TagContext) -> String {
    if let Some(p) = h::prefixed(entry, "player", ctx) {
        return p;
    }
    h::linked_player(ctx).map(|p| p.identify()).unwrap_or_default()
}

pub fn register_all() {
    register("chat", "chat [<text>] (player:<player>)", |entry, ctx, _q| {
        let text = h::payload(entry, ctx, &["player"], &[]);
        let target = player_target(entry, ctx);
        intent("chat", &format!("{}: {}", target, text));
        Ok(CommandOutcome::Continue)
    });

    register("actionbar", "actionbar [<text>] (targets:<player>|...)", |entry, ctx, _q| {
        let text = h::payload(entry, ctx, &["targets"], &[]);
        intent("actionbar", &text);
        Ok(CommandOutcome::Continue)
    });

    register("title", "title [<title>] (subtitle:<subtitle>) (fade_in:<duration>) (stay:<duration>) (fade_out:<duration>) (targets:<player>|...)", |entry, ctx, _q| {
        let title = h::arg_tag(entry, 0, ctx);
        let subtitle = h::prefixed(entry, "subtitle", ctx).unwrap_or_default();
        intent("title", &format!("{} / {}", title, subtitle));
        Ok(CommandOutcome::Continue)
    });

    register("kick", "kick (reason:<reason>) (targets:<player>|...)", |entry, ctx, _q| {
        let reason = h::prefixed(entry, "reason", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        let target = player_target(entry, ctx);
        intent("kick", &format!("{}: {}", target, reason));
        Ok(CommandOutcome::Continue)
    });

    register("gamemode", "gamemode [<gamemode>] (player:<player>)", |entry, ctx, _q| {
        let mode = h::arg_tag(entry, 0, ctx);
        let target = player_target(entry, ctx);
        intent("gamemode", &format!("{} {}", target, mode));
        Ok(CommandOutcome::Continue)
    });

    register("experience", "experience [<action>] (amount:<#>) (level:<#>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        let target = player_target(entry, ctx);
        intent("experience", &format!("{} {} {}", target, action, amount));
        Ok(CommandOutcome::Continue)
    });

    register("permission", "permission [<action>] [<permission>] (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let perm = h::arg_tag(entry, 1, ctx);
        let target = player_target(entry, ctx);
        intent("permission", &format!("{} {} {}", target, action, perm));
        Ok(CommandOutcome::Continue)
    });

    register("group", "group [<action>] [<group>] (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let group = h::arg_tag(entry, 1, ctx);
        let target = player_target(entry, ctx);
        intent("group", &format!("{} {} {}", target, action, group));
        Ok(CommandOutcome::Continue)
    });

    register("money", "money [give/take/set] (quantity:<#>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let qty = h::prefixed(entry, "quantity", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        let target = player_target(entry, ctx);
        intent("money", &format!("{} {} {}", action, qty, target));
        Ok(CommandOutcome::Continue)
    });

    register("sidebar", "sidebar [<action>] (id:<id>) (title:<title>) (value:<value>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("sidebar", &action);
        Ok(CommandOutcome::Continue)
    });

    register("tablist", "tablist [<action>] (header:<header>) (footer:<footer>) (targets:<player>|...)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("tablist", &action);
        Ok(CommandOutcome::Continue)
    });

    register("team", "team [<action>] [<team>] (players:<player>|...)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let team = h::arg_tag(entry, 1, ctx);
        intent("team", &format!("{} {}", action, team));
        Ok(CommandOutcome::Continue)
    });

    register("compass", "compass (target:<location>) (player:<player>)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "target", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        intent("compass", &loc);
        Ok(CommandOutcome::Continue)
    });

    register("oxygen", "oxygen [<action>] (amount:<#>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("oxygen", &format!("{} {}", action, amount));
        Ok(CommandOutcome::Continue)
    });

    register("resourcepack", "resourcepack [<action>] (url:<url>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("resourcepack", &action);
        Ok(CommandOutcome::Continue)
    });

    register("statistic", "statistic [<action>] [<statistic>] (amount:<#>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let stat = h::arg_tag(entry, 1, ctx);
        intent("statistic", &format!("{} {}", action, stat));
        Ok(CommandOutcome::Continue)
    });

    register("advancement", "advancement [<action>] [<advancement>] (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let adv = h::arg_tag(entry, 1, ctx);
        intent("advancement", &format!("{} {}", action, adv));
        Ok(CommandOutcome::Continue)
    });

    register("toast", "toast [<text>] (icon:<item>) (frame:<frame>) (player:<player>)", |entry, ctx, _q| {
        let text = h::arg_tag(entry, 0, ctx);
        intent("toast", &text);
        Ok(CommandOutcome::Continue)
    });

    register("bossbar", "bossbar [<action>] (id:<id>) (title:<title>) (progress:<#>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("bossbar", &action);
        Ok(CommandOutcome::Continue)
    });

    register("clickable", "clickable [<action>] (id:<id>) (chat:<chat>) (command:<command>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("clickable", &action);
        Ok(CommandOutcome::Continue)
    });

    register("itemcooldown", "itemcooldown [<action>] (item:<item>) (duration:<duration>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("itemcooldown", &action);
        Ok(CommandOutcome::Continue)
    });

    register("blockcrack", "blockcrack (location:<location>) (progress:<#>) (player:<player>)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("blockcrack", &loc);
        Ok(CommandOutcome::Continue)
    });

    register("opentrades", "opentrades (entity:<entity>) (player:<player>)", |entry, ctx, _q| {
        let target = h::prefixed(entry, "entity", ctx).unwrap_or_default();
        intent("opentrades", &target);
        Ok(CommandOutcome::Continue)
    });

}
