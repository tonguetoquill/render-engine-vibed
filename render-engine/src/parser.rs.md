# parser.rs design document

This component is responsible for parsing Quill Delta syntax and converting it into Typst markup. It uses the quill-delta-rs crate for parsing Markdown and potentially the typst crate for generating Typst markup (though not required). It supports the following Quill Delta features for my narrow use case:

- Paragraphs
- Nested lists
- Nested enums
- Styles: bold, italic, underline, strikethrough
- Potentially more with phased approach

