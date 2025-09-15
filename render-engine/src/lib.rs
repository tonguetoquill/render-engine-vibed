mod typst_wrapper;
pub mod delta_parser;
pub mod form_processor;

// Re-export only the necessary types for the public API
pub use typst_wrapper::{
    TypstWrapperError,
    OutputFormat,
    RenderConfig,
};

// Re-export parser types
pub use delta_parser::{
    DeltaParser,
    ParserError,
};

pub mod assets;
pub mod macros;

/// Render Typst markup to bytes (returns array of pages for SVG, single item for PDF)
/// 
/// # Arguments
/// * `markup` - The Typst markup string to render
/// * `config` - Optional render configuration (defaults to SVG output)
/// 
/// # Returns
/// * `Ok(Vec<Vec<u8>>)` - Vector of rendered pages as bytes
/// * `Err(TypstWrapperError)` - Compilation or rendering error
/// 
/// # Examples
/// ```
/// use render_engine::{render_markup, RenderConfig, OutputFormat};
/// 
/// // Render as SVG (default)
/// let markup = r#"
///     #set page(width: 8.5in, height: 11in)
///     #set text(font: "Times", size: 12pt)
///     
///     = Hello World
///     
///     This is a test document.
/// "#;
/// 
/// let svg_pages = render_markup(markup, None).unwrap();
/// 
/// // Render as PDF
/// let config = RenderConfig { format: OutputFormat::Pdf };
/// let pdf = render_markup(markup, Some(config)).unwrap();
/// ```
pub fn render_markup(
    markup: &str,
    config: Option<RenderConfig>,
) -> Result<Vec<Vec<u8>>, TypstWrapperError> {
    typst_wrapper::TypstWrapper::render_markup(markup, config)
}

/// Render a Typst form from JSON input
/// 
/// # Arguments
/// * `json_input` - The JSON string representing the Typst form
/// * `config` - Optional render configuration (defaults to SVG output)
/// 
/// # Returns
/// * `Ok(Vec<Vec<u8>>)` - Vector of rendered pages as bytes
/// * `Err(TypstWrapperError)` - Compilation or rendering error
/// 
/// # Examples
/// ```
/// use render_engine::{render_form, RenderConfig, OutputFormat};
/// 
/// // JSON input for the Typst form (official memorandum format)
/// let json_input = r#"
/// {
///     "memo-for": ["Recipient Name"],
///     "from-block": ["Sender Name", "Title", "Organization"], 
///     "subject": "Test Subject",
///     "signature-block": ["Signature Name", "Title"],
///     "body_raw": "Hello, world! This is the memo content."
/// }
/// "#;
/// 
/// // Render the form as SVG
/// let svg_pages = render_form(json_input, None).unwrap();
/// 
/// // Render the form as PDF
/// let config = RenderConfig { format: OutputFormat::Pdf };
/// let pdf = render_form(json_input, Some(config)).unwrap();
/// ```
pub fn render_form(
    json_input: &str,
    config: Option<RenderConfig>,
) -> Result<Vec<Vec<u8>>, TypstWrapperError> {
    typst_wrapper::TypstWrapper::render_form(json_input, config)
}