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
        "format": "delta",
        "data": "{\"ops\":[{\"insert\":\"This is the body content of the memorandum.\\n\\nYou can format text with \"},{\"attributes\":{\"bold\":true},\"insert\":\"bold\"},{\"insert\":\", \"},{\"attributes\":{\"italic\":true},\"insert\":\"italic\"},{\"insert\":\", and \"},{\"attributes\":{\"underline\":true},\"insert\":\"underlined\"},{\"insert\":\" text.\\n\\nYou can also create:\"},{\"attributes\":{\"header\":2},\"insert\":\"\\n\"},{\"insert\":\"Numbered lists\"},{\"attributes\":{\"list\":\"ordered\"},\"insert\":\"\\n\"},{\"insert\":\"Bullet points\"},{\"attributes\":{\"list\":\"bullet\"},\"insert\":\"\\n\"},{\"insert\":\"And much more!\"},{\"attributes\":{\"list\":\"bullet\"},\"insert\":\"\\n\"}]}"
    },
    "signature-block": [
        "[FIRST M. LAST, Rank, USAF]",
        "[Your Official Duty Title]",
        "[Organization (optional)]"
    ],
    "date": "2025-09-15"
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

#[test]
fn test_delta_parsing() {
    use render_engine::delta_parser::DeltaParser;
    
    // The specific Delta JSON to test
    let delta_json = r#"{"ops":[{"insert":"This is the body content of the memorandum.\n\nYou can format text with "},{"attributes":{"bold":true},"insert":"bold"},{"insert":", "},{"attributes":{"italic":true},"insert":"italic"},{"insert":", and "},{"attributes":{"underline":true},"insert":"underlined"},{"insert":" text.\n\nYou can also create:"},{"attributes":{"header":2},"insert":"\n"},{"insert":"Numbered lists"},{"attributes":{"list":"ordered"},"insert":"\n"},{"insert":"Bullet points"},{"attributes":{"list":"bullet"},"insert":"\n"},{"insert":"And much more!"},{"attributes":{"list":"bullet"},"insert":"\n"}]}"#;
    
    let mut parser = DeltaParser::new();
    let result = parser.parse(delta_json);
    
    assert!(result.is_ok(), "Delta parsing should succeed: {:?}", result.err());
    
    let typst_markup = result.unwrap();
    
    // Print the result for inspection
    println!("=== DELTA PARSING TEST RESULTS ===");
    println!("Input Delta JSON:");
    println!("{}", delta_json);
    println!("\nParsed Typst Markup:");
    println!("{}", typst_markup);
    println!("=== END DELTA PARSING TEST ===");
    
    // Basic assertions - checking for formatting elements
    assert!(typst_markup.contains("*bold*"), "Should contain bold formatting");
    assert!(typst_markup.contains("_italic_"), "Should contain italic formatting");
    assert!(typst_markup.contains("#underline[underlined]"), "Should contain underline formatting");
    
    // Check for list markers (the text positioning issue is a separate concern)
    assert!(typst_markup.contains("+"), "Should contain ordered list marker");
    assert!(typst_markup.contains("-"), "Should contain bullet list marker");
    assert!(typst_markup.contains("Numbered lists"), "Should contain list text");
    assert!(typst_markup.contains("Bullet points"), "Should contain list text");
}

#[test]
fn test_delta_parse() {
    use render_engine::delta_parser::DeltaParser;
    
    // Test parsing Delta JSON directly with the DeltaParser  
    // Using the original correct Delta format
    let delta_json = r#"{"ops":[{"insert":"This is the body content of the memorandum.\n\nYou can format text with "},{"attributes":{"bold":true},"insert":"bold"},{"insert":", "},{"attributes":{"italic":true},"insert":"italic"},{"insert":", and "},{"attributes":{"underline":true},"insert":"underlined"},{"insert":" text.\n\nasdf"},{"attributes":{"list":"ordered"},"insert":"\n"},{"insert":"\nYou can also create numbered lists:\n"}]}"#;

    println!("=== DELTA PARSER DIRECT TEST ===");
    println!("Input Delta JSON:");
    println!("{}", delta_json);
    
    // Parse using DeltaParser directly
    let mut parser = DeltaParser::new();
    let parse_result = parser.parse(delta_json);
    
    assert!(parse_result.is_ok(), "Delta parsing should succeed: {:?}", parse_result.err());
    
    let typst_markup = parse_result.unwrap();
    println!("\nParsed Typst Markup:");
    println!("{}", typst_markup);
    
    // Verify formatting elements are preserved
    assert!(typst_markup.contains("*bold*"), "Should contain bold formatting");
    assert!(typst_markup.contains("_italic_"), "Should contain italic formatting");
    assert!(typst_markup.contains("#underline[underlined]"), "Should contain underline formatting");
    
    // Verify text content is preserved
    assert!(typst_markup.contains("This is the body content"), "Should contain body text");
    assert!(typst_markup.contains("You can format text"), "Should contain formatting text");
    
    println!("=== END DELTA PARSER DIRECT TEST ===");
}
