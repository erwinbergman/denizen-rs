package com.denizenscript.denizen;

/**
 * NativeBridge, the java side of the native engine.
 *
 * <p>This class is intentionally tiny: every method is a {@code native} declaration that maps
 * 1:1 to an exported symbol in the Rust cdylib ({@code libdenizen.so}/{@code .dll}/{@code .dylib}).
 * All real logic lives in Rust. This file exists only to load and invoke the native library.</p>
 */
public final class NativeBridge {

    private NativeBridge() {
    }

    private static boolean loaded = false;

    /** The name of the native library (without platform prefix/extension). */
    public static final String LIBRARY_NAME = "denizen";

    /**
     * Load the native library. The plugin calls this from {@code onEnable()}.
     *
     * @return true if the library is loaded (or already was)
     */
    public static synchronized boolean loadLibrary() {
        if (loaded) {
            return true;
        }
        try {
            System.loadLibrary(LIBRARY_NAME);
            loaded = true;
            return true;
        } catch (UnsatisfiedLinkError error) {
            try {
                // Fall back to an explicit path extraction handled by the plugin.
                String path = NativeLoader.extractAndGetPath();
                if (path != null) {
                    System.load(path);
                    loaded = true;
                    return true;
                }
            } catch (Throwable ignored) {
            }
            throw error;
        }
    }

    public static boolean isLoaded() {
        return loaded;
    }

    // ------------------------------------------------------------------
    // Engine lifecycle
    // ------------------------------------------------------------------

    /** Initialize the native engine (registers object types, tags, commands, events). */
    public static native void nativeInit();

    /** Returns the native engine version string. */
    public static native String nativeVersion();

    /** Returns engine statistics. */
    public static native String nativeStats();

    /** Number of registered commands. */
    public static native int nativeCommandCount();

    /** Number of registered object types. */
    public static native int nativeObjectTypeCount();

    // ------------------------------------------------------------------
    // Tags
    // ------------------------------------------------------------------

    /** Parse a tag string and return the resolved text. */
    public static native String nativeTag(String input);

    /** Parse a tag string and return the object identify form. */
    public static native String nativeTagObject(String input);

    // ------------------------------------------------------------------
    // Scripts
    // ------------------------------------------------------------------

    /** Reload scripts from the scripts folder, returning the number of containers loaded. */
    public static native int nativeReloadScripts();

    /** Reload scripts from an explicit folder path, returning the number of containers loaded. */
    public static native int nativeReloadScriptsFrom(String folder);

    /** Number of loaded script containers. */
    public static native int nativeScriptCount();

    /** Run a script by name; returns entries executed, or -1 on error. */
    public static native int nativeRunScript(String name);

    /** Execute a raw command line; returns "ok:<n>" or "error:<message>". */
    public static native String nativeExecuteCommand(String line);

    /** Returns a {@code |}-separated list of loaded script names. */
    public static native String nativeScriptNames();

    // ------------------------------------------------------------------
    // Events
    // ------------------------------------------------------------------

    /** Fire a named event with a {@code key=value;...} context; returns script fires. */
    public static native int nativeFireEvent(String name, String context);

    /** Number of registered events. */
    public static native int nativeEventCount();

    // ------------------------------------------------------------------
    // Flags
    // ------------------------------------------------------------------

    /** Set a flag on an object id. */
    public static native boolean nativeFlagSet(String objectId, String key, String value);

    /** Get a flag value, or an empty string. */
    public static native String nativeFlagGet(String objectId, String key);

    /** Delete a flag; returns whether it existed. */
    public static native boolean nativeFlagDelete(String objectId, String key);

    // ------------------------------------------------------------------
    // Utilities
    // ------------------------------------------------------------------

    /** Strip Denizen color codes from a string. */
    public static native String nativeStripColors(String input);

    /** Enable or disable native debug logging. */
    public static native void nativeSetDebug(boolean enabled);

    /** Sanity check: returns 42 when the bridge is wired correctly. */
    public static native int nativePing();
}
