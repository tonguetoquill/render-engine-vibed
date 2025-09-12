#!/bin/bash

# Build optimized release WebAssembly binary
echo "Building optimized WASM binary..."

# Build with wasm-pack for release
# wasm-opt optimization is configured in Cargo.toml
wasm-pack build --target bundler --release --out-dir pkg

echo "Creating gzipped version for deployment..."
gzip -c pkg/wasm_wrapper_bg.wasm > pkg/wasm_wrapper_bg.wasm.gz

echo "Build complete! Output in ./pkg/"
echo "File sizes:"
ls -lh pkg/*.wasm pkg/*.wasm.gz | awk '{print $5 " " $9}'