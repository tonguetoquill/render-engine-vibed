# lib.rs: render-engine high level design doc

WASM binary that exposes typst-wrapper functionality for web browsers and JavaScript environments.

## Implementation Status

✅ **COMPLETED** - The render-engine has been successfully implemented with the following features:

## Exported functions

### `test()`

Calls `TypstWrapper::render()` with a small snippet of dummy Typst markup for testing SVG output. Returns a success message with page count and size information.

### `test_pdf()`

Similar to `test()` but renders to PDF format for testing PDF output functionality.

### `render_typst(markup: string, format?: string)`

Accepts arbitrary Typst markup from JavaScript and renders it to the specified format:
- `markup`: The Typst source code as a string
- `format`: Optional output format ("svg" or "pdf", defaults to "svg")

### `main()`

Initialization function that sets up panic hooks for better error reporting in development.

## Architecture

```
JavaScript ← WASM bindings ← render-engine ← typst-wrapper ← Typst
```

The render-engine acts as a thin WASM wrapper around the typst-wrapper crate, providing web-compatible interfaces while leveraging all the embedded fonts and assets from typst-wrapper.

## Build Output

- WASM binary: `pkg/render_engine_bg.wasm`
- JavaScript bindings: `pkg/render_engine.js` 
- TypeScript definitions: `pkg/render_engine.d.ts`
- Package configuration: `pkg/package.json`

## Testing

- Unit tests verify core functionality
- Interactive HTML test page (`test.html`) for browser testing
- Local server setup for development testing