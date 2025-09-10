use render_engine::RenderEngine;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the render engine
    let engine = RenderEngine::new();
    
    // Simple Typst markup
    let markup = r#"
        #set page(paper: "a4")
        #set text(font: "Times", 11pt)
        
        = Render Engine Demo
        
        This document demonstrates the render engine functionality.
        
        == Features
        - Custom Typst world implementation
        - Embedded fonts (Arial, Times, Copperplate CC)
        - Embedded assets (DoD seal image)
        - PDF generation
        
        == Sample Image
        
        #image("assets/dod_seal.gif", width: 100pt)
        
        The image above is loaded from our embedded assets.
    "#;
    
    // Render to PDF
    let pdf_bytes = engine.render_to_pdf(markup)?;
    
    // Write to file
    fs::write("demo.pdf", pdf_bytes)?;
    println!("Demo PDF written to demo.pdf");
    
    Ok(())
}
