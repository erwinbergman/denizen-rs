//! `ItemTag`: a holdable item stack, identified as `i@<material>[...]`.

use crate::obj::{get_processor, ElementTag, ListTag, MapTag, ObjectTag, ObjectTagImpl};
use crate::tags::Attribute;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// An item enchantment entry.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemEnchantment {
    pub name: String,
    pub level: i64,
}

/// A stack of an item type with optional metadata.
#[derive(Clone, Debug)]
pub struct ItemTag {
    pub material: String,
    pub quantity: i64,
    pub display_name: Option<String>,
    pub lore: Vec<String>,
    pub enchantments: Vec<ItemEnchantment>,
    pub durability: i64,
    pub unbreakable: bool,
    pub custom_model_data: Option<i64>,
}

/// Global per-item-type flag store keyed by item identity (used by flags).
static ITEM_DATA: LazyLock<RwLock<HashMap<String, MapTag>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

impl ItemTag {
    pub fn new(material: &str) -> Self {
        let mut m = material.trim().to_lowercase();
        if m.len() >= 2 && m[..2].eq_ignore_ascii_case("i@") {
            m = m[2..].to_string();
        }
        ItemTag {
            material: m,
            quantity: 1,
            display_name: None,
            lore: Vec::new(),
            enchantments: Vec::new(),
            durability: 0,
            unbreakable: false,
            custom_model_data: None,
        }
    }

    pub fn air() -> Self {
        ItemTag::new("air")
    }

    pub fn with_quantity(mut self, q: i64) -> Self {
        self.quantity = q;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.material == "air" || self.quantity <= 0
    }

    /// Parse `i@material[key=value;key2=value2]`.
    pub fn from_identify(s: &str) -> Option<Self> {
        let mut s = s.trim();
        if s.len() >= 2 && s[..2].eq_ignore_ascii_case("i@") {
            s = &s[2..];
        }
        if s.is_empty() {
            return None;
        }
        let (mat, props) = match s.find('[') {
            Some(i) if s.ends_with(']') => (&s[..i], Some(&s[i + 1..s.len() - 1])),
            _ => (s, None),
        };
        let mut item = ItemTag::new(mat);
        if let Some(props) = props {
            for entry in split_props(props) {
                let mut kv = entry.splitn(2, '=');
                let key = kv.next().unwrap_or("").trim().to_lowercase();
                let value = kv.next().unwrap_or("").trim();
                match key.as_str() {
                    "quantity" | "qty" => item.quantity = value.parse().unwrap_or(1),
                    "display_name" | "name" => item.display_name = Some(unescape(value)),
                    "lore" => {
                        item.lore = value.split('|').map(unescape).collect();
                    }
                    "durability" => item.durability = value.parse().unwrap_or(0),
                    "unbreakable" => item.unbreakable = value.eq_ignore_ascii_case("true"),
                    "custom_model_data" => item.custom_model_data = value.parse().ok(),
                    "enchantments" => {
                        for e in value.split('|') {
                            if let Some((n, l)) = e.split_once(':') {
                                item.enchantments.push(ItemEnchantment {
                                    name: n.trim().to_lowercase(),
                                    level: l.trim().parse().unwrap_or(1),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Some(item)
    }

    pub fn get_data(&self) -> MapTag {
        ITEM_DATA.read().unwrap().get(&self.identify()).cloned().unwrap_or_default()
    }

    pub fn set_data(&self, map: MapTag) {
        ITEM_DATA.write().unwrap().insert(self.identify(), map);
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

fn split_props(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(&n) = chars.peek() {
                    cur.push('\\');
                    cur.push(n);
                    chars.next();
                }
            }
            '[' => {
                depth += 1;
                cur.push(c);
            }
            ']' => {
                depth -= 1;
                cur.push(c);
            }
            ';' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn unescape(s: &str) -> String {
    s.replace("\\;", ";").replace("\\]", "]").replace("\\=", "=").replace("\\\\", "\\")
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace(';', "\\;").replace(']', "\\]").replace('=', "\\=")
}

impl ObjectTagImpl for ItemTag {
    fn prefix(&self) -> String {
        "Item".to_string()
    }
    fn identify(&self) -> String {
        if self.is_empty() {
            return "i@air".to_string();
        }
        let mut props = Vec::new();
        if self.quantity != 1 {
            props.push(format!("quantity={}", self.quantity));
        }
        if let Some(n) = &self.display_name {
            props.push(format!("display_name={}", escape(n)));
        }
        if !self.lore.is_empty() {
            let l: Vec<String> = self.lore.iter().map(|x| escape(x)).collect();
            props.push(format!("lore={}", l.join("|")));
        }
        if !self.enchantments.is_empty() {
            let e: Vec<String> = self
                .enchantments
                .iter()
                .map(|e| format!("{}:{}", e.name, e.level))
                .collect();
            props.push(format!("enchantments={}", e.join("|")));
        }
        if self.durability != 0 {
            props.push(format!("durability={}", self.durability));
        }
        if self.unbreakable {
            props.push("unbreakable=true".to_string());
        }
        if let Some(c) = self.custom_model_data {
            props.push(format!("custom_model_data={}", c));
        }
        if props.is_empty() {
            format!("i@{}", self.material)
        } else {
            format!("i@{}[{}]", self.material, props.join(";"))
        }
    }
    fn identify_simple(&self) -> String {
        if self.is_empty() {
            "i@air".to_string()
        } else {
            format!("i@{}", self.material)
        }
    }
    fn object_type_name(&self) -> &'static str {
        "ItemTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
    fn is_unique(&self) -> bool {
        false
    }
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }
}

fn param_item(attr: &mut Attribute) -> Option<ItemTag> {
    let p = attr.get_param_object()?;
    let s = p.as_element().value;
    ItemTag::from_identify(&s)
}

/// Register ItemTag's tags.
pub fn register_tags() {
    let p = get_processor("ItemTag");

    p.register_tag("MaterialTag", "material", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        Some(ObjectTag::new(crate::obj::core::material::MaterialTag::new(&i.material)))
    }, true, &[]);

    p.register_tag("ElementTag", "quantity", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_int(i.quantity)))
    }, true, &["qty", "count"]);

    p.register_tag("ElementTag", "display_name", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        Some(ObjectTag::new(ElementTag::new(i.display_name.clone().unwrap_or_default())))
    }, true, &["name"]);

    p.register_tag("ListTag", "lore", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        Some(ObjectTag::new(ListTag::from_strings(&i.lore)))
    }, true, &[]);

    p.register_tag("ListTag", "enchantments", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let items: Vec<ObjectTag> = i
            .enchantments
            .iter()
            .map(|e| ObjectTag::new(crate::obj::core::enchantment::EnchantmentTag::new(&e.name)))
            .collect();
        Some(ObjectTag::new(ListTag::from_objects(items)))
    }, true, &[]);

    p.register_tag("ElementTag", "durability", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_int(i.durability)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_empty", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_bool(i.is_empty())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_unbreakable", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_bool(i.unbreakable)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_book", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(matches!(i.material.as_str(), "writable_book" | "written_book" | "book"))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_firework", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_bool(matches!(i.material.as_str(), "firework_rocket" | "firework_star"))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_colorable", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let c = i.material.ends_with("_wool") || i.material.ends_with("_carpet") || i.material.ends_with("_concrete") || matches!(i.material.as_str(), "leather_helmet" | "leather_chestplate" | "leather_leggings" | "leather_boots");
        Some(ObjectTag::new(ElementTag::from_bool(c)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_repairable", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let m = crate::obj::core::material::MaterialTag::new(&i.material);
        Some(ObjectTag::new(ElementTag::from_bool(m.max_durability() > 0)))
    }, true, &["repairable"]);

    p.register_tag("ElementTag", "max_stack_size", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let m = crate::obj::core::material::MaterialTag::new(&i.material);
        Some(ObjectTag::new(ElementTag::from_int(m.max_stack_size())))
    }, true, &[]);

    p.register_tag("ElementTag", "max_durability", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let m = crate::obj::core::material::MaterialTag::new(&i.material);
        Some(ObjectTag::new(ElementTag::from_int(m.max_durability())))
    }, true, &[]);

    p.register_tag_with_param("ItemTag", "ElementTag", "with_quantity", |_a, o, param| {
        let i = o.downcast_ref::<ItemTag>()?;
        Some(ObjectTag::new(i.clone().with_quantity(param.as_element().to_int().unwrap_or(1))))
    }, true, &[]);

    p.register_tag_with_param("ItemTag", "ElementTag", "with_display_name", |_a, o, param| {
        let mut i = o.downcast_ref::<ItemTag>()?.clone();
        i.display_name = Some(param.as_element().value);
        Some(ObjectTag::new(i))
    }, true, &[]);

    p.register_tag_with_param("ItemTag", "ElementTag", "with_lore", |_a, o, param| {
        let mut i = o.downcast_ref::<ItemTag>()?.clone();
        i.lore = param.as_element().value.split('|').map(|s| s.to_string()).collect();
        Some(ObjectTag::new(i))
    }, true, &[]);

    p.register_tag_with_param("ItemTag", "ElementTag", "with_durability", |_a, o, param| {
        let mut i = o.downcast_ref::<ItemTag>()?.clone();
        i.durability = param.as_element().to_int().unwrap_or(0);
        Some(ObjectTag::new(i))
    }, true, &[]);

    p.register_tag_with_param("ItemTag", "ElementTag", "with_material", |_a, o, param| {
        let mut i = o.downcast_ref::<ItemTag>()?.clone();
        i.material = crate::obj::core::material::MaterialTag::new(&param.as_element().value).name;
        Some(ObjectTag::new(i))
    }, true, &[]);

    p.register_tag_with_param("ItemTag", "ElementTag", "enchant", |a, o, param| {
        let mut i = o.downcast_ref::<ItemTag>()?.clone();
        let arg = param.as_element().value;
        let (name, level) = match arg.split_once(':') {
            Some((n, l)) => (n.trim().to_lowercase(), l.trim().parse().unwrap_or(1)),
            None => (arg.trim().to_lowercase(), 1),
        };
        if let Some(e) = i.enchantments.iter_mut().find(|e| e.name == name) {
            e.level = level;
        } else {
            i.enchantments.push(ItemEnchantment { name, level });
        }
        let _ = a;
        Some(ObjectTag::new(i))
    }, true, &[]);

    p.register_tag("ElementTag", "is_similar", |a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let other = param_item(a)?;
        Some(ObjectTag::new(ElementTag::from_bool(i.material == other.material)))
    }, true, &[]);

    p.register_tag("ElementTag", "simple", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::new(i.identify_simple())))
    }, true, &[]);

    p.register_tag("MapTag", "data", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        Some(ObjectTag::new(i.get_data()))
    }, true, &[]);

    p.register_tag("ElementTag", "is_enchanted", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_bool(!i.enchantments.is_empty())))
    }, true, &[]);

    p.register_tag("ElementTag", "has_lore", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_bool(!i.lore.is_empty())))
    }, true, &[]);

    p.register_tag("ElementTag", "has_display_name", |_a, o| {
        o.downcast_ref::<ItemTag>().map(|i| ObjectTag::new(ElementTag::from_bool(i.display_name.is_some())))
    }, true, &[]);

    p.register_tag("ElementTag", "formatted", |_a, o| {
        let i = o.downcast_ref::<ItemTag>()?;
        let mut out = i.display_name.clone().unwrap_or_else(|| i.material.clone());
        if i.quantity > 1 {
            out.push_str(&format!(" x{}", i.quantity));
        }
        Some(ObjectTag::new(ElementTag::new(out)))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("ItemTag", |o| o.downcast_ref::<ItemTag>().map(|x| x.identify()));
}
