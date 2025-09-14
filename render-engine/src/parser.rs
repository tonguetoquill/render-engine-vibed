/// A parser for the render engine.
/// Converts Quill Delta syntax into Typst markup.
/// 
/// This module provides functionality to parse Quill Delta JSON format and convert it
/// into Typst markup language. It supports:
/// 
/// - Text formatting (bold, italic, underline, strikethrough)
/// - Paragraphs
/// - Bullet lists (nested)
/// - Ordered lists (nested)
/// 
/// # Example
/// 
/// ```
/// use render_engine::DeltaParser;
/// 
/// let mut parser = DeltaParser::new();
/// let delta_json = r#"{"ops":[{"insert":"Hello "},{"insert":"world","attributes":{"bold":true}}]}"#;
/// let typst_markup = parser.parse(delta_json).unwrap();
/// assert_eq!(typst_markup, "Hello *world*");
/// ```

use serde::{Deserialize, Serialize};
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

/// Represents a Quill Delta operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaOperation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retain: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Value>>,
}

/// Represents a complete Quill Delta document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuillDelta {
    pub ops: Vec<DeltaOperation>,
}

/// Parser for converting Quill Delta to Typst markup
pub struct DeltaParser {
    /// Current list nesting level
    list_level: usize,
    /// Current enum nesting level  
    enum_level: usize,
    /// Stack to track nested list types
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
            list_level: 0,
            enum_level: 0,
            list_stack: Vec::new(),
        }
    }

    /// Parse a Quill Delta JSON string and convert to Typst markup
    pub fn parse(&mut self, delta_json: &str) -> Result<String, ParserError> {
        let delta: QuillDelta = serde_json::from_str(delta_json)?;
        self.convert_delta_to_typst(&delta)
    }

    /// Convert a QuillDelta struct to Typst markup
    pub fn convert_delta_to_typst(&mut self, delta: &QuillDelta) -> Result<String, ParserError> {
        let mut result = String::new();
        let mut current_paragraph = String::new();
        let mut in_list = false;
        
        for op in &delta.ops {
            if let Some(insert) = &op.insert {
                match insert {
                    Value::String(text) => {
                        let formatted_text = self.apply_text_formatting(text, &op.attributes)?;
                        
                        // Handle list items specially
                        if let Some(attrs) = &op.attributes {
                            if let Some(list_attr) = attrs.get("list") {
                                // This is a list item - extract the text without the newline
                                let item_text = text.trim_end_matches('\n');
                                let list_item = self.handle_list_item(item_text, list_attr, attrs)?;
                                result.push_str(&list_item);
                                in_list = true;
                                continue;
                            }
                        }
                        
                        // Handle regular text with potential newlines
                        if text.contains('\n') {
                            let lines: Vec<&str> = text.split('\n').collect();
                            for (i, line) in lines.iter().enumerate() {
                                if i > 0 {
                                    // End any current list for regular paragraphs
                                    if in_list {
                                        result.push_str(&self.close_lists());
                                        in_list = false;
                                    }
                                    // Add paragraph break
                                    if !current_paragraph.is_empty() {
                                        result.push_str(&current_paragraph);
                                        result.push_str("\n\n");
                                        current_paragraph.clear();
                                    }
                                }
                                
                                if i < lines.len() - 1 || !line.is_empty() {
                                    let formatted_line = self.apply_text_formatting(line, &op.attributes)?;
                                    current_paragraph.push_str(&formatted_line);
                                }
                            }
                        } else {
                            current_paragraph.push_str(&formatted_text);
                        }
                    }
                    Value::Object(embed) => {
                        // Handle embedded objects (images, etc.) - placeholder for future expansion
                        return Err(ParserError::UnsupportedOperation(
                            format!("Embedded objects not yet supported: {:?}", embed)
                        ));
                    }
                    _ => {
                        return Err(ParserError::InvalidFormat(
                            format!("Unsupported insert type: {:?}", insert)
                        ));
                    }
                }
            }
        }

        // Add any remaining paragraph content
        if !current_paragraph.is_empty() {
            result.push_str(&current_paragraph);
        }

        // Close any remaining lists
        if in_list {
            result.push_str(&self.close_lists());
        }

        Ok(result)
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
        }

        Ok(formatted)
    }

    /// Handle list item formatting
    fn handle_list_item(
        &mut self,
        text: &str,
        list_attr: &Value,
        attributes: &HashMap<String, Value>,
    ) -> Result<String, ParserError> {
        let list_type = match list_attr.as_str() {
            Some("bullet") => ListType::Bullet,
            Some("ordered") => ListType::Ordered,
            _ => return Err(ParserError::UnsupportedOperation(
                format!("Unsupported list type: {:?}", list_attr)
            )),
        };

        // Handle nesting level
        let indent_level = attributes
            .get("indent")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Adjust list stack based on current level
        while self.list_stack.len() > indent_level + 1 {
            self.list_stack.pop();
        }

        if self.list_stack.len() == indent_level {
            self.list_stack.push(list_type.clone());
        } else if self.list_stack.len() == indent_level + 1 {
            self.list_stack[indent_level] = list_type.clone();
        }

        // Generate appropriate list marker
        let marker = match list_type {
            ListType::Bullet => "-",
            ListType::Ordered => "+",
        };

        let indent = "  ".repeat(indent_level);
        let formatted_text = self.apply_text_formatting(text, &Some(attributes.clone()))?;
        
        Ok(format!("{}{} {}\n", indent, marker, formatted_text))
    }

    /// Close all open lists
    fn close_lists(&mut self) -> String {
        self.list_stack.clear();
        self.list_level = 0;
        self.enum_level = 0;
        String::new() // Typst doesn't require explicit list closing
    }
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
        let delta_json = r#"{"ops":[{"insert":"Item 1\n","attributes":{"list":"bullet"}},{"insert":"Item 2\n","attributes":{"list":"bullet"}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 2"));
    }

    #[test]
    fn test_ordered_list() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"First item\n","attributes":{"list":"ordered"}},{"insert":"Second item\n","attributes":{"list":"ordered"}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert!(result.contains("+ First item"));
        assert!(result.contains("+ Second item"));
    }

    #[test]
    fn test_nested_list() {
        let mut parser = DeltaParser::new();
        let delta_json = r#"{"ops":[{"insert":"Top level\n","attributes":{"list":"bullet"}},{"insert":"Nested item\n","attributes":{"list":"bullet","indent":1}}]}"#;
        
        let result = parser.parse(delta_json).unwrap();
        assert!(result.contains("- Top level"));
        assert!(result.contains("  - Nested item"));
    }
}