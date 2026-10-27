//! `MaterialTag`: a block/item material type, identified as `m@<name>`.

use crate::obj::{get_processor, ElementTag, ListTag, ObjectTag, ObjectTagImpl};
use std::any::Any;

/// Known vanilla material categories for the lightweight engine model.
fn is_block_material(name: &str) -> bool {
    if name == "air" {
        return true;
    }
    !matches!(
        name,
        "stick" | "apple" | "bread" | "diamond" | "emerald" | "coal" | "iron_ingot" | "gold_ingot"
            | "iron_sword" | "diamond_sword" | "wooden_sword" | "stone_sword" | "golden_sword"
            | "netherite_sword" | "bow" | "arrow" | "paper" | "book" | "map" | "compass" | "clock"
            | "bucket" | "water_bucket" | "lava_bucket" | "milk_bucket" | "shears" | "fishing_rod"
            | "carrot" | "potato" | "wheat" | "wheat_seeds" | "bone" | "string" | "leather"
            | "feather" | "flint" | "gunpowder" | "redstone" | "glowstone_dust" | "sugar" | "egg"
            | "snowball" | "clay_ball" | "brick" | "nether_brick" | "ender_pearl" | "blaze_rod"
            | "blaze_powder" | "slime_ball" | "magma_cream" | "ghast_tear" | "spider_eye"
            | "rotten_flesh" | "golden_apple" | "cookie" | "melon_slice" | "pumpkin_pie"
            | "beef" | "cooked_beef" | "porkchop" | "cooked_porkchop" | "chicken" | "cooked_chicken"
            | "cod" | "cooked_cod" | "salmon" | "cooked_salmon" | "tropical_fish" | "pufferfish"
            | "mutton" | "cooked_mutton" | "rabbit" | "cooked_rabbit" | "rabbit_stew"
            | "mushroom_stew" | "beetroot" | "beetroot_soup" | "suspicious_stew" | "honey_bottle"
            | "experience_bottle" | "name_tag" | "lead" | "item_frame" | "painting" | "armor_stand"
            | "oak_boat" | "minecart" | "chest_minecart" | "furnace_minecart" | "tnt_minecart"
            | "hopper_minecart" | "saddle" | "elytra" | "totem_of_undying" | "trident"
            | "crossbow" | "shield" | "turtle_helmet" | "phantom_membrane" | "nautilus_shell"
            | "heart_of_the_sea" | "prismarine_shard" | "prismarine_crystals" | "shulker_shell"
            | "chorus_fruit" | "popped_chorus_fruit" | "end_crystal" | "firework_rocket"
            | "firework_star" | "knowledge_book" | "debug_stick" | "bundle" | "music_disc_13"
    )
}

fn is_edible(name: &str) -> bool {
    matches!(
        name,
        "apple" | "bread" | "carrot" | "potato" | "baked_potato" | "golden_apple" | "cookie"
            | "melon_slice" | "pumpkin_pie" | "beef" | "cooked_beef" | "porkchop" | "cooked_porkchop"
            | "chicken" | "cooked_chicken" | "cod" | "cooked_cod" | "salmon" | "cooked_salmon"
            | "tropical_fish" | "pufferfish" | "mutton" | "cooked_mutton" | "rabbit"
            | "cooked_rabbit" | "rabbit_stew" | "mushroom_stew" | "beetroot" | "beetroot_soup"
            | "suspicious_stew" | "honey_bottle" | "dried_kelp" | "enchanted_golden_apple"
            | "glow_berries" | "sweet_berries" | "chorus_fruit"
    )
}

/// A material type (block or item).
#[derive(Clone, Debug)]
pub struct MaterialTag {
    pub name: String,
    pub prefix: String,
}

impl MaterialTag {
    pub fn new(name: &str) -> Self {
        let mut n = name.trim().to_lowercase();
        if n.len() >= 2 && n[..2].eq_ignore_ascii_case("m@") {
            n = n[2..].to_string();
        }
        MaterialTag { name: n, prefix: "material".to_string() }
    }

    pub fn from_identify(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        Some(MaterialTag::new(s))
    }

    pub fn is_block(&self) -> bool {
        is_block_material(&self.name)
    }

    pub fn is_item(&self) -> bool {
        !matches!(self.name.as_str(), "air" | "cave_air" | "void_air" | "water" | "lava")
    }

    pub fn is_solid(&self) -> bool {
        is_block_material(&self.name)
            && !matches!(
                self.name.as_str(),
                "air" | "cave_air" | "void_air" | "water" | "lava" | "short_grass" | "tall_grass"
                    | "grass" | "fern" | "dead_bush" | "torch" | "wall_torch" | "fire" | "soul_fire"
                    | "vine" | "light" | "redstone_wire"
            )
    }

    pub fn max_stack_size(&self) -> i64 {
        match self.name.as_str() {
            "air" | "water" | "lava" => 64,
            _ => 64,
        }
    }

    pub fn max_durability(&self) -> i64 {
        match self.name.as_str() {
            n if n.ends_with("_sword") => 250,
            n if n.ends_with("_pickaxe") || n.ends_with("_axe") || n.ends_with("_shovel") => 250,
            "elytra" => 432,
            "shield" => 336,
            "trident" => 250,
            "bow" | "crossbow" | "fishing_rod" | "flint_and_steel" | "shears" => 384,
            n if n.ends_with("_helmet") => 363,
            n if n.ends_with("_chestplate") => 528,
            n if n.ends_with("_leggings") => 495,
            n if n.ends_with("_boots") => 429,
            _ => 0,
        }
    }

    pub fn hardness(&self) -> f64 {
        match self.name.as_str() {
            "obsidian" => 50.0,
            "bedrock" | "barrier" => -1.0,
            "stone" | "cobblestone" | "deepslate" => 1.5,
            "dirt" | "grass_block" | "sand" => 0.5,
            "oak_log" | "oak_planks" => 2.0,
            "iron_block" => 5.0,
            "diamond_block" => 5.0,
            _ => 1.0,
        }
    }

    pub fn obj(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

impl ObjectTagImpl for MaterialTag {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    fn identify(&self) -> String {
        format!("m@{}", self.name)
    }
    fn identify_simple(&self) -> String {
        format!("m@{}", self.name)
    }
    fn object_type_name(&self) -> &'static str {
        "MaterialTag"
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn duplicate(&self) -> ObjectTag {
        ObjectTag::new(self.clone())
    }
}

/// Register MaterialTag's tags.
pub fn register_tags() {
    let p = get_processor("MaterialTag");

    p.register_tag("ElementTag", "name", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::new(m.name.clone())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_block", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_bool(m.is_block())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_item", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_bool(m.is_item())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_solid", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_bool(m.is_solid())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_edible", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_bool(is_edible(&m.name))))
    }, true, &["is_food"]);

    p.register_tag("ElementTag", "is_burnable", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let burnable = m.name.ends_with("_log") || m.name.ends_with("_planks") || m.name.ends_with("_wood")
            || matches!(m.name.as_str(), "coal" | "charcoal" | "lava_bucket" | "stick" | "paper" | "bookshelf" | "crafting_table");
        Some(ObjectTag::new(ElementTag::from_bool(burnable)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_fuel", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let fuel = m.name.ends_with("_log") || m.name.ends_with("_planks") || matches!(m.name.as_str(), "coal" | "charcoal" | "lava_bucket" | "stick" | "blaze_rod");
        Some(ObjectTag::new(ElementTag::from_bool(fuel)))
    }, true, &[]);

    p.register_tag("ElementTag", "fuel_burn_time", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let ticks = match m.name.as_str() {
            "lava_bucket" => 20000,
            "coal" => 1600,
            "charcoal" => 1600,
            "blaze_rod" => 2400,
            "stick" => 100,
            n if n.ends_with("_log") || n.ends_with("_planks") => 300,
            _ => 0,
        };
        Some(ObjectTag::new(ElementTag::from_int(ticks)))
    }, true, &[]);

    p.register_tag("ElementTag", "max_stack_size", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_int(m.max_stack_size())))
    }, true, &[]);

    p.register_tag("ElementTag", "max_durability", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_int(m.max_durability())))
    }, true, &[]);

    p.register_tag("ElementTag", "hardness", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_double(m.hardness())))
    }, true, &[]);

    p.register_tag("ElementTag", "block_resistance", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let r = match m.name.as_str() {
            "obsidian" => 1200.0,
            "bedrock" => 3_600_000.0,
            "stone" => 6.0,
            _ => 0.0,
        };
        Some(ObjectTag::new(ElementTag::from_double(r)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_transparent", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let transparent = matches!(m.name.as_str(), "air" | "glass" | "water" | "ice" | "short_grass" | "tall_grass" | "torch");
        Some(ObjectTag::new(ElementTag::from_bool(transparent)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_occluding", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        Some(ObjectTag::new(ElementTag::from_bool(m.is_solid())))
    }, true, &[]);

    p.register_tag("ElementTag", "is_flammable", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let flam = m.name.ends_with("_planks") || m.name.ends_with("_log") || m.name.ends_with("_wool") || m.name.ends_with("_leaves");
        Some(ObjectTag::new(ElementTag::from_bool(flam)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_interactable", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let inter = matches!(m.name.as_str(), "chest" | "crafting_table" | "furnace" | "anvil" | "enchanting_table" | "lever" | "stone_button" | "oak_door");
        Some(ObjectTag::new(ElementTag::from_bool(inter)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_waterloggable", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let wl = m.name.ends_with("_slab") || m.name.ends_with("_stairs") || m.name.ends_with("_fence") || m.name.ends_with("_wall");
        Some(ObjectTag::new(ElementTag::from_bool(wl)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_leaves", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_bool(m.name.ends_with("_leaves"))))
    }, true, &[]);

    p.register_tag("ElementTag", "has_gravity", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let grav = matches!(m.name.as_str(), "sand" | "gravel" | "anvil" | "dragon_egg" | "suspicious_sand");
        Some(ObjectTag::new(ElementTag::from_bool(grav)))
    }, true, &[]);

    p.register_tag("ElementTag", "is_record", |_a, o| {
        o.downcast_ref::<MaterialTag>().map(|m| ObjectTag::new(ElementTag::from_bool(m.name.starts_with("music_disc_"))))
    }, true, &[]);

    p.register_tag("ElementTag", "food_points", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let pts = match m.name.as_str() {
            "golden_apple" => 4,
            "enchanted_golden_apple" => 4,
            "bread" => 5,
            "cooked_beef" => 8,
            "cooked_porkchop" => 8,
            "apple" => 4,
            "cookie" => 2,
            "carrot" => 3,
            "baked_potato" => 5,
            _ => 0,
        };
        Some(ObjectTag::new(ElementTag::from_int(pts)))
    }, true, &[]);

    p.register_tag("ItemTag", "as_item", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        Some(ObjectTag::new(crate::obj::core::item::ItemTag::new(&m.name)))
    }, true, &[]);

    p.register_tag("ListTag", "vanilla_tags", |_a, o| {
        let m = o.downcast_ref::<MaterialTag>()?;
        let mut tags = Vec::new();
        if m.is_block() {
            tags.push("minecraft:mineable/pickaxe".to_string());
        }
        if is_edible(&m.name) {
            tags.push("minecraft:edible".to_string());
        }
        Some(ObjectTag::new(ListTag::from_strings(&tags)))
    }, true, &[]);
}
