# status

what is implemented right now.

## numbers

| metric | value |
|--------|-------|
| rust source | around 16,600 lines over about 50 modules |
| java plugin | around 320 lines |
| object types | 29 |
| registered tags | about 450 |
| script commands | 189 |
| script events | 404 |
| native library | `libdenizen.so`, about 4 mb |
| tests | unit and integration, all passing |

## object system

the base unit is `ObjectTag`, which wraps an `Arc<dyn ObjectTagImpl>`. every type has a prefix,
an identify form, an object type name and a step down to the next type. tags and mechanisms for
a type register into an `ObjectTagProcessor`.

object types: `ElementTag`, `ListTag`, `MapTag`, `DurationTag`, `VectorObject`, `ColorTag`,
`TimeTag`, `ScriptTag`, `QueueTag`, `BinaryTag`, `SecretTag`, `CustomObjectTag`, `ImageTag`,
`LocationTag`, `EntityTag`, `PlayerTag`, `NPCTag`, `WorldTag`, `ItemTag`, `MaterialTag`,
`InventoryTag`, `EnchantmentTag`, `CuboidTag`, `BiomeTag`, `ChunkTag`, `PluginTag`, `TradeTag`,
`EllipsoidTag`, `PolygonTag`.

## tag engine

- `Attribute` and `AttributeComponent` handle dot separated chains with `[param]` brackets,
  number aware dot splitting so `1.5` stays intact, and fallbacks such as `if_null`, `exists`
  and `is_truthy`
- `TagContext` carries a `DefinitionProvider` and a `ContextSource`
- definitions through `<[...]>`, `<def[...]>` and `<definition[...]>`
- tag text parsing handles nested tags and merges adjacent plain text
- base tags like `<element[...]>`, `<list[...]>`, `<map[...]>`, `<location[...]>`,
  `<item[...]>`, `<cuboid[...]>`, `<biome[...]>`, `<vector[...]>` and `<ternary[...]>`, plus
  math tags and the per object tags

## script engine

- a yaml parser for block maps, sequences, flow lists, quoted scalars and block scalars
- comment stripping for script files
- script containers of every kind and a global registry
- entry building, including nested blocks such as `- if <cond>:` with indented bodies
- queues with definitions, context sources, wait and resume, procedure results and scheduling
- control flow: if and else, while, foreach, repeat, choose and case, switch, goto and mark,
  wait and waituntil, stop, run and inject, define and definemap, determine, ratelimit, queue

## commands

the command families cover queue control, core commands, entity, player, world, npc, item,
server, data, web and interact. examples include spawn, kill, hurt, heal, feed, burn, explode,
teleport, mount, sneak, look, walk, follow, equip, gamemode, fly, give, take, inventory,
experience, actionbar, chat, weather, time, playsound, particle, create, despawn, narrate,
debug, adjust, note, yaml, file, log, execute, zap, cooldown, reset and flag.

where a live server would be needed the command records the intent through the host hook. a
future bukkit host can turn those hooks into real effects.

## events

all of the event names from the java engine plus lifecycle and simple names, with switch
parsing, context sources and player and npc linking. `fire_event` matches world containers by
event name, ignoring switches, and runs their scripts.

## flags

per object flag trackers for players, entities, npcs, locations, worlds, inventories, items and
the server. set, get, remove and list, optional expiry, and save and load to disk. exposed
through the `flag` command, through flag tags and through the native flag calls.

## java plugin

`Denizen.java` is the plugin, `NativeBridge.java` holds the native declarations,
`NativeLoader.java` extracts and loads the library, and `plugin.yml` describes the plugin.
enabling loads the library, initialises the engine and reloads scripts. `/denizen` and `/ex`
go straight to native calls.

## known gaps

- live world effects like real block changes, fake entities over packets, map rendering and
  full nms interop are recorded as intents instead of direct bukkit calls, since the engine is
  kept separate from bukkit
- citizens and npc traits exist at the object level but are not bound to the citizens plugin
- network protocols, mongodb, redis and the web server are stubbed at the command level
