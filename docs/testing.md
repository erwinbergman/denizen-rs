# testing

everything runs through cargo.

## run the suite

```bash
cd denizen-rs
cargo test
```

## unit tests

unit tests sit inline in the source under `#[cfg(test)] mod tests`. they cover things like:

- md5 against known vectors, roman numerals and number formatting
- yaml mapping, list, nested and flow parsing, plus round trips
- object registration, tag dispatch and identify round trips
- script, command, event and flag parsing behaviour

to run only the unit tests:

```bash
cargo test --lib
```

## integration tests

`crates/denizen-core/tests/integration.rs` drives the engine through its public api:

- engine init and stats
- how many commands and events are registered
- object type registration
- tag parsing such as `<element[hi].to_uppercase>`, arithmetic and definitions
- list, map, duration and vector behaviour
- script runs with if, while, foreach and procedures that determine a value
- flags, both directly and through the `flag` command
- firing an event from a world script
- yaml parse and round trip

to run only the integration tests:

```bash
cargo test --test integration
```

## a note on global state

the engine keeps global registries for commands, events, object types, script containers and
flags. tests that register containers share that state, so registration is idempotent and
thread safe. the integration tests tolerate shared state and check for existence rather than
exact global counts where it matters.

## manual jni smoke test

```bash
cargo build --release -p denizen-jni
# compile a small java class that calls NativeBridge methods and run it with:
java -Djava.library.path=target/release YourTest
```

expected values: `nativeVersion()` returns `1.3.4`, `nativePing()` returns `42`,
`nativeCommandCount()` is around 189 and `nativeEventCount()` is around 404.
