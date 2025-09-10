use render_engine::RenderEngine;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the render engine
    let engine = RenderEngine::new();
    
    // Load the USAF template
    let template_content = fs::read_to_string("../tonguetoquill-usaf-memo/usaf-template.typ")?;
    
    // Try to render it
    println!("Attempting to render USAF template...");
    
    match engine.render_to_pdf(&template_content) {
        Ok(pdf_bytes) => {
            fs::write("usaf-template.pdf", pdf_bytes)?;
            println!("✅ Successfully rendered USAF template to PDF: usaf-template.pdf");
        }
        Err(e) => {
            println!("❌ Rendering failed: {}", e);
        }
    }
    
    Ok(())
}
