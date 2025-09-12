mod typst_wrapper;

// Re-export only the necessary types for the public API
pub use typst_wrapper::{
    TypstWrapperError,
    OutputFormat,
    RenderConfig,
};

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
    typst_wrapper::TypstWrapper::render(markup, config)
}