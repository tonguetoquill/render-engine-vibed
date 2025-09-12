# Render Engine Design Document

This project is ultimately a Rust web assembly that renders official USAF memos from Slate JSON. It uses the tonguetoquill-usaf-memo Typst package `official-memorandum` function.

## Crates

The project consists of multiple crates within a workspace:

### slate2typst (Separate Crate)

A standalone library crate that transpiles deserialized Slate JSON to Typst content blocks.

#### Dependencies

- serde
- serde_json

#### Input

Deserialized Slate JSON object (`serde_json::Value`).

#### Output

A Typst markup string that represents the memo body content.

#### Implementation Details

- Currently implements a mock transpiler that returns sample memo content
- Future implementation will parse Slate.js document structure and convert to appropriate Typst markup
- Can be compiled and used independently of the main render service
- See `tonguetoquill-usaf-memo` for example usage of the Typst markup `official-memorandum` function

### render-engine (Main WASM Crate)

This is the main crate that compiles to WASM. It orchestrates the workflow for rendering a Typst document from Slate JSON.

#### Dependencies

- Crates:
    - slate2typst (local workspace crate)
    - serde
    - serde_json
    - typst
    - typst-svg
    - comemo
- Fonts:
    - Arial
    - Times New Roman
    - tonguetoquill-usaf-memo/assets/fonts/CopperplateCC-Heavy.otf
- {more as needed}

#### Input

A JSON object conforming to `schemas/memo-render-input.json`.

#### Output

A byte array representing the rendered Typst document.

#### Workflow

1. Parse the input JSON to extract memo details.
1. Transpile the extracted body into Typst markup.
1. Use a templating engine to combine the Typst markup with the memo metadata.
1. Render the final SVG document and return it as a byte array.

#### Implementation

- Uses a minimal Typst `World` implementation for compilation
- Currently mocks the `official-memorandum` function since we can't easily bundle the Typst package in WASM
- Implements WASM bindings for JavaScript interaction
- Consider how to manage Typst package dependencies (i.e., @preview/tonguetoquill-usaf-memo:0.0.3) in future iterations

#### Build

- Compiles to a web app-friendly WASM module using `wasm-pack`