package com.denizenscript.denizen;

import org.bukkit.Bukkit;
import org.bukkit.command.Command;
import org.bukkit.command.CommandSender;
import org.bukkit.plugin.java.JavaPlugin;

import java.util.logging.Level;

/**
 * Denizen, the bukkit plugin entry point.
 *
 * <p>This shim does nothing but load the native Rust engine and forward lifecycle calls to it.
 * All scripting, tag parsing, command handling, event firing, and flag storage live in the
 * native library. This class is a tiny loader and should remain as small as possible.</p>
 */
public class Denizen extends JavaPlugin {

    private static Denizen instance;
    private boolean nativeReady = false;

    public static Denizen getInstance() {
        return instance;
    }

    public static boolean isNativeReady() {
        return instance != null && instance.nativeReady;
    }

    @Override
    public void onEnable() {
        instance = this;
        long start = System.currentTimeMillis();
        try {
            NativeBridge.loadLibrary();
            NativeBridge.nativeInit();
            nativeReady = true;
            String version = NativeBridge.nativeVersion();
            String stats = NativeBridge.nativeStats();
            getLogger().info("Denizen native engine v" + version + " loaded.");
            getLogger().info(stats);
            getLogger().info("Loaded in " + (System.currentTimeMillis() - start) + "ms.");
            // Reload scripts from the plugin's scripts folder.
            getDataFolder().mkdirs();
            int scripts = NativeBridge.nativeReloadScripts();
            getLogger().info("Loaded " + scripts + " script containers.");
        } catch (Throwable error) {
            getLogger().log(Level.SEVERE, "Failed to load the native Denizen engine!", error);
            nativeReady = false;
        }
    }

    @Override
    public void onDisable() {
        if (nativeReady) {
            try {
                NativeBridge.nativeFlagSet("server", "shutdown_time",
                        String.valueOf(System.currentTimeMillis()));
            } catch (Throwable ignored) {
            }
        }
        nativeReady = false;
        instance = null;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!nativeReady) {
            sender.sendMessage("Denizen native engine is not loaded.");
            return true;
        }
        switch (command.getName().toLowerCase()) {
            case "denizen": {
                if (args.length == 0) {
                    sender.sendMessage(NativeBridge.nativeStats());
                    return true;
                }
                switch (args[0].toLowerCase()) {
                    case "version":
                        sender.sendMessage("Denizen native v" + NativeBridge.nativeVersion());
                        return true;
                    case "stats":
                        sender.sendMessage(NativeBridge.nativeStats());
                        return true;
                    case "reload": {
                        int count = NativeBridge.nativeReloadScripts();
                        sender.sendMessage("Reloaded " + count + " script containers.");
                        return true;
                    }
                    case "scripts":
                        sender.sendMessage(NativeBridge.nativeScriptNames().replace("|", ", "));
                        return true;
                    default:
                        sender.sendMessage("Usage: /denizen <version|stats|reload|scripts>");
                        return true;
                }
            }
            case "ex":
            case "exs": {
                if (args.length == 0) {
                    sender.sendMessage("Usage: /ex <command>");
                    return true;
                }
                String line = String.join(" ", args);
                String result = NativeBridge.nativeExecuteCommand(line);
                sender.sendMessage(NativeBridge.nativeStripColors(result));
                return true;
            }
            case "denizenclickable":
                return true;
            default:
                return false;
        }
    }
}
