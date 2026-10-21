//! DenizenCore, the native engine behind denizen-rs.
//!
//! This crate contains the complete engine: the object/tag system, script containers and
//! queues, the command registry, the event system, flag storage, and the file/YAML layer.
//! It is exposed to Minecraft servers through the `denizen-jni` cdylib crate.

pub mod obj;
pub mod tags;
pub mod scripts;
pub mod commands;
pub mod events;
pub mod flags;
pub mod util;
pub mod engine;

pub use engine::*;

/// Version of the Rust engine. Mirrors the Denizen plugin version it targets.
pub const VERSION: &str = "1.3.4";

/// Version of the underlying core engine.
pub const CORE_VERSION: &str = "1.91.0";
