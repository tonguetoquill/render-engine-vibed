#import "@preview/tonguetoquill-usaf-memo:0.0.3": official-memorandum, indorsement
#import "utils.typ": validate-memo-input, apply-memo-defaults

#let input = json("input.json")

// Perform validation using consolidated validation framework
#let validation-errors = validate-memo-input(input)

// Display validation results with clear error reporting
#if validation-errors.len() > 0 {
  panic("Input validation failed:\n" + validation-errors.join("\n"))
}

// If validation passes, apply defaults and generate document
#let processed-input = apply-memo-defaults(input)

// Generate the official memorandum with validated and processed input
#official-memorandum(
  // Letterhead configuration
  letterhead-title: "DEPARTMENT OF THE AIR FORCE",
  letterhead-caption: "123RD EXAMPLE SQUADRON",
  letterhead-seal: image("assets/dod_seal.gif"),
  letterhead-font: "Copperplate CC",
  
  // Recipients
  memo-for: processed-input.memo-for,
  
  // Sender information
  from-block: processed-input.from-block,
  
  // Subject line
  subject: processed-input.subject,
  
  // Optional references
  references: processed-input.references,
  
  // Signature block
  signature-block: processed-input.signature-block,
  
)[
  // Body content from JSON
  #processed-input.body.data
]


