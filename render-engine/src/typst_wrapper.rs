use std::collections::HashMap;
use std::sync::LazyLock;

use crate::assets;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::layout::PagedDocument;
use typst::syntax::{FileId, Source, VirtualPath, package::PackageSpec};
use typst::text::{Font, FontBook, FontInfo};
use typst::utils::LazyHash;
use typst::{Library, World};



// Static font collections initialized at compile time
static FONT_BOOK: LazyLock<LazyHash<FontBook>> = LazyLock::new(|| {
    let mut book = FontBook::new();
    
    // Load all embedded fonts from assets
    let font_assets = assets::get_font_assets();
    
    for font_asset in font_assets {
        // Single font files (ttf, otf)
        if font_asset.path.ends_with(".ttf") || font_asset.path.ends_with(".otf") {
            if let Some(info) = FontInfo::new(font_asset.content, 0) {
                book.push(info);
            }
        }
        // Font collections (ttc) - try multiple indices
        else if font_asset.path.ends_with(".ttc") {
            for i in 0..10 {
                if let Some(info) = FontInfo::new(font_asset.content, i) {
                    book.push(info);
                } else {
                    break;
                }
            }
        }
    }
    
    LazyHash::new(book)
});

// Static font vector for font() method access
static FONTS: LazyLock<Vec<Font>> = LazyLock::new(|| {
    let mut fonts = Vec::new();
    
    // Load all embedded fonts using Font::iter from assets
    let font_assets = assets::get_font_assets();
    
    for font_asset in font_assets {
        for font in Font::iter(Bytes::new(font_asset.content)) {
            fonts.push(font);
        }
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
    pub fn render_markup(
        markup: &str,
        config: Option<RenderConfig>,
    ) -> Result<Vec<Vec<u8>>, TypstWrapperError> {
        let mut world = TypstWorld::new();
        
        // Parse the main source
        let source = Source::new(FileId::new(None, VirtualPath::new("main.typ")), markup.to_string());
        world.insert_source(source);
        
        Self::render_file(world, config)
    }
    
    /// Render form using JSON input and memo-loader template
    pub fn render_form(
        json_input: &str,
        config: Option<RenderConfig>,
    ) -> Result<Vec<Vec<u8>>, TypstWrapperError> {
        // Create a completely fresh world for each render to avoid state pollution
        let mut world = TypstWorld::new();
        
        // Use unique identifiers to ensure file IDs don't collide between renders
        // In WASM environments, SystemTime is not available, so we use a simple hash
        let timestamp = {
            #[cfg(target_arch = "wasm32")]
            {
                // Use a hash of the JSON input as a unique identifier for WASM
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                json_input.hash(&mut hasher);
                // Add some additional entropy based on string length and content
                (json_input.len() as u64).hash(&mut hasher);
                hasher.finish() as u128
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            }
        };
        
        // Add the JSON input as a virtual file with unique path
        let json_filename = format!("input-{}.json", timestamp);
        let json_path = format!("memo-loader/{}", json_filename);
        let json_file_id = FileId::new(None, VirtualPath::new(&json_path));
        let json_source = Source::new(json_file_id, json_input.to_string());
        world.insert_source(json_source);
        
        // Load the memo-loader main template
        let memo_loader_asset = assets::load_string_asset("memo-loader-main")
            .ok_or_else(|| TypstWrapperError::FileNotFound("memo-loader main template not found".to_string()))?;
        
        // Create modified main template that references the unique JSON file
        let modified_main_content = memo_loader_asset.content.replace(
            "#let input = json(\"input.json\")",
            &format!("#let input = json(\"{}\")", json_filename)
        );
        
        
        // Parse the memo-loader template as the main source with unique path
        let main_path = format!("memo-loader/main-{}.typ", timestamp);
        let memo_loader_file_id = FileId::new(None, VirtualPath::new(&main_path));
        let memo_loader_source = Source::new(memo_loader_file_id, modified_main_content);
        world.insert_source(memo_loader_source);
        
        Self::render_file(world, config)
    }
    
    /// Internal function to render a prepared world with sources
    fn render_file(
        world: TypstWorld,
        config: Option<RenderConfig>,
    ) -> Result<Vec<Vec<u8>>, TypstWrapperError> {
        let config = config.unwrap_or_default();
        
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
}

impl TypstWorld {
    fn new() -> Self {
        Self {
            library: LazyHash::new(Library::default()),
            sources: HashMap::new(),
            package_sources: HashMap::new(),
        }
    }
    
    fn insert_source(&mut self, source: Source) {
        self.sources.insert(source.id(), source);
    }
    
    fn resolve_asset(&self, path: &str) -> Option<&'static [u8]> {
        assets::resolve_binary_asset(path)
    }
    
    fn resolve_package_file(&self, spec: &PackageSpec, path: &str) -> Option<&'static str> {
        assets::resolve_package_file(spec, path)
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
        
        // Check if this is a virtual source file (like memo-loader/input.json)
        if let Some(source) = self.sources.get(&id) {
            return Ok(Bytes::new(source.text().to_string().into_bytes()));
        }
        
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
    
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        // Return a fixed date since we don't need dynamic dates for this use case
        // You can change this to the current date or make it configurable if needed
        Datetime::from_ymd(2024, 1, 1)
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
        
        let result = TypstWrapper::render_markup(markup, None);
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
        
        let result = TypstWrapper::render_markup(markup, Some(config));
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
        
        let result = TypstWrapper::render_markup(markup, None);
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
        
        let result = TypstWrapper::render_markup(markup, None);
        assert!(result.is_ok(), "Asset loading should work: {:?}", result.err());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert!(!pages[0].is_empty());
    }
    
    #[test]
    fn test_render_form() {
        // Test that render_form works with JSON input matching the correct schema
        let json_input = r#"{
            "memo-for": ["Test Recipient", "Another Recipient"],
            "from-block": ["Test Sender", "Test Title", "Test Organization"],
            "subject": "Test Subject",
            "signature-block": ["Test Signature", "Test Title"],
            "body": {
                "data": "This is a test memo content."
            }
        }"#;
        
        let result = TypstWrapper::render_form(json_input, None);
        assert!(result.is_ok(), "Form rendering should work: {:?}", result.err());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert!(!pages[0].is_empty());
    }
    
    #[test] 
    fn test_render_form_pdf() {
        // Test that render_form works with PDF output
        let json_input = r#"{
            "memo-for": ["PDF Test Recipient"],
            "from-block": ["Test Sender", "Test Title"],
            "subject": "PDF Test Subject",
            "signature-block": ["Test Signature", "Test Title"],
            "body": {
                "data": "This memo should be rendered as PDF."
            }
        }"#;
        
        let config = RenderConfig {
            format: OutputFormat::Pdf,
        };
        
        let result = TypstWrapper::render_form(json_input, Some(config));
        assert!(result.is_ok(), "PDF form rendering should work: {:?}", result.err());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty());
        assert_eq!(pages.len(), 1); // PDF returns single item
        
        // PDF files start with %PDF
        assert!(pages[0].starts_with(b"%PDF"));
    }
}
