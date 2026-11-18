# java plugin

this is the java side of denizen-rs. it is a small bukkit plugin whose only job is to load the
native rust engine and pass calls to it.

everything else, the object and tag system, the script engine, commands, events and flags,
lives in rust under `../crates/denizen-core` and is compiled into a native library.

## files

- `Denizen.java` is the plugin entry point. it loads the library, initialises the engine,
  reloads scripts and handles the `/denizen` and `/ex` commands.
- `NativeBridge.java` holds the native method declarations that match the rust exports.
- `NativeLoader.java` extracts the bundled library from the jar to a temp file.
- `plugin.yml` is the bukkit plugin descriptor.

## building

build the native library first:

```bash
cd ..
./build.sh                 # builds target/release/libdenizen.so and stages it here
cd java-shim
gradle build               # only needed if you want the packaged jar
```

the gradle build puts the library under `natives/<os>-<arch>/` inside the jar.
