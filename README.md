# Render Engine

A Rust-based render engine that compiles Typst markup to PDF documents, designed for generating official USAF memorandums and other documents.

## Architecture

The project consists of two main components:

1. **slate2typst** - A library crate for converting Slate JSON to Typst markup (future implementation)
2. **render-engine** - The main crate that renders Typst markup to PDF and compiles to WASM

## Render Engine

The render-engine crate provides a simple `render()` function that takes Typst markup as input and returns a PDF as a byte array.

### Core Function

```rust
pub fn render(typst_markup: &str) -> Result<Vec<u8>, String>
```

Takes Typst markup string and returns either:
- `Ok(Vec<u8>)` - PDF bytes on successful compilation
- `Err(String)` - Error message if compilation fails

### Features

- ✅ Compiles Typst markup to PDF
- ✅ WASM bindings for web usage
- ✅ Minimal Typst World implementation
- ✅ Error handling and reporting
- ✅ Unit tests with USAF memo examples

## Usage

### Building and Testing

```bash
# Run basic test
cargo run --bin test_render

# Run USAF template test  
cargo run --bin test_usaf

# Run unit tests
cargo test
```

### Building for WASM

```bash
# Build WASM module
./build_wasm.sh

# Or manually:
cd render-engine
wasm-pack build --target web --out-dir ../pkg
```

### Using in Rust

```rust
use render_engine::render;

let typst_markup = r#"
#set page(margin: 1in)
#set text(font: "Times New Roman", size: 12pt)

= My Document

This is a test document.
"#;

match render(typst_markup) {
    Ok(pdf_bytes) => {
        // Save PDF or send to client
        std::fs::write("output.pdf", pdf_bytes)?;
    }
    Err(e) => eprintln!("Render failed: {}", e),
}
```

### Using in JavaScript (WASM)

```javascript
import init, { render_typst } from './pkg/render_engine.js';

await init();

const typstMarkup = `
#set page(margin: 1in)
#set text(font: "Times New Roman", size: 12pt)

= My Document

This is a test document.
`;

try {
    const pdfBytes = render_typst(typstMarkup);
    // Use pdfBytes (Uint8Array) to create blob, download, etc.
    const blob = new Blob([pdfBytes], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    window.open(url);
} catch (error) {
    console.error('Render failed:', error);
}
```

## USAF Memo Example

The render engine can handle complex USAF memo formatting:

```typst
#set page(margin: (top: 1in, bottom: 1in, left: 1in, right: 1in))
#set text(font: "Times New Roman", size: 12pt)

// Header
#align(center)[
  #text(size: 14pt, weight: "bold")[DEPARTMENT OF THE AIR FORCE] \
  #text(size: 12pt, weight: "bold")[123RD EXAMPLE SQUADRON]
]

#v(0.5in)

// Date
#align(right)[9 September 2025]

#v(1em)

// Memo For
#text(weight: "bold")[MEMORANDUM FOR] #h(2em) 123 ES/CC

#v(1em)

// From Block  
#text(weight: "bold")[FROM:] #h(2em) ORG/SYMBOL \
#h(6em) Organization \
#h(6em) Street Address \
#h(6em) City ST  12345-6789

#v(1em)

// Subject
#text(weight: "bold")[SUBJECT:] #h(2em) Format for the Official Memorandum

#v(2em)

// Body content
This is the body of the memorandum...

#v(3em)

// Signature block
#align(right)[
  FIRST M. LAST, Rank, USAF \
  Duty Title
]
```

## Testing Output

The test binaries generate PDF files you can inspect:
- `test_output.pdf` - Basic render test output
- `usaf_template_test.pdf` - USAF memo template test output

Both should be valid PDFs that open in any PDF viewer.

## Dependencies

- `typst` - Typst compiler and runtime
- `typst-pdf` - PDF generation from Typst documents  
- `comemo` - Memoization for Typst compilation
- `wasm-bindgen` - WASM bindings for JavaScript interop

## Future Enhancements

- Integration with the `slate2typst` crate for JSON input
- Support for external assets (fonts, images)  
- Template management system
- Enhanced error reporting with line numbers
- Custom font loading for WASM
