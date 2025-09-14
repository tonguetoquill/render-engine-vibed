#!/bin/bash
set -euo pipefail

# Parse command line arguments
BUILD_MODE="debug"
if [ $# -gt 0 ]; then
    case "$1" in
        debug|--debug|-d)
            BUILD_MODE="debug"
            ;;
        release|--release|-r)
            BUILD_MODE="release"
            ;;
        -h|--help)
            echo "Usage: $0 [debug|release]"
            echo "  debug    Build in debug mode (faster compilation, larger binary)"
            echo "  release  Build in release mode with optimizations (default)"
            exit 0
            ;;
        *)
            echo "Error: Unknown argument '$1'"
            echo "Usage: $0 [debug|release]"
            exit 1
            ;;
    esac
fi

CRATE=wasm-wrapper
PKG_DIR="$CRATE/pkg"
TARGET_DIR="target/wasm32-unknown-unknown/$BUILD_MODE"
OUT_NAME=wasm_wrapper

if [ "$BUILD_MODE" = "debug" ]; then
    echo "==> Building debug WASM binary ($CRATE)…"
    CARGO_FLAGS=""
    WASM_OPT_FLAGS="-O1"
else
    echo "==> Building optimized WASM binary ($CRATE)…"
    CARGO_FLAGS="--release"
    WASM_OPT_FLAGS="-Oz"
fi

# Ensure wasm target exists
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

# 1) Compile the crate to wasm (from the workspace root)
cargo build -p "$CRATE" $CARGO_FLAGS --target wasm32-unknown-unknown

# 2) Generate JS bindings (web target) from the produced .wasm
mkdir -p "$PKG_DIR"
wasm-bindgen \
  --target web \
  --out-dir "$PKG_DIR" \
  --out-name "$OUT_NAME" \
  "$TARGET_DIR/${CRATE//-/_}.wasm"

WASM="$PKG_DIR/${OUT_NAME}_bg.wasm"
OPT_WASM="$PKG_DIR/${OUT_NAME}_bg.opt.wasm"

echo "Size before wasm-opt:"
ls -lh "$WASM" | awk '{print $5, $9}'

# 3) Optimize
if [ "$BUILD_MODE" = "debug" ]; then
    echo "==> Skipping wasm-opt in debug mode (creating symlink to original file)…"
    ln -sf "$(basename "$WASM")" "$OPT_WASM"
else
    echo "==> Running wasm-opt (full optimization for release)…"
    wasm-opt $WASM_OPT_FLAGS --enable-bulk-memory --enable-nontrapping-float-to-int --detect-features \
      -o "$OPT_WASM" "$WASM"
fi

echo "Size after processing:"
ls -lh "$OPT_WASM" | awk '{print $5, $9}'

# 4) Gzip artifacts
echo "==> Creating gzipped versions for deployment…"
gzip -c "$WASM" > "$WASM.gz"
gzip -c "$OPT_WASM" > "$OPT_WASM.gz"

echo "==> Build complete! Outputs in ./$PKG_DIR"
ls -lh "$PKG_DIR"/*.{wasm,wasm.gz} | awk '{print $5, $9}'
