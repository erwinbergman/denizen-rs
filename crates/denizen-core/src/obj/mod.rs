//! The Denizen object system: `ObjectTag`, object types, and the per-type tag/mechanism processors.

pub mod core;

use crate::tags::{Attribute, TagContext};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

pub use core::element::ElementTag;
pub use core::list::ListTag;
pub use core::map::MapTag;
pub use core::duration::DurationTag;
pub use core::vector::VectorObject;
pub use core::color::ColorTag;
pub use core::time::TimeTag;

/// The base trait implemented by every Denizen object type.
pub trait ObjectTagImpl: Send + Sync {
    /// The short prefix used for debugging (eg "List").
    fn prefix(&self) -> String;

    /// A precise, reconstructable string form (eg `li@a|b`).
    fn identify(&self) -> String;

    /// A simpler string form.
    fn identify_simple(&self) -> String {
        self.identify()
    }

    /// Debug-oriented string form.
    fn debuggable(&self) -> String {
        self.identify()
    }

    /// Short name of the object type (eg "ElementTag", "ListTag").
    fn object_type_name(&self) -> &'static str;

    /// Support for downcasting to the concrete type.
    fn as_any(&self) -> &dyn Any;

    /// Whether this object represents a unique instance in the world.
    fn is_unique(&self) -> bool {
        false
    }

    /// "Truthiness" of the value.
    fn is_truthy(&self) -> bool {
        true
    }

    /// Produce a copy of this object.
    fn duplicate(&self) -> ObjectTag;

    /// Refresh any cached state (eg re-read a live entity).
    fn refresh_state(&self) -> ObjectTag {
        self.duplicate()
    }

    /// Conversion to a plain element.
    fn as_element(&self) -> ElementTag {
        ElementTag::new(self.identify())
    }

    /// The object used for the next type "down" in the fallback chain.
    fn next_object_type_down(&self) -> ObjectTag {
        ObjectTag::new(self.as_element())
    }

    /// Called for values that should be considered raw text input.
    fn as_raw(&self) -> ObjectTag {
        self.duplicate()
    }
}

/// A reference-counted Denizen object.
#[derive(Clone)]
pub struct ObjectTag(pub Arc<dyn ObjectTagImpl>);

impl ObjectTag {
    pub fn new<T: ObjectTagImpl + 'static>(value: T) -> Self {
        ObjectTag(Arc::new(value))
    }

    pub fn from_arc(value: Arc<dyn ObjectTagImpl>) -> Self {
        ObjectTag(value)
    }

    pub fn prefix(&self) -> String {
        self.0.prefix()
    }

    pub fn identify(&self) -> String {
        self.0.identify()
    }

    pub fn identify_simple(&self) -> String {
        self.0.identify_simple()
    }

    pub fn debuggable(&self) -> String {
        self.0.debuggable()
    }

    pub fn savable(&self) -> String {
        self.0.refresh_state().identify()
    }

    pub fn object_type_name(&self) -> &'static str {
        self.0.object_type_name()
    }

    pub fn is_unique(&self) -> bool {
        self.0.is_unique()
    }

    pub fn is_truthy(&self) -> bool {
        self.0.is_truthy()
    }

    pub fn duplicate(&self) -> ObjectTag {
        self.0.duplicate()
    }

    pub fn refresh_state(&self) -> ObjectTag {
        self.0.refresh_state()
    }

    pub fn as_element(&self) -> ElementTag {
        self.0.as_element()
    }

    pub fn next_object_type_down(&self) -> ObjectTag {
        self.0.next_object_type_down()
    }

    pub fn downcast_ref<T: ObjectTagImpl + 'static>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref::<T>()
    }

    pub fn is<T: ObjectTagImpl + 'static>(&self) -> bool {
        self.0.as_any().is::<T>()
    }

    /// Returns a string usable by scripts.
    pub fn as_string(&self) -> String {
        self.identify()
    }

    /// Convert to an ElementTag.
    pub fn to_element(&self) -> ElementTag {
        self.as_element()
    }

    /// Convert to a ListTag (parsing the element form when needed).
    pub fn to_list(&self) -> ListTag {
        if let Some(l) = self.downcast_ref::<ListTag>() {
            return l.clone();
        }
        ListTag::from_identify(&self.identify())
    }

    /// Convert to a MapTag.
    pub fn to_map(&self) -> Option<MapTag> {
        if let Some(m) = self.downcast_ref::<MapTag>() {
            return Some(m.clone());
        }
        MapTag::from_identify(&self.identify())
    }

    /// Convert to a DurationTag.
    pub fn to_duration(&self) -> Option<DurationTag> {
        if let Some(d) = self.downcast_ref::<DurationTag>() {
            return Some(d.clone());
        }
        DurationTag::from_identify(&self.identify())
    }

    /// Resolve the object's tag chain.
    pub fn get_object_attribute(&self, attribute: &mut Attribute) -> Option<ObjectTag> {
        dispatch_object_attribute(self, attribute)
    }

    /// Generic type conversion. Returns None when not convertible.
    pub fn as_type(&self, type_name: &str, context: &TagContext) -> Option<ObjectTag> {
        if self.object_type_name() == type_name {
            return Some(self.clone());
        }
        let ot = get_object_type(type_name)?;
        if let Some(conv) = ot.converter {
            return conv(self, context);
        }
        Some(ObjectTag::new(ElementTag::new(self.identify())))
    }
}

impl std::fmt::Display for ObjectTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identify())
    }
}

impl std::fmt::Debug for ObjectTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectTag({})", self.identify())
    }
}

/// Signature for a per-object tag implementation.
pub type ObjectTagRunner = Arc<dyn Fn(&mut Attribute, &ObjectTag) -> Option<ObjectTag> + Send + Sync>;

/// Signature for a mechanism implementation.
pub type MechanismRunner = Arc<dyn Fn(&ObjectTag, &mut crate::obj::Mechanism) + Send + Sync>;

/// Data describing a registered object tag.
#[derive(Clone)]
pub struct TagData {
    pub name: String,
    pub runner: ObjectTagRunner,
    pub return_type: &'static str,
    pub is_static: bool,
}

/// Data describing a registered mechanism.
#[derive(Clone)]
pub struct MechanismData {
    pub name: String,
    pub runner: MechanismRunner,
    pub allow_property: bool,
}

/// Per-object-type tag & mechanism registry.
#[derive(Default)]
pub struct ObjectTagProcessor {
    pub tags: RwLock<HashMap<String, TagData>>,
    pub mechanisms: RwLock<HashMap<String, MechanismData>>,
    /// Custom advanced matchers registered for this type.
    pub custom_matchers: RwLock<Vec<Arc<dyn Fn(&ObjectTag, &str) -> Option<bool> + Send + Sync>>>,
}

impl ObjectTagProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_tag(
        &self,
        return_type: &'static str,
        name: &str,
        runner: impl Fn(&mut Attribute, &ObjectTag) -> Option<ObjectTag> + Send + Sync + 'static,
        is_static: bool,
        deprecated: &[&str],
    ) {
        let key = name.to_lowercase();
        let data = TagData { name: key.clone(), runner: Arc::new(runner), return_type, is_static };
        self.tags.write().unwrap().insert(key.clone(), data.clone());
        for dep in deprecated {
            let dep_key = dep.to_lowercase();
            let inner = data.runner.clone();
            let proper = key.clone();
            let dep_for_msg = dep_key.clone();
            let runner2 = Arc::new(move |attr: &mut Attribute, obj: &ObjectTag| {
                attr.echo_error(&format!("Using deprecated form of tag '{}': '{}'.", proper, dep_for_msg));
                (inner)(attr, obj)
            });
            self.tags.write().unwrap().insert(dep_key.clone(), TagData { name: dep_key, runner: runner2, return_type, is_static: false });
        }
    }

    pub fn register_tag_with_param(
        &self,
        return_type: &'static str,
        param_type: &'static str,
        name: &str,
        runner: impl Fn(&mut Attribute, &ObjectTag, ObjectTag) -> Option<ObjectTag> + Send + Sync + 'static,
        is_static: bool,
        deprecated: &[&str],
    ) {
        let name_owned = name.to_string();
        self.register_tag(
            return_type,
            name,
            move |attr: &mut Attribute, obj: &ObjectTag| {
                if !attr.has_param() {
                    return None;
                }
                let param = attr.get_param_object()?;
                let converted = param.as_type(param_type, &attr.context)?;
                runner(attr, obj, converted)
            },
            is_static,
            deprecated,
        );
        let _ = name_owned;
    }

    pub fn register_mechanism(
        &self,
        name: &str,
        allow_property: bool,
        runner: impl Fn(&ObjectTag, &mut crate::obj::Mechanism) + Send + Sync + 'static,
        deprecated: &[&str],
    ) {
        let key = name.to_lowercase();
        let data = MechanismData { name: key.clone(), runner: Arc::new(runner), allow_property };
        self.mechanisms.write().unwrap().insert(key.clone(), data.clone());
        for dep in deprecated {
            self.mechanisms.write().unwrap().insert(dep.to_lowercase(), MechanismData { name: dep.to_lowercase(), ..data.clone() });
        }
    }
}

/// Descriptor for an object type.
pub struct ObjectType {
    pub name: &'static str,
    pub short_name: &'static str,
    pub prefix: &'static str,
    pub processor: Arc<ObjectTagProcessor>,
    /// Conversion function from any object to this type.
    pub converter: Option<fn(&ObjectTag, &TagContext) -> Option<ObjectTag>>,
    /// Checks whether a value should be considered this type.
    pub should_be_checker: Option<fn(&ObjectTag) -> bool>,
}

static OBJECT_TYPES: LazyLock<RwLock<HashMap<&'static str, Arc<ObjectType>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static PREFIX_MAP: LazyLock<RwLock<HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register an object type. Re-registration replaces the previous descriptor.
pub fn register_object_type(ot: ObjectType) {
    let arc = Arc::new(ot);
    PREFIX_MAP.write().unwrap().insert(arc.prefix, arc.name);
    OBJECT_TYPES.write().unwrap().insert(arc.name, arc);
}

pub fn get_object_type(name: &str) -> Option<Arc<ObjectType>> {
    OBJECT_TYPES.read().unwrap().get(name).cloned()
}

/// Number of registered object types.
pub fn object_type_count() -> usize {
    OBJECT_TYPES.read().unwrap().len()
}

/// All registered object type names.
pub fn object_type_names() -> Vec<&'static str> {
    OBJECT_TYPES.read().unwrap().keys().copied().collect()
}

pub fn get_processor(type_name: &str) -> Arc<ObjectTagProcessor> {
    if let Some(ot) = get_object_type(type_name) {
        return ot.processor.clone();
    }
    static FALLBACK: LazyLock<Arc<ObjectTagProcessor>> = LazyLock::new(|| Arc::new(ObjectTagProcessor::new()));
    FALLBACK.clone()
}

pub fn type_for_prefix(prefix: &str) -> Option<&'static str> {
    PREFIX_MAP.read().unwrap().get(prefix).copied()
}

/// Dispatch a tag lookup against an object.
pub fn dispatch_object_attribute(obj: &ObjectTag, attribute: &mut Attribute) -> Option<ObjectTag> {
    if attribute.is_complete() {
        return Some(obj.clone());
    }
    let key = attribute.current_key();
    attribute.last_valid = Some(obj.clone());

    let processor = get_processor(obj.object_type_name());
    let data = processor.tags.read().unwrap().get(&key).cloned();
    if let Some(data) = data {
        if let Some(returned) = (data.runner)(attribute, obj) {
            attribute.fulfill(1);
            return attribute_chain(&returned, attribute);
        }
        attribute.track_last_tag_failure();
        return None;
    }

    if let Some(result) = core_object_tag(obj, &key) {
        attribute.fulfill(1);
        return attribute_chain(&result, attribute);
    }

    // Try mechanisms-as-properties and the fallback chain.
    let next = obj.next_object_type_down();
    if next.object_type_name() == obj.object_type_name() {
        attribute.track_last_tag_failure();
        return None;
    }
    dispatch_object_attribute(&next, attribute)
}

fn attribute_chain(obj: &ObjectTag, attribute: &mut Attribute) -> Option<ObjectTag> {
    if attribute.is_complete() {
        return Some(obj.clone());
    }
    dispatch_object_attribute(obj, attribute)
}

/// Tags available on every object type.
fn core_object_tag(obj: &ObjectTag, key: &str) -> Option<ObjectTag> {
    match key {
        "prefix" => Some(ObjectTag::new(ElementTag::new(obj.prefix()))),
        "object_type" | "type" => Some(ObjectTag::new(ElementTag::new(obj.object_type_name().to_string()))),
        _ => None,
    }
}

/// A mechanism application request.
pub struct Mechanism {
    pub raw_name: String,
    pub value: Option<ObjectTag>,
    pub adjusting: Option<ObjectTag>,
    pub context: TagContext,
    pub is_property: bool,
    pub fulfilled: bool,
    pub error: Option<String>,
}

impl Mechanism {
    pub fn new(name: &str, value: Option<ObjectTag>, context: TagContext) -> Self {
        Mechanism {
            raw_name: name.to_lowercase(),
            value,
            adjusting: None,
            context,
            is_property: false,
            fulfilled: false,
            error: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.raw_name
    }

    pub fn fulfill(&mut self) {
        self.fulfilled = true;
    }

    pub fn get_value(&self) -> ElementTag {
        match &self.value {
            Some(v) => v.as_element(),
            None => ElementTag::new(""),
        }
    }

    pub fn has_value(&self) -> bool {
        match &self.value {
            None => false,
            Some(v) => !v.as_element().as_string().is_empty(),
        }
    }

    pub fn matches(&mut self, name: &str) -> bool {
        if self.raw_name == name {
            self.fulfill();
            true
        } else {
            false
        }
    }

    pub fn echo_error(&mut self, message: &str) {
        if self.error.is_none() {
            self.error = Some(message.to_string());
        }
        self.context.echo_error(message);
    }
}

/// Apply a mechanism to an object, dispatching to its registered handler.
pub fn process_mechanism(obj: &ObjectTag, mechanism: &mut Mechanism) {
    let processor = get_processor(obj.object_type_name());
    let data = processor.mechanisms.read().unwrap().get(mechanism.name()).cloned();
    match data {
        Some(data) => {
            mechanism.fulfill();
            if mechanism.is_property && !data.allow_property {
                mechanism.echo_error(&format!("Error: mechanism '{}' may not be used as property input.", data.name));
                return;
            }
            (data.runner)(obj, mechanism);
        }
        None => {
            mechanism.echo_error(&format!("Invalid mechanism '{}' for object type '{}'.", mechanism.name(), obj.object_type_name()));
        }
    }
}

/// Register all core object types. Called during engine init.
///
/// Idempotent: repeated calls are no-ops. This is important because tests (and hosts) may
/// call it more than once, and re-creating the object-type processors would race with and
/// discard any tags registered by other threads.
pub fn register_core_objects() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        core::register();
    });
}
