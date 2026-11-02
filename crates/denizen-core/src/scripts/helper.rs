//! Script loading helpers: find and parse `.dsc`/`.yml` script files.

use crate::scripts::container::{ScriptContainer, ScriptContainerKind};
use crate::scripts::registry;
use crate::util::yaml::{self, Yaml};
use std::path::Path;

/// Recursively find script files under a directory.
pub fn list_script_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    collect_files(dir, &mut out);
    out
}

fn collect_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                collect_files(&path, out);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("dsc") || ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml") {
                    out.push(path);
                }
            }
        }
    }
}

/// Remove comments and normalize a script file's text before YAML parsing.
pub fn clear_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for raw_line in input.replace('\t', "    ").replace('\r', "").split('\n') {
        let trimmed = raw_line.trim();
        if trimmed.starts_with('#') {
            result.push('\n');
            continue;
        }
        result.push_str(raw_line);
        result.push('\n');
    }
    result.push('\n');
    result
}

/// Parse a script file's contents into containers, registering each.
pub fn load_script_file(path: &Path) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    let cleaned = clear_comments(&text);
    let parsed = yaml::parse(&cleaned);
    let mut count = 0;
    if let Yaml::Map(entries) = parsed {
        for (name, body) in entries {
            let kind = body
                .get_string("type")
                .map(|t| ScriptContainerKind::from_type_name(&t))
                .unwrap_or(ScriptContainerKind::Task);
            let container = ScriptContainer::new(&name, kind, body);
            registry::register_container(container);
            count += 1;
        }
    }
    Ok(count)
}

/// Load every script file in a directory tree.
pub fn load_scripts_dir(dir: &Path) -> (usize, Vec<String>) {
    let mut total = 0;
    let mut errors = Vec::new();
    for file in list_script_files(dir) {
        match load_script_file(&file) {
            Ok(n) => total += n,
            Err(e) => errors.push(e),
        }
    }
    (total, errors)
}

/// Parse script containers from raw text without touching the filesystem.
pub fn parse_scripts_text(text: &str) -> Vec<ScriptContainer> {
    let cleaned = clear_comments(text);
    let parsed = yaml::parse(&cleaned);
    let mut out = Vec::new();
    if let Yaml::Map(entries) = parsed {
        for (name, body) in entries {
            let kind = body
                .get_string("type")
                .map(|t| ScriptContainerKind::from_type_name(&t))
                .unwrap_or(ScriptContainerKind::Task);
            out.push(ScriptContainer::new(&name, kind, body));
        }
    }
    out
}
