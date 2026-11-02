//! The global script registry.

use crate::scripts::container::ScriptContainer;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

static CONTAINERS: LazyLock<RwLock<HashMap<String, Arc<ScriptContainer>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static ORIGINAL_NAMES: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a container under its lowercased name.
pub fn register_container(container: ScriptContainer) {
    let key = container.name.to_lowercase();
    let original = container.original_name.clone();
    let arc = Arc::new(container);
    CONTAINERS.write().unwrap().insert(key.clone(), arc);
    ORIGINAL_NAMES.write().unwrap().insert(key, original);
}

pub fn get_container(name: &str) -> Option<Arc<ScriptContainer>> {
    CONTAINERS.read().unwrap().get(&name.to_lowercase()).cloned()
}

pub fn get_original_name(name: &str) -> Option<String> {
    ORIGINAL_NAMES.read().unwrap().get(&name.to_lowercase()).cloned()
}

pub fn all_containers() -> Vec<Arc<ScriptContainer>> {
    CONTAINERS.read().unwrap().values().cloned().collect()
}

pub fn container_names() -> Vec<String> {
    CONTAINERS.read().unwrap().keys().cloned().collect()
}

pub fn clear_containers() {
    CONTAINERS.write().unwrap().clear();
    ORIGINAL_NAMES.write().unwrap().clear();
}

pub fn container_count() -> usize {
    CONTAINERS.read().unwrap().len()
}
