//! Core object types: Element, List, Map, Duration, Vector, Color, Time, Script, Queue, Binary, Secret, Custom.

pub mod element;
pub mod list;
pub mod map;
pub mod duration;
pub mod vector;
pub mod color;
pub mod time;
pub mod script;
pub mod queue;
pub mod binary;
pub mod secret;
pub mod custom;
pub mod image;
pub mod location;
pub mod entity;
pub mod player;
pub mod npc;
pub mod world;
pub mod item;
pub mod material;
pub mod inventory;
pub mod enchantment;
pub mod cuboid;
pub mod area;
pub mod biome;
pub mod chunk;
pub mod plugin;
pub mod trade;

pub use element::ElementTag;
pub use list::ListTag;
pub use map::MapTag;
pub use duration::DurationTag;
pub use vector::VectorObject;
pub use color::ColorTag;
pub use time::TimeTag;
pub use script::ScriptTag;
pub use queue::QueueTag;
pub use binary::BinaryTag;
pub use secret::SecretTag;
pub use custom::CustomObjectTag;
pub use image::ImageTag;
pub use location::LocationTag;
pub use entity::EntityTag;
pub use player::PlayerTag;
pub use npc::NPCTag;
pub use world::WorldTag;
pub use item::ItemTag;
pub use material::MaterialTag;
pub use inventory::InventoryTag;
pub use enchantment::EnchantmentTag;
pub use cuboid::CuboidTag;
pub use biome::BiomeTag;
pub use chunk::ChunkTag;
pub use plugin::PluginTag;
pub use trade::TradeTag;

use crate::obj::{register_object_type, ObjectTagProcessor, ObjectType};
use std::sync::Arc;

fn type_of(name: &'static str, short: &'static str, prefix: &'static str) -> ObjectType {
    ObjectType {
        name,
        short_name: short,
        prefix,
        processor: Arc::new(ObjectTagProcessor::new()),
        converter: None,
        should_be_checker: None,
    }
}

/// Register all core object types and their tags.
pub fn register() {
    register_object_type(type_of("ElementTag", "Element", "element"));
    register_object_type(type_of("ListTag", "List", "li"));
    register_object_type(type_of("MapTag", "Map", "map"));
    register_object_type(type_of("DurationTag", "Duration", "duration"));
    register_object_type(type_of("VectorObject", "Vector", "vector"));
    register_object_type(type_of("ColorTag", "Color", "color"));
    register_object_type(type_of("TimeTag", "Time", "time"));
    register_object_type(type_of("ScriptTag", "Script", "script"));
    register_object_type(type_of("QueueTag", "Queue", "queue"));
    register_object_type(type_of("BinaryTag", "Binary", "binary"));
    register_object_type(type_of("SecretTag", "Secret", "secret"));
    register_object_type(type_of("CustomObjectTag", "Custom", "custom"));
    register_object_type(type_of("ImageTag", "Image", "image"));

    element::register_tags();
    list::register_tags();
    map::register_tags();
    duration::register_tags();
    vector::register_tags();
    color::register_tags();
    time::register_tags();
    script::register_tags();
    queue::register_tags();
    binary::register_tags();

    register_object_type(type_of("LocationTag", "Location", "l"));
    register_object_type(type_of("EntityTag", "Entity", "e"));
    register_object_type(type_of("PlayerTag", "Player", "p"));
    register_object_type(type_of("NPCTag", "NPC", "n"));
    register_object_type(type_of("WorldTag", "World", "w"));
    register_object_type(type_of("ItemTag", "Item", "i"));
    register_object_type(type_of("MaterialTag", "Material", "m"));
    register_object_type(type_of("InventoryTag", "Inventory", "in"));
    register_object_type(type_of("EnchantmentTag", "Enchantment", "enchantment"));
    register_object_type(type_of("CuboidTag", "Cuboid", "cu"));
    register_object_type(type_of("BiomeTag", "Biome", "b"));
    register_object_type(type_of("ChunkTag", "Chunk", "ch"));
    register_object_type(type_of("PluginTag", "Plugin", "pl"));
    register_object_type(type_of("TradeTag", "Trade", "trade"));

    location::register_tags();
    entity::register_tags();
    player::register_tags();
    npc::register_tags();
    world::register_tags();
    item::register_tags();
    material::register_tags();
    inventory::register_tags();
    enchantment::register_tags();
    cuboid::register_tags();
    biome::register_tags();
    chunk::register_tags();
    plugin::register_tags();
    trade::register_tags();
    area::register_object_types();

    crate::tags::register_extra_base_tags();
}
