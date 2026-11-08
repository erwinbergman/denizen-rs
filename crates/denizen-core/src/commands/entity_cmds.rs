//! Entity commands: spawn, kill, hurt, heal, feed, fly, burn, explode, teleport,
//! mount, sneak, look, walk, follow, and more. Effects are recorded as intent via the host.

use crate::commands::helpers as h;
use crate::commands::registry::{register, CommandOutcome};
use crate::obj::ElementTag;
use crate::tags::tag_object;

/// Record an entity-affecting command intent.
fn intent(command: &str, target: &str, detail: &str) {
    if detail.is_empty() {
        crate::util::debug_log(&format!("[{}] {}", command, target));
    } else {
        crate::util::debug_log(&format!("[{}] {} {}", command, target, detail));
    }
}

/// Resolve the target entity text: explicit argument or the linked entity/player.
fn resolve_target(entry: &crate::scripts::entry::ScriptEntry, ctx: &crate::tags::TagContext) -> String {
    let text = h::arg(entry, 0);
    if text.trim().is_empty() {
        if let Some(p) = h::linked_player(ctx) {
            return p.identify();
        }
        if let Some(n) = h::linked_npc(ctx) {
            return n.identify();
        }
    }
    tag_object(&text, ctx).identify()
}

pub fn register_all() {
    register("spawn", "spawn [<entity>] (location:<location>) (reason:<cause>)", |entry, ctx, _q| {
        let entity = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("spawn", &entity, &loc);
        Ok(CommandOutcome::Continue)
    });

    register("kill", "kill (entity:<entity>|...)", |entry, ctx, _q| {
        let target = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        intent("kill", &target, "");
        Ok(CommandOutcome::Continue)
    });

    register("hurt", "hurt [<entity>] (amount:<#>) (cause:<cause>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_else(|| "1".to_string());
        let cause = h::prefixed(entry, "cause", ctx).unwrap_or_default();
        intent("hurt", &target, &format!("amount:{} cause:{}", amount, cause));
        Ok(CommandOutcome::Continue)
    });

    register("heal", "heal [<entity>] (amount:<#>) (cause:<cause>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_else(|| "full".to_string());
        intent("heal", &target, &format!("amount:{}", amount));
        Ok(CommandOutcome::Continue)
    });

    register("health", "health [<entity>] (amount:<#>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("health", &target, &amount);
        Ok(CommandOutcome::Continue)
    });

    register("feed", "feed [<entity>] (amount:<#>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_else(|| "full".to_string());
        intent("feed", &target, &amount);
        Ok(CommandOutcome::Continue)
    });

    register("fly", "fly [<entity>] (state:<true/false>) (speed:<#>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "toggle".to_string());
        intent("fly", &target, &state);
        Ok(CommandOutcome::Continue)
    });

    register("burn", "burn [<entity>] (duration:<duration>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let duration = h::prefixed(entry, "duration", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("burn", &target, &duration);
        Ok(CommandOutcome::Continue)
    });

    register("explode", "explode (entity:<entity>) (location:<location>) (power:<#>) (fire:<true/false>)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        let power = h::prefixed(entry, "power", ctx).unwrap_or_else(|| "1".to_string());
        intent("explode", &loc, &format!("power:{}", power));
        Ok(CommandOutcome::Continue)
    });

    register("teleport", "teleport [<location>] (entity:<entity>) (cause:<cause>)", |entry, ctx, _q| {
        let loc = h::arg_tag(entry, 0, ctx);
        let entity = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| {
            h::linked_player(ctx).map(|p| p.identify()).unwrap_or_default()
        });
        intent("teleport", &entity, &format!("to:{}", loc));
        Ok(CommandOutcome::Continue)
    });

    register("mount", "mount (entity:<entity>) (location:<location>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("mount", &entity, &loc);
        Ok(CommandOutcome::Continue)
    });

    register("dismount", "dismount (entity:<entity>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        intent("dismount", &entity, "");
        Ok(CommandOutcome::Continue)
    });

    register("sneak", "sneak (entity:<entity>) (state:<true/false>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "true".to_string());
        intent("sneak", &entity, &state);
        Ok(CommandOutcome::Continue)
    });

    register("look", "look [<entity>] (<location>|<entity>) (angle:<#>)", |entry, ctx, _q| {
        let entity = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        let at = h::arg_tag(entry, 0, ctx);
        intent("look", &entity, &format!("at:{}", at));
        Ok(CommandOutcome::Continue)
    });

    register("walk", "walk [<location>] (entity:<entity>) (speed:<#>)", |entry, ctx, _q| {
        let loc = h::arg_tag(entry, 0, ctx);
        let entity = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        intent("walk", &entity, &format!("to:{}", loc));
        Ok(CommandOutcome::Continue)
    });

    register("follow", "follow (entity:<entity>) (target:<entity>) (distance:<#>)", |entry, ctx, _q| {
        let entity = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        let target = h::prefixed(entry, "target", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        intent("follow", &entity, &format!("target:{}", target));
        Ok(CommandOutcome::Continue)
    });

    register("rotate", "rotate [<entity>] (yaw:<#>) (pitch:<#>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        let yaw = h::prefixed(entry, "yaw", ctx).unwrap_or_default();
        let pitch = h::prefixed(entry, "pitch", ctx).unwrap_or_default();
        intent("rotate", &entity, &format!("yaw:{} pitch:{}", yaw, pitch));
        Ok(CommandOutcome::Continue)
    });

    register("push", "push [<entity>] (direction:<location>) (speed:<#>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        let dir = h::prefixed(entry, "direction", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("push", &entity, &format!("dir:{}", dir));
        Ok(CommandOutcome::Continue)
    });

    register("shoot", "shoot [<entity>] (origin:<location>) (destination:<location>) (speed:<#>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        let dest = h::prefixed(entry, "destination", ctx).unwrap_or_default();
        intent("shoot", &entity, &format!("dest:{}", dest));
        Ok(CommandOutcome::Continue)
    });

    register("attack", "attack [<entity>] (target:<entity>)", |entry, ctx, _q| {
        let entity = resolve_target(entry, ctx);
        let target = h::prefixed(entry, "target", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("attack", &entity, &format!("target:{}", target));
        Ok(CommandOutcome::Continue)
    });

    register("equip", "equip (entity:<entity>) (location:<location>) (item:<item>)", |entry, ctx, _q| {
        let entity = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        let item = h::prefixed(entry, "item", ctx).unwrap_or_default();
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| "hand".to_string());
        intent("equip", &entity, &format!("{} {}", loc, item));
        Ok(CommandOutcome::Continue)
    });

    register("fakeequip", "fakeequip (entity:<entity>) (item:<item>) (slot:<slot>)", |entry, ctx, _q| {
        let entity = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        let item = h::prefixed(entry, "item", ctx).unwrap_or_default();
        intent("fakeequip", &entity, &item);
        Ok(CommandOutcome::Continue)
    });

    register("remove", "remove (entity:<entity>|...) (npc:<npc>)", |entry, ctx, _q| {
        let target = h::prefixed(entry, "entity", ctx).unwrap_or_else(|| resolve_target(entry, ctx));
        intent("remove", &target, "");
        Ok(CommandOutcome::Continue)
    });

    register("rename", "rename [<entity>] [<name>]", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let name = h::arg_tag(entry, 1, ctx);
        intent("rename", &target, &name);
        Ok(CommandOutcome::Continue)
    });

    register("glow", "glow [<entity>] (state:<true/false>) (color:<color>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "true".to_string());
        intent("glow", &target, &state);
        Ok(CommandOutcome::Continue)
    });

    register("invisible", "invisible [<entity>] (state:<true/false>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let state = h::prefixed(entry, "state", ctx).unwrap_or_else(|| "true".to_string());
        intent("invisible", &target, &state);
        Ok(CommandOutcome::Continue)
    });

    register("leash", "leash (entity:<entity>) (holder:<entity>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let holder = h::prefixed(entry, "holder", ctx).unwrap_or_default();
        intent("leash", &target, &holder);
        Ok(CommandOutcome::Continue)
    });

    register("attach", "attach [<entity>] (to:<entity>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let to = h::prefixed(entry, "to", ctx).unwrap_or_default();
        intent("attach", &target, &to);
        Ok(CommandOutcome::Continue)
    });

    register("animate", "animate [<entity>] [<animation>]", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let anim = h::arg_tag(entry, 1, ctx);
        intent("animate", &target, &anim);
        Ok(CommandOutcome::Continue)
    });

    register("age", "age (entity:<entity>) (amount:<#>) (lock:<true/false>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let amount = h::prefixed(entry, "amount", ctx).unwrap_or_default();
        intent("age", &target, &amount);
        Ok(CommandOutcome::Continue)
    });

    register("cast", "cast [<entity>] [<spell>] (power:<#>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let spell = h::arg_tag(entry, 1, ctx);
        intent("cast", &target, &spell);
        Ok(CommandOutcome::Continue)
    });

    register("head", "head [<entity>] (yaw:<#>) (pitch:<#>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let yaw = h::prefixed(entry, "yaw", ctx).unwrap_or_default();
        intent("head", &target, &yaw);
        Ok(CommandOutcome::Continue)
    });

    register("tame", "tame (entity:<entity>) (owner:<entity>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let owner = h::prefixed(entry, "owner", ctx).unwrap_or_default();
        intent("tame", &target, &owner);
        Ok(CommandOutcome::Continue)
    });

    register("disguise", "disguise (entity:<entity>) (disguise:<entity>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        let disguise = h::prefixed(entry, "disguise", ctx).unwrap_or_default();
        intent("disguise", &target, &disguise);
        Ok(CommandOutcome::Continue)
    });

    register("undisguise", "undisguise (entity:<entity>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        intent("undisguise", &target, "");
        Ok(CommandOutcome::Continue)
    });

    register("fakespawn", "fakespawn [<entity>] (location:<location>) (to:<player>)", |entry, ctx, _q| {
        let entity = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("fakespawn", &entity, &loc);
        Ok(CommandOutcome::Continue)
    });

    register("fakeinternaldata", "fakeinternaldata (entity:<entity>) (data:<map>)", |entry, ctx, _q| {
        let target = resolve_target(entry, ctx);
        intent("fakeinternaldata", &target, "");
        Ok(CommandOutcome::Continue)
    });

    #[allow(unused)]
    let _ = ElementTag::new("");
}
