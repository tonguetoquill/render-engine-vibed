#import "@preview/tonguetoquill-usaf-memo:latest": official-memorandum, indorsement


#let input = json("input.json")
#let try_get(key, default) = if key not in input { default } else { input.at(key) }

// Parse the date from input, supporting ISO formats
#let parsed-datetime = if "date" in input {
  // Parse ISO date strings (supports both YYYY-MM-DD and YYYY-MM-DDTHH:MM:SSZ formats)
  let parse-iso-date(iso-string) = {
    // Extract just the date part (YYYY-MM-DD) from ISO string
    let date-part = if iso-string.contains("T") {
      iso-string.split("T").at(0)
    } else {
      iso-string
    }
    
    // Parse using TOML date format
    let toml-content = "date = " + date-part
    let parsed = toml(bytes(toml-content))
    parsed.date
  }
  
  let parsed-date = parse-iso-date(input.date)
  assert(type(parsed-date) == datetime, message: "Error: 'date' must be in ISO (YYYY-MM-DD) or (YYYY-MM-DDTHH:MM:SSZ) format")
  parsed-date
} else {
  datetime.today()
}

// Generate the official memorandum with validated and processed input
#official-memorandum(
  // Letterhead configuration
  letterhead-title: try_get("letterhead-title", "DEPARTMENT OF THE AIR FORCE"),
  letterhead-caption: try_get("letterhead-caption", "123RD EXAMPLE SQUADRON"),
  letterhead-seal: image("assets/dod_seal.gif"),
  letterhead-font: "Copperplate CC",

  // Date
  date: parsed-datetime,
  
  // Recipients
  memo-for: input.memo-for,
  
  // Sender information
  from-block: input.from-block,
  
  // Subject line
  subject: input.subject,
  
  // Optional references
  references: try_get("references", none),
  
  // Signature block
  signature-block: input.signature-block,
  
)[
  // Body content from JSON
  #eval(input.body_raw, mode: "markup")
]


