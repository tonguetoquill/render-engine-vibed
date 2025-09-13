use wasm_bindgen::prelude::*;
use render_engine::{render_markup as engine_render_markup, render_form as engine_render_form, RenderConfig, OutputFormat};

// Import the `console.log` function from the `console` module - only in debug builds
#[cfg(feature = "debug")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make console logging easier - only in debug builds
#[cfg(feature = "debug")]
macro_rules! console_log {
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string()) })
}

// No-op console logging for release builds
#[cfg(not(feature = "debug"))]
macro_rules! console_log {
    ($($t:tt)*) => {}
}

/// Initialize panic hook for better error messages in development
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Render arbitrary Typst markup from JavaScript
#[wasm_bindgen]
pub fn render_markup(markup: &str, format: Option<String>) -> Result<Vec<u8>, JsValue> {
    // Parse format parameter
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
            
            // Return actual content as bytes (works for both SVG and PDF)
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

/// Render from JSON schemaed input
#[wasm_bindgen]
pub fn render_form(input_json: &str, format: Option<String>) -> Result<Vec<u8>, JsValue> {
    // Parse format parameter
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
            
            // Debug: Check what the first few bytes look like
            if !pages.is_empty() {
                let _first_bytes = &pages[0][..std::cmp::min(100, pages[0].len())];
                console_log!("First 100 bytes as string: {}", String::from_utf8_lossy(_first_bytes));
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_typst_render() {
        // Test basic functionality without WASM bindings
        let test_markup = "= Test\nThis is a test document.";
        
        let config = RenderConfig {
            format: OutputFormat::Svg,
        };
        
        let result = engine_render_markup(test_markup, Some(config));
        assert!(result.is_ok(), "Basic render should succeed");
        
        let pages = result.unwrap();
        assert!(!pages.is_empty(), "Should generate at least one page");
        assert!(!pages[0].is_empty(), "First page should have content");
    }
    
    #[test] 
    fn test_pdf_render() {
        let test_markup = "= PDF Test\nTesting PDF output.";
        
        let config = RenderConfig {
            format: OutputFormat::Pdf,
        };
        
        let result = engine_render_markup(test_markup, Some(config));
        assert!(result.is_ok(), "PDF render should succeed");
        
        let pages = result.unwrap();
        assert_eq!(pages.len(), 1, "PDF should generate exactly one item");
        assert!(!pages[0].is_empty(), "PDF should have content");
    }
    
    #[test]
    fn test_form_render() {
        // Test form rendering with correct JSON schema
        let json_input = r#"{
            "memo-for": ["Test Recipient"],
            "from-block": ["Test Sender", "Test Title", "Test Organization"],
            "subject": "Test Subject",
            "signature-block": ["Test Signature", "Test Title"],
            "body": {
                "data": "This is test memo content for WASM wrapper."
            }
        }"#;
        
        let config = RenderConfig {
            format: OutputFormat::Svg,
        };
        
        let result = engine_render_form(json_input, Some(config));
        assert!(result.is_ok(), "Form render should succeed: {:?}", result.err());
        
        let pages = result.unwrap();
        assert!(!pages.is_empty(), "Form should generate at least one page");
        assert!(!pages[0].is_empty(), "First page should have content");
    }
}
