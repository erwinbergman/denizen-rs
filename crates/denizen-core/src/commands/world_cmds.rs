//! World commands: weather, time, playsound, playeffect, explode, firework, strike,
//! gamerule, modifyblock, copyblock, createworld, worldborder, and more.

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
    register("weather", "weather [<type>] (world:<world>) (duration:<duration>)", |entry, ctx, _q| {
        let kind = h::arg_tag(entry, 0, ctx);
        let world = h::prefixed(entry, "world", ctx).unwrap_or_default();
        intent("weather", &format!("{} {}", kind, world));
        Ok(CommandOutcome::Continue)
    });

    register("time", "time [<time>] (world:<world>)", |entry, ctx, _q| {
        let time = h::arg_tag(entry, 0, ctx);
        let world = h::prefixed(entry, "world", ctx).unwrap_or_default();
        intent("time", &format!("{} {}", time, world));
        Ok(CommandOutcome::Continue)
    });

    register("playsound", "playsound [<sound>] (location:<location>) (volume:<#>) (pitch:<#>) (targets:<player>|...)", |entry, ctx, _q| {
        let sound = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        let volume = h::prefixed(entry, "volume", ctx).unwrap_or_else(|| "1".to_string());
        intent("playsound", &format!("{} at:{} vol:{}", sound, loc, volume));
        Ok(CommandOutcome::Continue)
    });

    register("play_sound", "play_sound [<sound>] (location:<location>) (volume:<#>) (pitch:<#>)", |entry, ctx, _q| {
        let sound = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("playsound", &format!("{} at:{}", sound, loc));
        Ok(CommandOutcome::Continue)
    });

    register("playeffect", "playeffect [<effect>] (location:<location>) (data:<#>) (speed:<#>) (count:<#>)", |entry, ctx, _q| {
        let effect = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("playeffect", &format!("{} at:{}", effect, loc));
        Ok(CommandOutcome::Continue)
    });

    register("particle", "particle [<effect>] (location:<location>) (count:<#>) (speed:<#>)", |entry, ctx, _q| {
        let effect = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("particle", &format!("{} at:{}", effect, loc));
        Ok(CommandOutcome::Continue)
    });

    register("firework", "firework (location:<location>) (power:<#>) (type:<type>) (flicker) (trail)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        intent("firework", &loc);
        Ok(CommandOutcome::Continue)
    });

    register("strike", "strike (location:<location>) (no_damage)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        intent("strike", &loc);
        Ok(CommandOutcome::Continue)
    });

    register("gamerule", "gamerule [<rule>] [<value>] (world:<world>)", |entry, ctx, _q| {
        let rule = h::arg_tag(entry, 0, ctx);
        let value = h::arg_tag(entry, 1, ctx);
        intent("gamerule", &format!("{} {}", rule, value));
        Ok(CommandOutcome::Continue)
    });

    register("modifyblock", "modifyblock [<location>] [<material>] (delayed) (physics)", |entry, ctx, _q| {
        let loc = h::arg_tag(entry, 0, ctx);
        let mat = h::arg_tag(entry, 1, ctx);
        intent("modifyblock", &format!("{} -> {}", loc, mat));
        Ok(CommandOutcome::Continue)
    });

    register("copyblock", "copyblock [<location>] [<location>]", |entry, ctx, _q| {
        let from = h::arg_tag(entry, 0, ctx);
        let to = h::arg_tag(entry, 1, ctx);
        intent("copyblock", &format!("{} -> {}", from, to));
        Ok(CommandOutcome::Continue)
    });

    register("adjustblock", "adjustblock [<location>] [<mechanism>] [<value>]", |entry, ctx, _q| {
        let loc = h::arg_tag(entry, 0, ctx);
        let mech = h::arg_tag(entry, 1, ctx);
        let value = h::tag_rest(entry, 2, ctx);
        intent("adjustblock", &format!("{} {} {}", loc, mech, value));
        Ok(CommandOutcome::Continue)
    });

    register("createworld", "createworld [<name>] (generator:<generator>) (type:<type>)", |entry, ctx, _q| {
        let name = h::arg_tag(entry, 0, ctx);
        intent("createworld", &name);
        Ok(CommandOutcome::Continue)
    });

    register("chunkload", "chunkload [<location>] (generate:<true/false>)", |entry, ctx, _q| {
        let loc = h::arg_tag(entry, 0, ctx);
        intent("chunkload", &loc);
        Ok(CommandOutcome::Continue)
    });

    register("light", "light (location:<location>) (level:<#>)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        let level = h::prefixed(entry, "level", ctx).unwrap_or_default();
        intent("light", &format!("{} {}", loc, level));
        Ok(CommandOutcome::Continue)
    });

    register("worldborder", "worldborder [<action>] (radius:<#>) (duration:<duration>) (world:<world>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("worldborder", &action);
        Ok(CommandOutcome::Continue)
    });

    register("drop", "drop [<item>] (location:<location>) (speed:<#>)", |entry, ctx, _q| {
        let item = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("drop", &format!("{} at:{}", item, loc));
        Ok(CommandOutcome::Continue)
    });

    register("sign", "sign (location:<location>) (line:<#>) (text:<text>)", |entry, ctx, _q| {
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        intent("sign", &loc);
        Ok(CommandOutcome::Continue)
    });

    register("schematic", "schematic [<action>] [<name>] (location:<location>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let name = h::arg_tag(entry, 1, ctx);
        intent("schematic", &format!("{} {}", action, name));
        Ok(CommandOutcome::Continue)
    });

    register("animatechest", "animatechest [<location>] (open:<true/false>)", |entry, ctx, _q| {
        let loc = h::arg_tag(entry, 0, ctx);
        let open = h::prefixed(entry, "open", ctx).unwrap_or_else(|| "true".to_string());
        intent("animatechest", &format!("{} {}", loc, open));
        Ok(CommandOutcome::Continue)
    });

    register("midi", "midi [<action>] (location:<location>) (file:<file>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("midi", &action);
        Ok(CommandOutcome::Continue)
    });

    register("tick", "tick [<action>] (amount:<#>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("tick", &action);
        Ok(CommandOutcome::Continue)
    });

    register("scoreboard", "scoreboard [<action>] (objective:<name>) (name:<name>) (value:<#>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("scoreboard", &action);
        Ok(CommandOutcome::Continue)
    });
}
