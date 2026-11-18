plugins {
    java
}

group = "com.denizenscript"
version = "1.3.4-native"

repositories {
    mavenCentral()
    maven("https://repo.papermc.io/repository/maven-public/")
}

dependencies {
    compileOnly("io.papermc.paper:paper-api:1.21.4-R0.1-SNAPSHOT")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
}

// Package the native library for the current platform into the jar's natives/ tree.
tasks.processResources {
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()
    val osName = when {
        os.contains("win") -> "windows"
        os.contains("mac") || os.contains("darwin") -> "macos"
        else -> "linux"
    }
    val archName = if (arch.contains("aarch64") || arch.contains("arm")) "aarch64" else "x86_64"
    val libName = when (osName) {
        "windows" -> "denizen.dll"
        "macos" -> "libdenizen.dylib"
        else -> "libdenizen.so"
    }
    val nativeFile = file("../target/release/$libName")
    if (nativeFile.exists()) {
        from(nativeFile) {
            into("natives/$osName-$archName")
        }
    } else {
        logger.warn("Native library not found at ${nativeFile.absolutePath}; run ../build.sh first.")
    }
}

tasks.jar {
    archiveBaseName.set("Denizen")
    archiveClassifier.set("")
    manifest {
        attributes(
            "Implementation-Title" to "Denizen",
            "Implementation-Version" to project.version,
        )
    }
}
