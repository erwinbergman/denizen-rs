//! Queue-control commands: define, if/else, while, foreach, repeat, choose, switch,
//! wait, stop, run, inject, goto/mark, random, ratelimit, and friends.

use crate::commands::registry::{register, CommandOutcome};
use crate::obj::{ElementTag, ListTag, MapTag, ObjectTag};
use crate::scripts::entry::ScriptEntry;
use crate::scripts::queue::ScriptQueue;
use crate::tags::{tag, tag_object, TagContext};
use std::sync::{Arc, RwLock};

fn arg(entry: &ScriptEntry, idx: usize) -> String {
    entry.args.get(idx).cloned().unwrap_or_default()
}

fn prefix_value(entry: &ScriptEntry, prefix: &str, ctx: &TagContext) -> Option<String> {
    entry.prefixed(prefix).map(|v| tag(v, ctx))
}

/// Build a tag context for an entry that is about to run in `queue`.
fn context_for(queue: &Arc<RwLock<ScriptQueue>>, entry: &ScriptEntry) -> TagContext {
    let q = queue.read().unwrap();
    TagContext {
        debug: entry.should_debug,
        show_errors: true,
        definition_provider: Some(Arc::new(crate::scripts::queue::QueueDefinitions { queue_id: q.id.clone() })),
        context_source: q.context_source.clone(),
        script_name: entry.script.as_ref().map(|s| s.name.clone()),
        queue_id: Some(q.id.clone()),
        entry_line: Some(entry.original_line.clone()),
    }
}

// ---------------------------------------------------------------------------
// Comparison engine (mirrors IfCommand.ArgComparer)
// ---------------------------------------------------------------------------

/// Evaluate a Denizen boolean comparison list against the tag context.
pub fn compare(args: &[String], ctx: &TagContext) -> bool {
    if args.is_empty() {
        return false;
    }
    // Handle grouping parens at top level.
    let tokens = tokenize_groups(args);
    eval_or(&tokens, ctx)
}

#[derive(Clone)]
enum Token {
    Group(Vec<Token>, bool),
    Word(String),
}

fn tokenize_groups(args: &[String]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let raw = &args[i];
        let lower = raw.to_lowercase();
        let neg = lower == "!(";
        if lower == "(" || lower == "!(" {
            let mut depth = 0i32;
            let mut inner: Vec<String> = Vec::new();
            let mut j = i;
            let mut closed = false;
            while j < args.len() {
                let l = args[j].to_lowercase();
                if l == "(" || l == "!(" {
                    depth += 1;
                    if depth > 1 {
                        inner.push(args[j].clone());
                    }
                } else if l == ")" {
                    depth -= 1;
                    if depth == 0 {
                        closed = true;
                        break;
                    }
                    inner.push(args[j].clone());
                } else {
                    inner.push(args[j].clone());
                }
                j += 1;
            }
            out.push(Token::Group(tokenize_groups(&inner), neg));
            i = if closed { j + 1 } else { args.len() };
            continue;
        }
        out.push(Token::Word(raw.clone()));
        i += 1;
    }
    out
}

fn token_word(t: &Token) -> Option<String> {
    match t {
        Token::Word(w) => Some(w.clone()),
        Token::Group(_, _) => None,
    }
}

fn eval_or(tokens: &[Token], ctx: &TagContext) -> bool {
    // Split on "||" / "or" at this level.
    let mut groups: Vec<Vec<Token>> = vec![Vec::new()];
    for t in tokens {
        if let Some(w) = token_word(t) {
            let lw = w.to_lowercase();
            if lw == "||" || lw == "or" {
                groups.push(Vec::new());
                continue;
            }
        }
        groups.last_mut().unwrap().push(t.clone());
    }
    if groups.len() == 1 {
        return eval_and(tokens, ctx);
    }
    groups.iter().any(|g| eval_and(g, ctx))
}

fn eval_and(tokens: &[Token], ctx: &TagContext) -> bool {
    let mut groups: Vec<Vec<Token>> = vec![Vec::new()];
    for t in tokens {
        if let Some(w) = token_word(t) {
            let lw = w.to_lowercase();
            if lw == "&&" || lw == "and" {
                groups.push(Vec::new());
                continue;
            }
        }
        groups.last_mut().unwrap().push(t.clone());
    }
    groups.iter().all(|g| eval_single(g, ctx))
}

fn eval_single(tokens: &[Token], ctx: &TagContext) -> bool {
    if tokens.is_empty() {
        return false;
    }
    if tokens.len() == 1 {
        return match &tokens[0] {
            Token::Group(inner, neg) => {
                let r = eval_or(inner, ctx);
                if *neg { !r } else { r }
            }
            Token::Word(w) => truthy(w, ctx),
        };
    }
    // A group mixed with words: evaluate group then remaining as boolean chain.
    if let Token::Group(inner, neg) = &tokens[0] {
        let r = eval_or(inner, ctx);
        let r = if *neg { !r } else { r };
        if tokens.len() == 1 {
            return r;
        }
        // Treat as: group <rest...> => compare group's truthiness
        return r;
    }
    // Build a flat word list for operator parsing.
    let mut words: Vec<String> = Vec::new();
    for t in tokens {
        match t {
            Token::Word(w) => words.push(w.clone()),
            Token::Group(inner, neg) => {
                let r = eval_or(inner, ctx);
                words.push(if *neg { (!r).to_string() } else { r.to_string() });
            }
        }
    }
    eval_words(&words, ctx)
}

fn eval_words(words: &[String], ctx: &TagContext) -> bool {
    // Handle `not` prefixes and keyword operators.
    let mut idx = 0;
    let mut negate = false;
    while idx < words.len() && words[idx].eq_ignore_ascii_case("not") {
        negate = !negate;
        idx += 1;
    }
    let rest = &words[idx..];
    if rest.is_empty() {
        return false;
    }
    if rest.len() == 1 {
        let r = truthy(&rest[0], ctx);
        return if negate { !r } else { r };
    }
    // Operator forms.
    let left = tag(&rest[0], ctx);
    let op_raw = rest[1].to_lowercase();
    let (op, right_start) = if op_raw == "not" && rest.len() >= 3 {
        let real = rest[2].to_lowercase();
        (format!("not {}", real), 3)
    } else {
        (op_raw, 2)
    };
    let right = if right_start < rest.len() {
        tag(&rest[right_start], ctx)
    } else {
        String::new()
    };
    let result = apply_operator(&op, &left, &right, ctx);
    if negate { !result } else { result }
}

fn apply_operator(op: &str, left: &str, right: &str, ctx: &TagContext) -> bool {
    let l = left.trim();
    let r = right.trim();
    match op {
        "==" | "=" | "equals" | "is" | "equal" => value_eq(l, r),
        "!=" | "neq" | "not equals" | "not is" => !value_eq(l, r),
        ">" | "greater" | "above" | "over" => cmp_f64(l, r).map(|o| o > 0.0).unwrap_or(l > r),
        ">=" | "at_least" | "or_more" => cmp_f64(l, r).map(|o| o >= 0.0).unwrap_or(l >= r),
        "<" | "less" | "below" | "under" => cmp_f64(l, r).map(|o| o < 0.0).unwrap_or(l < r),
        "<=" | "at_most" | "or_less" => cmp_f64(l, r).map(|o| o <= 0.0).unwrap_or(l <= r),
        "contains" | "includes" => {
            let needle = tag(right, ctx);
            l.to_lowercase().contains(&needle.to_lowercase())
        }
        "not contains" => {
            let needle = tag(right, ctx);
            !l.to_lowercase().contains(&needle.to_lowercase())
        }
        "starts_with" | "starts" => l.to_lowercase().starts_with(&r.to_lowercase()),
        "ends_with" | "ends" => l.to_lowercase().ends_with(&r.to_lowercase()),
        "matches" => regex_like(l, r),
        "in" => {
            let hay = r.to_lowercase();
            hay.split('|').any(|p| p.trim() == l.to_lowercase())
        }
        _ => value_eq(l, r),
    }
}

fn value_eq(a: &str, b: &str) -> bool {
    if a.eq_ignore_ascii_case(b) {
        return true;
    }
    if let (Ok(x), Ok(y)) = (a.parse::<f64>(), b.parse::<f64>()) {
        return (x - y).abs() < f64::EPSILON;
    }
    false
}

fn cmp_f64(a: &str, b: &str) -> Option<f64> {
    let x = a.parse::<f64>().ok()?;
    let y = b.parse::<f64>().ok()?;
    Some(x - y)
}

fn regex_like(hay: &str, pat: &str) -> bool {
    match regex::Regex::new(pat) {
        Ok(re) => re.is_match(hay),
        Err(_) => false,
    }
}

/// Truthiness of a single argument, honoring a leading `!`.
fn truthy(text: &str, ctx: &TagContext) -> bool {
    let trimmed = text.trim();
    let (neg, body) = if let Some(stripped) = trimmed.strip_prefix('!') {
        (true, stripped)
    } else {
        (false, trimmed)
    };
    let obj = tag_object(body, ctx);
    let r = obj.is_truthy();
    if neg { !r } else { r }
}

// ---------------------------------------------------------------------------
// Braced helper utilities
// ---------------------------------------------------------------------------

fn body_of(entry: &ScriptEntry) -> Vec<ScriptEntry> {
    entry.braced_body().unwrap_or_default()
}

fn make_callback(entry: &ScriptEntry, sentinel: &str) -> ScriptEntry {
    let mut cb = entry.clone();
    cb.args = vec![sentinel.to_string()];
    cb
}

fn inject_body(queue: &Arc<RwLock<ScriptQueue>>, body: Vec<ScriptEntry>) {
    queue.write().unwrap().inject_entries_at_start(body);
}

fn set_def(queue: &Arc<RwLock<ScriptQueue>>, key: &str, value: Option<ObjectTag>) {
    queue.write().unwrap().set_definition(key, value);
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

pub fn register_all() {
    register("define", "define [<name>] [<value>]", |entry, ctx, q| {
        let name = tag(&arg(entry, 0), ctx);
        let value: String = entry.args[1..].join(" ");
        let obj = tag_object(&value, ctx);
        q.write().unwrap().set_definition(&name, Some(obj));
        Ok(CommandOutcome::Continue)
    });

    register("definemap", "definemap [<name>] [<key>] [<value>]", |entry, ctx, q| {
        let name = tag(&arg(entry, 0), ctx);
        let key = tag(&arg(entry, 1), ctx);
        let value: String = entry.args[2..].join(" ");
        let obj = tag_object(&value, ctx);
        let mut q = q.write().unwrap();
        let mut m = q.get_definition(&name).and_then(|o| o.to_map()).unwrap_or_default();
        m.put_string(&key, obj);
        q.set_definition(&name, Some(ObjectTag::new(m)));
        Ok(CommandOutcome::Continue)
    });

    register("determine", "determine [<value>]", |entry, ctx, q| {
        let value: String = entry.args.join(" ");
        let obj = tag_object(&value, ctx);
        let mut q = q.write().unwrap();
        q.determined = Some(obj.clone());
        q.determinations.push(obj);
        Ok(CommandOutcome::Stop)
    });

    register("if", "if [<value>] (!)(<operator> <value>) [<commands>]", |entry, ctx, q| {
        let cond_args = entry.original_args();
        let passed = compare(&cond_args, ctx);
        if passed {
            let body = body_of(entry);
            // Consume any following else/else-if entries.
            consume_else_chain(q, true);
            if !body.is_empty() {
                inject_body(q, body);
            }
        } else {
            try_else_chain(q);
        }
        Ok(CommandOutcome::Continue)
    });

    register("else", "else (if <comparison logic>)", |_entry, ctx, q| {
        // Reaching an else normally means the preceding if ran; skip its body.
        let _ = ctx;
        let _ = q;
        Ok(CommandOutcome::Continue)
    });

    register("while", "while [stop/next/<value>] (<operator> <value>) [<commands>]", |entry, ctx, q| {
        let first = arg(entry, 0).to_lowercase();
        if first == "stop" {
            while_control(q, true);
            return Ok(CommandOutcome::Continue);
        }
        if first == "next" {
            while_control(q, false);
            return Ok(CommandOutcome::Continue);
        }
        if first == "__callback" {
            return Ok(run_while_callback(entry, q));
        }
        let cond_args = entry.original_args();
        if !compare(&cond_args, ctx) {
            return Ok(CommandOutcome::Continue);
        }
        let state_key = format!("__while_{}", entry.line_number);
        set_def(
            q,
            &format!("__while_cond_{}", entry.line_number),
            Some(ObjectTag::new(ListTag::from_strings(&cond_args))),
        );
        let count = q
            .read()
            .unwrap()
            .get_definition(&state_key)
            .and_then(|o| o.as_element().value.parse::<i64>().ok())
            .unwrap_or(0);
        if count > 100_000 {
            ctx.echo_error("While loop exceeded iteration limit.");
            set_def(q, &state_key, Some(ObjectTag::new(ElementTag::from_int(0))));
            return Ok(CommandOutcome::Continue);
        }
        set_def(q, &state_key, Some(ObjectTag::new(ElementTag::from_int(count + 1))));
        set_def(q, "loop_index", Some(ObjectTag::new(ElementTag::from_int(count + 1))));
        let mut body = body_of(entry);
        if body.is_empty() {
            ctx.echo_error("Empty while subsection - did you forget a ':'?");
            return Ok(CommandOutcome::Continue);
        }
        let mut cb = make_callback(entry, "__callback");
        cb.owner_line = entry.line_number;
        body.push(cb);
        inject_body(q, body);
        Ok(CommandOutcome::Continue)
    });

    register("foreach", "foreach [stop/next/<object>|...] (as:<name>) (key:<name>) [<commands>]", |entry, ctx, q| {
        let first = arg(entry, 0).to_lowercase();
        if first == "stop" {
            loop_control(q, "foreach", true);
            return Ok(CommandOutcome::Continue);
        }
        if first == "next" {
            loop_control(q, "foreach", false);
            return Ok(CommandOutcome::Continue);
        }
        let state_key = format!("__foreach_{}", entry.line_number);
        if first == "__callback" {
            return Ok(foreach_step(entry, q, &state_key));
        }
        let list_text: String = entry
            .args
            .iter()
            .take_while(|a| {
                let l = a.to_lowercase();
                !l.starts_with("as:") && !l.starts_with("key:") && l != "{"
            })
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        let list = tag_object(&list_text, ctx);
        let as_name = prefix_value(entry, "as", ctx).unwrap_or_else(|| "value".to_string());
        let key_name = prefix_value(entry, "key", ctx).unwrap_or_else(|| "key".to_string());
        // Build state.
        let mut state = MapTag::new();
        if let Some(map) = list.to_map() {
            let mut items = ListTag::new();
            let mut keys = ListTag::new();
            for (k, v) in &map.entries {
                keys.add_string(k);
                items.add_object(v.clone());
            }
            state.put_string("items", ObjectTag::new(items));
            state.put_string("keys", ObjectTag::new(keys));
        } else {
            state.put_string("items", ObjectTag::new(list.to_list()));
        }
        state.put_element("index", "0");
        state.put_element("value_name", &as_name);
        state.put_element("key_name", &key_name);
        set_def(q, &state_key, Some(ObjectTag::new(state)));
        Ok(foreach_step(entry, q, &state_key))
    });

    register("repeat", "repeat [stop/next/<amount>] (from:<#>) (as:<name>) [<commands>]", |entry, ctx, q| {
        let first = arg(entry, 0).to_lowercase();
        if first == "stop" {
            loop_control(q, "repeat", true);
            return Ok(CommandOutcome::Continue);
        }
        if first == "next" {
            loop_control(q, "repeat", false);
            return Ok(CommandOutcome::Continue);
        }
        let state_key = format!("__repeat_{}", entry.line_number);
        if first == "__callback" {
            return Ok(repeat_step(entry, q, &state_key));
        }
        let amount: i64 = tag(&arg(entry, 0), ctx).trim().parse().unwrap_or(0);
        if amount <= 0 {
            return Ok(CommandOutcome::Continue);
        }
        let from: i64 = prefix_value(entry, "from", ctx)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1);
        let as_name = prefix_value(entry, "as", ctx).unwrap_or_else(|| "value".to_string());
        let mut state = MapTag::new();
        state.put_element("index", &(from - 1).to_string());
        state.put_element("target", &(from + amount - 1).to_string());
        state.put_element("value_name", &as_name);
        set_def(q, &state_key, Some(ObjectTag::new(state)));
        Ok(repeat_step(entry, q, &state_key))
    });

    register("choose", "choose [<option>] [<cases>]", |entry, ctx, q| {
        let option = tag(&arg(entry, 0), ctx).to_lowercase();
        let body = body_of(entry);
        let mut default_idx: Option<usize> = None;
        let mut matched: Option<usize> = None;
        for (i, child) in body.iter().enumerate() {
            match child.command.as_str() {
                "case" => {
                    if child.args.iter().any(|a| a.to_lowercase() == option) {
                        matched = Some(i);
                        break;
                    }
                }
                "default" => default_idx = Some(i),
                _ => {}
            }
        }
        let chosen = matched.or(default_idx);
        if let Some(i) = chosen {
            let case_body = body_of(&body[i]);
            inject_body(q, case_body);
        }
        Ok(CommandOutcome::Continue)
    });

    register("switch", "switch [<value>] [<cases>]", |entry, ctx, q| {
        let value = tag(&arg(entry, 0), ctx).to_lowercase();
        let body = body_of(entry);
        let mut default_idx: Option<usize> = None;
        let mut matched: Option<usize> = None;
        for (i, child) in body.iter().enumerate() {
            match child.command.as_str() {
                "case" => {
                    if child.args.iter().any(|a| a.to_lowercase() == value) {
                        matched = Some(i);
                        break;
                    }
                }
                "default" => default_idx = Some(i),
                _ => {}
            }
        }
        if let Some(i) = matched.or(default_idx) {
            inject_body(q, body_of(&body[i]));
        }
        Ok(CommandOutcome::Continue)
    });

    register("case", "case [<value>...]", |_entry, _ctx, _q| Ok(CommandOutcome::Continue));
    register("default", "default", |_entry, _ctx, _q| Ok(CommandOutcome::Continue));

    register("wait", "wait (<duration>) (queue:<name>)", |entry, ctx, _q| {
        let dur_text = arg(entry, 0);
        if dur_text.is_empty() {
            return Ok(CommandOutcome::Wait(1));
        }
        let dur = crate::obj::DurationTag::from_identify(&tag(&dur_text, ctx));
        let millis = dur.map(|d| d.as_millis() as i64).unwrap_or(0);
        Ok(CommandOutcome::Wait(millis))
    });

    register("waituntil", "waituntil [<condition>]", |entry, ctx, q| {
        let cond = entry.original_args();
        if compare(&cond, ctx) {
            return Ok(CommandOutcome::Continue);
        }
        let mut e = entry.clone();
        e.args = cond;
        q.write().unwrap().inject_entry_at_start(e);
        Ok(CommandOutcome::Wait(50))
    });

    register("stop", "stop", |_entry, _ctx, _q| Ok(CommandOutcome::Stop));

    register("run", "run [<script>] (path:<path>) (def:<def>) (id:<id>) (instantly)", |entry, ctx, q| {
        run_script_command(entry, ctx, q, false)
    });

    register("runlater", "runlater [<script>] [delay:<duration>] (path:<path>)", |entry, ctx, q| {
        let delay_text = prefix_value(entry, "delay", ctx).unwrap_or_default();
        let delay = crate::obj::DurationTag::from_identify(&delay_text)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let script = tag(&arg(entry, 0), ctx);
        let path = prefix_value(entry, "path", ctx).unwrap_or_else(|| "script".to_string());
        let entries = crate::scripts::registry::get_container(&script)
            .and_then(|c| c.get_entries(&path))
            .map(|a| (*a).clone())
            .unwrap_or_default();
        if entries.is_empty() {
            ctx.echo_error(&format!("RunLater failed: script '{}' path '{}' not found.", script, path));
            return Ok(CommandOutcome::Continue);
        }
        let defs = collect_defs(entry, ctx);
        let queue_id = q.read().unwrap().id.clone();
        crate::scripts::queue::schedule_run(crate::util::current_time_millis() + delay, entries, defs, Some(queue_id));
        Ok(CommandOutcome::Continue)
    });

    register("goto", "goto [<name>]", |entry, ctx, q| {
        let name = tag(&arg(entry, 0), ctx).to_lowercase();
        let mut q = q.write().unwrap();
        let mut found = false;
        for e in q.entries.iter() {
            if e.command == "mark" && e.args.first().map(|a| a.to_lowercase()) == Some(name.clone()) {
                found = true;
                break;
            }
        }
        if !found {
            ctx.echo_error("Cannot go to that location - doesn't seem to exist!");
            return Ok(CommandOutcome::Continue);
        }
        while let Some(e) = q.entries.front() {
            if e.command == "mark" && e.args.first().map(|a| a.to_lowercase()) == Some(name.clone()) {
                break;
            }
            q.entries.pop_front();
        }
        Ok(CommandOutcome::Continue)
    });

    register("mark", "mark [<name>]", |_entry, _ctx, _q| Ok(CommandOutcome::Continue));

    register("random", "random [<commands>]", |entry, ctx, q| {
        let body = body_of(entry);
        if !body.is_empty() {
            use rand::Rng;
            let mut rng = rand::rng();
            let idx = rng.random_range(0..body.len());
            inject_body(q, vec![body[idx].clone()]);
            set_def(q, "possibilities", Some(ObjectTag::new(ElementTag::from_int(body.len() as i64))));
            set_def(q, "selected", Some(ObjectTag::new(ElementTag::from_int(idx as i64 + 1))));
            return Ok(CommandOutcome::Continue);
        }
        let count: i64 = tag(&arg(entry, 0), ctx).trim().parse().unwrap_or(1);
        use rand::Rng;
        let mut rng = rand::rng();
        let pick = rng.random_range(1..=count.max(1));
        set_def(q, "value", Some(ObjectTag::new(ElementTag::from_int(pick))));
        Ok(CommandOutcome::Continue)
    });

    register("inject", "inject [<script>] (path:<name>) (instantly)", |entry, ctx, q| {
        let target = tag(&arg(entry, 0), ctx);
        let path = prefix_value(entry, "path", ctx).unwrap_or_else(|| "script".to_string());
        if target.eq_ignore_ascii_case("locally") || target.eq_ignore_ascii_case("local") {
            return Ok(CommandOutcome::Continue);
        }
        let instant = entry.args.iter().any(|a| {
            let l = a.to_lowercase();
            l == "instant" || l == "instantly"
        });
        if let Some(container) = crate::scripts::registry::get_container(&target) {
            if let Some(entries) = container.get_entries(&path) {
                if instant {
                    crate::scripts::queue::run_now(q, (*entries).clone());
                } else {
                    q.write().unwrap().inject_entries_at_start((*entries).clone());
                }
            } else {
                ctx.echo_error(&format!("Script inject failed (invalid script path '{}').", path));
            }
        } else {
            ctx.echo_error(&format!("Script inject failed (invalid script '{}').", target));
        }
        Ok(CommandOutcome::Continue)
    });

    register("ratelimit", "ratelimit [<id>] [<duration>]", |entry, ctx, q| {
        let id = tag(&arg(entry, 0), ctx);
        let dur = tag(&arg(entry, 1), ctx);
        let key = format!("ratelimit_{}", id);
        let last = q.read().unwrap().get_definition(&key);
        if let Some(last) = last {
            if let Ok(last_ms) = last.identify().parse::<i64>() {
                let window = crate::obj::DurationTag::from_identify(&dur)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                if crate::util::current_time_millis() - last_ms < window {
                    return Ok(CommandOutcome::Stop);
                }
            }
        }
        set_def(q, &key, Some(ObjectTag::new(ElementTag::from_int(crate::util::current_time_millis()))));
        Ok(CommandOutcome::Continue)
    });

    register("queue", "queue [clear/stop/exists] (id:<id>)", |entry, ctx, q| {
        let action = tag(&arg(entry, 0), ctx).to_lowercase();
        let id = prefix_value(entry, "id", ctx).unwrap_or_default();
        match action.as_str() {
            "clear" => q.write().unwrap().clear(),
            "stop" => q.write().unwrap().is_stopping = true,
            "exists" => {
                let exists = crate::scripts::queue::queue_exists(&id);
                set_def(q, "value", Some(ObjectTag::new(ElementTag::from_bool(exists))));
            }
            _ => {}
        }
        Ok(CommandOutcome::Continue)
    });

    register("foreachelse", "foreachelse", |_e, _c, _q| Ok(CommandOutcome::Continue));

    register("procedure", "procedure [<script>] (def:<def>) (save:<name>)", |entry, ctx, q| {
        let script = tag(&arg(entry, 0), ctx);
        let save = prefix_value(entry, "save", ctx).unwrap_or_else(|| "value".to_string());
        let defs = collect_defs(entry, ctx);
        if let Some(container) = crate::scripts::registry::get_container(&script) {
            let result = container.run_procedure(Some(defs));
            match result {
                Some(v) => set_def(q, &save, Some(v)),
                None => ctx.echo_error(&format!("Procedure '{}' returned no value.", script)),
            }
        } else {
            ctx.echo_error(&format!("Procedure script '{}' not found.", script));
        }
        Ok(CommandOutcome::Continue)
    });

    register("task", "task [<script>] (path:<path>) (def:<def>) (instantly)", |entry, ctx, q| {
        run_script_command(entry, ctx, q, false)
    });
}

// ---------------------------------------------------------------------------
// If / else chain helpers
// ---------------------------------------------------------------------------

fn consume_else_chain(queue: &Arc<RwLock<ScriptQueue>>, _ran: bool) {
    let mut q = queue.write().unwrap();
    while let Some(front) = q.entries.front() {
        if front.command == "else" {
            q.entries.pop_front();
        } else {
            break;
        }
    }
}

fn try_else_chain(queue: &Arc<RwLock<ScriptQueue>>) {
    loop {
        let next_else = {
            let q = queue.read().unwrap();
            q.entries.front().filter(|e| e.command == "else").cloned()
        };
        let Some(else_entry) = next_else else {
            return;
        };
        queue.write().unwrap().entries.pop_front();
        // Determine whether this else has an `if` condition.
        let mut args = else_entry.args.clone();
        let has_cond = args.first().map(|a| a.eq_ignore_ascii_case("if")).unwrap_or(false);
        if has_cond {
            args.remove(0);
        }
        let ctx = context_for(queue, &else_entry);
        let passes = if has_cond { compare(&args, &ctx) } else { true };
        if passes {
            let body = body_of(&else_entry);
            // Consume remaining else entries.
            consume_else_chain(queue, true);
            if !body.is_empty() {
                inject_body(queue, body);
            }
            return;
        }
        // else-if failed: continue to next else entry.
    }
}

// ---------------------------------------------------------------------------
// While helpers
// ---------------------------------------------------------------------------

fn run_while_callback(entry: &ScriptEntry, queue: &Arc<RwLock<ScriptQueue>>) -> CommandOutcome {
    let ctx = context_for(queue, entry);
    let state_key = format!("__while_{}", entry.line_number);
    let count = queue
        .read()
        .unwrap()
        .get_definition(&state_key)
        .and_then(|o| o.as_element().value.parse::<i64>().ok())
        .unwrap_or(0);
    if count > 100_000 {
        ctx.echo_error("While loop exceeded iteration limit.");
        set_def(queue, &state_key, Some(ObjectTag::new(ElementTag::from_int(0))));
        return CommandOutcome::Continue;
    }
    // Re-evaluate using the owner's condition, stored when the loop started.
    let cond = queue
        .read()
        .unwrap()
        .get_definition(&format!("__while_cond_{}", entry.line_number))
        .and_then(|o| Some(o.to_list()))
        .map(|l| l.strings())
        .unwrap_or_default();
    if cond.is_empty() || !compare(&cond, &ctx) {
        set_def(queue, &state_key, Some(ObjectTag::new(ElementTag::from_int(0))));
        return CommandOutcome::Continue;
    }
    set_def(queue, &state_key, Some(ObjectTag::new(ElementTag::from_int(count + 1))));
    set_def(queue, "loop_index", Some(ObjectTag::new(ElementTag::from_int(count + 1))));
    let mut body = body_of(entry);
    if body.is_empty() {
        return CommandOutcome::Continue;
    }
    let mut cb = make_callback(entry, "__callback");
    cb.owner_line = entry.line_number;
    body.push(cb);
    inject_body(queue, body);
    CommandOutcome::Continue
}

fn while_control(queue: &Arc<RwLock<ScriptQueue>>, stop: bool) {
    // Remove entries up to the next while callback (and the callback itself when stopping).
    let mut q = queue.write().unwrap();
    let mut idx = None;
    for (i, e) in q.entries.iter().enumerate() {
        if e.command == "while" && e.args.first().map(|a| a == "__callback").unwrap_or(false) {
            idx = Some(i);
            break;
        }
    }
    if let Some(i) = idx {
        let count = if stop { i + 1 } else { i };
        for _ in 0..count {
            q.entries.pop_front();
        }
    }
}

// ---------------------------------------------------------------------------
// Foreach helpers
// ---------------------------------------------------------------------------

fn foreach_step(entry: &ScriptEntry, queue: &Arc<RwLock<ScriptQueue>>, state_key: &str) -> CommandOutcome {
    let state = queue.read().unwrap().get_definition(state_key).and_then(|o| o.to_map());
    let Some(mut state) = state else {
        return CommandOutcome::Continue;
    };
    let index: i64 = state.get("index").map(|o| o.as_element().value.parse().unwrap_or(0)).unwrap_or(0);
    let items = state.get("items").map(|o| o.to_list()).unwrap_or_default();
    let next_index = index + 1;
    if next_index > items.size() as i64 {
        // Loop complete: restore definitions? Leave loop_index for reference.
        set_def(queue, state_key, None);
        return CommandOutcome::Continue;
    }
    let value_name = state.get("value_name").map(|o| o.identify()).unwrap_or_else(|| "value".to_string());
    let key_name = state.get("key_name").map(|o| o.identify()).unwrap_or_else(|| "key".to_string());
    state.put_element("index", &next_index.to_string());
    set_def(queue, state_key, Some(ObjectTag::new(state.clone())));
    if let Some(keys) = state.get("keys").map(|o| o.to_list()) {
        if let Some(k) = keys.get((next_index - 1) as usize) {
            set_def(queue, &key_name, Some(k.clone()));
        }
    }
    if let Some(item) = items.get((next_index - 1) as usize) {
        set_def(queue, &value_name, Some(item.clone()));
    }
    set_def(queue, "loop_index", Some(ObjectTag::new(ElementTag::from_int(next_index))));
    let mut body = body_of(entry);
    if body.is_empty() {
        return CommandOutcome::Continue;
    }
    let mut cb = make_callback(entry, "__callback");
    cb.owner_line = entry.line_number;
    body.push(cb);
    inject_body(queue, body);
    CommandOutcome::Continue
}

// ---------------------------------------------------------------------------
// Repeat helpers
// ---------------------------------------------------------------------------

fn repeat_step(entry: &ScriptEntry, queue: &Arc<RwLock<ScriptQueue>>, state_key: &str) -> CommandOutcome {
    let state = queue.read().unwrap().get_definition(state_key).and_then(|o| o.to_map());
    let Some(mut state) = state else {
        return CommandOutcome::Continue;
    };
    let index: i64 = state.get("index").map(|o| o.as_element().value.parse().unwrap_or(0)).unwrap_or(0);
    let target: i64 = state.get("target").map(|o| o.as_element().value.parse().unwrap_or(0)).unwrap_or(0);
    let next_index = index + 1;
    if next_index > target {
        set_def(queue, state_key, None);
        return CommandOutcome::Continue;
    }
    let value_name = state.get("value_name").map(|o| o.identify()).unwrap_or_else(|| "value".to_string());
    state.put_element("index", &next_index.to_string());
    set_def(queue, state_key, Some(ObjectTag::new(state)));
    set_def(queue, &value_name, Some(ObjectTag::new(ElementTag::from_int(next_index))));
    set_def(queue, "loop_index", Some(ObjectTag::new(ElementTag::from_int(next_index))));
    let mut body = body_of(entry);
    if body.is_empty() {
        return CommandOutcome::Continue;
    }
    let mut cb = make_callback(entry, "__callback");
    cb.owner_line = entry.line_number;
    body.push(cb);
    inject_body(queue, body);
    CommandOutcome::Continue
}

fn loop_control(queue: &Arc<RwLock<ScriptQueue>>, command: &str, stop: bool) {
    let mut q = queue.write().unwrap();
    let mut idx = None;
    for (i, e) in q.entries.iter().enumerate() {
        if e.command == command && e.args.first().map(|a| a == "__callback").unwrap_or(false) {
            idx = Some(i);
            break;
        }
    }
    if let Some(i) = idx {
        let count = if stop { i + 1 } else { i };
        for _ in 0..count {
            q.entries.pop_front();
        }
    }
}

// ---------------------------------------------------------------------------
// Run helpers
// ---------------------------------------------------------------------------

fn collect_defs(entry: &ScriptEntry, ctx: &TagContext) -> MapTag {
    let mut defs = MapTag::new();
    for a in &entry.args {
        let lower = a.to_lowercase();
        // `def.<name>:<value>` (note: the prefix parser splits at the first colon).
        if lower.starts_with("def.") {
            let rest = &a[4..];
            if let Some(colon) = rest.find(':') {
                let name = &rest[..colon];
                let value = &rest[colon + 1..];
                defs.put_string(name, tag_object(value, ctx));
            }
        }
        // `def:<name>|<value>` or `def:<name>` with a following argument.
        else if lower == "def" || lower.starts_with("def:") {
            if let Some(rest) = a.split_once(':').map(|(_, v)| v) {
                if let Some(bar) = rest.find('|') {
                    let name = &rest[..bar];
                    let value = &rest[bar + 1..];
                    defs.put_string(name, tag_object(value, ctx));
                }
            }
        }
    }
    defs
}

fn run_script_command(
    entry: &ScriptEntry,
    ctx: &TagContext,
    q: &Arc<RwLock<ScriptQueue>>,
    _inject: bool,
) -> Result<CommandOutcome, String> {
    let script = tag(&arg(entry, 0), ctx);
    let instant = entry.args.iter().any(|a| {
        let l = a.to_lowercase();
        l == "instant" || l == "instantly"
    });
    let path = prefix_value(entry, "path", ctx).unwrap_or_else(|| "script".to_string());
    let defs = collect_defs(entry, ctx);
    let parent_defs = q.read().unwrap().definitions.clone();

    if let Some(container) = crate::scripts::registry::get_container(&script) {
        if let Some(entries) = container.get_entries(&path) {
            let mut merged = parent_defs;
            for (k, v) in defs.entries.iter() {
                merged.put_string(k, v.clone());
            }
            let context_source = q.read().unwrap().context_source.clone();
            if instant {
                let sub = crate::scripts::queue::run_script_entries(&entries, Some(merged), context_source);
                crate::scripts::queue::run_queue(&sub);
            } else {
                let sub = crate::scripts::queue::build_queue(&entries, Some(merged), context_source);
                crate::scripts::queue::run_queue(&sub);
            }
            return Ok(CommandOutcome::Continue);
        }
        ctx.echo_error(&format!("Script run failed (invalid path '{}').", path));
        return Ok(CommandOutcome::Continue);
    }
    // Fallback: treat the argument as a raw command line.
    let _ = crate::engine::execute_command_line(&script);
    Ok(CommandOutcome::Continue)
}

#[allow(dead_code)]
fn _unused(_m: MapTag, _l: ListTag) {}
