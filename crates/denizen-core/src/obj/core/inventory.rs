//! `InventoryTag`: a collection of item slots, identified as `in@<type>[...]`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// Mutable inventory contents keyed by identity string.
static INVENTORIES: LazyLock<RwLock<HashMap<String, Vec<Option<crate::obj::core::item::ItemTag>>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// An inventory object.
#[derive(Clone, Debug)]
pub struct InventoryTag {
    pub id_type: String,
    pub size: usize,
    pub items: Vec<Option<crate::obj::core::item::ItemTag>>,
    pub prefix: String,
}

impl InventoryTag {
    pub fn new(id_type: &str, size: usize) -> Self {
        InventoryTag {
            id_type: id_type.to_lowercase(),
            size,
            items: vec![None; size],
            prefix: "Inventory".to_string(),
        }
    }

    pub fn generic() -> Self {
        InventoryTag::new("generic", 27)
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 3 && s[..3].eq_ignore_ascii_case("in@") {
            s = &s[3..];
        }
        if s.is_empty() {
            return None;
        }
        let (id_type, props) = match s.find('[') {
            Some(i) if s.ends_with(']') => (&s[..i], Some(&s[i + 1..s.len() - 1])),
            _ => (s, None),
        };
        let mut inv = InventoryTag::new(id_type, default_size(id_type));
        if let Some(props) = props {
            for entry in props.split(';') {
                if let Some((k, v)) = entry.split_once('=') {
                    if k.trim().eq_ignore_ascii_case("size") {
                        if let Ok(n) = v.trim().parse::<usize>() {
                            inv.size = n;
                            inv.items = vec![None; n];
                        }
                    }
                }
            }
        }
        Some(inv)
    }

    pub fn set_slot(&mut self, slot: usize, item: Option<crate::obj::core::item::ItemTag>) {
        if slot < self.items.len() {
            self.items[slot] = item;
        }
    }

    pub fn get_slot(&self, slot: usize) -> Option<&crate::obj::core::item::ItemTag> {
        self.items.get(slot).and_then(|x| x.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|i| match i {
            None => true,
            Some(it) => it.is_empty(),
        })
    }

    pub fn empty_slots(&self) -> i64 {
        self.items
            .iter()
            .filter(|i| match i {
                None => true,
                Some(it) => it.is_empty(),
            })
            .count() as i64
    }

    pub fn first_empty(&self) -> i64 {
        self.items
            .iter()
            .position(|i| match i {
                None => true,
                Some(it) => it.is_empty(),
            })
            .map(|x| x as i64)
            .unwrap_or(-1)
    }

    pub fn save(&self) {
        INVENTORIES.write().unwrap().insert(self.identify(), self.items.clone());
    }

    pub fn contents_list(&self) -> ListTag {
        let items: Vec<ObjectTag> = self
            .items
            .iter()
            .filter_map(|i| i.as_ref().filter(|it| !it.is_empty()).map(|it| it.obj()))
            .collect();
        ListTag::from_objects(items)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

fn default_size(id_type: &str) -> usize {
    match id_type.to_lowercase().as_str() {
        "player" => 36,
        "generic" | "chest" => 27,
        "workbench" | "crafting" => 9,
        "enderchest" => 27,
        "furnace" => 3,
        "dispenser" | "dropper" => 9,
        "hopper" => 5,
        "shulker_box" => 27,
        "anvil" => 3,
        "enchanting" => 2,
        "brewing" => 5,
        "merchant" => 3,
        _ => 27,
    }
}

impl ObjectTagImpl for InventoryTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        if self.size == default_size(&self.id_type) {
            format!("in@{}", self.id_type)
        } else {
            format!("in@{}[size={}]", self.id_type, self.size)
        }
    }
    fn identify_simple(&self) -> String {
        format!("in@{}", self.id_type)
    }
    fn object_type_name(&self) -> &'static str {
        "InventoryTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_unique(&self) -> bool {
        true
    }
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }
}

fn param_item(attr: &mut Attribute) -> Option<crate::obj::core::item::ItemTag> {
    let p = attr.get_param_object()?;
    crate::obj::core::item::ItemTag::from_identify(&p.as_element().value)
}

/// Register InventoryTag's tags.
pub fn register_tags() {
    let p = get_processor("InventoryTag");

    p.register_tag("ElementTag", "size", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(ElementTag::from_int(i.size as i64)))
    }, true, &[]);

    p.register_tag("ListTag", "contents", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(i.contents_list()))
    }, true, &[]);

    p.register_tag("ElementTag", "is_empty", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(ElementTag::from_bool(i.is_empty())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_full", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(ElementTag::from_bool(i.empty_slots() == 0)))
    }, true, &[]);

    p.register_tag("ElementTag", "empty_slots", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(ElementTag::from_int(i.empty_slots())))
    }, true, &[]);

    p.register_tag("ElementTag", "first_empty", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(ElementTag::from_int(i.first_empty())))
    }, true, &[]);

    p.register_tag("ElementTag", "id_type", |_a, o| {
        o.downcast_ref::<InventoryTag>().map(|i| ObjectTag::new(ElementTag::new(i.id_type.clone())))
    }, true, &["type"]);

    p.register_tag_with_param("ItemTag", "ElementTag", "get_slot", |a, o, param| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        let slot = param.as_element().to_int().unwrap_or(0) as usize;
        let _ = a;
        Some(inv.get_slot(slot).map(|it| it.obj()).unwrap_or_else(|| ObjectTag::new(crate::obj::core::item::ItemTag::air())))
    }, true, &[]);

    p.register_tag_with_param("InventoryTag", "ElementTag", "with_slot", |a, o, param| {
        let mut inv = o.downcast_ref::<InventoryTag>()?.clone();
        let arg = param.as_element().value;
        let parts: Vec<&str> = arg.splitn(2, '|').collect();
        let slot = parts.first().and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(0);
        let item = parts.get(1).and_then(|s| crate::obj::core::item::ItemTag::from_identify(s));
        inv.set_slot(slot, item);
        let _ = a;
        Some(ObjectTag::new(inv))
    }, true, &[]);

    p.register_tag_with_param("InventoryTag", "ElementTag", "with_contents", |a, o, param| {
        let mut inv = o.downcast_ref::<InventoryTag>()?.clone();
        let list = param.to_list();
        for (i, item) in list.items.iter().enumerate() {
            if i >= inv.items.len() {
                break;
            }
            inv.items[i] = crate::obj::core::item::ItemTag::from_identify(&item.as_element().value);
        }
        let _ = a;
        Some(ObjectTag::new(inv))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "contains_item", |a, o, param| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        let want = param.as_element().value.to_lowercase();
        let has = inv.items.iter().flatten().any(|i| i.material == want);
        let _ = a;
        Some(ObjectTag::new(ElementTag::from_bool(has)))
    }, true, &[]);

    p.register_tag("ListTag", "find_empty_slots", |_a, o| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        let slots: Vec<String> = inv
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| match i {
                None => true,
                Some(it) => it.is_empty(),
            })
            .map(|(idx, _)| idx.to_string())
            .collect();
        Some(ObjectTag::new(ListTag::from_strings(&slots)))
    }, true, &["empty_slots_list"]);

    p.register_tag("ElementTag", "quantity", |_a, o| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        let total: i64 = inv.items.iter().flatten().map(|i| i.quantity).sum();
        Some(ObjectTag::new(ElementTag::from_int(total)))
    }, true, &[]);

    p.register_tag("ElementTag", "quantity_item", |a, o| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        let want = a.get_param_object().map(|p| p.as_element().value.to_lowercase()).unwrap_or_default();
        let total: i64 = inv.items.iter().flatten().filter(|i| i.material == want).map(|i| i.quantity).sum();
        Some(ObjectTag::new(ElementTag::from_int(total)))
    }, true, &[]);

    p.register_tag("ElementTag", "note_name", |_a, o| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        Some(ObjectTag::new(ElementTag::new(inv.identify_simple())))
    }, true, &[]);

    p.register_tag("ElementTag", "formatted", |_a, o| {
        let inv = o.downcast_ref::<InventoryTag>()?;
        Some(ObjectTag::new(ElementTag::new(format!(
            "{}x {} ({} slots, {} empty)",
            inv.size,
            inv.id_type,
            inv.size,
            inv.empty_slots()
        ))))
    }, true, &[]);

    let _ = param_item;

    crate::obj::core::area::register_flag_tags("InventoryTag", |o| o.downcast_ref::<InventoryTag>().map(|x| x.identify()));
}
