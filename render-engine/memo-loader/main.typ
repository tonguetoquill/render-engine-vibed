#import "@preview/tonguetoquill-usaf-memo:0.0.3": official-memorandum, indorsement

#let default(value, default) = if value == none { default } else { value }

#let input = json("input.json")
#let parsed-date(iso-string) = toml(bytes("date =  "+ iso-string )).date
#input.insert("date", parsed-date(input.date))

// Generate the official memorandum with validated and processed input
#official-memorandum(
  // Letterhead configuration
  letterhead-title: default(input.letterhead-title, "DEPARTMENT OF THE AIR FORCE"),
  letterhead-caption: default(input.letterhead-caption, "123RD EXAMPLE SQUADRON"),
  letterhead-seal: image("assets/dod_seal.gif"),
  letterhead-font: "Copperplate CC",

  // Date
  date: input.date,
  
  // Recipients
  memo-for: input.memo-for,
  
  // Sender information
  from-block: input.from-block,
  
  // Subject line
  subject: input.subject,
  
  // Optional references
  references: input.references,
  
  // Signature block
  signature-block: input.signature-block,
  
)[
  // Body content from JSON
  #input.body.data
]


