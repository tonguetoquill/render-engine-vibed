#import "@preview/tonguetoquill-usaf-memo:0.1.0": official-memorandum, indorsement


#let input = json("input.json")
#let try_get(key, default) = if key not in input { default } else { input.at(key) }

#if "date" in input {
  let parsed-date(iso-string) = toml(bytes("date =  "+ iso-string )).date
  let date = parsed-date(input.date)
  assert(type(date) == datetime,message:  "Error: 'date' must be in YYYY-MM-DD format")
  input.insert("date", date)
} else {
  input.insert("date", datetime.today())
}

// Generate the official memorandum with validated and processed input
#official-memorandum(
  // Letterhead configuration
  letterhead-title: try_get("letterhead-title", "DEPARTMENT OF THE AIR FORCE"),
  letterhead-caption: try_get("letterhead-caption", "123RD EXAMPLE SQUADRON"),
  letterhead-seal: image("assets/dod_seal.gif"),
  letterhead-font: "Copperplate CC",

  // Date
  date: try_get("date", datetime.today()),
  
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


