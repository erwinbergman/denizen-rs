//! Item commands: give, take, inventory, equip, displayitem, fakeitem, map, and more.

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
    register("give", "give [<item>] (quantity:<#>) (player:<player>) (slot:<slot>)", |entry, ctx, _q| {
        let item = h::arg_tag(entry, 0, ctx);
        let qty = h::prefixed(entry, "quantity", ctx).unwrap_or_else(|| "1".to_string());
        let target = h::prefixed(entry, "player", ctx)
            .or_else(|| h::linked_player(ctx).map(|p| p.identify()))
            .unwrap_or_default();
        intent("give", &format!("{} x{} to:{}", item, qty, target));
        Ok(CommandOutcome::Continue)
    });

    register("take", "take [<item>] (quantity:<#>) (player:<player>)", |entry, ctx, _q| {
        let item = h::arg_tag(entry, 0, ctx);
        let qty = h::prefixed(entry, "quantity", ctx).unwrap_or_else(|| "1".to_string());
        let target = h::prefixed(entry, "player", ctx)
            .or_else(|| h::linked_player(ctx).map(|p| p.identify()))
            .unwrap_or_default();
        intent("take", &format!("{} x{} from:{}", item, qty, target));
        Ok(CommandOutcome::Continue)
    });

    register("inventory", "inventory [<action>] (id:<id>) (slot:<slot>) (item:<item>) (player:<player>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        let id = h::prefixed(entry, "id", ctx).unwrap_or_else(|| "inventory".to_string());
        let slot = h::prefixed(entry, "slot", ctx).unwrap_or_default();
        let item = h::prefixed(entry, "item", ctx).unwrap_or_default();
        intent("inventory", &format!("{} {} slot:{} item:{}", action, id, slot, item));
        Ok(CommandOutcome::Continue)
    });

    register("displayitem", "displayitem [<item>] (location:<location>)", |entry, ctx, _q| {
        let item = h::arg_tag(entry, 0, ctx);
        let loc = h::prefixed(entry, "location", ctx).unwrap_or_default();
        intent("displayitem", &format!("{} at:{}", item, loc));
        Ok(CommandOutcome::Continue)
    });

    register("fakeitem", "fakeitem [<item>] (player:<player>) (slot:<slot>)", |entry, ctx, _q| {
        let item = h::arg_tag(entry, 0, ctx);
        intent("fakeitem", &item);
        Ok(CommandOutcome::Continue)
    });

    register("map", "map [<action>] (id:<id>) (location:<location>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("map", &action);
        Ok(CommandOutcome::Continue)
    });

    register("book", "book [<action>] (id:<id>) (title:<title>) (author:<author>)", |entry, ctx, _q| {
        let action = h::arg_tag(entry, 0, ctx);
        intent("book", &action);
        Ok(CommandOutcome::Continue)
    });

    register("enchant", "enchant (item:<item>) (enchantment:<enchantment>) (level:<#>)", |entry, ctx, _q| {
        let item = h::prefixed(entry, "item", ctx).unwrap_or_else(|| h::arg_tag(entry, 0, ctx));
        let ench = h::prefixed(entry, "enchantment", ctx).unwrap_or_else(|| h::arg_tag(entry, 1, ctx));
        intent("enchant", &format!("{} {}", item, ench));
        Ok(CommandOutcome::Continue)
    });

}
