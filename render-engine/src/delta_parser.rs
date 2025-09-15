/// A parser for the render engine.
/// Converts Quill Delta syntax into Typst markup.
/// 
/// This module provides functionality to parse Quill Delta JSON format and convert it
/// into Typst markup language according to the official Quill Delta specification
/// (https://quilljs.com/docs/delta/). It supports:
/// 
/// - Text formatting (bold, italic, underline, strikethrough, code)
/// - Paragraphs with proper line breaks
/// - Bullet lists (nested)
/// - Ordered lists (nested)
/// - Headers (levels 1-6)
/// - Blockquotes
/// - Code blocks
/// - Image embeds
/// 
/// # Example
/// 
/// ```
/// use render_engine::DeltaParser;
/// 
/// let parser = DeltaParser::new();
/// let delta_json = r#"{"ops":[{"insert":"Hello "},{"insert":"world","attributes":{"bold":true}}]}"#;
/// let typst_markup = parser.parse(delta_json).unwrap();
/// assert_eq!(typst_markup, "Hello *world*");
/// ```


use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Invalid Quill Delta format: {0}")]
    InvalidFormat(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Parser for converting Quill Delta to Typst markup
pub struct DeltaParser {
    /// Stack to track nested list types and levels
    list_stack: Vec<ListType>,
}

#[derive(Debug, Clone, PartialEq)]
enum ListType {
    Bullet,
    Ordered,
}

impl DeltaParser {
    pub fn new() -> Self {
        Self {
            list_stack: Vec::new(),
        }
    }

    /// Parse a Quill Delta JSON string and convert to Typst markup
    pub fn parse(&self, delta_json: &str) -> Result<String, ParserError> {
        // Parse JSON directly since quill-delta-rs expects a different format
        let json_value: Value = serde_json::from_str(delta_json)?;
        self.convert_json_to_typst(&json_value)
    }

    /// Convert JSON Delta format to Typst markup
    fn convert_json_to_typst(&self, json_value: &Value) -> Result<String, ParserError> {
        let mut result = String::new();
        let mut current_line = String::new();
        let mut in_list = false;
        
        // Get operations from JSON
        let ops = json_value["ops"].as_array()
            .ok_or_else(|| ParserError::InvalidFormat("Missing ops array".to_string()))?;
        
        for op in ops {
            if let Some(insert) = op.get("insert") {
                let attributes = op.get("attributes")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<HashMap<String, Value>>());
                
                match insert {
                    Value::String(text) => {
                        // Check if this is a newline with line formatting
                        if text == "\n" {
                            if let Some(attrs) = &attributes {
                                // Handle line-level formatting (lists, headers, etc.)
                                let formatted_line = self.handle_line_formatting(&current_line, attrs)?;
                                
                                if let Some(list_info) = self.extract_list_info(attrs) {
                                    // This is a list item
                                    let list_item = self.format_list_item(&current_line, &list_info, attrs)?;
                                    result.push_str(&list_item);
                                    in_list = true;
                                } else {
                                    // End any current list
                                    if in_list {
                                        result.push_str("\n");
                                        in_list = false;
                                    }
                                    
                                    // Handle other line formats (headers, etc.)
                                    if formatted_line.is_empty() && !current_line.is_empty() {
                                        result.push_str(&current_line);
                                    } else {
                                        result.push_str(&formatted_line);
                                    }
                                    
                                    if !current_line.is_empty() || !formatted_line.is_empty() {
                                        result.push_str("\n");
                                    }
                                }
                            } else {
                                // Regular newline without formatting
                                if in_list {
                                    result.push_str("\n");
                                    in_list = false;
                                }
                                if !current_line.is_empty() {
                                    result.push_str(&current_line);
                                    result.push_str("\n");
                                } else {
                                    result.push_str("\n");
                                }
                            }
                            current_line.clear();
                        } else {
                            // Regular text content
                            let formatted_text = self.apply_text_formatting(text, &attributes)?;
                            current_line.push_str(&formatted_text);
                        }
                    }
                    Value::Object(embed) => {
                        // Handle embedded objects (images, etc.)
                        let embed_typst = self.handle_embed(embed)?;
                        current_line.push_str(&embed_typst);
                    }
                    _ => {
                        return Err(ParserError::InvalidFormat(
                            format!("Unsupported insert type: {:?}", insert)
                        ));
                    }
                }
            } else if op.get("retain").is_some() {
                // Retain operations are used for changes, not document representation
                return Err(ParserError::UnsupportedOperation(
                    "Retain operations are not supported in document parsing".to_string()
                ));
            } else if op.get("delete").is_some() {
                // Delete operations are used for changes, not document representation
                return Err(ParserError::UnsupportedOperation(
                    "Delete operations are not supported in document parsing".to_string()
                ));
            }
        }
        
        // Handle any remaining content
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }
        
        Ok(result.trim_end().to_string())
    }

    /// Apply text formatting based on Quill Delta attributes
    fn apply_text_formatting(
        &self,
        text: &str,
        attributes: &Option<HashMap<String, Value>>,
    ) -> Result<String, ParserError> {
        let mut formatted = text.to_string();

        if let Some(attrs) = attributes {
            // Apply bold formatting
            if attrs.get("bold").and_then(|v| v.as_bool()).unwrap_or(false) {
                formatted = format!("*{}*", formatted);
            }

            // Apply italic formatting
            if attrs.get("italic").and_then(|v| v.as_bool()).unwrap_or(false) {
                formatted = format!("_{}_", formatted);
            }

            // Apply underline formatting
            if attrs.get("underline").and_then(|v| v.as_bool()).unwrap_or(false) {
                formatted = format!("#underline[{}]", formatted);
            }

            // Apply strikethrough formatting
            if attrs.get("strike").and_then(|v| v.as_bool()).unwrap_or(false) {
                formatted = format!("#strike[{}]", formatted);
            }

            // Apply code formatting
            if attrs.get("code").and_then(|v| v.as_bool()).unwrap_or(false) {
                formatted = format!("`{}`", formatted);
            }
        }

        Ok(formatted)
    }

    /// Handle line-level formatting (headers, etc.)
    fn handle_line_formatting(
        &self,
        text: &str,
        attributes: &HashMap<String, Value>,
    ) -> Result<String, ParserError> {
        let mut result = text.to_string();

        // Handle headers
        if let Some(header_level) = attributes.get("header").and_then(|v| v.as_u64()) {
            let header_prefix = "=".repeat(header_level as usize);
            result = format!("{} {}", header_prefix, result);
        }

        // Handle blockquotes
        if attributes.get("blockquote").and_then(|v| v.as_bool()).unwrap_or(false) {
            result = format!("> {}", result);
        }

        // Handle code blocks
        if attributes.get("code-block").and_then(|v| v.as_bool()).unwrap_or(false) {
            result = format!("```\n{}\n```", result);
        }

        Ok(result)
    }

    /// Extract list information from attributes
    fn extract_list_info(&self, attributes: &HashMap<String, Value>) -> Option<ListInfo> {
        if let Some(list_type) = attributes.get("list") {
            let indent_level = attributes
                .get("indent")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            let list_type = match list_type.as_str()? {
                "bullet" => ListType::Bullet,
                "ordered" => ListType::Ordered,
                _ => return None,
            };

            Some(ListInfo {
                list_type,
                indent_level,
            })
        } else {
            None
        }
    }

    /// Format a list item
    fn format_list_item(
        &self,
        text: &str,
        list_info: &ListInfo,
        _attributes: &HashMap<String, Value>,
    ) -> Result<String, ParserError> {
        let marker = match list_info.list_type {
            ListType::Bullet => "-",
            ListType::Ordered => "+",
        };

        let indent = "  ".repeat(list_info.indent_level);
        Ok(format!("{}{} {}\n", indent, marker, text))
    }

    /// Handle embedded objects
    fn handle_embed(&self, embed: &serde_json::Map<String, Value>) -> Result<String, ParserError> {
        // Handle different types of embeds
        if let Some(image_url) = embed.get("image").and_then(|v| v.as_str()) {
            Ok(format!("#image(\"{}\")", image_url))
        } else {
            Err(ParserError::UnsupportedOperation(
                format!("Unsupported embed type: {:?}", embed)
            ))
        }
    }
}

#[derive(Debug, Clone)]
struct ListInfo {
    list_type: ListType,
    indent_level: usize,
}

impl Default for DeltaParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_text_parsing() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Hello, World!"}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_bold_text() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Bold text","attributes":{"bold":true}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert_eq!(result, "*Bold text*");
    }

    #[test]
    fn test_italic_text() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Italic text","attributes":{"italic":true}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert_eq!(result, "_Italic text_");
    }

    #[test]
    fn test_underline_text() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Underlined text","attributes":{"underline":true}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert_eq!(result, "#underline[Underlined text]");
    }

    #[test]
    fn test_strikethrough_text() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Strikethrough text","attributes":{"strike":true}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert_eq!(result, "#strike[Strikethrough text]");
    }

    #[test]
    fn test_combined_formatting() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Bold and italic","attributes":{"bold":true,"italic":true}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert_eq!(result, "_*Bold and italic*_");
    }

    #[test]
    fn test_bullet_list() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Item 1"},{"attributes":{"list":"bullet"},"insert":"\n"},{"insert":"Item 2"},{"attributes":{"list":"bullet"},"insert":"\n"}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 2"));
    }

    #[test]
    fn test_ordered_list() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"First item"},{"attributes":{"list":"ordered"},"insert":"\n"},{"insert":"Second item"},{"attributes":{"list":"ordered"},"insert":"\n"}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert!(result.contains("+ First item"));
        assert!(result.contains("+ Second item"));
    }

    #[test]
    fn test_nested_list() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Top level"},{"attributes":{"list":"bullet"},"insert":"\n"},{"insert":"Nested item"},{"attributes":{"list":"bullet","indent":1},"insert":"\n"}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert!(result.contains("- Top level"));
        assert!(result.contains("  - Nested item"));
    }
}