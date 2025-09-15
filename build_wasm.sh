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

if [ "$BUILD_MODE" = "debug" ]; then
    echo "==> Building debug WASM package with wasm-pack ($CRATE)…"
    WASM_PACK_FLAGS="--dev"
    WASM_OPT_FLAGS="-O1"
else
    echo "==> Building optimized WASM package with wasm-pack ($CRATE)…"
    WASM_PACK_FLAGS=""
    WASM_OPT_FLAGS="-Oz --enable-bulk-memory --enable-nontrapping-float-to-int --detect-features --enable-simd"
fi

# Ensure getrandom uses the wasm_js backend for wasm32-unknown-unknown
export RUSTFLAGS="${RUSTFLAGS:-} --cfg getrandom_backend=\"wasm_js\""

# Build with wasm-pack (bundler target for better integration)
cd "$CRATE"
wasm-pack build \
  --target bundler \
  --out-dir pkg \
  $WASM_PACK_FLAGS
cd ..

# Optimize the generated WASM with wasm-opt
WASM_FILE="$PKG_DIR/wasm_wrapper_bg.wasm"
if [ -f "$WASM_FILE" ]; then
    echo "==> Optimizing WASM binary with wasm-opt ($WASM_OPT_FLAGS)…"
    wasm-opt $WASM_OPT_FLAGS "$WASM_FILE" -o "$WASM_FILE.opt"
    mv "$WASM_FILE.opt" "$WASM_FILE"
    
    # Create gzipped version for size comparison
    gzip -9 -c "$WASM_FILE" > "$WASM_FILE.gz"
    
    echo "==> Build complete!"
    echo "    WASM file: $WASM_FILE"
    echo "    Size: $(du -h "$WASM_FILE" | cut -f1) ($(du -h "$WASM_FILE.gz" | cut -f1) gzipped)"
else
    echo "Warning: Expected WASM file not found at $WASM_FILE"
fi