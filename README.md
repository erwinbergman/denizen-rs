# denizen-rs

a native rust implementation of the denizen scripting engine for minecraft.

the whole engine is compiled into a native shared library (`libdenizen.so`, `denizen.dll` or
`libdenizen.dylib`). a small java plugin loads that library through jni and hands everything
off to it. all of the actual work, the object and tag system, the script engine, commands,
events and flag storage, happens in rust. the java side is just a loader.

## layout

```
denizen-rs/
├── Cargo.toml                 # cargo workspace
├── crates/
│   ├── denizen-core/          # the engine itself
│   │   ├── src/obj/           # object system and object types
│   │   ├── src/tags/          # tag parsing and base tags
│   │   ├── src/scripts/       # containers, entries, queues
│   │   ├── src/commands/      # script commands
│   │   ├── src/events/        # event definitions and firing
│   │   ├── src/flags/         # per object flag storage
│   │   ├── src/util/          # yaml parser, logging, helpers
│   │   └── src/engine.rs      # init, reload, stats, host hooks
│   └── denizen-jni/           # cdylib that exposes the engine over jni
├── java-shim/                 # the bukkit plugin
├── docs/                      # build, testing and status notes
└── build.sh                   # builds the library and stages it for the plugin
```

## what is here

| area | notes |
|------|-------|
| object system | 29 object types |
| tags | about 450 registered tags |
| object types | element, list, map, duration, vector, color, time, script, queue, binary, secret, custom, image, location, entity, player, npc, world, item, material, inventory, enchantment, cuboid, biome, chunk, plugin, trade, ellipsoid, polygon |
| script containers | task, world, procedure, format, data, custom, command, economy, item, inventory, book, map |
| script engine | queues, if and else, while, foreach, choose, switch, goto, procedures |
| commands | 189 |
| events | 404 |
| flags | persistent, with expiry |
| yaml | native parser for script files |

see `docs/status.md` for a more detailed breakdown and `docs/build.md` for how to build.

## building

```bash
./build.sh               # builds target/release/libdenizen.so and stages it for the plugin
cargo test               # runs the test suite
```

you only need a rust toolchain. gradle is not required to build or test the engine.

## how the java side works

`java-shim/` is a regular bukkit plugin. when it enables it loads the native library and calls
`nativeInit()`. after that, tag parsing, script reloads, command runs, event firing and flag
lookups are all native calls into the rust engine. the java code is intentionally small.

## license

mit. see `LICENSE.txt`. the original denizen project is mit licensed by the denizen script
team, and this work follows the same terms.
