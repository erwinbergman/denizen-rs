//! Extra base tags: object constructors (`<location[...]>`, `<item[...]>`, ...) and math helpers.

use crate::obj::core::area::{Ellipsoid, Polygon};
use crate::obj::core::biome::BiomeTag;
use crate::obj::core::chunk::ChunkTag;
use crate::obj::core::cuboid::CuboidTag;
use crate::obj::core::enchantment::EnchantmentTag;
use crate::obj::core::entity::EntityTag;
use crate::obj::core::inventory::InventoryTag;
use crate::obj::core::item::ItemTag;
use crate::obj::core::location::LocationTag;
use crate::obj::core::material::MaterialTag;
use crate::obj::core::npc::NPCTag;
use crate::obj::core::player::PlayerTag;
use crate::obj::core::plugin::PluginTag;
use crate::obj::core::trade::TradeTag;
use crate::obj::core::world::WorldTag;
use crate::obj::{ElementTag, ObjectTag};
use crate::tags::{register_tag_handler_with_param, Attribute};
use std::sync::Arc;

/// Register all extra base tags.
pub fn register() {
    // Object constructors that take a single element parameter.
    register_tag_handler_with_param(
        "LocationTag",
        "ElementTag",
        "location",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            LocationTag::from_identify(&param.as_element().value).map(|l| l.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "LocationTag",
        "ElementTag",
        "l",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            LocationTag::from_identify(&param.as_element().value).map(|l| l.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "WorldTag",
        "ElementTag",
        "world",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            WorldTag::from_identify(&param.as_element().value).map(|w| w.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "ItemTag",
        "ElementTag",
        "item",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            ItemTag::from_identify(&param.as_element().value).map(|i| i.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "MaterialTag",
        "ElementTag",
        "material",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            MaterialTag::from_identify(&param.as_element().value).map(|m| m.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "EntityTag",
        "ElementTag",
        "entity",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            EntityTag::from_identify(&param.as_element().value).map(|e| e.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "PlayerTag",
        "ElementTag",
        "player",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            PlayerTag::from_identify(&param.as_element().value).map(|p| p.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "NPCTag",
        "ElementTag",
        "npc",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            NPCTag::from_identify(&param.as_element().value).map(|n| n.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "InventoryTag",
        "ElementTag",
        "inventory",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            InventoryTag::from_identify(&param.as_element().value).map(|i| i.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "EnchantmentTag",
        "ElementTag",
        "enchantment",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            EnchantmentTag::from_identify(&param.as_element().value).map(|e| e.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "CuboidTag",
        "ElementTag",
        "cuboid",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            CuboidTag::from_identify(&param.as_element().value).map(|c| c.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "BiomeTag",
        "ElementTag",
        "biome",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            BiomeTag::from_identify(&param.as_element().value).map(|b| b.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "ChunkTag",
        "ElementTag",
        "chunk",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            ChunkTag::from_identify(&param.as_element().value).map(|c| c.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "PluginTag",
        "ElementTag",
        "plugin",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            PluginTag::from_identify(&param.as_element().value).map(|p| p.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "TradeTag",
        "ElementTag",
        "trade",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            TradeTag::from_identify(&param.as_element().value).map(|t| t.obj())
        }),
        true,
    );
    register_tag_handler_with_param(
        "EllipsoidTag",
        "ElementTag",
        "ellipsoid",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            Ellipsoid::from_identify(&param.as_element().value)
                .map(|e| ObjectTag::new(crate::obj::core::area::EllipsoidObject(e)))
        }),
        true,
    );
    register_tag_handler_with_param(
        "PolygonTag",
        "ElementTag",
        "polygon",
        Arc::new(|_a: &mut Attribute, param: ObjectTag| {
            Polygon::from_identify(&param.as_element().value)
                .map(|p| ObjectTag::new(crate::obj::core::area::PolygonObject(p)))
        }),
        true,
    );

    register_math_bases();
    register_element_math_tags();
}

/// Add arithmetic object tags to ElementTag (kept here to avoid touching element.rs).
fn register_element_math_tags() {
    use crate::obj::get_processor;
    let p = get_processor("ElementTag");

    p.register_tag_with_param("ElementTag", "ElementTag", "add", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_double(a + b)))
    }, true, &["plus"]);

    p.register_tag_with_param("ElementTag", "ElementTag", "subtract", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_double(a - b)))
    }, true, &["minus"]);

    p.register_tag_with_param("ElementTag", "ElementTag", "multiply", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        Some(ObjectTag::new(ElementTag::from_double(a * b)))
    }, true, &["times"]);

    p.register_tag_with_param("ElementTag", "ElementTag", "divide", |_a, o, param| {
        let a = o.as_element().to_double().unwrap_or(0.0);
        let b = param.as_element().to_double().unwrap_or(0.0);
        if b == 0.0 {
            return None;
        }
        Some(ObjectTag::new(ElementTag::from_double(a / b)))
    }, true, &[]);

    p.register_tag("ElementTag", "abs", |_a, o| {
        Some(ObjectTag::new(ElementTag::from_double(o.as_element().to_double().unwrap_or(0.0).abs())))
    }, true, &[]);

    p.register_tag("ElementTag", "floor", |_a, o| {
        Some(ObjectTag::new(ElementTag::from_double(o.as_element().to_double().unwrap_or(0.0).floor())))
    }, true, &[]);

    p.register_tag("ElementTag", "ceil", |_a, o| {
        Some(ObjectTag::new(ElementTag::from_double(o.as_element().to_double().unwrap_or(0.0).ceil())))
    }, true, &[]);
}

fn register_math_bases() {
    register_tag_handler_with_param(
        "ElementTag",
        "ElementTag",
        "math",
        Arc::new(|a: &mut Attribute, param: ObjectTag| {
            let sub = a.get_attribute_without_param(1);
            let input = param.as_element().value;
            let parts: Vec<&str> = input.split('|').collect();
            let n = |idx: usize| parts.get(idx).and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0);
            let result = match sub.as_str() {
                "add" | "plus" => n(0) + n(1),
                "subtract" | "minus" => n(0) - n(1),
                "multiply" | "times" => n(0) * n(1),
                "divide" => {
                    if n(1) == 0.0 {
                        return None;
                    }
                    n(0) / n(1)
                }
                "mod" | "modulo" => {
                    if n(1) == 0.0 {
                        return None;
                    }
                    n(0) % n(1)
                }
                "abs" => n(0).abs(),
                "round" => n(0).round(),
                "floor" => n(0).floor(),
                "ceil" => n(0).ceil(),
                "min" => n(0).min(n(1)),
                "max" => n(0).max(n(1)),
                "pow" | "power" => n(0).powf(n(1)),
                "sqrt" => n(0).sqrt(),
                "sin" => n(0).to_radians().sin(),
                "cos" => n(0).to_radians().cos(),
                "tan" => n(0).to_radians().tan(),
                "log" => n(0).ln(),
                "log10" => n(0).log10(),
                _ => return None,
            };
            a.fulfill(1);
            Some(ObjectTag::new(ElementTag::from_double(result)))
        }),
        false,
    );
}
