//! A YAML subset parser sufficient for Denizen script files.
//!
//! Supports block mappings, block sequences, nested structures, quoted scalars,
//! simple flow sequences (`[a, b]`), comments, and multi-line strings via `|`/`>`.

use std::collections::HashMap;

/// A parsed YAML node.
#[derive(Clone, Debug, PartialEq)]
pub enum Yaml {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Yaml>),
    Map(Vec<(String, Yaml)>),
}

impl Default for Yaml {
    fn default() -> Self {
        Yaml::Null
    }
}

impl Yaml {
    pub fn new_map() -> Self {
        Yaml::Map(Vec::new())
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Yaml::Null)
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Yaml::Map(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Yaml::List(_))
    }

    pub fn as_map_entries(&self) -> Option<&Vec<(String, Yaml)>> {
        match self {
            Yaml::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut Vec<(String, Yaml)>> {
        match self {
            Yaml::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Yaml::String(s) => Some(s.clone()),
            Yaml::Int(i) => Some(i.to_string()),
            Yaml::Float(f) => Some(crate::util::double_to_string(*f)),
            Yaml::Bool(b) => Some(b.to_string()),
            Yaml::Null => Some(String::new()),
            _ => None,
        }
    }

    /// Fetch a value by dotted path.
    pub fn get(&self, path: &str) -> Option<&Yaml> {
        let mut cur = self;
        for part in path.split('.') {
            match cur {
                Yaml::Map(entries) => {
                    cur = &entries.iter().find(|(k, _)| k == part)?.1;
                }
                _ => return None,
            }
        }
        Some(cur)
    }

    pub fn get_string(&self, path: &str) -> Option<String> {
        self.get(path).and_then(|v| v.as_string())
    }

    pub fn get_string_list(&self, path: &str) -> Vec<String> {
        match self.get(path) {
            Some(Yaml::List(items)) => items.iter().filter_map(|i| i.as_string()).collect(),
            Some(other) => other.as_string().map(|s| vec![s]).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    pub fn get_section(&self, path: &str) -> Option<&Yaml> {
        match self.get(path) {
            Some(v @ Yaml::Map(_)) => Some(v),
            _ => None,
        }
    }

    pub fn as_string_list(&self) -> Option<Vec<String>> {
        match self {
            Yaml::List(items) => Some(items.iter().filter_map(|i| i.as_string()).collect()),
            Yaml::String(s) => Some(vec![s.clone()]),
            Yaml::Null => Some(Vec::new()),
            _ => None,
        }
    }

    /// Get or insert a key in a map.
    pub fn set(&mut self, key: &str, value: Yaml) {
        if let Yaml::Map(entries) = self {
            if let Some(e) = entries.iter_mut().find(|(k, _)| k == key) {
                e.1 = value;
            } else {
                entries.push((key.to_string(), value));
            }
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Yaml::Map(entries) = self {
            entries.retain(|(k, _)| k != key);
        }
    }

    /// Render back to a YAML string.
    pub fn save_to_string(&self) -> String {
        let mut out = String::new();
        write_yaml(self, 0, &mut out, false);
        out
    }
}

fn write_yaml(node: &Yaml, indent: usize, out: &mut String, inline_list: bool) {
    let pad = "  ".repeat(indent);
    match node {
        Yaml::Map(entries) => {
            if entries.is_empty() {
                if !inline_list {
                    out.push_str("{}");
                }
                return;
            }
            for (i, (k, v)) in entries.iter().enumerate() {
                if inline_list && i == 0 {
                    out.push_str(k);
                } else {
                    if i > 0 || !inline_list {
                        out.push_str(&pad);
                    }
                    out.push_str(k);
                }
                match v {
                    Yaml::Map(m) if !m.is_empty() => {
                        out.push_str(":\n");
                        write_yaml(v, indent + 1, out, false);
                    }
                    Yaml::List(l) if !l.is_empty() => {
                        out.push_str(":\n");
                        write_yaml(v, indent + 1, out, false);
                    }
                    Yaml::String(s) if s.contains('\n') => {
                        out.push_str(": |\n");
                        for line in s.lines() {
                            out.push_str(&pad);
                            out.push_str("  ");
                            out.push_str(line);
                            out.push('\n');
                        }
                    }
                    other => {
                        out.push_str(": ");
                        out.push_str(&scalar_to_string(other));
                        out.push('\n');
                    }
                }
            }
        }
        Yaml::List(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            for item in items {
                out.push_str(&pad);
                out.push_str("- ");
                match item {
                    Yaml::Map(m) if !m.is_empty() => {
                        write_yaml(item, indent + 1, out, true);
                        out.push('\n');
                        let _ = m;
                    }
                    Yaml::List(l) if !l.is_empty() => {
                        out.push('\n');
                        write_yaml(item, indent + 1, out, false);
                    }
                    other => {
                        out.push_str(&scalar_to_string(other));
                        out.push('\n');
                    }
                }
            }
        }
        other => {
            out.push_str(&pad);
            out.push_str(&scalar_to_string(other));
            out.push('\n');
        }
    }
}

fn scalar_to_string(node: &Yaml) -> String {
    match node {
        Yaml::Null => "null".to_string(),
        Yaml::Bool(b) => b.to_string(),
        Yaml::Int(i) => i.to_string(),
        Yaml::Float(f) => crate::util::double_to_string(*f),
        Yaml::String(s) => {
            let needs_quote = s.is_empty()
                || s.starts_with(' ')
                || s.ends_with(' ')
                || s.contains(": ")
                || s.contains(" #")
                || s.starts_with('-')
                || s.starts_with('[')
                || s.starts_with('{')
                || s.starts_with('#')
                || s.starts_with('&')
                || s.starts_with('*')
                || s.starts_with('!')
                || s.contains('\n');
            if needs_quote {
                format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        Yaml::List(l) => {
            let parts: Vec<String> = l.iter().map(scalar_to_string).collect();
            format!("[{}]", parts.join(", "))
        }
        Yaml::Map(m) => {
            let parts: Vec<String> = m.iter().map(|(k, v)| format!("{}: {}", k, scalar_to_string(v))).collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Line {
    indent: usize,
    text: String,
    #[allow(dead_code)]
    number: usize,
}

/// Parse a YAML document into a `Yaml` value.
pub fn parse(input: &str) -> Yaml {
    let lines = preprocess(input);
    if lines.is_empty() {
        return Yaml::Null;
    }
    let mut pos = 0usize;
    let value = parse_block(&lines, &mut pos, lines[0].indent);
    value
}

fn preprocess(input: &str) -> Vec<Line> {
    let mut out = Vec::new();
    let normalized = input.replace('\r', "");
    let raw_lines: Vec<&str> = normalized.split('\n').collect();
    let mut i = 0;
    while i < raw_lines.len() {
        let raw = raw_lines[i];
        if raw.trim().is_empty() {
            i += 1;
            continue;
        }
        let without_comment = strip_comment(raw);
        let trimmed = without_comment.trim_end();
        if trimmed.trim().is_empty() {
            i += 1;
            continue;
        }
        if trimmed.trim_start().starts_with("---") || trimmed.trim_start().starts_with("...") {
            i += 1;
            continue;
        }
        let indent = trimmed.len() - trimmed.trim_start().len();
        let mut text = trimmed.trim_start().to_string();

        // Block scalar support.
        if let Some(block_marker) = block_scalar_marker(&text) {
            let base_indent = indent;
            let mut collected = Vec::new();
            let mut j = i + 1;
            let mut content_indent: Option<usize> = None;
            while j < raw_lines.len() {
                let rl = raw_lines[j];
                if rl.trim().is_empty() {
                    collected.push(String::new());
                    j += 1;
                    continue;
                }
                let ind = rl.len() - rl.trim_start().len();
                if ind <= base_indent {
                    break;
                }
                if content_indent.is_none() {
                    content_indent = Some(ind);
                }
                let ci = content_indent.unwrap();
                let slice = if rl.len() >= ci { &rl[ci..] } else { "" };
                collected.push(slice.to_string());
                j += 1;
            }
            while collected.last().map(|s| s.is_empty()).unwrap_or(false) {
                collected.pop();
            }
            let joined = if block_marker == '>' {
                collected.join(" ")
            } else {
                collected.join("\n")
            };
            let key_part = text.trim_end_matches(|c| c == '|' || c == '>' || c == '-' || c == '+').trim_end();
            text = format!("{} {}", key_part, quote_scalar(&joined));
            out.push(Line { indent, text, number: i + 1 });
            i = j;
            continue;
        }

        out.push(Line { indent, text, number: i + 1 });
        i += 1;
    }
    out
}

fn block_scalar_marker(text: &str) -> Option<char> {
    let t = text.trim_end();
    if t.ends_with(": |") || t.ends_with(": |-") || t.ends_with(": |+") {
        return Some('|');
    }
    if t.ends_with(": >") || t.ends_with(": >-") || t.ends_with(": >+") {
        return Some('>');
    }
    if t.ends_with(" |") || t.ends_with(" |-") {
        return Some('|');
    }
    if t.ends_with(" >") || t.ends_with(" >-") {
        return Some('>');
    }
    None
}

fn quote_scalar(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
}

fn strip_comment(line: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if c == '#' && !in_single && !in_double {
            if i == 0 || chars[i - 1].is_whitespace() {
                return chars[..i].iter().collect();
            }
        }
        i += 1;
    }
    line.to_string()
}

fn parse_block(lines: &[Line], pos: &mut usize, indent: usize) -> Yaml {
    if *pos >= lines.len() {
        return Yaml::Null;
    }
    let first = &lines[*pos];
    if first.indent < indent {
        return Yaml::Null;
    }
    if first.text.starts_with("- ") || first.text == "-" {
        parse_sequence(lines, pos, first.indent)
    } else {
        parse_mapping(lines, pos, first.indent)
    }
}

fn parse_sequence(lines: &[Line], pos: &mut usize, indent: usize) -> Yaml {
    let mut items = Vec::new();
    while *pos < lines.len() {
        let line = &lines[*pos];
        if line.indent != indent || !(line.text.starts_with("- ") || line.text == "-") {
            break;
        }
        let content = if line.text == "-" { String::new() } else { line.text[2..].to_string() };
        let item_indent = indent + 2;
        if content.is_empty() {
            *pos += 1;
            if *pos < lines.len() && lines[*pos].indent > indent {
                items.push(parse_block(lines, pos, lines[*pos].indent));
            } else {
                items.push(Yaml::Null);
            }
            continue;
        }
        // Is this a `- key: value` mapping start?
        if let Some((key, rest)) = split_key_value(&content) {
            // Build an inline map, then continue nested mapping lines at item_indent.
            let mut map_entries: Vec<(String, Yaml)> = Vec::new();
            let mut current_key = key;
            let mut current_rest = rest;
            loop {
                if current_rest.is_empty() {
                    *pos += 1;
                    if *pos < lines.len() && lines[*pos].indent > indent {
                        let sub = parse_block(lines, pos, lines[*pos].indent);
                        map_entries.push((current_key.clone(), sub));
                    } else {
                        map_entries.push((current_key.clone(), Yaml::Null));
                    }
                    break;
                } else {
                    *pos += 1;
                    map_entries.push((current_key.clone(), parse_scalar(&current_rest)));
                    // Continue reading additional keys at item_indent level.
                    if *pos < lines.len() && lines[*pos].indent >= item_indent && !lines[*pos].text.starts_with("- ") {
                        let next = &lines[*pos];
                        if next.indent == item_indent {
                            if let Some((k, r)) = split_key_value(&next.text) {
                                current_key = k;
                                current_rest = r;
                                continue;
                            }
                        }
                    }
                    break;
                }
            }
            items.push(Yaml::Map(map_entries));
        } else {
            items.push(parse_scalar(&content));
            *pos += 1;
        }
    }
    Yaml::List(items)
}

fn parse_mapping(lines: &[Line], pos: &mut usize, indent: usize) -> Yaml {
    let mut entries: Vec<(String, Yaml)> = Vec::new();
    while *pos < lines.len() {
        let line = &lines[*pos];
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            // Unexpected deeper indentation; skip.
            *pos += 1;
            continue;
        }
        if line.text.starts_with("- ") || line.text == "-" {
            break;
        }
        let (key, rest) = match split_key_value(&line.text) {
            Some(kv) => kv,
            None => {
                *pos += 1;
                continue;
            }
        };
        if rest.is_empty() {
            *pos += 1;
            if *pos < lines.len() {
                let next = &lines[*pos];
                if next.indent > indent {
                    let sub = parse_block(lines, pos, next.indent);
                    entries.push((key, sub));
                } else if next.indent == indent && (next.text.starts_with("- ") || next.text == "-") {
                    let sub = parse_sequence(lines, pos, next.indent);
                    entries.push((key, sub));
                } else {
                    entries.push((key, Yaml::Null));
                }
            } else {
                entries.push((key, Yaml::Null));
            }
        } else {
            entries.push((key, parse_scalar(&rest)));
            *pos += 1;
        }
    }
    Yaml::Map(entries)
}

fn split_key_value(text: &str) -> Option<(String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut in_single = false;
    let mut in_double = false;
    let mut depth = 0i32;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            in_single = !in_single;
        } else if c == '"' && !in_single {
            in_double = !in_double;
        } else if !in_single && !in_double {
            if c == '[' || c == '{' {
                depth += 1;
            } else if c == ']' || c == '}' {
                depth -= 1;
            } else if c == ':' && depth == 0 {
                let next_is_space = i + 1 >= chars.len() || chars[i + 1].is_whitespace();
                if next_is_space {
                    let key: String = chars[..i].iter().collect();
                    let rest: String = chars[i + 1..].iter().collect();
                    return Some((unquote_key(key.trim()), rest.trim().to_string()));
                }
            }
        }
        i += 1;
    }
    None
}

fn unquote_key(key: &str) -> String {
    let key = key.trim();
    if (key.starts_with('"') && key.ends_with('"')) || (key.starts_with('\'') && key.ends_with('\'')) {
        key[1..key.len() - 1].to_string()
    } else {
        key.to_string()
    }
}

fn parse_scalar(text: &str) -> Yaml {
    let t = text.trim();
    if t.is_empty() || t == "~" || t.eq_ignore_ascii_case("null") {
        return Yaml::Null;
    }
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        return Yaml::String(unescape_double(&t[1..t.len() - 1]));
    }
    if t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2 {
        return Yaml::String(t[1..t.len() - 1].replace("''", "'"));
    }
    if t.starts_with('[') && t.ends_with(']') {
        let inner = &t[1..t.len() - 1];
        let items = split_flow(inner).into_iter().map(|s| parse_scalar(&s)).collect();
        return Yaml::List(items);
    }
    if t.starts_with('{') && t.ends_with('}') {
        let inner = &t[1..t.len() - 1];
        let mut entries = Vec::new();
        for part in split_flow(inner) {
            if let Some((k, v)) = split_key_value(&part) {
                entries.push((k, parse_scalar(&v)));
            }
        }
        return Yaml::Map(entries);
    }
    if t.eq_ignore_ascii_case("true") {
        return Yaml::Bool(true);
    }
    if t.eq_ignore_ascii_case("false") {
        return Yaml::Bool(false);
    }
    if let Ok(i) = t.parse::<i64>() {
        return Yaml::Int(i);
    }
    if let Ok(f) = t.parse::<f64>() {
        if t.contains('.') || t.contains('e') || t.contains('E') {
            return Yaml::Float(f);
        }
    }
    Yaml::String(t.to_string())
}

fn unescape_double(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn split_flow(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    for c in s.chars() {
        match c {
            '\'' if !in_double => {
                in_single = !in_single;
                cur.push(c);
            }
            '"' if !in_single => {
                in_double = !in_double;
                cur.push(c);
            }
            '[' | '{' if !in_single && !in_double => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' if !in_single && !in_double => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 && !in_single && !in_double => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out.into_iter().map(|s| s.trim().to_string()).collect()
}

/// Convenience: parse a YAML string.
pub fn load(data: &str) -> Yaml {
    parse(data)
}

/// Load into a map, returning an empty map on failure.
pub fn load_map(data: &str) -> Yaml {
    match parse(data) {
        Yaml::Map(m) => Yaml::Map(m),
        _ => Yaml::new_map(),
    }
}

/// Deep-merge helper used for configs.
pub fn merge(base: &mut Yaml, other: &Yaml) {
    if let (Yaml::Map(a), Yaml::Map(b)) = (&mut *base, other) {
        for (k, v) in b {
            if let Some(existing) = a.iter_mut().find(|(ek, _)| ek == k) {
                merge(&mut existing.1, v);
            } else {
                a.push((k.clone(), v.clone()));
            }
        }
    } else {
        *base = other.clone();
    }
}

/// Convert a map to a HashMap<String,String> of scalars.
pub fn flatten_strings(node: &Yaml) -> HashMap<String, String> {
    let mut out = HashMap::new();
    fn walk(prefix: &str, node: &Yaml, out: &mut HashMap<String, String>) {
        match node {
            Yaml::Map(entries) => {
                for (k, v) in entries {
                    let key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                    walk(&key, v, out);
                }
            }
            other => {
                if let Some(s) = other.as_string() {
                    out.insert(prefix.to_string(), s);
                }
            }
        }
    }
    walk("", node, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_map() {
        let y = parse("name: hello\nversion: 3\n");
        assert_eq!(y.get_string("name"), Some("hello".to_string()));
        assert_eq!(y.get("version"), Some(&Yaml::Int(3)));
    }

    #[test]
    fn parse_nested() {
        let text = "task:\n  type: task\n  script:\n    - narrate \"hi\"\n    - define x 3\n";
        let y = parse(text);
        let task = y.get_section("task").unwrap();
        assert_eq!(task.get_string("type"), Some("task".to_string()));
        let script = task.get_string_list("script");
        assert_eq!(script, vec!["narrate \"hi\"", "define x 3"]);
    }

    #[test]
    fn parse_list_of_maps() {
        let text = "events:\n  - on player joins:\n      script:\n        - narrate hi\n";
        let y = parse(text);
        let events = y.get("events").unwrap();
        if let Yaml::List(items) = events {
            assert_eq!(items.len(), 1);
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn roundtrip() {
        let mut m = Yaml::new_map();
        m.set("a", Yaml::String("x".into()));
        m.set("b", Yaml::Int(5));
        let s = m.save_to_string();
        let back = parse(&s);
        assert_eq!(back.get_string("a"), Some("x".to_string()));
        assert_eq!(back.get("b"), Some(&Yaml::Int(5)));
    }
}
