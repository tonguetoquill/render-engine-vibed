#!/bin/bash

# Ultra-optimized WASM build script with additional post-processing
echo "Building ultra-optimized WASM binary..."

# Clean previous builds
rm -rf pkg/

# Build with wasm-pack for release with maximum optimizations
echo "Step 1: Building with wasm-pack..."
wasm-pack build --target bundler --release --out-dir pkg

# Check if we have wasm-strip available for additional stripping
if command -v wasm-strip &> /dev/null; then
    echo "Step 2: Additional stripping with wasm-strip..."
    wasm-strip pkg/wasm_wrapper_bg.wasm
else
    echo "Step 2: wasm-strip not available, skipping additional stripping"
fi

# Show final size
echo "Final WASM size:"
ls -lh pkg/*.wasm

# Show compression potential
if command -v gzip &> /dev/null; then
    echo "Gzipped size (typical web compression):"
    gzip -c pkg/wasm_wrapper_bg.wasm | wc -c | awk '{printf "%.2f MB\n", $1/1024/1024}'
fi

echo "Ultra-optimized build complete! Output in ./pkg/"