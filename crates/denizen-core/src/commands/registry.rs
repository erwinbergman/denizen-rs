//! The command registry.

use crate::scripts::entry::ScriptEntry;
use crate::scripts::queue::ScriptQueue;
use crate::tags::TagContext;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

/// Control-flow outcome of executing a command.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandOutcome {
    /// Continue with the next entry.
    Continue,
    /// Stop the queue.
    Stop,
    /// Pause the queue for the given number of milliseconds.
    Wait(i64),
}

/// Signature of a command implementation.
pub type CommandRunner = Arc<
    dyn Fn(&ScriptEntry, &TagContext, &Arc<RwLock<ScriptQueue>>) -> Result<CommandOutcome, String>
        + Send
        + Sync,
>;

/// A registered command.
#[derive(Clone)]
pub struct RegisteredCommand {
    pub name: String,
    pub syntax: String,
    pub runner: CommandRunner,
    pub is_procedural: bool,
    pub minimum_args: usize,
    pub maximum_args: usize,
}

static COMMANDS: LazyLock<RwLock<HashMap<String, RegisteredCommand>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a command.
pub fn register_command(cmd: RegisteredCommand) {
    COMMANDS.write().unwrap().insert(cmd.name.to_lowercase(), cmd);
}

/// Register a command from a closure.
pub fn register(
    name: &str,
    syntax: &str,
    runner: impl Fn(&ScriptEntry, &TagContext, &Arc<RwLock<ScriptQueue>>) -> Result<CommandOutcome, String> + Send + Sync + 'static,
) {
    register_command(RegisteredCommand {
        name: name.to_lowercase(),
        syntax: syntax.to_string(),
        runner: Arc::new(runner),
        is_procedural: false,
        minimum_args: 0,
        maximum_args: usize::MAX,
    });
}

pub fn get_command(name: &str) -> Option<RegisteredCommand> {
    COMMANDS.read().unwrap().get(&name.to_lowercase()).cloned()
}

pub fn command_exists(name: &str) -> bool {
    COMMANDS.read().unwrap().contains_key(&name.to_lowercase())
}

pub fn command_names() -> Vec<String> {
    COMMANDS.read().unwrap().keys().cloned().collect()
}

pub fn command_count() -> usize {
    COMMANDS.read().unwrap().len()
}

pub fn clear_commands() {
    COMMANDS.write().unwrap().clear();
}
