package com.denizenscript.denizen;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.Locale;

/**
 * NativeLoader extracts the bundled native library from the jar to a temp file and returns its
 * absolute path, so {@link NativeBridge} can {@code System.load} it.
 */
public final class NativeLoader {

    private NativeLoader() {
    }

    private static String cachedPath = null;

    /** Determine the resource path for the current OS/arch. */
    public static String resourcePath() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        String arch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
        String osName;
        String libName;
        if (os.contains("win")) {
            osName = "windows";
            libName = "denizen.dll";
        } else if (os.contains("mac") || os.contains("darwin")) {
            osName = "macos";
            libName = "libdenizen.dylib";
        } else {
            osName = "linux";
            libName = "libdenizen.so";
        }
        String archName = (arch.contains("aarch64") || arch.contains("arm")) ? "aarch64" : "x86_64";
        return "natives/" + osName + "-" + archName + "/" + libName;
    }

    /** Extract the native library to a temp file, returning its absolute path (or null). */
    public static synchronized String extractAndGetPath() {
        if (cachedPath != null) {
            return cachedPath;
        }
        String resource = resourcePath();
        try (InputStream in = NativeLoader.class.getClassLoader().getResourceAsStream(resource)) {
            if (in == null) {
                return null;
            }
            String fileName = resource.substring(resource.lastIndexOf('/') + 1);
            File temp = File.createTempFile("denizen-native-", "-" + fileName);
            temp.deleteOnExit();
            try (OutputStream out = new FileOutputStream(temp)) {
                byte[] buffer = new byte[8192];
                int read;
                while ((read = in.read(buffer)) != -1) {
                    out.write(buffer, 0, read);
                }
            }
            temp.setExecutable(true, false);
            cachedPath = temp.getAbsolutePath();
            return cachedPath;
        } catch (IOException e) {
            return null;
        }
    }
}
