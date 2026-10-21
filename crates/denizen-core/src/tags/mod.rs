//! The Denizen tag engine: attribute chains, tag contexts, base-tag dispatch, and text parsing.

pub mod core;
pub mod bukkit_bases;

use crate::obj::ObjectTag;
use crate::obj::ElementTag;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

/// Provides definitions (`<[name]>`) and special definitions.
pub trait DefinitionProvider: Send + Sync {
    fn get_definition_object(&self, definition: &str) -> Option<ObjectTag>;
    fn add_definition(&self, definition: &str, value: Option<ObjectTag>);
    fn get_all_definitions(&self) -> Option<crate::obj::MapTag>;
    fn has_definition(&self, definition: &str) -> bool {
        self.get_definition_object(definition).is_some()
    }
}

/// A minimal definition provider backed by a map.
#[derive(Default, Clone)]
pub struct SimpleDefinitionProvider {
    pub map: Arc<RwLock<HashMap<String, Option<ObjectTag>>>>,
}

impl SimpleDefinitionProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DefinitionProvider for SimpleDefinitionProvider {
    fn get_definition_object(&self, definition: &str) -> Option<ObjectTag> {
        self.map.read().unwrap().get(&definition.to_lowercase()).cloned().flatten()
    }
    fn add_definition(&self, definition: &str, value: Option<ObjectTag>) {
        self.map.write().unwrap().insert(definition.to_lowercase(), value);
    }
    fn get_all_definitions(&self) -> Option<crate::obj::MapTag> {
        let mut m = crate::obj::MapTag::new();
        for (k, v) in self.map.read().unwrap().iter() {
            if let Some(v) = v {
                m.put_string(k, v.clone());
            }
        }
        Some(m)
    }
}

/// Supplies objects for `<context[...]>` lookups.
pub trait ContextSource: Send + Sync {
    fn get_context(&self, name: &str) -> Option<ObjectTag>;
}

/// Context passed down through tag parsing and command execution.
#[derive(Clone)]
pub struct TagContext {
    pub debug: bool,
    pub show_errors: bool,
    pub definition_provider: Option<Arc<dyn DefinitionProvider>>,
    pub context_source: Option<Arc<dyn ContextSource>>,
    /// Identifier of the script this context belongs to, if any.
    pub script_name: Option<String>,
    /// The queue id this context belongs to, if any.
    pub queue_id: Option<String>,
    pub entry_line: Option<String>,
}

impl Default for TagContext {
    fn default() -> Self {
        TagContext {
            debug: true,
            show_errors: true,
            definition_provider: None,
            context_source: None,
            script_name: None,
            queue_id: None,
            entry_line: None,
        }
    }
}

impl TagContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn no_debug() -> Self {
        TagContext { debug: false, show_errors: false, ..Default::default() }
    }

    pub fn should_show_errors(&self) -> bool {
        self.debug || self.show_errors
    }

    pub fn echo_error(&self, message: &str) {
        if self.should_show_errors() {
            crate::util::debug_log(&format!("[Denizen] ERROR: {}", crate::util::strip_colors(message)));
        }
    }

    pub fn get_definition(&self, name: &str) -> Option<ObjectTag> {
        self.definition_provider.as_ref().and_then(|p| p.get_definition_object(name))
    }

    pub fn with_definition_provider(mut self, provider: Arc<dyn DefinitionProvider>) -> Self {
        self.definition_provider = Some(provider);
        self
    }
}

/// A single piece of a parsed tag chain.
#[derive(Clone)]
pub struct AttributeComponent {
    pub raw_key: String,
    pub key: String,
    pub raw_param: Option<String>,
}

impl AttributeComponent {
    pub fn new(inp: &str) -> Self {
        let inp = inp.trim();
        if inp.ends_with(']') {
            if let Some(ind) = inp.find('[') {
                let raw_key = inp[..ind].to_string();
                let raw_param = inp[ind + 1..inp.len() - 1].to_string();
                let key = raw_key.to_lowercase();
                return AttributeComponent { raw_key, key, raw_param: Some(raw_param) };
            }
        }
        AttributeComponent { raw_key: inp.to_string(), key: inp.to_lowercase(), raw_param: None }
    }
}

impl std::fmt::Display for AttributeComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.raw_param {
            Some(p) => write!(f, "{}[{}]", self.key, p),
            None => write!(f, "{}", self.key),
        }
    }
}

static ATTRIBS_LOOKUP: LazyLock<Mutex<HashMap<String, Arc<Vec<AttributeComponent>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Split a tag attribute path on dots, respecting `[...]` nesting.
pub fn separate_attributes(attributes: &str) -> Result<Arc<Vec<AttributeComponent>>, String> {
    if let Some(cached) = ATTRIBS_LOOKUP.lock().unwrap().get(attributes) {
        return Ok(cached.clone());
    }
    if attributes.starts_with('.') || attributes.ends_with('.') {
        return Err(format!(
            "The tag '{}' is invalid due to a misplaced dot at the start or end of the tag.",
            attributes
        ));
    }
    let chars: Vec<char> = attributes.chars().collect();
    let mut matches = Vec::with_capacity(attributes.len() / 7);
    let mut x1 = 0usize;
    let mut x2: Option<usize> = None;
    let mut braced = 0i32;
    for (x, &chr) in chars.iter().enumerate() {
        if chr == '[' {
            braced += 1;
        }
        if x == chars.len() - 1 {
            x2 = Some(x + 1);
        }
        if chr == ']' {
            if braced > 0 {
                braced -= 1;
            }
        } else if chr == '.'
            && !(x > 0 && x + 1 < chars.len() && chars[x + 1].is_ascii_digit() && chars[x - 1].is_ascii_digit())
            && braced == 0
        {
            x2 = Some(x);
        }
        if let Some(end) = x2 {
            if end <= x1 {
                return Err(format!(
                    "The tag '{}' is invalid, likely due to double dots '..' somewhere.",
                    attributes
                ));
            }
            let piece: String = chars[x1..end].iter().collect();
            matches.push(AttributeComponent::new(&piece));
            x2 = None;
            x1 = x + 1;
        }
    }
    if braced != 0 {
        return Err(format!(
            "The tag '{}' is invalid due to misplaced [square brackets].",
            attributes
        ));
    }
    let arc = Arc::new(matches);
    ATTRIBS_LOOKUP.lock().unwrap().insert(attributes.to_string(), arc.clone());
    Ok(arc)
}

/// An attribute chain being evaluated against an object.
pub struct Attribute {
    pub attributes: Arc<Vec<AttributeComponent>>,
    pub contexts: Vec<Option<ObjectTag>>,
    pub fulfilled: usize,
    pub context: TagContext,
    pub last_valid: Option<ObjectTag>,
    pub had_alternative: bool,
    pub seeming_successes: Vec<String>,
    pub has_context_failed: bool,
    pub origin: String,
}

impl Attribute {
    pub fn new(attributes: &str, context: TagContext) -> Result<Self, String> {
        let comps = separate_attributes(attributes)?;
        let n = comps.len();
        Ok(Attribute {
            attributes: comps,
            contexts: vec![None; n],
            fulfilled: 0,
            context,
            last_valid: None,
            had_alternative: false,
            seeming_successes: Vec::new(),
            has_context_failed: false,
            origin: attributes.to_string(),
        })
    }

    pub fn from_components(components: Arc<Vec<AttributeComponent>>, context: TagContext, skippable: usize) -> Self {
        let n = components.len();
        Attribute {
            attributes: components,
            contexts: vec![None; n],
            fulfilled: skippable.min(n),
            context,
            last_valid: None,
            had_alternative: false,
            seeming_successes: Vec::new(),
            has_context_failed: false,
            origin: String::new(),
        }
    }

    /// Build a sub-attribute that continues from the current fulfillment index.
    pub fn sub_attribute(&self, skippable: usize) -> Attribute {
        Attribute {
            attributes: self.attributes.clone(),
            contexts: self.contexts.clone(),
            fulfilled: (self.fulfilled + skippable).min(self.attributes.len()),
            context: self.context.clone(),
            last_valid: self.last_valid.clone(),
            had_alternative: self.had_alternative,
            seeming_successes: Vec::new(),
            has_context_failed: false,
            origin: self.origin.clone(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.fulfilled >= self.attributes.len()
    }

    pub fn current_key(&self) -> String {
        if self.fulfilled >= self.attributes.len() {
            return String::new();
        }
        self.attributes[self.fulfilled].key.clone()
    }

    pub fn matches(&self, s: &str) -> bool {
        if self.fulfilled >= self.attributes.len() {
            return false;
        }
        self.attributes[self.fulfilled].key == s
    }

    pub fn starts_with(&self, s: &str) -> bool {
        if self.fulfilled >= self.attributes.len() {
            return false;
        }
        if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() + self.fulfilled > self.attributes.len() {
                return false;
            }
            for (i, part) in parts.iter().enumerate() {
                if self.attributes[self.fulfilled + i].key != *part {
                    return false;
                }
            }
            return true;
        }
        self.attributes[self.fulfilled].key == s
    }

    pub fn starts_with_at(&self, s: &str, attribute: i64) -> bool {
        self.get_attribute_without_param(attribute).eq_ignore_ascii_case(s)
    }

    pub fn fulfill(&mut self, count: usize) -> &mut Self {
        self.fulfilled = (self.fulfilled + count).min(self.attributes.len());
        self.seeming_successes.clear();
        self.has_context_failed = false;
        self
    }

    pub fn fulfill_one(&mut self, obj: &ObjectTag) -> &mut Self {
        self.last_valid = Some(obj.clone());
        self.fulfilled = (self.fulfilled + 1).min(self.attributes.len());
        self
    }

    pub fn track_last_tag_failure(&mut self) {
        if self.fulfilled < self.attributes.len() {
            self.seeming_successes.push(self.attributes[self.fulfilled].key.clone());
        }
    }

    pub fn has_param(&mut self) -> bool {
        if self.fulfilled >= self.attributes.len() {
            return false;
        }
        if self.attributes[self.fulfilled].raw_param.is_some() {
            return true;
        }
        self.has_context_failed = true;
        false
    }

    pub fn get_raw_param(&self) -> Option<String> {
        if self.fulfilled >= self.attributes.len() {
            return None;
        }
        self.attributes[self.fulfilled].raw_param.clone()
    }

    pub fn get_param_object(&mut self) -> Option<ObjectTag> {
        self.get_context_object(1)
    }

    pub fn get_context_object(&mut self, attribute: i64) -> Option<ObjectTag> {
        let idx = attribute + self.fulfilled as i64 - 1;
        if idx < 0 || idx >= self.attributes.len() as i64 {
            return None;
        }
        let idx = idx as usize;
        if let Some(c) = &self.contexts[idx] {
            return Some(c.clone());
        }
        let raw = self.attributes[idx].raw_param.clone()?;
        let parsed = tag_object(&raw, &self.context);
        self.contexts[idx] = Some(parsed.clone());
        Some(parsed)
    }

    pub fn get_param(&mut self) -> String {
        self.get_param_object().map(|o| o.identify()).unwrap_or_else(|| "null".to_string())
    }

    pub fn get_param_element(&mut self) -> Option<ElementTag> {
        self.get_param_object().map(|o| o.as_element())
    }

    pub fn get_long_param(&mut self) -> i64 {
        match self.get_param().parse::<i64>() {
            Ok(v) => v,
            Err(_) => {
                if !self.has_alternative() {
                    self.echo_error(&format!("Tag <{}> has invalid input - expected a (non-decimal) number.", self.to_string()));
                }
                0
            }
        }
    }

    pub fn get_int_param(&mut self) -> i32 {
        self.get_long_param() as i32
    }

    pub fn get_double_param(&mut self) -> f64 {
        match self.get_param().parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                if !self.has_alternative() {
                    self.echo_error(&format!("Tag <{}> has invalid input - expected a decimal number.", self.to_string()));
                }
                0.0
            }
        }
    }

    pub fn get_attribute(&self, num: i64) -> String {
        let idx = num + self.fulfilled as i64 - 1;
        if idx < 0 || idx >= self.attributes.len() as i64 {
            return String::new();
        }
        self.attributes[idx as usize].to_string()
    }

    pub fn get_attribute_without_param(&self, num: i64) -> String {
        let idx = num + self.fulfilled as i64 - 1;
        if idx < 0 || idx >= self.attributes.len() as i64 {
            return String::new();
        }
        self.attributes[idx as usize].key.clone()
    }

    pub fn echo_error(&self, message: &str) {
        if !self.has_alternative() {
            self.context.echo_error(message);
        }
    }

    pub fn set_had_alternative(&mut self, had: bool) {
        self.had_alternative = had;
        if had {
            self.context.debug = false;
        }
    }

    pub fn has_alternative(&self) -> bool {
        self.had_alternative || self.get_fallback_tag_index() != -1
    }

    pub fn get_fallback_tag_index(&self) -> i64 {
        for i in (self.fulfilled + 1)..self.attributes.len() {
            if FALLBACK_TAGS.with(|f| f.borrow().contains_key(&self.attributes[i].key)) {
                return i as i64;
            }
        }
        -1
    }

    pub fn filled_string(&self) -> String {
        let mut out = Vec::new();
        for i in 0..self.fulfilled.min(self.attributes.len()) {
            match &self.contexts[i] {
                Some(c) => out.push(format!("{}[{}]", self.attributes[i].key, c.identify())),
                None => out.push(self.attributes[i].to_string()),
            }
        }
        out.join(".")
    }

    pub fn unfilled_string(&self) -> String {
        let mut out = Vec::new();
        for i in self.fulfilled..self.attributes.len() {
            match &self.contexts[i] {
                Some(c) => out.push(format!("{}[{}]", self.attributes[i].key, c.identify())),
                None => out.push(self.attributes[i].to_string()),
            }
        }
        out.join(".")
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = Vec::new();
        for (i, comp) in self.attributes.iter().enumerate() {
            let mut s = comp.key.clone();
            if let Some(c) = &self.contexts[i] {
                s.push_str(&format!("[{}]", c.identify()));
            } else if let Some(p) = &comp.raw_param {
                s.push_str(&format!("[{}]", p));
            }
            out.push(s);
        }
        write!(f, "{}", out.join("."))
    }
}

thread_local! {
    static FALLBACK_TAGS: std::cell::RefCell<HashMap<String, FallbackTag>> = std::cell::RefCell::new(HashMap::new());
}

#[derive(Clone)]
pub struct FallbackTag {
    pub name: String,
    pub runner: Arc<dyn Fn(&mut Attribute) -> Option<ObjectTag> + Send + Sync>,
    pub return_type: &'static str,
}

/// Signature for a base tag handler.
pub type BaseTagRunner = Arc<dyn Fn(&mut Attribute) -> Option<ObjectTag> + Send + Sync>;

#[derive(Clone)]
pub struct TagBaseData {
    pub name: String,
    pub base_form: Option<BaseTagRunner>,
    pub return_type: &'static str,
    pub is_static: bool,
}

static BASE_TAGS: LazyLock<RwLock<HashMap<String, TagBaseData>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn register_tag_handler(return_type: &'static str, name: &str, run: BaseTagRunner, is_static: bool) {
    BASE_TAGS.write().unwrap().insert(
        name.to_string(),
        TagBaseData { name: name.to_string(), base_form: Some(run), return_type, is_static },
    );
}

pub fn register_tag_handler_with_param(
    return_type: &'static str,
    param_type: &'static str,
    name: &str,
    run: Arc<dyn Fn(&mut Attribute, ObjectTag) -> Option<ObjectTag> + Send + Sync>,
    is_static: bool,
) {
    let name_owned = name.to_string();
    register_tag_handler(
        return_type,
        name,
        Arc::new(move |attr: &mut Attribute| {
            if !attr.has_param() {
                return None;
            }
            let param = attr.get_param_object()?;
            let converted = param.as_type(param_type, &attr.context)?;
            run(attr, converted)
        }),
        is_static,
    );
    let _ = name_owned;
}

pub fn get_base_tag(name: &str) -> Option<TagBaseData> {
    BASE_TAGS.read().unwrap().get(name).cloned()
}

pub fn init_fallbacks() {
    FALLBACK_TAGS.with(|f| {
        let mut f = f.borrow_mut();
        f.insert(
            "if_null".to_string(),
            FallbackTag {
                name: "if_null".to_string(),
                runner: Arc::new(|attr: &mut Attribute| {
                    if !attr.has_param() {
                        return None;
                    }
                    attr.get_param_object()
                }),
                return_type: "ObjectTag",
            },
        );
        f.insert(
            "exists".to_string(),
            FallbackTag {
                name: "exists".to_string(),
                runner: Arc::new(|_attr: &mut Attribute| Some(ObjectTag::new(ElementTag::from_bool(false)))),
                return_type: "ElementTag",
            },
        );
        f.insert(
            "is_truthy".to_string(),
            FallbackTag {
                name: "is_truthy".to_string(),
                runner: Arc::new(|_attr: &mut Attribute| Some(ObjectTag::new(ElementTag::from_bool(false)))),
                return_type: "ElementTag",
            },
        );
    });
}

/// Result of parsing text into tag pieces.
#[derive(Clone)]
pub struct ParseableTag {
    pub raw_object: Option<ObjectTag>,
    pub pieces: Vec<ParseableTagPiece>,
    pub single_tag: Option<ParseableTagPiece>,
    pub has_tag: bool,
}

#[derive(Clone)]
pub struct ParseableTagPiece {
    pub content: String,
    pub is_tag: bool,
    pub is_error: bool,
    pub raw_tag: Option<String>,
    pub raw_object: Option<ObjectTag>,
}

impl ParseableTag {
    pub fn plain(text: &str) -> Self {
        ParseableTag {
            raw_object: Some(ObjectTag::new(ElementTag::raw(text))),
            pieces: Vec::new(),
            single_tag: None,
            has_tag: false,
        }
    }

    pub fn parse(&self, context: &TagContext) -> ObjectTag {
        if let Some(obj) = &self.raw_object {
            return obj.clone();
        }
        if let Some(single) = &self.single_tag {
            return read_single_tag(single, context);
        }
        parse_chain_object(&self.pieces, context)
    }
}

static PRE_CALCED: LazyLock<RwLock<HashMap<String, ParseableTag>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn parse_text_to_tag(arg: &str, context: &TagContext) -> ParseableTag {
    if let Some(cached) = PRE_CALCED.read().unwrap().get(arg) {
        return cached.clone();
    }
    let result = parse_text_to_tag_internal(arg, context, false);
    let mut w = PRE_CALCED.write().unwrap();
    if w.len() < 100_000 {
        w.insert(arg.to_string(), result.clone());
    }
    result
}

fn valid_tag_first_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '&' || c == '_' || c == '['
}

fn locate_tag(arg: &str, start: usize, use_brace: bool) -> Option<(usize, usize)> {
    let chars: Vec<char> = arg.chars().collect();
    let len = chars.len();
    let mut first = None;
    let mut i = start;
    while i < len {
        if chars[i] == '<' && (!use_brace || (i + 1 < len && chars[i + 1] == '{')) {
            first = Some(i);
            break;
        }
        i += 1;
    }
    let first = first?;
    if !use_brace && first + 1 < len && !valid_tag_first_char(chars[first + 1]) {
        // Not a tag start, continue searching after this '<'.
        return locate_tag(arg, first + 1, false);
    }
    let mut bracks = 1;
    let mut i = first + 1;
    while i < len {
        if chars[i] == '<' && (!use_brace || (i + 1 < len && chars[i + 1] == '{')) {
            bracks += 1;
        } else if chars[i] == '>' && (!use_brace || chars[i - 1] == '}') {
            bracks -= 1;
            if bracks == 0 {
                return Some((first, i));
            }
        }
        i += 1;
    }
    None
}

fn char_substring(arg: &str, start: usize, end: usize) -> String {
    arg.chars().skip(start).take(end.saturating_sub(start)).collect()
}

fn parse_text_to_tag_internal(arg: &str, context: &TagContext, use_brace: bool) -> ParseableTag {
    let mut pieces: Vec<ParseableTagPiece> = Vec::new();
    if !arg.contains('>') || arg.chars().count() < 3 {
        return ParseableTag::plain(arg);
    }
    let mut current = arg.to_string();
    let mut first = locate_tag(&current, 0, use_brace);
    if first.is_none() {
        return ParseableTag::plain(arg);
    }
    while let Some((start, end)) = first {
        if start > 0 {
            pieces.push(ParseableTagPiece {
                content: char_substring(&current, 0, start),
                is_tag: false,
                is_error: false,
                raw_tag: None,
                raw_object: None,
            });
        }
        let tag_start = start + if use_brace { 2 } else { 1 };
        let tag_end = end - if use_brace { 1 } else { 0 };
        let tag_to_proc = char_substring(&current, tag_start, tag_end);
        pieces.push(ParseableTagPiece {
            content: tag_to_proc.clone(),
            is_tag: true,
            is_error: false,
            raw_tag: Some(tag_to_proc.clone()),
            raw_object: None,
        });
        current = char_substring(&current, end + 1, current.chars().count());
        first = locate_tag(&current, 0, use_brace);
    }
    if !current.is_empty() {
        pieces.push(ParseableTagPiece {
            content: current.clone(),
            is_tag: false,
            is_error: false,
            raw_tag: None,
            raw_object: None,
        });
    }
    // Simplify adjacent plaintext pieces.
    let mut i = 1;
    while i < pieces.len() {
        if !pieces[i - 1].is_tag && !pieces[i - 1].is_error && !pieces[i].is_tag && !pieces[i].is_error {
            let merged = format!("{}{}", pieces[i - 1].content, pieces[i].content);
            pieces[i - 1] = ParseableTagPiece {
                content: merged.clone(),
                is_tag: false,
                is_error: false,
                raw_tag: None,
                raw_object: Some(ObjectTag::new(ElementTag::raw(&merged))),
            };
            pieces.remove(i);
        } else {
            i += 1;
        }
    }
    let has_tag = pieces.iter().any(|p| p.is_tag);
    let mut result = ParseableTag { raw_object: None, pieces: pieces.clone(), single_tag: None, has_tag };
    if pieces.len() == 1 {
        let p = &pieces[0];
        if p.is_tag {
            result.single_tag = Some(p.clone());
        } else {
            result.raw_object = p.raw_object.clone();
            result.has_tag = false;
        }
    }
    let _ = context;
    result
}

/// Parse a string and return the resulting object.
pub fn tag_object(arg: &str, context: &TagContext) -> ObjectTag {
    parse_text_to_tag(arg, context).parse(context)
}

/// Parse a string and return its resulting string value.
pub fn tag(arg: &str, context: &TagContext) -> String {
    tag_object(arg, context).identify()
}

fn parse_chain_object(pieces: &[ParseableTagPiece], context: &TagContext) -> ObjectTag {
    if pieces.is_empty() {
        return ObjectTag::new(ElementTag::raw(""));
    }
    if pieces.len() < 2 {
        let p = &pieces[0];
        if p.is_error {
            context.echo_error(&p.content);
        } else if p.is_tag {
            return read_single_tag(p, context);
        }
        return ObjectTag::new(ElementTag::raw(&pieces[0].content));
    }
    let mut out = String::new();
    for p in pieces {
        if p.is_error {
            context.echo_error(&p.content);
        } else if p.is_tag {
            out.push_str(&read_single_tag(p, context).identify());
        } else {
            out.push_str(&p.content);
        }
    }
    ObjectTag::new(ElementTag::raw(&out))
}

/// Execute a single parsed tag piece.
pub fn read_single_tag(piece: &ParseableTagPiece, context: &TagContext) -> ObjectTag {
    let raw = piece.raw_tag.clone().unwrap_or_else(|| piece.content.clone());
    read_raw_tag(&raw, context)
}

/// Execute a raw tag string (without the surrounding `<` `>`).
pub fn read_raw_tag(raw: &str, context: &TagContext) -> ObjectTag {
    match evaluate_raw_tag(raw, context) {
        Some(obj) => obj,
        None => ObjectTag::new(ElementTag::raw(raw)),
    }
}

/// Core single-tag evaluation. Returns None when the tag is invalid/unhandled.
pub fn evaluate_raw_tag(raw: &str, context: &TagContext) -> Option<ObjectTag> {
    let (tag_body, alternative) = split_alternative(raw);
    let (tag_body, _value) = split_value(&tag_body);
    let tag_body = tag_body.trim().to_string();

    let mut attr = Attribute::new(&tag_body, context.clone()).ok()?;
    attr.set_had_alternative(alternative.is_some());
    let base_name = attr.attributes.first().map(|c| c.key.clone()).unwrap_or_default();

    let result = run_base_tag(&base_name, &mut attr);
    if let Some(obj) = result {
        return Some(obj);
    }
    // Alternative handling.
    if let Some(alt) = alternative {
        return Some(tag_object(&alt, context));
    }
    let index = attr.get_fallback_tag_index();
    if index >= 0 {
        let name = attr.attributes[index as usize].key.clone();
        attr.fulfilled = index as usize;
        if let Some(fb) = FALLBACK_TAGS.with(|f| f.borrow().get(&name).cloned()) {
            return (fb.runner)(&mut attr);
        }
    }
    None
}

fn split_alternative(raw: &str) -> (String, Option<String>) {
    let chars: Vec<char> = raw.chars().collect();
    let mut bracks = 0i32;
    let mut bracks2 = 0i32;
    let mut previous_was_target = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' {
            bracks += 1;
        } else if c == '>' {
            bracks -= 1;
        } else if bracks == 0 && c == '[' {
            bracks2 += 1;
        } else if bracks == 0 && c == ']' {
            bracks2 -= 1;
        } else if c == '|' && bracks == 0 && bracks2 == 0 {
            if previous_was_target {
                let body: String = chars[..i - 1].iter().collect();
                let alt: String = chars[i + 1..].iter().collect();
                return (body, Some(alt.trim().to_string()));
            }
            previous_was_target = true;
        } else {
            previous_was_target = false;
        }
        i += 1;
    }
    (raw.to_string(), None)
}

fn split_value(raw: &str) -> (String, Option<String>) {
    let chars: Vec<char> = raw.chars().collect();
    let mut bracks = 0i32;
    let mut bracks2 = 0i32;
    for (i, &c) in chars.iter().enumerate() {
        if c == '<' {
            bracks += 1;
        } else if c == '>' {
            bracks -= 1;
        } else if bracks == 0 && c == '[' {
            bracks2 += 1;
        } else if bracks == 0 && c == ']' {
            bracks2 -= 1;
        } else if c == ':' && bracks == 0 && bracks2 == 0 {
            let body: String = chars[..i].iter().collect();
            let value: String = chars[i + 1..].iter().collect();
            return (body, Some(value));
        }
    }
    (raw.to_string(), None)
}

/// Run a base tag handler, chaining the resulting object's attributes.
pub fn run_base_tag(name: &str, attr: &mut Attribute) -> Option<ObjectTag> {
    if let Some(base) = get_base_tag(name) {
        if let Some(runner) = base.base_form {
            let result = (runner)(attr)?;
            attr.fulfill(1);
            if attr.is_complete() {
                return Some(result);
            }
            return result.get_object_attribute(attr);
        }
    }
    // Empty base = definition tag (`<[name]>`).
    if name.is_empty() {
        if let Some(obj) = crate::tags::core::definition_tag(attr) {
            attr.fulfill(1);
            if attr.is_complete() {
                return Some(obj);
            }
            return obj.get_object_attribute(attr);
        }
    }
    None
}

/// Debug helper.
pub mod debug {
    pub use crate::util::debug_log;
}

/// Re-export object tag types.
pub use crate::obj::{ElementTag as Elem, ListTag, MapTag};

/// Register extra base tags (object constructors, math). Called from obj/core/mod.rs::register().
pub fn register_extra_base_tags() {
    bukkit_bases::register();
}
