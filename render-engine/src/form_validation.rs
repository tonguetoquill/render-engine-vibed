use serde_json::{Value, Map};
use std::collections::HashMap;

/// Errors that can occur during form validation
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub field_path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field_path: &str, message: &str) -> Self {
        Self {
            field_path: field_path.to_string(),
            message: message.to_string(),
        }
    }
}

/// Result type for validation operations
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Schema definition for memo validation
#[derive(Debug, Clone)]
pub struct MemoSchema {
    pub allowed_properties: Vec<String>,
    pub required_properties: Vec<String>,
    pub properties: HashMap<String, Vec<ValidationRule>>,
    pub body_schema: BodySchema,
}

/// Schema for the body object within a memo
#[derive(Debug, Clone)]
pub struct BodySchema {
    pub allowed_properties: Vec<String>,
    pub properties: HashMap<String, Vec<ValidationRule>>,
}

/// Validation rules that can be applied to properties
#[derive(Debug, Clone)]
pub enum ValidationRule {
    Type { expected: ValueType },
    ArrayItems { expected: ValueType },
    StringLength { min: usize },
    ArrayLength { min: usize },
    NullableArray,
    Enum { allowed: Vec<String> },
}

/// Supported value types for validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    String,
    Array,
    Object,
    Number,
    Boolean,
    Null,
}

impl MemoSchema {
    /// Creates the default official memorandum schema
    pub fn official_memorandum() -> Self {
        let mut properties = HashMap::new();
        
        // memo-for validation
        properties.insert("memo-for".to_string(), vec![
            ValidationRule::Type { expected: ValueType::Array },
            ValidationRule::ArrayLength { min: 1 },
            ValidationRule::ArrayItems { expected: ValueType::String },
        ]);
        
        // from-block validation  
        properties.insert("from-block".to_string(), vec![
            ValidationRule::Type { expected: ValueType::Array },
            ValidationRule::ArrayLength { min: 1 },
            ValidationRule::ArrayItems { expected: ValueType::String },
        ]);
        
        // subject validation
        properties.insert("subject".to_string(), vec![
            ValidationRule::Type { expected: ValueType::String },
            ValidationRule::StringLength { min: 1 },
        ]);
        
        // references validation (optional)
        properties.insert("references".to_string(), vec![
            ValidationRule::NullableArray,
        ]);
        
        // signature-block validation (AFH 33-337 requirement: min 2 items)
        properties.insert("signature-block".to_string(), vec![
            ValidationRule::Type { expected: ValueType::Array },
            ValidationRule::ArrayLength { min: 2 },
            ValidationRule::ArrayItems { expected: ValueType::String },
        ]);
        
        // body validation
        properties.insert("body".to_string(), vec![
            ValidationRule::Type { expected: ValueType::Object },
        ]);
        
        // Body schema properties
        let mut body_properties = HashMap::new();
        
        body_properties.insert("format".to_string(), vec![
            ValidationRule::Type { expected: ValueType::String },
            ValidationRule::Enum { allowed: vec!["plaintext".to_string()] },
        ]);
        
        body_properties.insert("data".to_string(), vec![
            ValidationRule::Type { expected: ValueType::String },
        ]);
        
        Self {
            allowed_properties: vec![
                "memo-for".to_string(),
                "from-block".to_string(),
                "subject".to_string(),
                "references".to_string(),
                "signature-block".to_string(),
                "body".to_string(),
            ],
            required_properties: vec![
                "memo-for".to_string(),
                "from-block".to_string(),
                "subject".to_string(),
                "signature-block".to_string(),
                "body".to_string(),
            ],
            properties,
            body_schema: BodySchema {
                allowed_properties: vec![
                    "format".to_string(),
                    "data".to_string(),
                ],
                properties: body_properties,
            },
        }
    }
}

/// Main validator for memo forms
pub struct MemoValidator {
    schema: MemoSchema,
}

impl MemoValidator {
    /// Creates a new validator with the official memorandum schema
    pub fn new() -> Self {
        Self {
            schema: MemoSchema::official_memorandum(),
        }
    }
    
    /// Validates a JSON string representing a memo form
    pub fn validate_json(&self, json_input: &str) -> ValidationResult {
        let value: Value = serde_json::from_str(json_input)
            .map_err(|e| vec![ValidationError::new("root", &format!("Invalid JSON: {}", e))])?;
        
        if let Value::Object(obj) = value {
            self.validate_memo(&obj)
        } else {
            Err(vec![ValidationError::new("root", "Root must be a JSON object")])
        }
    }
    
    /// Validates a parsed JSON object representing a memo
    pub fn validate_memo(&self, data: &Map<String, Value>) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Check for unexpected properties
        for key in data.keys() {
            if !self.schema.allowed_properties.contains(key) {
                errors.push(ValidationError::new(
                    key,
                    &format!("Unexpected property '{}'. Allowed properties: {}", 
                        key, 
                        self.schema.allowed_properties.join(", ")
                    )
                ));
            }
        }
        
        // Check for required properties
        for required_prop in &self.schema.required_properties {
            if !data.contains_key(required_prop) {
                errors.push(ValidationError::new(
                    required_prop,
                    &format!("Required property '{}' is missing", required_prop)
                ));
            }
        }
        
        // Validate individual properties
        for (prop_name, prop_rules) in &self.schema.properties {
            if let Some(value) = data.get(prop_name) {
                if let Err(mut prop_errors) = self.validate_property(value, prop_rules, prop_name) {
                    errors.append(&mut prop_errors);
                }
            }
        }
        
        // Special handling for references array items (if not null)
        if let Some(references) = data.get("references") {
            if !references.is_null() {
                if let Value::Array(ref_array) = references {
                    if let Err(mut ref_errors) = self.validate_array_items(ref_array, &ValueType::String, "references") {
                        errors.append(&mut ref_errors);
                    }
                }
            }
        }
        
        // Validate nested body object
        if let Some(Value::Object(body_obj)) = data.get("body") {
            if let Err(mut body_errors) = self.validate_body(body_obj) {
                errors.append(&mut body_errors);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validates the body object of a memo
    fn validate_body(&self, body: &Map<String, Value>) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Check for unexpected properties in body
        for key in body.keys() {
            if !self.schema.body_schema.allowed_properties.contains(key) {
                errors.push(ValidationError::new(
                    &format!("body.{}", key),
                    &format!("Unexpected property in body: '{}'. Allowed properties: {}", 
                        key,
                        self.schema.body_schema.allowed_properties.join(", ")
                    )
                ));
            }
        }
        
        // Validate body properties
        for (prop_name, prop_rules) in &self.schema.body_schema.properties {
            if let Some(value) = body.get(prop_name) {
                if let Err(mut prop_errors) = self.validate_property(value, prop_rules, &format!("body.{}", prop_name)) {
                    errors.append(&mut prop_errors);
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validates a single property against its rules
    fn validate_property(&self, value: &Value, rules: &[ValidationRule], path: &str) -> ValidationResult {
        let mut errors = Vec::new();
        
        for rule in rules {
            match rule {
                ValidationRule::Type { expected } => {
                    if let Err(mut type_errors) = self.validate_type(value, expected, path) {
                        errors.append(&mut type_errors);
                    }
                }
                ValidationRule::ArrayItems { expected } => {
                    if let Value::Array(arr) = value {
                        if let Err(mut item_errors) = self.validate_array_items(arr, expected, path) {
                            errors.append(&mut item_errors);
                        }
                    }
                }
                ValidationRule::StringLength { min } => {
                    if let Value::String(s) = value {
                        if let Err(mut length_errors) = self.validate_string_length(s, *min, path) {
                            errors.append(&mut length_errors);
                        }
                    }
                }
                ValidationRule::ArrayLength { min } => {
                    if let Value::Array(arr) = value {
                        if let Err(mut length_errors) = self.validate_array_length(arr, *min, path) {
                            errors.append(&mut length_errors);
                        }
                    }
                }
                ValidationRule::NullableArray => {
                    if let Err(mut nullable_errors) = self.validate_nullable_array(value, path) {
                        errors.append(&mut nullable_errors);
                    }
                }
                ValidationRule::Enum { allowed } => {
                    if let Value::String(s) = value {
                        if let Err(mut enum_errors) = self.validate_enum(s, allowed, path) {
                            errors.append(&mut enum_errors);
                        }
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validates that a value matches the expected type
    fn validate_type(&self, value: &Value, expected_type: &ValueType, path: &str) -> ValidationResult {
        let actual_type = match value {
            Value::String(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
            Value::Number(_) => ValueType::Number,
            Value::Bool(_) => ValueType::Boolean,
            Value::Null => ValueType::Null,
        };
        
        if actual_type != *expected_type {
            Err(vec![ValidationError::new(
                path,
                &format!("Property '{}' must be {:?} (got {:?})", path, expected_type, actual_type)
            )])
        } else {
            Ok(())
        }
    }
    
    /// Validates that all array items are of the expected type
    fn validate_array_items(&self, arr: &[Value], expected_type: &ValueType, path: &str) -> ValidationResult {
        let mut errors = Vec::new();
        
        for (i, item) in arr.iter().enumerate() {
            let actual_type = match item {
                Value::String(_) => ValueType::String,
                Value::Array(_) => ValueType::Array,
                Value::Object(_) => ValueType::Object,
                Value::Number(_) => ValueType::Number,
                Value::Bool(_) => ValueType::Boolean,
                Value::Null => ValueType::Null,
            };
            
            if actual_type != *expected_type {
                errors.push(ValidationError::new(
                    path,
                    &format!("All items in '{}' array must be {:?} (item {} is {:?})", 
                        path, expected_type, i, actual_type)
                ));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validates string minimum length
    fn validate_string_length(&self, value: &str, min_length: usize, path: &str) -> ValidationResult {
        if value.len() < min_length {
            Err(vec![ValidationError::new(
                path,
                &format!("Property '{}' cannot be empty (minLength: {})", path, min_length)
            )])
        } else {
            Ok(())
        }
    }
    
    /// Validates array minimum length
    fn validate_array_length(&self, arr: &[Value], min_items: usize, path: &str) -> ValidationResult {
        if arr.len() < min_items {
            let plural = if min_items > 1 { "s" } else { "" };
            Err(vec![ValidationError::new(
                path,
                &format!("Property '{}' must contain at least {} item{}", path, min_items, plural)
            )])
        } else {
            Ok(())
        }
    }
    
    /// Validates that a value is either null or an array
    fn validate_nullable_array(&self, value: &Value, path: &str) -> ValidationResult {
        match value {
            Value::Null | Value::Array(_) => Ok(()),
            _ => Err(vec![ValidationError::new(
                path,
                &format!("Property '{}' must be an array or null", path)
            )])
        }
    }
    
    /// Validates enum values
    fn validate_enum(&self, value: &str, allowed_values: &[String], path: &str) -> ValidationResult {
        if allowed_values.contains(&value.to_string()) {
            Ok(())
        } else {
            Err(vec![ValidationError::new(
                path,
                &format!("Property '{}' must be one of: {} (got '{}')", 
                    path, 
                    allowed_values.join(", "), 
                    value)
            )])
        }
    }
    
    /// Applies default values to a memo JSON object
    pub fn apply_defaults(&self, json_input: &str) -> Result<String, serde_json::Error> {
        let mut value: Value = serde_json::from_str(json_input)?;
        
        if let Value::Object(ref mut obj) = value {
            // Apply default for references if not present
            if !obj.contains_key("references") {
                obj.insert("references".to_string(), Value::Null);
            }
            
            // Apply default for body.format if not present
            if let Some(Value::Object(ref mut body_obj)) = obj.get_mut("body") {
                if !body_obj.contains_key("format") {
                    body_obj.insert("format".to_string(), Value::String("plaintext".to_string()));
                }
            }
        }
        
        Ok(serde_json::to_string_pretty(&value)?)
    }
}

impl Default for MemoValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_memo() {
        let validator = MemoValidator::new();
        let valid_json = r#"
        {
            "memo-for": ["Recipient"],
            "from-block": ["Sender", "Title"],
            "subject": "Test Subject",
            "signature-block": ["Name", "Title"],
            "body": {
                "format": "plaintext",
                "data": "Test content"
            }
        }"#;
        
        assert!(validator.validate_json(valid_json).is_ok());
    }
    
    #[test]
    fn test_missing_required_field() {
        let validator = MemoValidator::new();
        let invalid_json = r#"
        {
            "memo-for": ["Recipient"],
            "subject": "Test Subject"
        }"#;
        
        let result = validator.validate_json(invalid_json);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("Required property 'from-block' is missing")));
    }
    
    #[test]
    fn test_invalid_type() {
        let validator = MemoValidator::new();
        let invalid_json = r#"
        {
            "memo-for": "Should be array",
            "from-block": ["Sender"],
            "subject": "Test Subject", 
            "signature-block": ["Name", "Title"],
            "body": {
                "data": "Test content"
            }
        }"#;
        
        let result = validator.validate_json(invalid_json);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("must be Array")));
    }
    
    #[test]
    fn test_apply_defaults() {
        let validator = MemoValidator::new();
        let input_json = r#"
        {
            "memo-for": ["Recipient"],
            "from-block": ["Sender"],
            "subject": "Test",
            "signature-block": ["Name", "Title"],
            "body": {
                "data": "Content"
            }
        }"#;
        
        let result = validator.apply_defaults(input_json).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        
        if let Value::Object(obj) = parsed {
            // Should have references set to null
            assert!(obj.contains_key("references"));
            assert!(obj["references"].is_null());
            
            // Should have body.format set to "plaintext"
            if let Value::Object(body) = &obj["body"] {
                assert_eq!(body["format"], Value::String("plaintext".to_string()));
            }
        }
    }
}