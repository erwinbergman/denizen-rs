//! The event system: script events matched against server occurrences.

use crate::obj::ObjectTag;
use crate::scripts::entry::ScriptEntry;
use crate::tags::TagContext;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// A fired server event instance.
#[derive(Clone)]
pub struct ScriptEvent {
    pub name: String,
    pub context: HashMap<String, ObjectTag>,
    /// Switches supplied by the firing side (eg `type:zombie`).
    pub switches: HashMap<String, String>,
    /// Linked player/npc objects, if any.
    pub player: Option<ObjectTag>,
    pub npc: Option<ObjectTag>,
}

impl ScriptEvent {
    pub fn new(name: &str) -> Self {
        ScriptEvent {
            name: name.to_lowercase(),
            context: HashMap::new(),
            switches: HashMap::new(),
            player: None,
            npc: None,
        }
    }

    pub fn with(mut self, key: &str, value: ObjectTag) -> Self {
        self.context.insert(key.to_lowercase(), value);
        self
    }

    pub fn with_switch(mut self, key: &str, value: &str) -> Self {
        self.switches.insert(key.to_lowercase(), value.to_string());
        self
    }

    pub fn with_player(mut self, player: ObjectTag) -> Self {
        self.player = Some(player);
        self
    }

    pub fn with_npc(mut self, npc: ObjectTag) -> Self {
        self.npc = Some(npc);
        self
    }
}

/// A registered event definition (name + optional label matcher + switches).
#[derive(Clone)]
pub struct EventInfo {
    pub name: String,
    /// The full event label (eg `<entity> damaged (by <'cause'>)`), if registered from Java metadata.
    pub matcher: Option<String>,
    pub switches: Vec<String>,
}

static EVENT_INFOS: LazyLock<RwLock<HashMap<String, EventInfo>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn register_event(name: &str, switches: &[&str]) {
    let key = name.to_lowercase();
    EVENT_INFOS.write().unwrap().insert(
        key.clone(),
        EventInfo {
            name: key,
            matcher: None,
            switches: switches.iter().map(|s| s.to_string()).collect(),
        },
    );
}

/// Register an event with its full label matcher (used for advanced matching).
pub fn register_event_with_matcher(name: &str, matcher: &str, switches: &[&str]) {
    let key = name.to_lowercase();
    EVENT_INFOS.write().unwrap().insert(
        key.clone(),
        EventInfo {
            name: key,
            matcher: Some(matcher.to_string()),
            switches: switches.iter().map(|s| s.to_string()).collect(),
        },
    );
}

pub fn event_exists(name: &str) -> bool {
    EVENT_INFOS.read().unwrap().contains_key(&name.to_lowercase())
}

pub fn get_event(name: &str) -> Option<EventInfo> {
    EVENT_INFOS.read().unwrap().get(&name.to_lowercase()).cloned()
}

pub fn event_names() -> Vec<String> {
    let mut v: Vec<String> = EVENT_INFOS.read().unwrap().keys().cloned().collect();
    v.sort();
    v
}

pub fn event_count() -> usize {
    EVENT_INFOS.read().unwrap().len()
}

/// An event context source for `<context[...]>` lookups during event firing.
pub struct EventContextSource {
    pub context: HashMap<String, ObjectTag>,
}

impl crate::tags::ContextSource for EventContextSource {
    fn get_context(&self, name: &str) -> Option<ObjectTag> {
        self.context.get(&name.to_lowercase()).cloned()
    }
}

/// A parsed event line from a script container (`events:` section).
#[derive(Clone, Debug)]
pub struct EventLine {
    /// The base event name (eg "entity damaged").
    pub name: String,
    /// Switches given in the event line, eg `type:zombie` -> ("type", "zombie").
    pub switches: HashMap<String, String>,
    /// Original line text.
    pub raw: String,
}

/// Parse an event line like `player breaks block` or `delta time every:5`.
pub fn parse_event_line(line: &str) -> EventLine {
    let raw = line.trim().to_string();
    let mut switches = HashMap::new();
    let mut name_parts = Vec::new();
    for part in raw.split_whitespace() {
        if let Some(colon) = part.find(':') {
            let key = part[..colon].to_lowercase();
            // Only treat as switch if the key is a plain identifier.
            if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                let value = part[colon + 1..].to_string();
                switches.insert(key, value);
                continue;
            }
        }
        name_parts.push(part);
    }
    EventLine {
        name: name_parts.join(" ").to_lowercase(),
        switches,
        raw,
    }
}

/// Fire an event: find matching event containers and run their scripts.
pub fn fire_event(event: &ScriptEvent) -> usize {
    let mut fired = 0;
    for container in crate::scripts::registry::all_containers() {
        let event_names = container.event_names();
        if event_names.is_empty() {
            continue;
        }
        let mut matched = false;
        for line in &event_names {
            let parsed = parse_event_line(line);
            if !event_name_matches(&parsed.name, &event.name) {
                continue;
            }
            if !switches_match(&parsed.switches, &event.switches) {
                continue;
            }
            matched = true;
            break;
        }
        if !matched {
            continue;
        }
        if let Some(entries) = container.get_entries("script") {
            let ctx_source = Arc::new(EventContextSource { context: event.context.clone() });
            let mut defs = crate::obj::MapTag::new();
            for (k, v) in &event.context {
                defs.put_string(k, v.clone());
            }
            let mut queue = crate::scripts::queue::ScriptQueue::new("");
            queue.context_source = Some(ctx_source);
            queue.definitions = defs;
            if let Some(p) = &event.player {
                queue.set_definition("__player", Some(p.clone()));
            }
            if let Some(n) = &event.npc {
                queue.set_definition("__npc", Some(n.clone()));
            }
            for e in entries.iter() {
                let mut entry = e.clone();
                if let Some(p) = &event.player {
                    entry.entry_data.player = Some(p.clone());
                }
                if let Some(n) = &event.npc {
                    entry.entry_data.npc = Some(n.clone());
                }
                queue.add_entry(entry);
            }
            let arc = crate::scripts::queue::register_queue(queue);
            fired += crate::scripts::queue::run_queue(&arc);
        }
    }
    fired
}

/// Fire an event by name with no context.
pub fn fire(name: &str) -> usize {
    fire_event(&ScriptEvent::new(name))
}

fn switches_match(required: &HashMap<String, String>, provided: &HashMap<String, String>) -> bool {
    for (k, v) in required {
        match provided.get(k) {
            Some(pv) if pv.eq_ignore_ascii_case(v) => {}
            _ => return false,
        }
    }
    true
}

fn event_name_matches(script_event: &str, fired: &str) -> bool {
    let script_event = script_event.trim().to_lowercase();
    let fired = fired.trim().to_lowercase();
    if script_event == fired {
        return true;
    }
    // If the registered event has a matcher, try matching against the full label.
    if let Some(info) = get_event(&fired) {
        if let Some(matcher) = &info.matcher {
            if matcher_matches(&matcher.to_lowercase(), &script_event) {
                return true;
            }
        }
    }
    // Try matching a script event label against a registered matcher.
    if let Some(info) = get_event(&script_event) {
        if let Some(matcher) = &info.matcher {
            if matcher_matches(&matcher.to_lowercase(), &fired) {
                return true;
            }
        }
    }
    // Prefix match: allow "entity damaged" to match "entity damaged by entity".
    if fired.starts_with(&script_event) || script_event.starts_with(&fired) {
        // Only when the boundary is a space.
        let a = fired.strip_prefix(&script_event).map(|r| r.starts_with(' ')).unwrap_or(false);
        let b = script_event.strip_prefix(&fired).map(|r| r.starts_with(' ')).unwrap_or(false);
        if a || b {
            return true;
        }
    }
    false
}

/// Match a Denizen event label (with `<type>` placeholders) against a concrete fired name.
fn matcher_matches(matcher: &str, concrete: &str) -> bool {
    // Strip optional parenthetical groups from the matcher.
    let mut base = String::new();
    let mut depth = 0;
    for c in matcher.chars() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => base.push(c),
            _ => {}
        }
    }
    let base = base.trim();
    // Remove angle-bracket placeholders, collapsing whitespace.
    let cleaned: String = {
        let mut out = String::new();
        let mut in_bracket = false;
        for c in base.chars() {
            if c == '<' {
                in_bracket = true;
                out.push(' ');
            } else if c == '>' {
                in_bracket = false;
                out.push(' ');
            } else if !in_bracket {
                out.push(c);
            }
        }
        out.split_whitespace().collect::<Vec<_>>().join(" ")
    };
    if cleaned.is_empty() {
        return true;
    }
    // Allow simple prefix matching of the static words.
    let words: Vec<&str> = cleaned.split(' ').collect();
    let cwords: Vec<&str> = concrete.split_whitespace().collect();
    let mut ci = 0;
    for w in words {
        if w.is_empty() {
            continue;
        }
        // Skip to the next matching word in the concrete name.
        let mut found = false;
        while ci < cwords.len() {
            if cwords[ci] == w {
                found = true;
                ci += 1;
                break;
            }
            ci += 1;
        }
        if !found {
            return false;
        }
    }
    true
}

/// Register every Denizen event name (and full label matcher where known).
pub fn register_core_events() {
    // Full event labels (Java metadata: registerCouldMatcher).
    let matchers: &[&str] = &[
        "area effect cloud applies",
        "bell rings",
        "<block> being built (on <block>)",
        "<block> burns",
        "<block> cooks|smelts <item> (into <item>)",
        "<block> decay",
        "<block> destroyed by explosion",
        "<block> dispenses <item>",
        "block equips <item>",
        "<block> explodes",
        "<block> extends",
        "<block> fades",
        "<block> falls",
        "<block> forms",
        "<block> grows",
        "block ignites",
        "<block> physics",
        "<block> retracts",
        "<block> shears <entity>",
        "<block> spreads",
        "block spreads",
        "brewing stand brews",
        "brewing stand fueled (with <item>)",
        "brewing starts",
        "cauldron level changes|raises|lowers",
        "chunk loads entities",
        "chunk loads (for the first time)",
        "chunk unloads",
        "chunk unloads entities",
        "command",
        "<'command_name'> command",
        "crafter crafts <item>",
        "creeper powered (because <'cause'>)",
        "dragon egg moves",
        "<entity> breaks <hanging> (because <'cause'>)",
        "<entity> breeds",
        "<entity> casts <'spell'>",
        "<entity> changes air level",
        "<entity> changes <block> (into <block>)",
        "<entity> changes food level",
        "<entity> changes phase",
        "<entity> changes pose",
        "<entity> combusts",
        "<entity> creates portal",
        "<entity> damaged (by <'cause'>)",
        "<entity> damaged by <entity>",
        "<entity> damages <entity>",
        "<entity> damages <vehicle>",
        "<entity> despawns",
        "<entity> dies|death",
        "<entity> drops <item>",
        "<entity> enters <entity>",
        "<entity> enters|exits <area>",
        "<entity> enters portal",
        "<entity> exits <entity>",
        "<entity> exits portal",
        "<entity> explodes",
        "<entity> explosion primes",
        "<entity> forms <block>",
        "<entity> goes into <block>",
        "<entity> heals (because <'cause'>)",
        "<entity> interacts with <block>",
        "<entity> jumps",
        "<entity> killed (by <'cause'>)",
        "<entity> killed (by <entity>)",
        "<entity> kills <entity>",
        "<entity> launched",
        "<entity> picks up <item>",
        "<entity> potion effects <'change_action'>",
        "<entity> potion effects modified",
        "<entity> resurrected",
        "<entity> shoots <block> (with <projectile>)",
        "<entity> shoots <item>",
        "<entity> spawns (because <'cause'>)",
        "<entity> takes <item>",
        "<entity> tamed",
        "<entity> targets (<entity>) (because <'cause'>)",
        "<entity> toggles|starts|stops gliding",
        "<entity> toggles|starts|stops swimming",
        "<entity> unleashed (because <'reason'>)",
        "farmland moisture level changes",
        "furnace burns <item>",
        "furnace starts smelting <item>",
        "generic game event",
        "<hanging> breaks (because <'cause'>)",
        "internal bukkit event",
        "<inventory> picks up <item>",
        "<item> despawns",
        "<item> enchanted",
        "<item> merges",
        "<item> moves from <inventory> (to <inventory>)",
        "<item> recipe formed",
        "<item> spawns",
        "<item> splash|splashes",
        "leaves decay",
        "lightning strikes",
        "lingering <item> splash|splashes",
        "liquid|lava|water level changes",
        "liquid spreads",
        "loot dispenses from <block>",
        "loot generates",
        "noteblock plays note",
        "npc begins|completes|cancels navigation",
        "npc opens <block>",
        "npc spawns",
        "npc stuck",
        "piglin barter",
        "pig zapped",
        "piston extends",
        "piston retracts",
        "player animates (<'animation'>)",
        "player changes armor stand item",
        "player changes gamemode (to <'gamemode'>)",
        "player changes main hand",
        "player changes world (from <world>) (to <world>)",
        "player (<'click_type'>) clicks (<item>) in <inventory>",
        "player damages <block>",
        "player discovers recipe",
        "player dyes sheep (<'color'>)",
        "player edits book",
        "player empties bucket",
        "player empties <item>",
        "player enters|exits <biome>",
        "player exhaustion level increases",
        "player fills bucket",
        "player fills <item>",
        "player fishes (<entity>) (while <'state'>)",
        "player fishes (<item>) (while <'state'>)",
        "player hears sound",
        "player input",
        "player <item> takes damage",
        "player jumps",
        "player kicked (for flying)",
        "player levels up (from <'level'>) (to <'level'>)",
        "player mends <item>",
        "player places <hanging>",
        "player places <material>",
        "player quits|quit",
        "player raises|lowers|toggles <item>",
        "player receives packet",
        "player receives tablist update",
        "player respawns (at bed)",
        "player respawns elsewhere",
        "player right clicks <entity>",
        "player (right|left) clicks <block>",
        "player sends packet",
        "player shears <'color'> sheep",
        "player shears <entity>",
        "player signs book",
        "player statistic increments",
        "player statistic <'statistic'> increments",
        "player steps on <block>",
        "player stops damaging <block>",
        "player tames <entity>",
        "player throws (hatching|non-hatching) egg",
        "player toggles|starts|stops sneaking",
        "player toggles|starts|stops sprinting",
        "player triggers raid",
        "player uses portal",
        "player walks over <'location'>",
        "player walks over notable",
        "portal created (because <'reason'>)",
        "<projectile> hits (<block>)",
        "<projectile> hits <entity>",
        "raid finishes",
        "raid spawns wave",
        "raid stops",
        "redstone recalculated",
        "resource pack status",
        "server list ping",
        "server prestart",
        "server start",
        "sheep dyed (<'color'>)",
        "slime splits (into <'#'>)",
        "spawn changes",
        "spawner spawns <entity>",
        "sponge absorbs water",
        "<'structure/plant'> grows from bonemeal",
        "<'structure/plant'> grows (naturally)",
        "tab complete",
        "thunder changes|begins|clears",
        "time <'0-23'> (in <world>)",
        "time changes (in <world>)",
        "tnt primes",
        "vault displays <item>",
        "<vehicle> collides with <entity>",
        "<vehicle> damaged",
        "villager replenishes trade",
        "weather changes|rains|clears (in <world>)",
        "<world> initializes",
        "<world> loads",
        "<world> saves",
        "<world> unloads",

    ];
    for m in matchers {
        let canonical = canonical_event_name(m);
        if !canonical.is_empty() {
            register_event_with_matcher(&canonical, m, &[]);
        }
        // Also register the raw lowercase form for direct name matching.
        let raw = m.to_lowercase();
        if raw != canonical {
            register_event(&raw, &[]);
        }
    }

    // Plain event names (no matcher metadata): engine lifecycle, legacy names, and
    // additional events that fire by simple name.
    let names: &[&str] = &[
        "server start",
        "server shutdown",
        "server tick",
        "system time",
        "delta time",
        "scripts loaded",
        "reload scripts",
        "custom",
        "console output",
        "exception",
        "player joins",
        "player quits",
        "player chats",
        "player breaks block",
        "player places block",
        "player right clicks block",
        "player left clicks block",
        "player dies",
        "player respawns",
        "player moves",
        "player teleports",
        "player takes damage",
        "player changes world",
        "player changes gamemode",
        "player changes main hand",
        "player sneaks",
        "player unsneaks",
        "player sprint toggles",
        "player flight toggles",
        "player drops item",
        "player picks up item",
        "player crafts item",
        "player enchants item",
        "player trades",
        "player level changes",
        "player experience changes",
        "player health changes",
        "player food changes",
        "player opens inventory",
        "player closes inventory",
        "player clicks inventory",
        "player drags inventory",
        "player right clicks entity",
        "player left clicks entity",
        "player right clicks",
        "player left clicks",
        "player equips armor",
        "player unequips armor",
        "player swaps item",
        "player consumes item",
        "player uses item",
        "player shoots bow",
        "player shoots",
        "player kills entity",
        "entity dies",
        "entity spawns",
        "entity damaged",
        "entity damaged by entity",
        "entity kills entity",
        "entity target",
        "entity enters area",
        "entity exits area",
        "player enters area",
        "player exits area",
        "block breaks",
        "block places",
        "block physics",
        "block burns",
        "block fades",
        "block ignites",
        "block forms",
        "block grows",
        "block decays",
        "block spreads",
        "inventory clicks",
        "inventory opens",
        "inventory closes",
        "item is crafted",
        "item is smelted",
        "item spawns",
        "item despawns",
        "item merges",
        "world loads",
        "world unloads",
        "world saves",
        "world initializes",
        "weather changes",
        "thunder changes",
        "time changes",
        "vehicle moves",
        "vehicle damaged",
        "npc is spawned",
        "npc is despawned",
        "npc assignment",
        "npc is pushed",
        "npc is clicked",
        "npc is looked at",
        "npc navigates",
        "npc stuck",
        "npc spawns",
        "npc despawns",
        "npc opens",
        "npc collides",
        "chunk loads",
        "chunk unloads",
        "potion effect applied",
        "server list ping",
        "tab complete",
        "command",
    ];
    for n in names {
        if !event_exists(n) {
            register_event(n, &[]);
        }
    }
}

/// Produce a canonical lookup name from a matcher by removing placeholders,
/// optional groups, and alternatives.
fn canonical_event_name(matcher: &str) -> String {
    let mut out = String::new();
    let mut in_bracket = false;
    let mut depth = 0;
    for c in matcher.chars() {
        match c {
            '<' => {
                in_bracket = true;
                out.push(' ');
            }
            '>' => {
                in_bracket = false;
                out.push(' ');
            }
            '(' if !in_bracket => depth += 1,
            ')' if !in_bracket => {
                if depth > 0 {
                    depth -= 1;
                }
                out.push(' ');
            }
            _ if in_bracket || depth > 0 => {}
            '|' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Clear all registered events.
pub fn clear_events() {
    EVENT_INFOS.write().unwrap().clear();
}

#[allow(dead_code)]
fn _unused(_e: ScriptEntry, _c: TagContext) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_registered() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        assert!(event_count() > 150);
        assert!(event_exists("player joins"));
        assert!(event_exists("entity damaged"));
        assert!(event_exists("server start"));
    }

    #[test]
    fn event_line_parsing() {
        let _g = crate::scripts::test_guard();
        let e = parse_event_line("delta time every:5");
        assert_eq!(e.name, "delta time");
        assert_eq!(e.switches.get("every").map(|s| s.as_str()), Some("5"));
    }

    #[test]
    fn name_matching() {
        let _g = crate::scripts::test_guard();
        assert!(event_name_matches("entity damaged", "entity damaged"));
        assert!(event_name_matches("entity damaged", "entity damaged by entity"));
        assert!(!event_name_matches("player joins", "player quits"));
    }

    #[test]
    fn fires_container() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        crate::scripts::registry::clear_containers();
        let parsed = crate::util::yaml::parse(
            "on join:\n  type: world\n  events:\n  - player joins\n  script:\n  - define fired yes\n",
        );
        if let crate::util::yaml::Yaml::Map(entries) = parsed {
            for (name, body) in entries {
                let c = crate::scripts::container::ScriptContainer::new(&name, crate::scripts::container::ScriptContainerKind::World, body);
                crate::scripts::registry::register_container(c);
            }
        }
        let n = fire("player joins");
        assert!(n > 0);
        crate::scripts::registry::clear_containers();
    }
}
