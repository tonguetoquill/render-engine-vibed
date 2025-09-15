//! # WASM Wrapper for Render Engine
//! 
//! This crate provides WebAssembly bindings for the render engine, allowing
//! JavaScript applications to render Typst documents directly in the browser.
//! 
//! ## Features
//! 
//! - Render arbitrary Typst markup to SVG or PDF
//! - Render structured memo forms from JSON input
//! - Debug logging support (enabled with "debug" feature)
//! - Optimized for web deployment with wasm-bindgen
//! 
//! ## Usage
//! 
//! ```javascript
//! import init, { render_markup, render_form } from './pkg/wasm_wrapper.js';
//! 
//! // Initialize the WASM module
//! await init();
//! 
//! // Render Typst markup
//! const svg = render_markup('= Hello World\nThis is a test.', 'svg');
//! 
//! // Render structured form data
//! const formData = {
//!   "memo-for": ["Recipient"],
//!   "from-block": ["Sender", "Title"],
//!   "subject": "Test Subject",
//!   "signature-block": ["Name", "Title"],
//!   "body_raw": "Content here"
//! };
//! const pdf = render_form(JSON.stringify(formData), 'pdf');
//! ```

use wasm_bindgen::prelude::*;
use render_engine::{render_markup as engine_render_markup, render_form as engine_render_form, RenderConfig, OutputFormat};

/// Import the `console.log` function from the `console` module.
/// Only available in debug builds to reduce binary size in production.
#[cfg(feature = "debug")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Macro for convenient console logging in debug builds.
/// 
/// In debug builds, this forwards to `console.log()` for debugging.
/// In release builds, this becomes a no-op to minimize overhead.
/// 
/// # Usage
/// 
/// ```rust
/// console_log!("Debug message: {}", value);
/// ```
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string()) })
}

/// No-op console logging for release builds.
/// This ensures debug logging has zero runtime cost in production.
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {}
}

/// Initialize the WASM module with enhanced error handling.
/// 
/// This function is automatically called when the WASM module is loaded.
/// It sets up better panic messages in development builds to help with debugging.
/// 
/// # Features
/// 
/// - Installs `console_error_panic_hook` for readable panic messages in browser console
/// - Only active when the "console_error_panic_hook" feature is enabled
/// - Improves developer experience by showing Rust panic traces in JavaScript
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Render arbitrary Typst markup to SVG or PDF format.
/// 
/// This function takes raw Typst markup code and renders it to the specified format.
/// It's useful for rendering custom documents or testing Typst code directly.
/// 
/// # Parameters
/// 
/// - `markup`: Typst markup code as a string (e.g., "= Title\nContent here")
/// - `format`: Output format, either "pdf" or "svg" (defaults to SVG if not specified)
/// 
/// # Returns
/// 
/// Returns `Ok(Vec<u8>)` containing the rendered document bytes, or `Err(JsValue)` on failure.
/// 
/// # JavaScript Usage
/// 
/// ```javascript
/// // Render as SVG (default)
/// const svgBytes = render_markup('= Hello\nThis is a test document.');
/// 
/// // Render as PDF
/// const pdfBytes = render_markup('= Hello\nThis is a test document.', 'pdf');
/// 
/// // Convert SVG bytes to string
/// const svgText = new TextDecoder().decode(svgBytes);
/// 
/// // Create PDF blob for download
/// const pdfBlob = new Blob([pdfBytes], { type: 'application/pdf' });
/// ```
/// 
/// # Errors
/// 
/// Common error cases:
/// - Invalid Typst syntax in markup
/// - Rendering engine internal errors
/// - Empty document (no pages generated)
#[wasm_bindgen]
pub fn render_markup(markup: &str, format: Option<String>) -> Result<Vec<u8>, JsValue> {
    // Parse format parameter - defaults to SVG for web compatibility
    let output_format = match format.as_deref() {
        Some("pdf") => OutputFormat::Pdf,
        _ => OutputFormat::Svg,
    };
    
    let config = RenderConfig {
        format: output_format,
    };
    
    match engine_render_markup(markup, Some(config)) {
        Ok(pages) => {
            console_log!("Markup render successful! Generated {} page(s)", pages.len());
            
            // Return the first page as bytes (SVG text or PDF binary data)
            if !pages.is_empty() {
                Ok(pages[0].clone())
            } else {
                Err(JsValue::from_str("Error: No pages generated"))
            }
        }
        Err(e) => {
            console_log!("Markup render failed: {:?}", e);
            Err(JsValue::from_str(&format!("Markup render failed: {:?}", e)))
        }
    }
}

/// Render structured form data to official memorandum format.
/// 
/// This function takes JSON input conforming to the official memorandum schema
/// and renders it using the appropriate Typst template. It's designed for
/// generating formal documents like military memos, official correspondence, etc.
/// 
/// # Parameters
/// 
/// - `input_json`: JSON string matching the official memorandum schema
/// - `format`: Output format, "pdf" or "svg" (case-insensitive, defaults to SVG)
/// 
/// # JSON Schema
/// 
/// The input JSON must contain these fields:
/// 
/// ```json
/// {
///   "memo-for": ["Recipient 1", "Recipient 2"],
///   "from-block": ["Sender Name", "Title", "Organization"],
///   "subject": "Subject Line",
///   "signature-block": ["SIGNATURE NAME", "Title"],
///   "body": {
///     "format": "markup|delta",
///     "data": "Content or serialized delta"
///   },
///   "references": ["Optional reference 1", "Optional reference 2"]
/// }
/// ```
/// 
/// # Body Formats
/// 
/// - `"markup"`: Plain text content
/// - `"delta"`: Quill.js Delta JSON (for rich text editing)
/// 
/// # Returns
/// 
/// Returns `Ok(Vec<u8>)` with the rendered document, or `Err(JsValue)` on failure.
/// 
/// # JavaScript Usage
/// 
/// ```javascript
/// const formData = {
///   "memo-for": ["Commander, Test Wing"],
///   "from-block": ["John A. Smith", "Colonel, USAF", "412th Test Wing"],
///   "subject": "Test Memorandum",
///   "signature-block": ["JOHN A. SMITH", "Colonel, USAF"],
///   "body": {
///     "format": "markup",
///     "data": "This is the memo content."
///   }
/// };
/// 
/// const pdfBytes = render_form(JSON.stringify(formData), 'pdf');
/// const svgBytes = render_form(JSON.stringify(formData), 'svg');
/// ```
/// 
/// # Errors
/// 
/// Common error cases:
/// - Invalid JSON format
/// - Missing required schema fields
/// - Invalid Delta format (for rich text)
/// - Template rendering errors
/// - Empty document generation
#[wasm_bindgen]
pub fn render_form(input_json: &str, format: Option<String>) -> Result<Vec<u8>, JsValue> {
    // Parse format parameter - case insensitive, defaults to SVG
    let output_format = match format.as_deref() {
        Some("pdf") | Some("PDF") => OutputFormat::Pdf,
        _ => OutputFormat::Svg,
    };
    
    let config = RenderConfig {
        format: output_format,
    };
    
    console_log!("Attempting to render form with JSON: {}", input_json);
    console_log!("Output format: {:?}", output_format);
    
    match engine_render_form(input_json, Some(config)) {
        Ok(pages) => {
            console_log!("Form render successful! Generated {} page(s)", pages.len());
            
            // Return the first page as bytes
            if !pages.is_empty() {
                Ok(pages[0].clone())
            } else {
                Err(JsValue::from_str("Error: No pages generated"))
            }
        }
        Err(e) => {
            console_log!("Form render failed: {:?}", e);
            Err(JsValue::from_str(&format!("Form render failed: {:?}", e)))
        }
    }
}


