# parser.rs design document

This component is responsible for parsing Quill Delta syntax and converting it into Typst markup. It uses native Rust JSON parsing with serde for handling Quill Delta format and converts operations into Typst-compatible markup syntax.

## Implemented Features

The parser supports the following Quill Delta features:

### Text Formatting
- **Bold**: `{"bold": true}` → `*text*`
- **Italic**: `{"italic": true}` → `_text_`
- **Underline**: `{"underline": true}` → `#underline[text]`
- **Strikethrough**: `{"strike": true}` → `#strike[text]`
- **Combined formatting**: Multiple attributes can be combined

### Lists
- **Bullet lists**: `{"list": "bullet"}` → `- item`
- **Ordered lists**: `{"list": "ordered"}` → `+ item`
- **Nested lists**: Support for `{"indent": N}` attribute for nesting levels

### Document Structure
- **Paragraphs**: Automatic paragraph separation on newlines
- **Text blocks**: Continuous text without special formatting

## API

The main interface is the `DeltaParser` struct:

```rust
let mut parser = DeltaParser::new();
let typst_markup = parser.parse(delta_json)?;
```

## Error Handling

The parser provides comprehensive error handling through the `ParserError` enum:
- Invalid JSON format errors
- Unsupported operation errors
- Malformed Delta structure errors

## Future Expansion

The architecture supports easy addition of new features:
- Headers/headings
- Links and images
- Tables
- Block quotes
- Code blocks

