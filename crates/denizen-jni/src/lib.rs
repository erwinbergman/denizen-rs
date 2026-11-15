//! JNI bridge: exposes the native Denizen engine to the minimal Java shim.

use denizen_core::{engine, obj, scripts};
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jint, jlong, jstring};
use jni::JNIEnv;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn jstr(env: &mut JNIEnv, s: &str) -> jstring {
    match env.new_string(s) {
        Ok(js) => js.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn get_string(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(|s| s.into()).unwrap_or_default()
}

fn _unused(_: jlong, _: jint, _: jboolean, _: JObject, _: JClass) {}

// ---------------------------------------------------------------------------
// Engine lifecycle
// ---------------------------------------------------------------------------

/// `nativeInit()`: initialize the native engine.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeInit(
    _env: JNIEnv,
    _class: JClass,
) {
    engine::init(None);
}

/// `nativeVersion()`: returns the engine version string.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeVersion(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    jstr(&mut env, denizen_core::VERSION)
}

/// `nativeStats()`: returns engine statistics.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeStats(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    jstr(&mut env, &engine::stats())
}

/// `nativeCommandCount()`: number of registered commands.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeCommandCount(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    denizen_core::commands::registry::command_count() as jint
}

/// `nativeObjectTypeCount()`: number of object types.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeObjectTypeCount(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    denizen_core::obj::object_type_count() as jint
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

/// `nativeTag(String)`: parse a tag string and return the result.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeTag(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let text = get_string(&mut env, &input);
    let result = engine::tag(&text);
    jstr(&mut env, &result)
}

/// `nativeTagObject(String)`: parse a tag string, returning its identify form.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeTagObject(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let text = get_string(&mut env, &input);
    let result = engine::tag_object(&text).identify();
    jstr(&mut env, &result)
}

// ---------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------

/// `nativeReloadScripts()`: reload scripts from disk, returning the count.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeReloadScripts(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let (count, _errors) = engine::reload_scripts();
    count as jint
}

/// `nativeReloadScriptsFrom(String)`: reload scripts from an explicit folder path.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeReloadScriptsFrom(
    mut env: JNIEnv,
    _class: JClass,
    folder: JString,
) -> jint {
    let path = get_string(&mut env, &folder);
    let (count, _errors) = engine::reload_scripts_from(&path);
    count as jint
}

/// `nativeScriptCount()`: number of loaded script containers.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeScriptCount(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    scripts::registry::container_count() as jint
}

/// `nativeRunScript(String)`: run a script by name; returns entries executed or -1.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeRunScript(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) -> jint {
    let script = get_string(&mut env, &name);
    match engine::run_script(&script) {
        Ok(n) => n as jint,
        Err(_) => -1,
    }
}

/// `nativeExecuteCommand(String)`: execute a raw command line.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeExecuteCommand(
    mut env: JNIEnv,
    _class: JClass,
    line: JString,
) -> jstring {
    let cmd = get_string(&mut env, &line);
    match engine::execute_command_line(&cmd) {
        Ok(n) => jstr(&mut env, &format!("ok:{}", n)),
        Err(e) => jstr(&mut env, &format!("error:{}", e)),
    }
}

/// `nativeScriptNames()`: returns a `|`-separated list of script names.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeScriptNames(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let names = scripts::registry::container_names();
    jstr(&mut env, &names.join("|"))
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// `nativeFireEvent(String, String)`: fire a named event with a `key=value;...` context.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeFireEvent(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
    context: JString,
) -> jint {
    let name = get_string(&mut env, &name);
    let context = get_string(&mut env, &context);
    let mut event = denizen_core::events::ScriptEvent::new(&name);
    for pair in context.split(';') {
        if let Some(eq) = pair.find('=') {
            let k = &pair[..eq];
            let v = &pair[eq + 1..];
            event = event.with(k, obj::ObjectTag::new(obj::ElementTag::new(v)));
        }
    }
    denizen_core::events::fire_event(&event) as jint
}

/// `nativeEventCount()`: number of registered events.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeEventCount(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    denizen_core::events::event_count() as jint
}

// ---------------------------------------------------------------------------
// Flags
// ---------------------------------------------------------------------------

/// `nativeFlagSet(String, String, String)`: set a flag on an object id.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeFlagSet(
    mut env: JNIEnv,
    _class: JClass,
    object_id: JString,
    key: JString,
    value: JString,
) -> jboolean {
    let id = get_string(&mut env, &object_id);
    let k = get_string(&mut env, &key);
    let v = get_string(&mut env, &value);
    let tracker = denizen_core::flags::tracker_for(&id);
    tracker.set(&k, Some(obj::ObjectTag::new(obj::ElementTag::new(v))));
    1
}

/// `nativeFlagGet(String, String)`: get a flag value, or empty string.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeFlagGet(
    mut env: JNIEnv,
    _class: JClass,
    object_id: JString,
    key: JString,
) -> jstring {
    let id = get_string(&mut env, &object_id);
    let k = get_string(&mut env, &key);
    let tracker = denizen_core::flags::tracker_for(&id);
    let value = tracker.get(&k).map(|o| o.identify()).unwrap_or_default();
    jstr(&mut env, &value)
}

/// `nativeFlagDelete(String, String)`: delete a flag.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeFlagDelete(
    mut env: JNIEnv,
    _class: JClass,
    object_id: JString,
    key: JString,
) -> jboolean {
    let id = get_string(&mut env, &object_id);
    let k = get_string(&mut env, &key);
    let tracker = denizen_core::flags::tracker_for(&id);
    let existed = tracker.has(&k);
    tracker.remove(&k);
    existed as jboolean
}

// ---------------------------------------------------------------------------
// Debug/utility
// ---------------------------------------------------------------------------

/// `nativeStripColors(String)`: strip color codes from a string.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeStripColors(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let text = get_string(&mut env, &input);
    jstr(&mut env, &denizen_core::util::strip_colors(&text))
}

/// `nativeSetDebug(boolean)`: enable/disable debug logging.
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativeSetDebug(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) {
    denizen_core::util::set_debug_enabled(enabled != 0);
}

/// `nativeVersionMajor()`: returns the engine major version as an int (sanity check).
#[no_mangle]
pub extern "system" fn Java_com_denizenscript_denizen_NativeBridge_nativePing(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    42
}

#[allow(dead_code)]
fn _keep(_: jlong) {}
