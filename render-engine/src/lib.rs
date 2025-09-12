use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::{DateTime, Datelike, Utc};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source, VirtualPath, package::PackageSpec};
use typst::text::{Font, FontBook, FontInfo};
use typst::utils::LazyHash;
use typst::{Library, World};

// Embedded assets - fonts
static DOD_SEAL: &[u8] = include_bytes!("../assets/dod_seal.gif");
static ARIAL_FONT: &[u8] = include_bytes!("../assets/arial.ttf");
static TIMES_FONT: &[u8] = include_bytes!("../assets/times.ttf");
static TIMES_TTC_FONT: &[u8] = include_bytes!("../assets/Times.ttc");
static COPPERPLATE_FONT: &[u8] = include_bytes!("../assets/CopperplateCC-Heavy.otf");



// Embedded package files for @preview/tonguetoquill-usaf-memo:0.0.3
static PACKAGE_TYPST_TOML: &str = include_str!("../tonguetoquill-usaf-memo/typst.toml");
static PACKAGE_LIB_TYP: &str = include_str!("../tonguetoquill-usaf-memo/src/lib.typ");
static PACKAGE_UTILS_TYP: &str = include_str!("../tonguetoquill-usaf-memo/src/utils.typ");

// Static font collections initialized at compile time
static FONT_BOOK: LazyLock<LazyHash<FontBook>> = LazyLock::new(|| {
    let mut book = FontBook::new();
    
    // Load all embedded fonts
    if let Some(info) = FontInfo::new(ARIAL_FONT, 0) {
        book.push(info);
    }
    
    if let Some(info) = FontInfo::new(TIMES_FONT, 0) {
        book.push(info);
    }
    
    // Times.ttc is a collection, try multiple indices
    for i in 0..10 {
        if let Some(info) = FontInfo::new(TIMES_TTC_FONT, i) {
            book.push(info);
        } else {
            break;
        }
    }
    
    if let Some(info) = FontInfo::new(COPPERPLATE_FONT, 0) {
        book.push(info);
    }
    
    LazyHash::new(book)
});

// Static font vector for font() method access
static FONTS: LazyLock<Vec<Font>> = LazyLock::new(|| {
    let mut fonts = Vec::new();
    
    // Load all embedded fonts using Font::iter
    for font in Font::iter(Bytes::new(ARIAL_FONT)) {
        fonts.push(font);
    }
    
    for font in Font::iter(Bytes::new(TIMES_FONT)) {
        fonts.push(font);
    }
    
    for font in Font::iter(Bytes::new(TIMES_TTC_FONT)) {
        fonts.push(font);
    }
    
    for font in Font::iter(Bytes::new(COPPERPLATE_FONT)) {
        fonts.push(font);
    }
    
    fonts
});

/// Error types for the Typst wrapper
#[derive(Debug)]
pub enum TypstWrapperError {
    Compilation(String),
    Font(String),
    OutputFormat(String),
    FileNotFound(String),
    Io(std::io::Error),
}

impl std::fmt::Display for TypstWrapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypstWrapperError::Compilation(msg) => write!(f, "Compilation failed: {}", msg),
            TypstWrapperError::Font(msg) => write!(f, "Font loading error: {}", msg),
            TypstWrapperError::OutputFormat(msg) => write!(f, "Output format error: {}", msg),
            TypstWrapperError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            TypstWrapperError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for TypstWrapperError {}

impl From<std::io::Error> for TypstWrapperError {
    fn from(error: std::io::Error) -> Self {
        TypstWrapperError::Io(error)
    }
}

/// Output format configuration
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Svg,
    Pdf,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Svg
    }
}

/// Render configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub format: OutputFormat,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Svg,
        }
    }
}

/// Stateless Typst wrapper with embedded assets
#[derive(Debug)]
pub struct TypstWrapper;

impl TypstWrapper {
    /// Create a new wrapper instance (zero-cost)
    pub fn new() -> Self {
        Self
    }
    
    /// Render Typst markup to bytes (returns array of pages for SVG, single item for PDF)
    pub fn render(
        markup: &str,
        config: Option<RenderConfig>,
    ) -> Result<Vec<Vec<u8>>, TypstWrapperError> {
        let config = config.unwrap_or_default();
        
        // Create a fresh world for this render
        let mut world = TypstWorld::new();
        
        // Parse the main source
        let source = Source::new(FileId::new(None, VirtualPath::new("main.typ")), markup.to_string());
        world.insert_source(source.clone());
        
        // Compile the document
        let document = match typst::compile::<PagedDocument>(&world).output {
            Ok(doc) => doc,
            Err(errors) => {
                let error_msg = errors
                    .into_iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(TypstWrapperError::Compilation(error_msg));
            }
        };
        
                // Generate output based on format
        match config.format {
            OutputFormat::Svg => {
                // Render all pages as SVG
                let mut svg_pages = Vec::new();
                for page in &document.pages {
                    let svg = typst_svg::svg(page);
                    svg_pages.push(svg.into_bytes());
                }
                
                if svg_pages.is_empty() {
                    Err(TypstWrapperError::Compilation("No pages to render".to_string()))
                } else {
                    Ok(svg_pages)
                }
            }
            OutputFormat::Pdf => {
                let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
                    .map_err(|e| TypstWrapperError::Compilation(format!("PDF generation failed: {:?}", e)))?;
                Ok(vec![pdf])
            }
        }
    }
}

impl Default for TypstWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal Typst world implementation
struct TypstWorld {
    library: LazyHash<Library>,
    sources: HashMap<FileId, Source>,
    package_sources: HashMap<FileId, Source>,
    now: DateTime<Utc>,
}

impl TypstWorld {
    fn new() -> Self {
        Self {
            library: LazyHash::new(Library::default()),
            sources: HashMap::new(),
            package_sources: HashMap::new(),
            now: Utc::now(),
        }
    }
    
    fn insert_source(&mut self, source: Source) {
        self.sources.insert(source.id(), source);
    }
    
    fn resolve_asset(&self, path: &str) -> Option<&'static [u8]> {
        match path {
            "assets/dod_seal.gif" => Some(DOD_SEAL),
            _ => None,
        }
    }
    
    fn resolve_package_file(&self, spec: &PackageSpec, path: &str) -> Option<&'static str> {
        // Handle the tonguetoquill-usaf-memo package
        if spec.namespace == "preview" && spec.name == "tonguetoquill-usaf-memo" && spec.version.to_string() == "0.0.3" {
            match path {
                "typst.toml" => Some(PACKAGE_TYPST_TOML),
                "src/lib.typ" => Some(PACKAGE_LIB_TYP),
                "src/utils.typ" => Some(PACKAGE_UTILS_TYP),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    
    fn book(&self) -> &LazyHash<FontBook> {
        &FONT_BOOK
    }
    
    fn main(&self) -> FileId {
        self.sources
            .values()
            .find(|source| source.id().package().is_none())
            .unwrap()
            .id()
    }
    
    fn source(&self, id: FileId) -> FileResult<Source> {
        // Check main sources first
        if let Some(source) = self.sources.get(&id) {
            return Ok(source.clone());
        }
        
        // Check package sources
        if let Some(source) = self.package_sources.get(&id) {
            return Ok(source.clone());
        }
        
        // Try to load package source
        if let Some(spec) = id.package() {
            let path = id.vpath().as_rootless_path().to_string_lossy();
            if let Some(content) = self.resolve_package_file(&spec, &path) {
                let source = Source::new(id, content.to_string());
                // We can't mutate self here, but we can return the source
                return Ok(source);
            }
        }
        
        Err(FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
    }
    
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().as_rootless_path().to_string_lossy();
        
        // Try to resolve as embedded asset
        if let Some(data) = self.resolve_asset(&path) {
            return Ok(Bytes::new(data));
        }
        
        // Try package files
        if let Some(spec) = id.package() {
            if let Some(content) = self.resolve_package_file(&spec, &path) {
                return Ok(Bytes::new(content.as_bytes()));
            }
        }
        
        // File not found
        Err(FileError::NotFound(id.vpath().as_rootless_path().to_path_buf()))
    }
    
    fn font(&self, index: usize) -> Option<Font> {
        FONTS.get(index).cloned()
    }
    
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset_duration = chrono::Duration::hours(offset.unwrap_or(0));
        let datetime = self.now + offset_duration;
        
        Datetime::from_ymd(
            datetime.year(),
            datetime.month() as u8,
            datetime.day() as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wrapper_creation() {
        let wrapper = TypstWrapper::new();
        assert!(std::mem::size_of_val(&wrapper) == 0);
    }
    
    #[test]
    fn test_simple_render() {
        let markup = r#"
            #set page(width: 8.5in, height: 11in)
            #set text(font: "Times", size: 12pt)
            
            = Test Document
            
            This is a test document.
        "#;
        
        let result = TypstWrapper::render(markup, None);
        assert!(result.is_ok());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert!(!pages[0].is_empty());
    }
    
    #[test] 
    fn test_pdf_render() {
        let markup = r#"
            #set page(width: 8.5in, height: 11in)
            #set text(font: "Times", size: 12pt)
            
            = PDF Test Document
            
            This should render as PDF.
        "#;
        
        let config = RenderConfig {
            format: OutputFormat::Pdf,
        };
        
        let result = TypstWrapper::render(markup, Some(config));
        assert!(result.is_ok());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert_eq!(pages.len(), 1); // PDF returns single item
        
        // PDF files start with %PDF
        assert!(pages[0].starts_with(b"%PDF"));
    }
    
    #[test]
    fn test_package_import() {
        // Test that the package system works
        let markup = r#"
            #import "@preview/tonguetoquill-usaf-memo:0.0.3": official-memorandum
            
            #set page(width: 8.5in, height: 11in)
            #set text(font: "Times", size: 12pt)
            
            = Package Import Test
            
            The package imported successfully.
        "#;
        
        let result = TypstWrapper::render(markup, None);
        assert!(result.is_ok(), "Package import should work: {:?}", result.err());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert!(!pages[0].is_empty());
    }
    
    #[test]
    fn test_asset_loading() {
        // Test that embedded assets can be loaded
        let markup = r#"
            #set page(width: 8.5in, height: 11in)
            #set text(font: "Times", size: 12pt)
            
            = Asset Loading Test
            
            #image("assets/dod_seal.gif", width: 1in)
            
            This tests that embedded assets like the DOD seal can be loaded.
        "#;
        
        let result = TypstWrapper::render(markup, None);
        assert!(result.is_ok(), "Asset loading should work: {:?}", result.err());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert!(!pages[0].is_empty());
    }
}
