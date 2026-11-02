//! Script builders: turn parsed YAML content into `ScriptEntry` lists.
//!
//! Denizen script sections are YAML sequences of command lines. A command may be a plain
//! scalar (`- narrate hello`) or a mapping whose key is the command and whose value is the
//! braced sub-command body (`- if <cond>:` followed by an indented `- ...` list).

use crate::scripts::container::ScriptContainer;
use crate::scripts::entry::ScriptEntry;
use crate::util::yaml::Yaml;
use std::sync::Arc;

/// Marker char used to track line numbers through the comment-clearing phase.
pub const LINE_PREFIX_CHAR: char = '^';

/// Strip a leading line-number prefix from a raw line.
pub fn strip_line_prefix(raw_line: &str) -> &str {
    if !raw_line.starts_with(LINE_PREFIX_CHAR) {
        return raw_line;
    }
    match raw_line[1..].find(LINE_PREFIX_CHAR) {
        Some(idx) => &raw_line[idx + 2..],
        None => raw_line,
    }
}

/// Split a command line's argument string into individual arguments, respecting quotes and braces.
pub fn build_args(input: &str, quotes: bool) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut quote: Option<char> = None;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                current.push(c);
            }
        } else if quotes && (c == '\'' || c == '"') {
            quote = Some(c);
        } else if c == ' ' {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
        } else if c == '\\' && i + 1 < chars.len() && chars[i + 1] == ' ' {
            current.push(' ');
            i += 1;
        } else {
            current.push(c);
        }
        i += 1;
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Build a list of script entries from a YAML value (list of lines, a single string, or a map).
pub fn build_script_entries(contents: &Yaml, parent: &ScriptContainer) -> Vec<ScriptEntry> {
    let mut out = Vec::new();
    build_entries_into(contents, parent, &mut out);
    out
}

fn build_entries_into(contents: &Yaml, parent: &ScriptContainer, out: &mut Vec<ScriptEntry>) {
    match contents {
        Yaml::List(items) => {
            for (i, item) in items.iter().enumerate() {
                build_node(item, parent, i + 1, out);
            }
        }
        Yaml::Map(entries) => {
            if entries.len() == 1 {
                let (key, value) = &entries[0];
                build_key_value(key, value, parent, 1, out);
            } else {
                // Multiple keys in one node: treat as a sequence of commands.
                for (i, (key, value)) in entries.iter().enumerate() {
                    build_key_value(key, value, parent, i + 1, out);
                }
            }
        }
        Yaml::String(s) => {
            let mut line_no = 1;
            for raw in s.split('\n') {
                build_raw_line(raw, parent, line_no, out);
                line_no += 1;
            }
        }
        Yaml::Null => {}
        other => {
            if let Some(s) = other.as_string() {
                build_raw_line(&s, parent, 1, out);
            }
        }
    }
}

fn build_node(item: &Yaml, parent: &ScriptContainer, line_number: usize, out: &mut Vec<ScriptEntry>) {
    match item {
        Yaml::String(s) => build_raw_line(s, parent, line_number, out),
        Yaml::Map(entries) => {
            if entries.len() == 1 {
                let (key, value) = &entries[0];
                build_key_value(key, value, parent, line_number, out);
            } else {
                // A mapping inside a list: the first key carries the body of the command,
                // subsequent keys are sibling commands within the same block.
                for (i, (key, value)) in entries.iter().enumerate() {
                    build_key_value(key, value, parent, line_number + i, out);
                }
            }
        }
        Yaml::List(_) => {
            // A nested list without a command key: flatten into entries.
            build_entries_into(item, parent, out);
        }
        Yaml::Null => {}
        other => {
            if let Some(s) = other.as_string() {
                build_raw_line(&s, parent, line_number, out);
            }
        }
    }
}

fn build_key_value(key: &str, value: &Yaml, parent: &ScriptContainer, line_number: usize, out: &mut Vec<ScriptEntry>) {
    let (actual_key, mut line_num) = strip_prefix_number(key, line_number);
    let actual_key = actual_key.trim();
    if actual_key.is_empty() {
        return;
    }
    // A body is any non-empty nested block or an inline command string.
    let body: Option<Vec<ScriptEntry>> = match value {
        Yaml::Null => None,
        Yaml::String(s) if s.trim().is_empty() => None,
        _ => {
            let mut sub = Vec::new();
            build_entries_into(value, parent, &mut sub);
            if sub.is_empty() { None } else { Some(sub) }
        }
    };
    let entry = make_entry(actual_key, parent, line_num, body);
    if let Some(e) = entry {
        out.push(e);
    }
    let _ = &mut line_num;
}

fn build_raw_line(raw: &str, parent: &ScriptContainer, line_number: usize, out: &mut Vec<ScriptEntry>) {
    let mut line = raw.trim();
    if line.is_empty() {
        return;
    }
    // Strip a leading YAML list marker if present.
    if let Some(stripped) = line.strip_prefix("- ") {
        line = stripped;
    } else if line == "-" {
        return;
    }
    let (content, line_num) = strip_prefix_number(line, line_number);
    let content = content.trim();
    if content.is_empty() {
        return;
    }
    if let Some(e) = make_entry(content, parent, line_num, None) {
        out.push(e);
    }
}

fn strip_prefix_number(line: &str, default_line: usize) -> (&str, usize) {
    if line.starts_with(LINE_PREFIX_CHAR) {
        if let Some(idx) = line[1..].find(LINE_PREFIX_CHAR) {
            if let Ok(n) = line[1..idx + 1].parse::<usize>() {
                return (&line[idx + 2..], n);
            }
            return (&line[idx + 2..], default_line);
        }
    }
    (line, default_line)
}

fn make_entry(content: &str, parent: &ScriptContainer, line_number: usize, braced: Option<Vec<ScriptEntry>>) -> Option<ScriptEntry> {
    let mut parts = content.splitn(2, ' ');
    let command = parts.next().unwrap_or("").trim();
    if command.is_empty() {
        return None;
    }
    let args = parts.next().map(|a| build_args(a, true)).unwrap_or_default();
    let mut entry = ScriptEntry::new(command, args, Some(Arc::new(clone_container(parent))), line_number);
    entry.original_line = content.to_string();
    entry.owner_line = line_number;
    entry.braced = braced;
    Some(entry)
}

fn clone_container(parent: &ScriptContainer) -> ScriptContainer {
    // Containers are immutable after registration, so we re-use the parsed body.
    ScriptContainer::new(&parent.name, parent.kind.clone(), parent.contents.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripts::container::{ScriptContainer, ScriptContainerKind};

    fn container() -> ScriptContainer {
        ScriptContainer::new("test", ScriptContainerKind::Task, crate::util::yaml::Yaml::new_map())
    }

    #[test]
    fn parses_braced_if() {
        let _g = crate::scripts::test_guard();
        let yaml = crate::util::yaml::parse(
            "script:\n  - if 1 == 1:\n    - narrate yes\n    - narrate two\n  - narrate done\n",
        );
        let script = yaml.get("script").unwrap();
        let entries = build_script_entries(script, &container());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "if");
        assert_eq!(entries[0].args, vec!["1", "==", "1"]);
        let body = entries[0].braced.as_ref().unwrap();
        assert_eq!(body.len(), 2);
        assert_eq!(body[0].command, "narrate");
        assert_eq!(body[1].args, vec!["two"]);
        assert_eq!(entries[1].command, "narrate");
    }

    #[test]
    fn parses_choose_cases() {
        let _g = crate::scripts::test_guard();
        let yaml = crate::util::yaml::parse(
            "script:\n  - choose <[x]>:\n    - case a:\n      - narrate A\n    - case b c:\n      - narrate B\n    - default:\n      - narrate D\n",
        );
        let entries = build_script_entries(yaml.get("script").unwrap(), &container());
        assert_eq!(entries.len(), 1);
        let body = entries[0].braced.as_ref().unwrap();
        assert_eq!(body.len(), 3);
        assert_eq!(body[0].command, "case");
        assert_eq!(body[0].braced.as_ref().unwrap()[0].command, "narrate");
        assert_eq!(body[2].command, "default");
    }
}
