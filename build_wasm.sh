#!/bin/bash
set -euo pipefail

CRATE=wasm-wrapper
PKG_DIR="$CRATE/pkg"
TARGET_DIR=target/wasm32-unknown-unknown/release
OUT_NAME=wasm_wrapper

echo "==> Building optimized WASM binary ($CRATE)…"

# Ensure wasm target exists
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

# 1) Compile the crate to wasm (from the workspace root)
cargo build -p "$CRATE" --release --target wasm32-unknown-unknown

# 2) Generate JS bindings (bundler target) from the produced .wasm
mkdir -p "$PKG_DIR"
wasm-bindgen \
  --target bundler \
  --out-dir "$PKG_DIR" \
  --out-name "$OUT_NAME" \
  "$TARGET_DIR/${CRATE//-/_}.wasm"

WASM="$PKG_DIR/${OUT_NAME}_bg.wasm"
OPT_WASM="$PKG_DIR/${OUT_NAME}_bg.opt.wasm"

echo "Size before wasm-opt:"
ls -lh "$WASM" | awk '{print $5, $9}'

# 3) Optimize
echo "==> Running wasm-opt…"
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --detect-features \
  -o "$OPT_WASM" "$WASM"

echo "Size after wasm-opt:"
ls -lh "$OPT_WASM" | awk '{print $5, $9}'

# 4) Gzip artifacts
echo "==> Creating gzipped versions for deployment…"
gzip -c "$WASM" > "$WASM.gz"
gzip -c "$OPT_WASM" > "$OPT_WASM.gz"

echo "==> Build complete! Outputs in ./$PKG_DIR"
ls -lh "$PKG_DIR"/*.{wasm,wasm.gz} | awk '{print $5, $9}'
