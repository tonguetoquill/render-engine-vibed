#import "@preview/tonguetoquill-usaf-memo:0.0.2": official-memorandum, indorsement
#import "@preview/cmarker:0.1.6"

#set page(margin: 0.75in)
#set text(size: 10pt)

#official-memorandum(
  // LETTERHEAD CONFIGURATION
  letterhead-title: "DEPARTMENT OF THE AIR FORCE",
  letterhead-caption: "[YOUR SQUADRON/UNIT NAME]",
  letterhead-seal: image("assets/dod_seal.png"),
  
  // RECIPIENTS - Multiple format options shown@preview/tonguetoquill-usaf-memo:0.0.2
  memo-for: (
    // Grid layout example - replace with your recipients
    ("[FIRST/OFFICE]", "[SECOND/OFFICE]", "[THIRD/OFFICE]"),
    ("[FOURTH/OFFICE]", "[FIFTH/OFFICE]", "[SIXTH/OFFICE]"),
    // Alternative single recipient: "[SINGLE/OFFICE]"
    // Alternative list: ("[FIRST/OFFICE]", "[SECOND/OFFICE]")
  ),
  
  // SENDER INFORMATION BLOCK
  from-block: (
    "[YOUR/SYMBOL]",                    // Organization symbol
    "[Your Organization Name]",         // Full organization name  
    "[Street Address]",                 // Mailing address
    "[City ST 12345-6789]"             // City, state, ZIP
  ),
  
  // MEMO SUBJECT LINE
  subject: "[Your Subject in Title Case - Required Field]",
  
  // OPTIONAL: REFERENCE DOCUMENTS
  references: (
    "[Reference 1: Regulation/Directive, Date, Title]",
    "[Reference 2: AFI/AFH Number, Date, Title]",
    "[Reference 3: Local instruction or guidance]"
  ),
  
  // SIGNATURE BLOCK - Who signs the memo
  signature-block: (
    "[FIRST M. LAST, Rank, USAF]",     // Full name and rank
    "[Your Official Duty Title]",      // Position/title
    "[Organization (optional)]"        // Optional third line
  ),
  
  // ATTACHMENTS - Supporting documents
  attachments: (
    "[Description for first attachment, Date]",
    "[Description for second attachment, Date]"
  ),
  
  // COURTESY COPIES - Who gets copies
  cc: (
    "[First CC Recipient, ORG/SYMBOL]",
    "[Second CC Recipient, ORG/SYMBOL]",
  ),
  
  // DISTRIBUTION LIST - Who receives memo
  distribution: (
    "[ORGANIZATION/SYMBOL]",
    "[Another Organization Name]",
    "[Third Distribution Point]"
  ),
  
  // FORMATTING OPTIONS - Customize appearance
  letterhead-font: "Arial",                    // Letterhead font (Arial recommended)
  body-font: "Times New Roman",               // Body text font (TNR for AFH 33-337)
  paragraph-block-indent: false,             // true = indent paragraphs, false = block style
  leading-backmatter-pagebreak: false,         // true = force page break before attachments/cc
  
  // INDORSEMENTS - For routing through multiple offices
  indorsements: (
    indorsement(
      office-symbol: "[REVIEWING/OFFICE]",
      memo-for: "[NEXT/OFFICE]",
      signature-block: (
        "[REVIEWER NAME], [Rank], USAF",
        "[REVIEWER TITLE]"
      ),
    )[
      [First indorsement body text. This is where the reviewing office adds their comments, recommendations, or approval. Indorsements are automatically numbered as "1st Ind", "2d Ind", etc.]
    ],
    
    indorsement(
      office-symbol: "[FINAL/AUTHORITY]",
      memo-for: "[ORIGINAL/SENDER]",
      signature-block: (
        "[FINAL OFFICIAL, Rank, USAF]",
        "[Final Authority Title]"
      ),
      separate-page: true,                   // Use separate page format (common for final authority)
      original-office: "[ORIGINAL/SENDER]", // Original memo office symbol
      original-subject: "[Original Subject]", // Original memo subject
    )[
      [Final indorsement text. This indorsement uses separate page format, commonly used when returning to the original sender with final approval or disapproval.]
    ]
  )
)[
    // Main memo body text goes here. Use #cmarker for rich text formatting if needed.
    // Example:
  #cmarker.render(
    ```
This is the main body of the memorandum. Use this space to clearly and concisely communicate your message.

1. test 123
2. fdsa
    1. nested 1
    2. nested 2
- bullet 1
- bullet 2
    ```
      )

]
