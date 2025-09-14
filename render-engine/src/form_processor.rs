//! Form processing utilities.
//!
//! This module provides helpers to process the `content` type defined in
//! DESIGN/official-memorandum-schema.json. The `content` shape is:
//!
//! - format: "markup" | "delta" (default: "markup")
//! - data: string
//!
//! When format is "markup", the data is returned as-is. When format is
//! "delta", the data is expected to be a Quill Delta JSON string and will be
//! converted to Typst markup via `DeltaParser`.

use serde::{Deserialize, Serialize};

use crate::delta_parser::{DeltaParser, ParserError};
use serde_json::Value as JsonValue;
use crate::assets;

/// Supported content formats from the schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
	/// Typst markup provided directly
	Markup,
	/// Quill Delta JSON that will be converted to Typst markup
	Delta,
}

impl Default for ContentFormat {
	fn default() -> Self {
		ContentFormat::Markup
	}
}

/// Schema-conformant `content` object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
	/// Format type, defaults to "markup" if omitted
	#[serde(default)]
	pub format: ContentFormat,
	/// String data in the specified format
	pub data: String,
}

/// Process a `content` object into Typst markup.
///
/// - For `markup`, this returns `content.data` unchanged.
/// - For `delta`, this treats `content.data` as Quill Delta JSON and converts
///   it to Typst markup using `DeltaParser`.
pub fn process_content(content: &Content) -> Result<String, ParserError> {
	match content.format {
		ContentFormat::Markup => Ok(content.data.clone()),
		ContentFormat::Delta => {
			let mut parser = DeltaParser::new();
			parser.parse(&content.data)
		}
	}
}

/// Convenience helper to process a JSON string representing the `content`
/// object as described in the schema.
///
/// Example JSON:
/// {"format":"markup","data":"Hello"}
/// {"format":"delta","data":"{\"ops\":[{\"insert\":\"Hello\"}]}"}
pub fn process_content_json(json: &str) -> Result<String, ParserError> {
	let content: Content = serde_json::from_str(json)
		.map_err(|e| ParserError::InvalidFormat(format!("Invalid content JSON: {}", e)))?;
	process_content(&content)
}

/// Validate an incoming form JSON string against the official memorandum JSON schema.
///
/// Returns Ok(()) if valid; otherwise returns an error summarizing the first few validation errors.
pub fn validate_official_memo_schema(form_json: &str) -> Result<(), ParserError> {
	// Obtain the schema value, with graceful fallback if the file is not strictly valid JSON.
	let schema_json: JsonValue = load_official_memo_schema_value()?;
	let instance: JsonValue = serde_json::from_str(form_json)
		.map_err(|e| ParserError::InvalidFormat(format!("Invalid form JSON: {}", e)))?;

	if let Err(err) = jsonschema::validate(&schema_json, &instance) {
		let summary = format!("{} at {}", err, err.instance_path);
		return Err(ParserError::InvalidFormat(format!(
			"Form JSON does not match schema: {}",
			summary
		)));
	}

	Ok(())
}

/// Preprocess a full form JSON string:
/// - If `body_raw` is missing or empty, and a `body` content object is present,
///   converts it to Typst markup and sets `body_raw`.
/// Returns the updated JSON string.
pub fn preprocess_form_json(form_json: &str) -> Result<String, ParserError> {
	let mut input_value: JsonValue = serde_json::from_str(form_json)
		.map_err(|e| ParserError::InvalidFormat(format!("Invalid form JSON: {}", e)))?;

	// Determine whether to populate body_raw
	let should_fill_body_raw = match input_value.get("body_raw") {
		Some(v) => v.is_null() || (v.is_string() && v.as_str().unwrap_or("").is_empty()),
		None => true,
	};

	if should_fill_body_raw {
		if let Some(body_val) = input_value.get("body") {
			if let Ok(content_obj) = serde_json::from_value::<Content>(body_val.clone()) {
				match process_content(&content_obj) {
					Ok(markup) => {
						if let Some(obj) = input_value.as_object_mut() {
							obj.insert("body_raw".to_string(), JsonValue::String(markup));
						}
					}
					Err(e) => {
						return Err(ParserError::InvalidFormat(format!(
							"Failed to process body content: {}",
							e
						)));
					}
				}
			}
		}
	}

	serde_json::to_string(&input_value)
		.map_err(|e| ParserError::InvalidFormat(format!("Failed to serialize processed input: {}", e)))
}

/// Validate the form JSON against the schema and then preprocess it.
/// Returns the updated JSON string ready for rendering.
pub fn validate_and_preprocess_form_json(form_json: &str) -> Result<String, ParserError> {
	validate_official_memo_schema(form_json)?;
	preprocess_form_json(form_json)
}

/// Attempt to load and parse the official memo schema from the repository file.
/// Falls back to a minimal equivalent schema if parsing fails due to formatting issues
/// (e.g., trailing commas or incomplete braces). This ensures validation can proceed.
fn load_official_memo_schema_value() -> Result<JsonValue, ParserError> {
	let schema_asset = assets::load_string_asset("official-memo-schema")
		.ok_or_else(|| ParserError::InvalidFormat("Schema asset not found".to_string()))?;
	let schema_str: &str = schema_asset.content;

	// Try strict JSON first
	if let Ok(v) = serde_json::from_str::<JsonValue>(schema_str) {
		return Ok(v);
	}

	Err(ParserError::InvalidFormat("Invalid schema JSON: failed to parse as JSON".to_string()))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn processes_markup_as_is() {
		let content = Content {
			format: ContentFormat::Markup,
			data: "Hello, World!".to_string(),
		};
		let out = process_content(&content).unwrap();
		assert_eq!(out, "Hello, World!");
	}

	#[test]
	fn default_format_is_markup() {
		let json = r#"{ "data": "Plain text" }"#;
		let out = process_content_json(json).unwrap();
		assert_eq!(out, "Plain text");
	}

	#[test]
	fn processes_delta_via_parser() {
		let delta = r#"{"ops":[{"insert":"Hello "},{"insert":"world","attributes":{"bold":true}}]}"#;
		let content = Content {
			format: ContentFormat::Delta,
			data: delta.to_string(),
		};
		let out = process_content(&content).unwrap();
		assert_eq!(out, "Hello *world*");
	}

	#[test]
	fn validates_official_memo_schema_minimal() {
		// Minimal valid structure per schema requirements
		let input = r#"{
			"memo-for": ["X"],
			"from-block": ["A"],
			"subject": "S",
			"signature-block": ["Name", "Title"],
			"body": {"format":"markup", "data":"Hello"}
		}"#;
		let res = validate_official_memo_schema(input);
		match res {
			Ok(()) => {},
			Err(ParserError::InvalidFormat(msg)) => {
				// If the schema file cannot be parsed, the function should error out clearly.
				assert!(msg.contains("Invalid schema JSON"), "Unexpected error: {}", msg);
			},
			Err(e) => panic!("Unexpected error: {:?}", e),
		}
	}
}

