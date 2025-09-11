# Render Engine

A WebAssembly (WASM) binary that exposes [typst-wrapper](../typst-wrapper) functionality for web browsers and JavaScript environments.

## Overview

The render-engine provides a WASM interface to render Typst documents to SVG or PDF formats. It wraps the functionality of the typst-wrapper crate and makes it accessible from JavaScript.

## Features

- **WASM Binary**: Compiled as a WebAssembly module for use in browsers
- **SVG Output**: Render Typst documents as SVG for web display
- **PDF Output**: Render Typst documents as PDF for printing/download
- **Test Functions**: Built-in test functions for validation
- **Custom Rendering**: Accept arbitrary Typst markup from JavaScript

## Exported Functions

### `test()`
A test function that calls `typst-wrapper.render()` with a small snippet of dummy Typst markup for testing SVG output.

Returns: Result with success message or error details.

### `test_pdf()`
Similar to `test()` but renders to PDF format.

Returns: Result with success message or error details.

### `render_typst(markup: string, format?: string)`
Renders arbitrary Typst markup from JavaScript.

- `markup`: The Typst source code as a string
- `format`: Optional output format ("svg" or "pdf", defaults to "svg")

Returns: Result with success message or error details.

## Building

To build the WASM module:

```bash
wasm-pack build --target web --out-dir pkg
```

This creates the WASM binary and JavaScript bindings in the `pkg/` directory.

## Usage

### In a Web Page

```html
<script type="module">
import init, { test, test_pdf, render_typst } from './pkg/render_engine.js';

async function main() {
    // Initialize the WASM module
    await init();
    
    // Test basic functionality
    const result = test();
    console.log('Test result:', result);
    
    // Render custom Typst markup
    const custom = render_typst(`
        = Hello World
        This is *bold* and _italic_ text.
    `, 'svg');
    console.log('Custom render:', custom);
}

main();
</script>
```

### Testing

Open `test.html` in a web browser to test the functionality interactively. The test page includes:

- Built-in test buttons for SVG and PDF rendering
- A text area for custom Typst markup input
- Real-time output display

## Dependencies

- [typst-wrapper](../typst-wrapper) - The core Typst rendering functionality
- wasm-bindgen - For generating JavaScript bindings
- web-sys - For web API access
- console_error_panic_hook - For better error reporting in development

## Architecture

The render-engine acts as a thin WASM wrapper around the typst-wrapper crate:

```
JavaScript ← WASM bindings ← render-engine ← typst-wrapper ← Typst
```

All the heavy lifting is done by the typst-wrapper, which includes embedded fonts and assets. The render-engine simply provides a WASM-compatible interface.
