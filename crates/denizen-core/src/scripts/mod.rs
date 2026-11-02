//! Script containers, registry, entries, queues and the script execution engine.

pub mod container;
pub mod registry;
pub mod entry;
pub mod queue;
pub mod builder;
pub mod helper;

pub use container::{ScriptContainer, ScriptContainerKind};
pub use entry::{ScriptEntry, ScriptEntryData};
pub use queue::ScriptQueue;

/// Serializes engine initialization across parallel tests, since `engine::init`
/// briefly clears the global command registry.
#[cfg(test)]
pub fn test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    match LOCK.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}
