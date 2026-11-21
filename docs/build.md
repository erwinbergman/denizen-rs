# build

## requirements

- a rust toolchain, stable, edition 2021. `cargo` and `rustc` need to be on your path.
- only for the java plugin, a jdk 21 or newer and gradle. neither is needed to build or test
  the engine.

## build the native library

```bash
cd denizen-rs
cargo build --release -p denizen-jni
```

this produces the shared library:

| platform | artifact |
|----------|----------|
| linux    | `target/release/libdenizen.so` |
| macos    | `target/release/libdenizen.dylib` |
| windows  | `target/release/denizen.dll` |

there is also a helper script that builds the library and copies it into the plugin resources:

```bash
./build.sh              # release
./build.sh --debug      # debug
```

## tests

```bash
cargo test
```

the suite has unit tests inside the crate and integration tests in
`crates/denizen-core/tests/integration.rs` that drive the engine through its public api.

## packaging the plugin

packaging needs gradle, but the engine itself does not.

```bash
./build.sh                       # make sure the release library exists
cd java-shim
gradle build                     # produces build/libs/Denizen-1.3.4-native.jar
```

the gradle build puts the native library under `natives/<os>-<arch>/` inside the jar.
`NativeLoader` extracts it at runtime and `NativeBridge` loads it.

## checking the jni symbols

```bash
# after cargo build --release -p denizen-jni
java -Djava.library.path=target/release YourTestClass
```

the exported names all start with `Java_com_denizenscript_denizen_NativeBridge_native`.
you can list them with:

```bash
nm -D target/release/libdenizen.so | grep ' T Java_'
```
