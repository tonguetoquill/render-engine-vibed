# Render Engine Design Document

This project is ultimately a Rust web assembly  that renders official USAF memos from Slate JSON. It uses the tonguetoquill-usaf-memo Typst package `official-memorandum` function.

## Crates

The project consists of multiple crates within a monorepo:

### slate2typst

Transpiles deserialized Slate JSON to Typst content blocks.

#### Dependencies

- {whatever is needed}

#### Input

Deserialized Typst JSON object.

#### Output

A Typst markup document that represents the official memorandum. 

#### Implementation Details

- See `tonguetoquill-usaf-memo` for example usage of the Typst markup `official-memorandum` function.

### typst-wrapper

Orchestrates Typst crates for our functionality.

#### Dependencies

- typst
- typst-svg

#### Input

Typst markup document (string)

#### Output

Rendered SVG byte array

### Rendering Service

This component orchestrates the workflow for rendering a Typst document from Slate JSON.

#### Dependencies

- slate2typst
- serde_json

#### Input

A JSON object conforming to `schemas/memo-render-input.json`.

#### Output

A byte array representing the rendered Typst document.

#### Workflow

1. Parse the input JSON to extract memo details.
1. Transpile the extracted body into Typst markup.
1. Use a templating engine to combine the Typst markup with the memo metadata.
1. Render the final document and return it as a byte array.

#### Build

- Builds into a web app-friendly WASM module