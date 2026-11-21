#!/usr/bin/env bash
#
# build.sh, build the native engine (cdylib) and stage it for the java plugin.
#
# Usage: ./build.sh [--debug]
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PROFILE="release"
CARGO_FLAGS="--release"
if [[ "${1:-}" == "--debug" ]]; then
    PROFILE="debug"
    CARGO_FLAGS=""
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$SCRIPT_DIR/target}"

echo "==> Building native Denizen engine ($PROFILE)..."
cargo build -p denizen-jni $CARGO_FLAGS

# Locate the produced dynamic library.
OS="$(uname -s)"
case "$OS" in
    Linux*)   LIB_NAME="libdenizen.so"    ; OS_DIR="linux"   ;;
    Darwin*)  LIB_NAME="libdenizen.dylib" ; OS_DIR="macos"   ;;
    MINGW*|MSYS*|CYGWIN*) LIB_NAME="denizen.dll" ; OS_DIR="windows" ;;
    *)        LIB_NAME="libdenizen.so"    ; OS_DIR="linux"   ;;
esac

ARCH="$(uname -m)"
case "$ARCH" in
    aarch64|arm64) ARCH_DIR="aarch64" ;;
    *)             ARCH_DIR="x86_64"  ;;
esac

SRC="$CARGO_TARGET_DIR/$PROFILE/$LIB_NAME"
if [[ ! -f "$SRC" ]]; then
    echo "ERROR: native library not found at $SRC" >&2
    exit 1
fi

DEST_DIR="$SCRIPT_DIR/java-shim/src/main/resources/natives/${OS_DIR}-${ARCH_DIR}"
mkdir -p "$DEST_DIR"
cp "$SRC" "$DEST_DIR/$LIB_NAME"

echo "==> Native library: $SRC"
echo "==> Staged to:      $DEST_DIR/$LIB_NAME"
echo "==> Done. Use 'cargo test' to run the test suite, and gradle in java-shim/ to package the jar."
