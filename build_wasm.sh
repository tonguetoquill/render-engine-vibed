#!/bin/bash
set -euo pipefail

echo "==> Building optimized WASM binary (wasm-wrapper)…"

# Always build from root, point at wasm-wrapper
wasm-pack build \
  --manifest-path wasm-wrapper/Cargo.toml \
  --target bundler \
  --release \
  --out-dir wasm-wrapper/pkg \
  --out-name wasm_wrapper

WASM=wasm-wrapper/pkg/wasm_wrapper_bg.wasm
OPT_WASM=wasm-wrapper/pkg/wasm_wrapper_bg.opt.wasm

echo "Size before wasm-opt:"
ls -lh "$WASM" | awk '{print $5, $9}'

echo "==> Running wasm-opt…"
wasm-opt -Oz --detect-features -o "$OPT_WASM" "$WASM"

echo "Size after wasm-opt:"
ls -lh "$OPT_WASM" | awk '{print $5, $9}'

echo "==> Creating gzipped versions for deployment…"
gzip -c "$WASM" > "$WASM.gz"
gzip -c "$OPT_WASM" > "$OPT_WASM.gz"

echo "==> Build complete! Outputs in ./wasm-wrapper/pkg/"
ls -lh wasm-wrapper/pkg/*.wasm wasm-wrapper/pkg/*.wasm.gz | awk '{print $5, $9}'
