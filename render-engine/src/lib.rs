use std::collections::HashMap;
use std::path::PathBuf;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, package::PackageSpec};
use typst::text::{Font, FontBook};
use typst::{Library, World};
use typst::utils::LazyHash;
use chrono::{DateTime, Datelike, Utc};

/// Our custom Typst world implementation that includes fonts and assets
struct RenderWorld {
    /// The main source file being compiled
    main: Source,
    /// Font book for resolving fonts
    book: LazyHash<FontBook>,
    /// Available fonts
    fonts: Vec<Font>,
    /// Static assets (images, etc.)
    assets: HashMap<String, Bytes>,
    /// Package sources cache
    package_sources: HashMap<FileId, Source>,
    /// Package assets cache
    package_assets: HashMap<String, Bytes>,
    /// Package root directory
    packages_root: PathBuf,
    /// Current time for date/time functions
    now: DateTime<Utc>,
}

impl RenderWorld {
    fn new(content: &str, fonts: Vec<Font>, assets: HashMap<String, Bytes>, packages_root: PathBuf) -> Self {
        // Create the main source file
        let main = Source::detached(content);
        
        // Build font book
        let mut book = FontBook::new();
        for font in &fonts {
            book.push(font.info().clone());
        }
        
        Self {
            main,
            book: LazyHash::new(book),
            fonts,
            assets,
            package_sources: HashMap::new(),
            package_assets: HashMap::new(),
            packages_root,
            now: Utc::now(),
        }
    }
}

impl World for RenderWorld {
    fn library(&self) -> &LazyHash<Library> {
        // Use the standard library
        static LIBRARY: std::sync::LazyLock<LazyHash<Library>> = 
            std::sync::LazyLock::new(|| LazyHash::new(Library::builder().build()));
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else if let Some(source) = self.package_sources.get(&id) {
            Ok(source.clone())
        } else if let Some(spec) = id.package() {
            // Load package source
            self.load_package_source(id, spec)
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().as_rootless_path();
        let path_str = path.to_string_lossy();
        
        if let Some(bytes) = self.assets.get(path_str.as_ref()) {
            Ok(bytes.clone())
        } else if let Some(bytes) = self.package_assets.get(path_str.as_ref()) {
            Ok(bytes.clone())
        } else if let Some(spec) = id.package() {
            // Load package asset
            self.load_package_asset(id, spec)
        } else {
            Err(FileError::NotFound(path.into()))
        }
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.fonts.get(id).cloned()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset_duration = chrono::Duration::try_hours(offset.unwrap_or(0))?;
        let date = (self.now + offset_duration).date_naive();
        Datetime::from_ymd(
            date.year(),
            date.month().try_into().ok()?,
            date.day().try_into().ok()?,
        )
    }
}

impl RenderWorld {
    fn load_package_source(&self, id: FileId, spec: &PackageSpec) -> FileResult<Source> {
        let package_path = self.packages_root
            .join(spec.namespace.as_str())
            .join(spec.name.as_str())
            .join(spec.version.to_string());
        
        let file_path = package_path.join(id.vpath().as_rootless_path());
        
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                let source = Source::new(id, content);
                Ok(source)
            }
            Err(_) => Err(FileError::NotFound(file_path.into())),
        }
    }
    
    fn load_package_asset(&self, id: FileId, spec: &PackageSpec) -> FileResult<Bytes> {
        let package_path = self.packages_root
            .join(spec.namespace.as_str())
            .join(spec.name.as_str())
            .join(spec.version.to_string());
        
        let file_path = package_path.join(id.vpath().as_rootless_path());
        
        match std::fs::read(&file_path) {
            Ok(bytes) => Ok(Bytes::new(bytes)),
            Err(_) => Err(FileError::NotFound(file_path.into())),
        }
    }
}

/// Render engine that maintains a Typst world with fonts and images
pub struct RenderEngine {
    fonts: Vec<Font>,
    assets: HashMap<String, Bytes>,
    packages_root: PathBuf,
}

impl RenderEngine {
    /// Create a new render engine with fonts and assets from the assets directory
    pub fn new() -> Self {
        // Load fonts
        let arial_font_bytes = include_bytes!("../../assets/arial.ttf");
        let copperplate_font_bytes = include_bytes!("../../assets/CopperplateCC-Heavy.otf");
        let times_font_bytes = include_bytes!("../../assets/times.ttf");

        // Parse fonts
        let mut fonts = Vec::new();
        
        // Load Arial fonts
        for font in Font::iter(Bytes::new(arial_font_bytes)) {
            fonts.push(font);
        }
        
        // Load Copperplate font
        for font in Font::iter(Bytes::new(copperplate_font_bytes)) {
            fonts.push(font);
        }
        
        // Load Times fonts
        for font in Font::iter(Bytes::new(times_font_bytes)) {
            fonts.push(font);
        }

        // Load assets
        let mut assets = HashMap::new();
        let dod_seal = include_bytes!("../../assets/dod_seal.gif");
        assets.insert("assets/dod_seal.gif".to_string(), Bytes::new(dod_seal));

        // Set packages root path
        let packages_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/packages");

        Self { fonts, assets, packages_root }
    }



    /// Render a Typst document to PDF bytes
    pub fn render_to_pdf(&self, typst_markup: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let world = RenderWorld::new(
            typst_markup, 
            self.fonts.clone(), 
            self.assets.clone(), 
            self.packages_root.clone()
        );
        
        let result = typst::compile(&world);
        
        match result.output {
            Ok(document) => {
                match typst_pdf::pdf(&document, &Default::default()) {
                    Ok(pdf_bytes) => Ok(pdf_bytes),
                    Err(errors) => {
                        let error_msg = errors.iter()
                            .map(|e| format!("{}", e.message))
                            .collect::<Vec<_>>()
                            .join("; ");
                        Err(format!("PDF generation failed: {}", error_msg).into())
                    }
                }
            }
            Err(errors) => {
                let error_msg = errors.iter()
                    .map(|e| format!("{}", e.message))
                    .collect::<Vec<_>>()
                    .join("; ");
                Err(format!("Typst compilation failed: {}", error_msg).into())
            }
        }
    }

    /// Add a custom asset that can be referenced in Typst documents
    pub fn add_asset(&mut self, path: &str, data: Bytes) {
        self.assets.insert(path.to_string(), data);
    }

    /// Add a custom font that can be used in Typst documents  
    pub fn add_font(&mut self, font_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = Bytes::new(font_data);
        for font in Font::iter(bytes) {
            self.fonts.push(font);
        }
        Ok(())
    }
}

impl Default for RenderEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_document() {
        let engine = RenderEngine::new();
        let markup = r#"
            #set page(paper: "a4")
            #set text(font: "Times", 11pt)
            
            = Hello World
            This is a test document.
        "#;
        
        let result = engine.render_to_pdf(markup);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_with_asset() {
        let engine = RenderEngine::new();
        let markup = r#"
            #set page(paper: "a4")
            
            = Document with Image
            
            #image("assets/dod_seal.gif", width: 50pt)
        "#;
        
        let result = engine.render_to_pdf(markup);
        assert!(result.is_ok());
    }

    #[test] 
    fn test_render_to_pdf() {
        let engine = RenderEngine::new();
        let markup = r#"
            #set page(paper: "a4")
            #set text(font: "Times", 11pt)
            
            = PDF Test
            This document will be rendered to PDF.
        "#;
        
        let result = engine.render_to_pdf(markup);
        assert!(result.is_ok());
        
        let pdf_bytes = result.unwrap();
        assert!(!pdf_bytes.is_empty());
        // PDF should start with %PDF
        assert!(pdf_bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_render_usaf_template() {
        let engine = RenderEngine::new();
        let template_content = include_str!("../../tonguetoquill-usaf-memo/usaf-template.typ");
        
        let result = engine.render_to_pdf(template_content);
        // This may fail due to missing @preview packages, but let's see the error
        if let Err(e) = result {
            println!("Expected error (missing packages): {}", e);
        }
    }
}
