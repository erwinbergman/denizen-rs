//! Logging and color handling used across the engine.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(true);

type LogSink = Arc<dyn Fn(&str) + Send + Sync>;

static LOG_SINK: LazyLock<Mutex<Option<LogSink>>> = LazyLock::new(|| Mutex::new(None));

/// Enable or disable debug logging.
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
}

pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::SeqCst)
}

/// Install a sink that receives log lines (eg to forward to a Java logger).
pub fn set_log_sink(sink: Option<LogSink>) {
    *LOG_SINK.lock().unwrap() = sink;
}

/// Log a message. Routes to the installed sink, else stdout.
pub fn debug_log(message: &str) {
    if let Some(sink) = LOG_SINK.lock().unwrap().as_ref() {
        sink(message);
    } else {
        println!("{}", strip_colors(message));
    }
}

/// Remove Denizen color codes like `<&co>` and Minecraft `§` codes.
pub fn strip_colors(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let chars: Vec<char> = message.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '§' {
            i += 2;
            continue;
        }
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '&' {
            if let Some(end) = chars[i..].iter().position(|&c| c == '>') {
                i += end + 1;
                continue;
            }
        }
        if chars[i] == '<' && i + 2 < chars.len() && chars[i + 2] == '>' {
            let tag: String = chars[i + 1..i + 2].iter().collect();
            if is_color_tag(&tag) {
                i += 3;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn is_color_tag(tag: &str) -> bool {
    matches!(
        tag,
        "W" | "Y" | "R" | "G" | "B" | "A" | "C" | "D" | "E" | "F" | "L" | "S"
            | "LR" | "LG" | "LY" | "LB" | "LA" | "LC" | "LD" | "LE" | "LF" | "LL" | "LS"
            | "GR" | "GG" | "GB" | "GA" | "GC" | "GD" | "GE" | "GF" | "GL" | "GS"
    )
}

/// Colorize text by replacing Denizen color codes with ANSI escapes (console output).
pub fn apply_ansi_colors(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    let chars: Vec<char> = message.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '&' {
            if let Some(end) = chars[i..].iter().position(|&c| c == '>') {
                let name: String = chars[i + 2..i + end].iter().collect();
                out.push_str(&denizen_code_to_ansi(&name));
                i += end + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn denizen_code_to_ansi(name: &str) -> String {
    match name {
        "co" => "\u{1b}[0m".to_string(),
        "ns" => "\u{1b}[0m".to_string(),
        "dq" => "\"".to_string(),
        "dot" => ".".to_string(),
        "amp" => "&".to_string(),
        "base" => "\u{1b}[0m".to_string(),
        "emphasis" => "\u{1b}[1m".to_string(),
        "error" => "\u{1b}[31m".to_string(),
        "warning" => "\u{1b}[33m".to_string(),
        "success" => "\u{1b}[32m".to_string(),
        _ => {
            if let Some(stripped) = name.strip_prefix('#') {
                if let Some(c) = crate::obj::ColorTag::from_hex(stripped) {
                    return format!("\u{1b}[38;2;{};{};{}m", c.red, c.green, c.blue);
                }
            }
            String::new()
        }
    }
}
