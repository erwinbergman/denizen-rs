//! Script queues: the execution engine that runs script entries.

use crate::commands::registry::{get_command, CommandOutcome};
use crate::obj::{ElementTag, MapTag, ObjectTag};
use crate::scripts::entry::ScriptEntry;
use crate::tags::{ContextSource, DefinitionProvider, TagContext};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

static ALL_QUEUES: LazyLock<RwLock<HashMap<String, Arc<RwLock<ScriptQueue>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static TOTAL_QUEUES: AtomicU64 = AtomicU64::new(0);

pub fn queue_exists(id: &str) -> bool {
    ALL_QUEUES.read().unwrap().contains_key(id)
}

pub fn get_queue(id: &str) -> Option<Arc<RwLock<ScriptQueue>>> {
    ALL_QUEUES.read().unwrap().get(id).cloned()
}

pub fn all_queue_ids() -> Vec<String> {
    ALL_QUEUES.read().unwrap().keys().cloned().collect()
}

pub fn queue_count() -> usize {
    ALL_QUEUES.read().unwrap().len()
}

pub fn all_queues() -> Vec<Arc<RwLock<ScriptQueue>>> {
    ALL_QUEUES.read().unwrap().values().cloned().collect()
}

/// A queue of script entries to run.
pub struct ScriptQueue {
    pub id: String,
    pub entries: VecDeque<ScriptEntry>,
    pub definitions: MapTag,
    pub context_source: Option<Arc<dyn ContextSource>>,
    pub procedural: bool,
    pub is_stopping: bool,
    pub is_stopped: bool,
    pub wait_when_empty: bool,
    pub numeric_id: u64,
    pub delay_until_millis: i64,
    pub debug_output: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    pub callback: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Determinations produced by the queue (for procedure scripts).
    pub determinations: Vec<ObjectTag>,
    /// The current "determine" result, if a determine command ran.
    pub determined: Option<ObjectTag>,
    /// Number of entries executed in this queue (for diagnostics).
    pub executed: usize,
}

impl ScriptQueue {
    pub fn new(id: &str) -> Self {
        let numeric_id = TOTAL_QUEUES.fetch_add(1, Ordering::SeqCst);
        let id = if id.is_empty() { format!("QUEUE_{}", numeric_id) } else { id.to_string() };
        ScriptQueue {
            id,
            entries: VecDeque::new(),
            definitions: MapTag::new(),
            context_source: None,
            procedural: false,
            is_stopping: false,
            is_stopped: false,
            wait_when_empty: false,
            numeric_id,
            delay_until_millis: 0,
            debug_output: None,
            callback: None,
            determinations: Vec::new(),
            determined: None,
            executed: 0,
        }
    }

    pub fn add_entry(&mut self, entry: ScriptEntry) {
        self.entries.push_back(entry);
    }

    pub fn add_entries(&mut self, entries: impl IntoIterator<Item = ScriptEntry>) {
        for e in entries {
            self.entries.push_back(e);
        }
    }

    /// Inject entries so they run next (before the existing queue contents).
    pub fn inject_entries_at_start(&mut self, entries: Vec<ScriptEntry>) {
        for e in entries.into_iter().rev() {
            self.entries.push_front(e);
        }
    }

    pub fn inject_entry_at_start(&mut self, entry: ScriptEntry) {
        self.entries.push_front(entry);
    }

    pub fn remove_first(&mut self) -> Option<ScriptEntry> {
        self.entries.pop_front()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn set_definition(&mut self, key: &str, value: Option<ObjectTag>) {
        if key.starts_with("__") {
            // Special definitions are handled by the host implementation.
            crate::engine::special_def_set(key, Some(self.id.clone()), value.clone());
        }
        self.definitions.put_deep(key, value);
    }

    pub fn get_definition(&self, key: &str) -> Option<ObjectTag> {
        if key.starts_with("__") {
            if let Some(v) = crate::engine::special_def_get(key, Some(self.id.clone())) {
                return Some(v);
            }
        }
        self.definitions.get_deep(key)
    }

    /// Produce a TagContext bound to this queue.
    pub fn tag_context(&self) -> TagContext {
        TagContext {
            debug: true,
            show_errors: true,
            definition_provider: Some(Arc::new(QueueDefinitions { queue_id: self.id.clone() })),
            context_source: self.context_source.clone(),
            script_name: None,
            queue_id: Some(self.id.clone()),
            entry_line: None,
        }
    }

    /// Whether the queue is currently delayed (waiting on a `wait`).
    pub fn is_delayed(&self, now_millis: i64) -> bool {
        self.delay_until_millis > now_millis
    }
}

/// Definition provider backed by a queue id.
pub struct QueueDefinitions {
    pub queue_id: String,
}

impl DefinitionProvider for QueueDefinitions {
    fn get_definition_object(&self, definition: &str) -> Option<ObjectTag> {
        let q = get_queue(&self.queue_id)?;
        let guard = q.read().unwrap();
        guard.get_definition(definition)
    }
    fn add_definition(&self, definition: &str, value: Option<ObjectTag>) {
        if let Some(q) = get_queue(&self.queue_id) {
            q.write().unwrap().set_definition(definition, value);
        }
    }
    fn get_all_definitions(&self) -> Option<MapTag> {
        let q = get_queue(&self.queue_id)?;
        let guard = q.read().unwrap();
        Some(guard.definitions.clone())
    }
}

/// Register a queue in the global table.
pub fn register_queue(queue: ScriptQueue) -> Arc<RwLock<ScriptQueue>> {
    let id = queue.id.clone();
    let arc = Arc::new(RwLock::new(queue));
    ALL_QUEUES.write().unwrap().insert(id, arc.clone());
    arc
}

pub fn remove_queue(id: &str) {
    ALL_QUEUES.write().unwrap().remove(id);
}

/// Run a queue to completion, returning the number of entries executed.
pub fn run_queue(queue: &Arc<RwLock<ScriptQueue>>) -> usize {
    let mut executed = 0;
    loop {
        let (entry, context) = {
            let mut q = queue.write().unwrap();
            if q.is_stopping || q.entries.is_empty() {
                break;
            }
            if q.is_delayed(crate::util::current_time_millis()) {
                break;
            }
            let mut entry = q.entries.pop_front().unwrap();
            entry.queue_id = Some(q.id.clone());
            let ctx = TagContext {
                debug: entry.should_debug,
                show_errors: true,
                definition_provider: Some(Arc::new(QueueDefinitions { queue_id: q.id.clone() })),
                context_source: q.context_source.clone(),
                script_name: entry.script.as_ref().map(|s| s.name.clone()),
                queue_id: Some(q.id.clone()),
                entry_line: Some(entry.original_line.clone()),
            };
            q.executed += 1;
            (entry, ctx)
        };
        executed += 1;
        let outcome = execute_entry(&entry, &context, queue);
        match outcome {
            CommandOutcome::Continue => {}
            CommandOutcome::Stop => {
                let mut q = queue.write().unwrap();
                q.is_stopping = true;
                break;
            }
            CommandOutcome::Wait(millis) => {
                let mut q = queue.write().unwrap();
                q.delay_until_millis = crate::util::current_time_millis() + millis.max(0);
                break;
            }
        }
    }
    let should_remove = {
        let q = queue.read().unwrap();
        q.entries.is_empty() && !q.wait_when_empty && !q.is_delayed(crate::util::current_time_millis())
    };
    if should_remove {
        finish_queue(queue);
    }
    executed
}

/// A pending scheduled script run (from `runlater`).
pub struct ScheduledRun {
    pub execute_at: i64,
    pub entries: Vec<ScriptEntry>,
    pub definitions: MapTag,
    pub parent_queue: Option<String>,
}

static SCHEDULED: LazyLock<RwLock<Vec<ScheduledRun>>> = LazyLock::new(|| RwLock::new(Vec::new()));

/// Schedule a script to run at a given wall-clock millisecond timestamp.
pub fn schedule_run(execute_at: i64, entries: Vec<ScriptEntry>, definitions: MapTag, parent_queue: Option<String>) {
    SCHEDULED.write().unwrap().push(ScheduledRun { execute_at, entries, definitions, parent_queue });
}

/// Number of currently scheduled runs.
pub fn scheduled_count() -> usize {
    SCHEDULED.read().unwrap().len()
}

/// Run any scheduled scripts whose time has come. Returns the number executed.
pub fn tick_scheduled() -> usize {
    let now = crate::util::current_time_millis();
    let due: Vec<ScheduledRun> = {
        let mut pending = SCHEDULED.write().unwrap();
        let mut due = Vec::new();
        let mut keep = Vec::new();
        for run in pending.drain(..) {
            if run.execute_at <= now {
                due.push(run);
            } else {
                keep.push(run);
            }
        }
        *pending = keep;
        due
    };
    let count = due.len();
    for run in due {
        let queue = build_queue(&run.entries, Some(run.definitions), None);
        run_queue(&queue);
    }
    count
}

/// Advance all timed queues and scheduled runs. Intended to be called each server tick.
pub fn tick() -> usize {
    tick_scheduled() + tick_queues()
}

/// Mark a queue as stopped and invoke its callback.
pub fn finish_queue(queue: &Arc<RwLock<ScriptQueue>>) {
    let (id, callback) = {
        let mut q = queue.write().unwrap();
        if q.is_stopped {
            return;
        }
        q.is_stopped = true;
        (q.id.clone(), q.callback.clone())
    };
    remove_queue(&id);
    if let Some(cb) = callback {
        cb();
    }
}

/// Advance every timed queue whose delay has elapsed. Returns the number of queues advanced.
pub fn tick_queues() -> usize {
    let now = crate::util::current_time_millis();
    let queues = all_queues();
    let mut advanced = 0;
    for queue in queues {
        let ready = {
            let q = queue.read().unwrap();
            !q.is_stopped && q.delay_until_millis <= now
        };
        if ready {
            run_queue(&queue);
            advanced += 1;
        }
    }
    advanced
}

/// Execute a single entry, returning the control-flow outcome.
fn execute_entry(entry: &ScriptEntry, context: &TagContext, queue: &Arc<RwLock<ScriptQueue>>) -> CommandOutcome {
    if let Some(cmd) = get_command(&entry.command) {
        match (cmd.runner)(entry, context, queue) {
            Ok(outcome) => outcome,
            Err(err) => {
                context.echo_error(&format!("Error executing command '{}': {}", entry.command, err));
                CommandOutcome::Continue
            }
        }
    } else if entry.command.is_empty() {
        CommandOutcome::Continue
    } else {
        context.echo_error(&format!("Unknown command '{}'.", entry.command));
        CommandOutcome::Continue
    }
}

/// Start a queue synchronously and run it.
pub fn start_queue(mut queue: ScriptQueue) -> Arc<RwLock<ScriptQueue>> {
    queue.is_stopped = false;
    let arc = register_queue(queue);
    run_queue(&arc);
    arc
}

/// Create a queue for a script's "script" section and run it.
pub fn run_script_entries(entries: &[ScriptEntry], definitions: Option<MapTag>, context_source: Option<Arc<dyn ContextSource>>) -> Arc<RwLock<ScriptQueue>> {
    let mut queue = ScriptQueue::new("");
    queue.context_source = context_source;
    if let Some(d) = definitions {
        queue.definitions = d;
    }
    for e in entries {
        queue.add_entry(e.clone());
    }
    start_queue(queue)
}

/// Build a fresh queue from entries without running it.
pub fn build_queue(entries: &[ScriptEntry], definitions: Option<MapTag>, context_source: Option<Arc<dyn ContextSource>>) -> Arc<RwLock<ScriptQueue>> {
    let mut queue = ScriptQueue::new("");
    queue.context_source = context_source;
    if let Some(d) = definitions {
        queue.definitions = d;
    }
    for e in entries {
        queue.add_entry(e.clone());
    }
    register_queue(queue)
}

/// Run a set of entries synchronously inside an existing queue's definition scope.
pub fn run_now(queue: &Arc<RwLock<ScriptQueue>>, entries: Vec<ScriptEntry>) {
    {
        let mut q = queue.write().unwrap();
        q.inject_entries_at_start(entries);
    }
    run_queue(queue);
}

/// Convenience: build a TagContext that has no definitions.
pub fn empty_context() -> TagContext {
    TagContext {
        definition_provider: Some(Arc::new(crate::tags::SimpleDefinitionProvider::new())),
        ..Default::default()
    }
}

#[allow(dead_code)]
fn _unused(_e: ElementTag) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Arc<RwLock<ScriptQueue>> {
        crate::engine::init(None);
        let parsed = crate::util::yaml::parse(text);
        let container = crate::scripts::container::ScriptContainer::new(
            "test",
            crate::scripts::container::ScriptContainerKind::Task,
            parsed,
        );
        let entries = container.get_entries("script").unwrap();
        run_script_entries(&entries, None, None)
    }

    #[test]
    fn if_else_runs_correct_branch() {
        let _g = crate::scripts::test_guard();
        let q = run("script:\n  - if 1 == 2:\n    - define branch wrong\n  - else:\n    - define branch right\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("branch").unwrap().identify(), "right");
    }

    #[test]
    fn if_true_branch() {
        let _g = crate::scripts::test_guard();
        let q = run("script:\n  - if 2 > 1:\n    - define branch yes\n  - else:\n    - define branch no\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("branch").unwrap().identify(), "yes");
    }

    #[test]
    fn repeat_loops() {
        let _g = crate::scripts::test_guard();
        let q = run("script:\n  - define count 0\n  - repeat 5:\n    - define last <[value]>\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("last").unwrap().identify(), "5");
    }

    #[test]
    fn foreach_loops() {
        let _g = crate::scripts::test_guard();
        let q = run("script:\n  - define total 0\n  - foreach li@a|b|c as:item:\n    - define last <[item]>\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("last").unwrap().identify(), "c");
    }

    #[test]
    fn choose_matches() {
        let _g = crate::scripts::test_guard();
        let q = run("script:\n  - define x b\n  - choose <[x]>:\n    - case a:\n      - define result A\n    - case b:\n      - define result B\n    - default:\n      - define result D\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("result").unwrap().identify(), "B");
    }

    #[test]
    fn while_loops() {
        let _g = crate::scripts::test_guard();
        let q = run("script:\n  - define loop true\n  - while <[loop]>:\n    - define loop false\n    - define ran yes\n  - define done <[ran]>\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("done").unwrap().identify(), "yes");
    }
}


#[cfg(test)]
mod integration_tests {
    use super::*;

    fn run_text(text: &str) -> Arc<RwLock<ScriptQueue>> {
        crate::engine::init(None);
        let containers = crate::scripts::helper::parse_scripts_text(text);
        let c = &containers[0];
        let entries = c.get_entries("script").unwrap();
        run_script_entries(&entries, None, None)
    }

    #[test]
    fn goto_mark() {
        let _g = crate::scripts::test_guard();
        let q = run_text("s:\n  type: task\n  script:\n  - define x before\n  - goto skip\n  - define x skipped\n  - mark skip\n  - define y after\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("x").unwrap().identify(), "before");
        assert_eq!(g.get_definition("y").unwrap().identify(), "after");
    }

    #[test]
    fn switch_case() {
        let _g = crate::scripts::test_guard();
        let q = run_text("s:\n  type: task\n  script:\n  - define v two\n  - switch <[v]>:\n    - case one:\n      - define r 1\n    - case two:\n      - define r 2\n    - default:\n      - define r 0\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("r").unwrap().identify(), "2");
    }

    #[test]
    fn nested_if_in_foreach() {
        let _g = crate::scripts::test_guard();
        let q = run_text("s:\n  type: task\n  script:\n  - define hits 0\n  - foreach li@a|b|c as:item:\n    - if <[item]> == b:\n      - define hits 1\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("hits").unwrap().identify(), "1");
    }

    #[test]
    fn else_if_chain() {
        let _g = crate::scripts::test_guard();
        let q = run_text("s:\n  type: task\n  script:\n  - define n 5\n  - if <[n]> > 10:\n    - define r big\n  - else if <[n]> > 3:\n    - define r medium\n  - else:\n    - define r small\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("r").unwrap().identify(), "medium");
    }

    #[test]
    fn foreach_map_keys() {
        let _g = crate::scripts::test_guard();
        let q = run_text("s:\n  type: task\n  script:\n  - define last none\n  - foreach map@[a=1;b=2] as:val key:k:\n    - define last <[k]>=<[val]>\n");
        let g = q.read().unwrap();
        assert_eq!(g.get_definition("last").unwrap().identify(), "b=2");
    }

    #[test]
    fn random_braced() {
        let _g = crate::scripts::test_guard();
        let q = run_text("s:\n  type: task\n  script:\n  - random:\n    - define r a\n    - define r b\n    - define r c\n");
        let g = q.read().unwrap();
        let r = g.get_definition("r").unwrap().identify();
        assert!(["a", "b", "c"].contains(&r.as_str()));
    }

    #[test]
    fn run_another_script() {
        let _g = crate::scripts::test_guard();
        crate::engine::init(None);
        let containers = crate::scripts::helper::parse_scripts_text(
            "child:\n  type: task\n  script:\n  - define child_ran yes\nparent:\n  type: task\n  script:\n  - define ran 0\n  - run child\n  - define ran 1\n",
        );
        for c in &containers {
            crate::scripts::registry::register_container(crate::scripts::container::ScriptContainer::new(
                &c.name, c.kind.clone(), c.contents.clone()));
        }
        let parent = crate::scripts::registry::get_container("parent").unwrap();
        let entries = parent.get_entries("script").unwrap();
        let q = run_script_entries(&entries, None, None);
        let g = q.read().unwrap();
        // The parent continues after the child run completes.
        assert_eq!(g.get_definition("ran").unwrap().identify(), "1");
        crate::scripts::registry::clear_containers();
    }
}

