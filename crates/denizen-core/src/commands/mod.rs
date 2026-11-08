//! Commands: the registry, argument parsing, and control-flow outcomes.

pub mod registry;
pub mod helpers;
pub mod queue_cmds;
pub mod core_cmds;
pub mod flag_cmds;
pub mod util_cmds;
pub mod entity_cmds;
pub mod player_cmds;
pub mod world_cmds;
pub mod npc_cmds;
pub mod item_cmds;
pub mod server_cmds;
pub mod data_cmds;
pub mod web_cmds;
pub mod interact_cmds;

pub use registry::{CommandOutcome, RegisteredCommand};

#[cfg(test)]
mod count_tests {
    #[test]
    fn count_all() {
        crate::engine::init(None);
        let cmds = crate::commands::registry::command_count();
        let events = crate::events::event_count();
        println!("COMMANDS={} EVENTS={}", cmds, events);
        assert!(cmds > 100);
        assert!(events > 150);
    }
}

