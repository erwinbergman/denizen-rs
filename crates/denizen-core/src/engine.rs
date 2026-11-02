//! The engine facade: initialization, host implementation hooks, and the JNI-facing API surface.

use crate::commands::registry;
use crate::obj;
use crate::tags;
use std::sync::{Arc, LazyLock, Mutex};
use std::collections::HashMap;

/// Hooks a host (eg a Minecraft server) can provide to the engine.
pub trait HostImplementation: Send + Sync {
    fn implementation_name(&self) -> String {
        "Rust".to_string()
    }
    fn implementation_version(&self) -> String {
        "0.1".to_string()
    }
    fn data_folder(&self) -> String {
        ".".to_string()
    }
    fn scripts_folder(&self) -> String {
        "./scripts".to_string()
    }
    /// Deliver a narrate-style message to a target.
    fn narrate(&self, _target: Option<&crate::obj::ObjectTag>, _text: &str) {}
    /// Run a raw server command.
    fn run_server_command(&self, _command: &str) {}
    /// Called after scripts reload.
    fn on_script_reload(&self) {}
}

static HOST: LazyLock<Mutex<Option<Arc<dyn HostImplementation>>>> = LazyLock::new(|| Mutex::new(None));

/// Install the host implementation.
pub fn set_host(host: Arc<dyn HostImplementation>) {
    *HOST.lock().unwrap() = Some(host);
}

pub fn get_host() -> Option<Arc<dyn HostImplementation>> {
    HOST.lock().unwrap().clone()
}

static SPECIAL_DEFS: LazyLock<Mutex<HashMap<String, crate::obj::ObjectTag>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Get a special definition (eg `__player`).
pub fn special_def_get(name: &str, _queue: Option<String>) -> Option<crate::obj::ObjectTag> {
    SPECIAL_DEFS.lock().unwrap().get(&name.to_lowercase()).cloned()
}

/// Set a special definition.
pub fn special_def_set(name: &str, _queue: Option<String>, value: Option<crate::obj::ObjectTag>) {
    let mut map = SPECIAL_DEFS.lock().unwrap();
    match value {
        Some(v) => {
            map.insert(name.to_lowercase(), v);
        }
        None => {
            map.remove(&name.to_lowercase());
        }
    }
}

/// Whether the engine has been initialized.
static INITIALIZED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// Initialize the engine: register object types, tags, and commands.
pub fn init(host: Option<Arc<dyn HostImplementation>>) {
    let mut initialized = INITIALIZED.lock().unwrap();
    if *initialized {
        return;
    }
    if let Some(h) = host {
        set_host(h);
    }
    tags::init_fallbacks();
    obj::register_core_objects();
    tags::core::register_core_tags();
    registry::clear_commands();
    crate::commands::queue_cmds::register_all();
    crate::commands::core_cmds::register_all();
    crate::commands::flag_cmds::register_all();
    crate::commands::util_cmds::register_all();
    crate::events::register_core_events();
    *initialized = true;
    crate::util::debug_log(&format!(
        "Denizen Rust core v{} initialized ({} commands, {} object types).",
        crate::VERSION,
        registry::command_count(),
        obj::object_type_count()
    ));
}

pub fn is_initialized() -> bool {
    *INITIALIZED.lock().unwrap()
}

/// Reload scripts from the host's script folder.
pub fn reload_scripts() -> (usize, Vec<String>) {
    let folder = match get_host() {
        Some(h) => h.scripts_folder(),
        None => "./scripts".to_string(),
    };
    reload_scripts_from(&folder)
}

/// Reload scripts from an explicit folder path.
pub fn reload_scripts_from(folder: &str) -> (usize, Vec<String>) {
    crate::scripts::registry::clear_containers();
    let (count, errors) = crate::scripts::helper::load_scripts_dir(std::path::Path::new(folder));
    if let Some(h) = get_host() {
        h.on_script_reload();
    }
    (count, errors)
}

/// Tag-parse a string using a fresh context.
pub fn tag(text: &str) -> String {
    let ctx = tags::TagContext::default();
    tags::tag(text, &ctx)
}

/// Tag-parse a string and return its object form.
pub fn tag_object(text: &str) -> crate::obj::ObjectTag {
    let ctx = tags::TagContext::default();
    tags::tag_object(text, &ctx)
}

/// Run a script container's `script` section by name.
pub fn run_script(name: &str) -> Result<usize, String> {
    let container = crate::scripts::registry::get_container(name)
        .ok_or_else(|| format!("Script '{}' not found.", name))?;
    let entries = container
        .get_entries("script")
        .ok_or_else(|| format!("Script '{}' has no 'script' section.", name))?;
    let queue = crate::scripts::queue::build_queue(&entries, None, None);
    Ok(crate::scripts::queue::run_queue(&queue))
}

/// Execute a single raw script command line (eg `narrate hello`).
pub fn execute_command_line(line: &str) -> Result<usize, String> {
    let args = crate::scripts::builder::build_args(line, true);
    if args.is_empty() {
        return Err("Empty command line.".to_string());
    }
    let cmd = args[0].clone();
    let rest = args[1..].to_vec();
    let entry = crate::scripts::entry::ScriptEntry::new(&cmd, rest, None, 1);
    let queue = crate::scripts::queue::build_queue(&[entry], None, None);
    Ok(crate::scripts::queue::run_queue(&queue))
}

/// Engine statistics.
pub fn stats() -> String {
    format!(
        "Denizen Rust core v{}: {} commands, {} object types, {} scripts, {} active queues.",
        crate::VERSION,
        registry::command_count(),
        obj::object_type_count(),
        crate::scripts::registry::container_count(),
        crate::scripts::queue::queue_count()
    )
}

pub mod config {
    //! Engine configuration flags mirroring `CoreConfiguration`.
    use std::sync::atomic::{AtomicBool, Ordering};
    pub static DEBUG_VERBOSE: AtomicBool = AtomicBool::new(false);
    pub static DEBUG_OVERRIDE: AtomicBool = AtomicBool::new(false);
    pub static DEBUG_SCRIPT_BUILDER: AtomicBool = AtomicBool::new(false);
    pub static TAG_TIMEOUT_UNSAFE: AtomicBool = AtomicBool::new(false);
    pub static ALLOW_NETWORK: AtomicBool = AtomicBool::new(true);
    pub static SCRIPTS_COMPILED: AtomicBool = AtomicBool::new(false);

    pub fn debug_verbose() -> bool {
        DEBUG_VERBOSE.load(Ordering::Relaxed)
    }
    pub fn set_debug_verbose(v: bool) {
        DEBUG_VERBOSE.store(v, Ordering::Relaxed);
    }
}
