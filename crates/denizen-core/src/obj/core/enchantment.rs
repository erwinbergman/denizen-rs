//! `EnchantmentTag`: an item enchantment type, identified as `enchantment@<name>`.

use crate::obj::{get_processor, ElementTag, ObjectTag, ObjectTagImpl};
use std::any::Any;

/// Known vanilla enchantment data: (min_level, max_level, category, is_curse, is_treasure).
fn enchantment_info(name: &str) -> Option<(i64, i64, &'static str, bool, bool)> {
    let n = name.to_lowercase();
    let n = n.trim_start_matches("minecraft:");
    Some(match n {
        "protection" => (1, 4, "armor", false, false),
        "fire_protection" => (1, 4, "armor", false, false),
        "feather_falling" => (1, 4, "armor", false, false),
        "blast_protection" => (1, 4, "armor", false, false),
        "projectile_protection" => (1, 4, "armor", false, false),
        "respiration" => (1, 3, "armor", false, false),
        "aqua_affinity" => (1, 1, "armor", false, false),
        "thorns" => (1, 3, "armor", false, false),
        "depth_strider" => (1, 3, "armor", false, false),
        "frost_walker" => (1, 2, "armor", false, true),
        "binding_curse" => (1, 1, "armor", true, true),
        "soul_speed" => (1, 3, "armor", false, true),
        "swift_sneak" => (1, 3, "armor", false, true),
        "sharpness" => (1, 5, "weapon", false, false),
        "smite" => (1, 5, "weapon", false, false),
        "bane_of_arthropods" => (1, 5, "weapon", false, false),
        "knockback" => (1, 2, "weapon", false, false),
        "fire_aspect" => (1, 2, "weapon", false, false),
        "looting" => (1, 3, "weapon", false, false),
        "sweeping_edge" => (1, 3, "weapon", false, false),
        "efficiency" => (1, 5, "tool", false, false),
        "silk_touch" => (1, 1, "tool", false, false),
        "unbreaking" => (1, 3, "tool", false, false),
        "fortune" => (1, 3, "tool", false, false),
        "power" => (1, 5, "bow", false, false),
        "punch" => (1, 2, "bow", false, false),
        "flame" => (1, 1, "bow", false, false),
        "infinity" => (1, 1, "bow", false, false),
        "luck_of_the_sea" => (1, 3, "fishing", false, false),
        "lure" => (1, 3, "fishing", false, false),
        "loyalty" => (1, 3, "trident", false, false),
        "impaling" => (1, 5, "trident", false, false),
        "riptide" => (1, 3, "trident", false, false),
        "channeling" => (1, 1, "trident", false, false),
        "multishot" => (1, 1, "crossbow", false, false),
        "quick_charge" => (1, 3, "crossbow", false, false),
        "piercing" => (1, 4, "crossbow", false, false),
        "mending" => (1, 1, "tool", false, true),
        "vanishing_curse" => (1, 1, "tool", true, true),
        _ => return None,
    })
}

/// An enchantment type.
#[derive(Clone, Debug)]
pub struct EnchantmentTag {
    pub name: String,
    pub prefix: String,
}

impl EnchantmentTag {
    pub fn new(name: &str) -> Self {
        let mut n = name.trim().to_lowercase();
        if n.len() >= 12 && n[..12].eq_ignore_ascii_case("enchantment@") {
            n = n[12..].to_string();
        }
        n = n.trim_start_matches("minecraft:").to_string();
        EnchantmentTag { name: n, prefix: "enchantment".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        Some(EnchantmentTag::new(s))
    }

    pub fn max_level(&self) -> i64 {
        enchantment_info(&self.name).map(|i| i.1).unwrap_or(1)
    }

    pub fn min_level(&self) -> i64 {
        enchantment_info(&self.name).map(|i| i.0).unwrap_or(1)
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for EnchantmentTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("enchantment@{}", self.name)
    }
    fn identify_simple(&self) -> String {
        format!("enchantment@{}", self.name)
    }
    fn object_type_name(&self) -> &'static str {
        "EnchantmentTag"
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
}

/// Register EnchantmentTag's tags.
pub fn register_tags() {
    let p = get_processor("EnchantmentTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<EnchantmentTag>().map(|e| ObjectTag::new(ElementTag::new(e.name.clone())))
    }, true, &[]);

    p.register_tag("ElementTag", "key", |_a, o| {
        o.downcast_ref::<EnchantmentTag>().map(|e| ObjectTag::new(ElementTag::new(format!("minecraft:{}", e.name))))
    }, true, &[]);

    p.register_tag("ElementTag", "full_name", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::new(format!("minecraft:{}", e.name))))
    }, true, &[]);

    p.register_tag("ElementTag", "max_level", |_a, o| {
        o.downcast_ref::<EnchantmentTag>().map(|e| ObjectTag::new(ElementTag::from_int(e.max_level())))
    }, true, &[]);

    p.register_tag("ElementTag", "min_level", |_a, o| {
        o.downcast_ref::<EnchantmentTag>().map(|e| ObjectTag::new(ElementTag::from_int(e.min_level())))
    }, true, &[]);

    p.register_tag("ElementTag", "category", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        let cat = enchantment_info(&e.name).map(|i| i.2).unwrap_or("misc");
        Some(ObjectTag::new(ElementTag::new(cat)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_curse", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(enchantment_info(&e.name).map(|i| i.3).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_treasure_only", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(enchantment_info(&e.name).map(|i| i.4).unwrap_or(false))))
    }, true, &["treasure_only"]);

    p.register_tag("ElementTag", "is_tradable", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(!enchantment_info(&e.name).map(|i| i.4).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "is_discoverable", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(!enchantment_info(&e.name).map(|i| i.3).unwrap_or(false))))
    }, true, &[]);

    p.register_tag("ElementTag", "rarity", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        let rarity = match e.max_level() {
            5 => "rare",
            4 => "uncommon",
            3 => "uncommon",
            _ => "common",
        };
        Some(ObjectTag::new(ElementTag::new(rarity)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_compatible", |a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        let other = a.get_param_object().map(|p| EnchantmentTag::new(&p.as_element().value));
        let compat = match other {
            Some(other) => enchantment_info(&e.name).map(|i| i.2) == enchantment_info(&other.name).map(|i| i.2),
            None => true,
        };
        Some(ObjectTag::new(ElementTag::from_bool(compat)))
    }, true, &[]);

    p.register_tag("ElementTag", "min_cost", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(e.min_level() * 5)))
    }, true, &[]);

    p.register_tag("ElementTag", "max_cost", |_a, o| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        Some(ObjectTag::new(ElementTag::from_int(e.max_level() * 10)))
    }, true, &[]);

    p.register_tag_with_param("ElementTag", "ElementTag", "equals", |_a, o, param| {
        let e = o.downcast_ref::<EnchantmentTag>()?;
        let other = EnchantmentTag::new(&param.as_element().value);
        Some(ObjectTag::new(ElementTag::from_bool(e.name.eq_ignore_ascii_case(&other.name))))
    }, true, &[]);

    crate::obj::core::area::register_flag_tags("EnchantmentTag", |o| o.downcast_ref::<EnchantmentTag>().map(|x| x.identify()));
}
