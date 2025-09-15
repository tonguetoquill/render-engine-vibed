use std::fs;
use std::path::Path;
use render_engine::{render_markup, render_form, RenderConfig, OutputFormat};

#[test]
fn basic_test() {
    // Just make sure the crate can be compiled and loaded
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_usaf_template_render() {
    // Load the USAF template markup
    let usaf_template = include_str!("../tonguetoquill-usaf-memo/template/usaf-template.typ");
    
    // Create target/tmp/svg directory if it doesn't exist
    let output_dir = Path::new("target/tmp");
    let svg_dir = output_dir.join("svg");
    fs::create_dir_all(&svg_dir).expect("Failed to create target/tmp/svg directory");
    
    // Test SVG rendering (all pages) - SVG is the default format
    let svg_pages_result = render_markup(usaf_template, None);
    assert!(svg_pages_result.is_ok(), "SVG rendering failed: {:?}", svg_pages_result.err());
    
    let svg_pages = svg_pages_result.unwrap();
    assert!(!svg_pages.is_empty(), "SVG pages should not be empty");
    
    // Write each SVG page to a separate file in svg subfolder
    for (page_num, svg_output) in svg_pages.iter().enumerate() {
        let svg_path = svg_dir.join(format!("usaf_template_test_page_{}.svg", page_num + 1));
        fs::write(&svg_path, svg_output).expect("Failed to write SVG file");
        println!("SVG page {} output written to: {}", page_num + 1, svg_path.display());
    }
    
    println!("Total SVG pages rendered: {}", svg_pages.len());
    
    // Test PDF rendering
    let pdf_config = RenderConfig {
        format: OutputFormat::Pdf,
    };
    
    let pdf_result = render_markup(usaf_template, Some(pdf_config));
    assert!(pdf_result.is_ok(), "PDF rendering failed: {:?}", pdf_result.err());
    
    let pdf_pages = pdf_result.unwrap();
    assert!(!pdf_pages.is_empty(), "PDF output should not be empty");
    assert_eq!(pdf_pages.len(), 1, "PDF should return exactly one item");
    
    let pdf_output = &pdf_pages[0];
    assert!(pdf_output.starts_with(b"%PDF"), "PDF output should start with %PDF header");
    
    // Write PDF output to file
    let pdf_path = output_dir.join("usaf_template_test.pdf");
    fs::write(&pdf_path, pdf_output).expect("Failed to write PDF file");
    println!("PDF output written to: {}", pdf_path.display());
}

#[test]
fn test_render_form_with_provided_input() {
        let json_input = r#"{
  "letterhead-title": "DEPARTMENT OF THE AIR FORCE",
  "letterhead-caption": "123RD EXAMPLE SQUADRON",
  "memo-for": [
    "[FIRST/OFFICE]",
    "[SECOND/OFFICE]",
    "[THIRD/OFFICE]",
    "[FOURTH/OFFICE]",
    "[FIFTH/OFFICE]",
    "[SIXTH/OFFICE]"
  ],
  "from-block": [
    "[YOUR/SYMBOL]",
    "[Your Organization Name]",
    "[Street Address]",
    "[City ST 12345-6789]"
  ],
  "subject": "[Your Subject in Title Case - Required Field]",
  "body": {
    "format": "markup",
    "data": ""
  },
  "signature-block": [
    "[FIRST M. LAST, Rank, USAF]",
    "[Your Official Duty Title]",
    "[Organization (optional)]"
  ],
  "date": "2025-09-01T00:00:00Z"
}"#;
        // Render as SVG (default)
        let svg_result = render_form(json_input, None);
        assert!(svg_result.is_ok(), "SVG render_form failed: {:?}", svg_result.err());
        let svg_pages = svg_result.unwrap();
        assert!(!svg_pages.is_empty(), "SVG pages should not be empty");
        assert!(!svg_pages[0].is_empty(), "First SVG page should have content");

        // Render as PDF
        let pdf_config = RenderConfig { format: OutputFormat::Pdf };
        let pdf_result = render_form(json_input, Some(pdf_config));
        assert!(pdf_result.is_ok(), "PDF render_form failed: {:?}", pdf_result.err());
        let pdf_pages = pdf_result.unwrap();
        assert_eq!(pdf_pages.len(), 1, "PDF should return exactly one item");
        assert!(pdf_pages[0].starts_with(b"%PDF"), "PDF output should start with %PDF header");
}
