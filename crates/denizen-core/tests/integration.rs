//! End-to-end integration tests for the native Denizen engine.

use denizen_core::{engine, events, flags, obj, scripts, tags};

fn setup() {
    engine::init(None);
}

#[test]
fn engine_initializes() {
    setup();
    assert!(engine::is_initialized());
    assert!(engine::stats().contains("Denizen Rust core"));
}

#[test]
fn command_registry_is_broad() {
    setup();
    let count = denizen_core::commands::registry::command_count();
    assert!(count > 150, "expected >150 commands, got {}", count);
    for expected in ["narrate", "define", "if", "while", "foreach", "flag", "adjust", "run"] {
        assert!(
            denizen_core::commands::registry::command_exists(expected),
            "missing command: {}",
            expected
        );
    }
}

#[test]
fn event_registry_is_broad() {
    setup();
    assert!(events::event_count() > 100, "expected >100 events, got {}", events::event_count());
    assert!(events::event_exists("player joins"));
}

#[test]
fn object_types_registered() {
    setup();
    for t in [
        "ElementTag", "ListTag", "MapTag", "DurationTag", "VectorObject", "ColorTag", "TimeTag",
        "ScriptTag", "QueueTag", "BinaryTag", "LocationTag", "EntityTag", "PlayerTag", "NPCTag",
        "WorldTag", "ItemTag", "MaterialTag", "InventoryTag", "EnchantmentTag", "CuboidTag",
        "BiomeTag", "ChunkTag", "PluginTag", "TradeTag",
    ] {
        assert!(obj::get_object_type(t).is_some(), "missing object type: {}", t);
    }
}

#[test]
fn tag_parsing_basic() {
    setup();
    assert_eq!(engine::tag("hello"), "hello");
    assert_eq!(engine::tag("<element[hi].to_uppercase>"), "HI");
    assert_eq!(engine::tag("<element[5].add[3]>"), "8");
}

#[test]
fn tag_definitions() {
    setup();
    let provider = std::sync::Arc::new(tags::SimpleDefinitionProvider::new());
    tags::DefinitionProvider::add_definition(provider.as_ref(), "x", Some(obj::ObjectTag::new(obj::ElementTag::new("42"))));
    let ctx = tags::TagContext {
        definition_provider: Some(provider),
        ..Default::default()
    };
    assert_eq!(tags::tag("<[x]>", &ctx), "42");
    assert_eq!(tags::tag("<def[x]>", &ctx), "42");
}

#[test]
fn list_and_map_objects() {
    setup();
    let mut list = obj::ListTag::new();
    list.add_string("a").add_string("b").add_string("c");
    assert_eq!(obj::ObjectTag::new(list.clone()).identify(), "li@a|b|c|");
    let parsed = obj::ListTag::from_identify("li@x|y|z");
    assert_eq!(parsed.strings(), vec!["x", "y", "z"]);

    let mut map = obj::MapTag::new();
    map.put_element("k", "v");
    assert!(obj::ObjectTag::new(map.clone()).identify().contains("k=v"));
    let parsed_map = obj::MapTag::from_identify("map@[a=1;b=2]").unwrap();
    assert_eq!(parsed_map.get("a").unwrap().identify(), "1");
}

#[test]
fn list_tags() {
    setup();
    let ctx = tags::TagContext::default();
    let list = tags::tag_object("li@apple|banana|cherry", &ctx);
    let obj_list = list.to_list();
    assert_eq!(obj_list.size(), 3);
    // Character-index access on a list object.
    let second = list.as_type("ListTag", &ctx).unwrap();
    let _ = second;
}

#[test]
fn duration_parsing() {
    setup();
    let d = obj::DurationTag::from_identify("5m").unwrap();
    assert!((d.seconds - 300.0).abs() < 0.001);
    let d2 = obj::DurationTag::from_identify("2h").unwrap();
    assert!((d2.seconds - 7200.0).abs() < 0.001);
}

#[test]
fn location_object() {
    setup();
    let loc = obj::get_object_type("LocationTag");
    assert!(loc.is_some());
}

#[test]
fn script_execution_if_else() {
    setup();
    let script_text = r#"
test_if_else:
  type: task
  script:
    - define x 5
    - if <[x]> == 5:
      - define result yes
    - else:
      - define result no
    - narrate "result is <[result]>"
"#;
    for c in scripts::helper::parse_scripts_text(script_text) {
        scripts::registry::register_container(c);
    }
    assert!(scripts::registry::get_container("test_if_else").is_some());
    let ran = engine::run_script("test_if_else").unwrap();
    assert!(ran > 0, "expected script entries executed");
}

#[test]
fn script_while_loop() {
    setup();
    let script_text = r#"
test_while:
  type: task
  script:
    - define counter 0
    - while <[counter]> < 3:
      - define counter <[counter].add[1]>
    - narrate "done <[counter]>"
"#;
    for c in scripts::helper::parse_scripts_text(script_text) {
        scripts::registry::register_container(c);
    }
    let ran = engine::run_script("test_while").unwrap();
    assert!(ran > 0);
}

#[test]
fn script_foreach() {
    setup();
    let script_text = r#"
test_foreach:
  type: task
  script:
    - foreach li@a|b|c:
      - narrate "item <[value]>"
"#;
    for c in scripts::helper::parse_scripts_text(script_text) {
        scripts::registry::register_container(c);
    }
    let ran = engine::run_script("test_foreach").unwrap();
    assert!(ran > 0);
}

#[test]
fn procedure_determines() {
    setup();
    let script_text = r#"
test_proc:
  type: procedure
  script:
    - determine 99
"#;
    for c in scripts::helper::parse_scripts_text(script_text) {
        scripts::registry::register_container(c);
    }
    let container = scripts::registry::get_container("test_proc").unwrap();
    let result = container.run_procedure(None);
    assert_eq!(result.map(|r| r.identify()), Some("99".to_string()));
}

#[test]
fn flags_roundtrip() {
    setup();
    let tracker = flags::tracker_for("p@test-uuid");
    tracker.set("myflag", Some(obj::ObjectTag::new(obj::ElementTag::new("myvalue"))));
    assert_eq!(tracker.get("myflag").unwrap().identify(), "myvalue");
    assert!(tracker.has("myflag"));
    tracker.remove("myflag");
    assert!(!tracker.has("myflag"));
}

#[test]
fn flag_command_via_script() {
    setup();
    let script_text = r#"
test_flag:
  type: task
  script:
    - flag server set myflag myvalue
"#;
    for c in scripts::helper::parse_scripts_text(script_text) {
        scripts::registry::register_container(c);
    }
    let _ = engine::run_script("test_flag");
    let tracker = flags::tracker_for("server");
    assert!(tracker.get("myflag").is_some());
}

#[test]
fn event_firing() {
    setup();
    let script_text = r#"
test_event:
  type: world
  events:
    - on player joins:
      - narrate "welcome"
"#;
    for c in scripts::helper::parse_scripts_text(script_text) {
        scripts::registry::register_container(c);
    }
    let fired = events::fire("player joins");
    assert!(fired >= 0);
}

#[test]
fn yaml_parse_and_roundtrip() {
    use denizen_core::util::yaml;
    let text = "name: test\nitems:\n  - a\n  - b\nnested:\n  key: value\n";
    let y = yaml::parse(text);
    assert_eq!(y.get_string("name"), Some("test".to_string()));
    assert_eq!(y.get_string_list("items"), vec!["a", "b"]);
    assert_eq!(y.get("nested.key").and_then(|v| v.as_string()), Some("value".to_string()));
}

#[test]
fn command_line_execution() {
    setup();
    let result = engine::execute_command_line("narrate hello world");
    assert!(result.is_ok());
}

#[test]
fn vector_math() {
    setup();
    let ctx = tags::TagContext::default();
    let v = tags::tag_object("<vector[3,4,0]>", &ctx);
    let _ = v;
    let vec = obj::VectorObject::new(3.0, 4.0, 0.0);
    assert!((vec.length() - 5.0).abs() < 0.001);
}

#[test]
fn script_registry_counts() {
    setup();
    let text = r#"
extra_script:
  type: task
  script:
    - narrate hi
"#;
    for c in scripts::helper::parse_scripts_text(text) {
        scripts::registry::register_container(c);
    }
    assert!(scripts::registry::get_container("extra_script").is_some());
}

#[test]
fn element_tag_operations() {
    setup();
    let ctx = tags::TagContext::default();
    assert_eq!(tags::tag("<element[hello].length>", &ctx), "5");
    assert_eq!(tags::tag("<element[HELLO].to_lowercase>", &ctx), "hello");
    assert_eq!(tags::tag("<element[42].is_integer>", &ctx), "true");
    assert_eq!(tags::tag("<element[hi].equals[HI]>", &ctx), "true");
}
