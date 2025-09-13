// memo-loader-utils.typ: Consolidated validation utilities and schema for memo loading

// =============================================================================
// MISC UTILS
// =============================================================================

/// Checks if a value is "falsey" (none, false, empty array, or empty string).
/// - value (any): The value to check.
#let falsey(value) = {
  value == none or value == false or (type(value) == array and value.len() == 0) or (type(value) == str and value == "")
}

// =============================================================================
// SCHEMA DEFINITIONS
// =============================================================================

// Define the complete validation schema based on official-memorandum-schema.json
#let official-memorandum-schema = (
  allowed-properties: (
    "memo-for",
    "from-block", 
    "subject",
    "references",
    "signature-block",
    "body"
  ),
  
  required-properties: (
    "memo-for",
    "from-block",
    "subject", 
    "signature-block",
    "body"
  ),
  
  properties: (
    "memo-for": (
      (type: "type", expected: array),
      (type: "array-length", min: 1),
      (type: "array-items", expected: str)
    ),
    
    "from-block": (
      (type: "type", expected: array),
      (type: "array-length", min: 1),
      (type: "array-items", expected: str)
    ),
    
    "subject": (
      (type: "type", expected: str),
      (type: "string-length", min: 1)
    ),
    
    "references": (
      (type: "nullable-array"),
    ),
    
    "signature-block": (
      (type: "type", expected: array),
      (type: "array-length", min: 2),  // AFH 33-337 requirement
      (type: "array-items", expected: str)
    ),
    
    "body": (
      (type: "type", expected: dictionary),
    )
  ),
  
  // Nested schema for body object
  body-schema: (
    allowed-properties: ("format", "data"),
    
    properties: (
      "format": (
        (type: "type", expected: str),
        (type: "enum", allowed: ("plaintext",))
      ),
      
      "data": (
        (type: "type", expected: str),
      )
    )
  )
)

// =============================================================================
// VALIDATION UTILITIES
// =============================================================================

/// Validates that a value is of the expected type.
/// - value (any): The value to check.
/// - expected-type (type): The expected type.
/// - path (str): Property path for error messages.
/// -> array: Empty array if valid, array with error message if invalid.
#let validate-type(value, expected-type, path) = {
  if type(value) != expected-type {
    return ("Property '" + path + "' must be " + repr(expected-type) + " (got " + repr(type(value)) + ")")
  }
  ()
}

/// Validates that an array contains only items of the expected type.
/// - arr (array): The array to validate.
/// - expected-item-type (type): The expected type for array items.
/// - path (str): Property path for error messages.
/// -> array: Array of error messages (empty if valid).
#let validate-array-items(arr, expected-item-type, path) = {
  let errors = ()
  for (i, item) in arr.enumerate() {
    if type(item) != expected-item-type {
      errors.push("All items in '" + path + "' array must be " + repr(expected-item-type) + "s (item " + str(i) + " is " + repr(type(item)) + ")")
    }
  }
  errors
}

/// Validates that a string meets minimum length requirements.
/// - value (str): The string to validate.
/// - min-length (int): Minimum required length.
/// - path (str): Property path for error messages.
/// -> array: Empty array if valid, array with error message if invalid.
#let validate-string-length(value, min-length, path) = {
  if value.len() < min-length {
    return ("Property '" + path + "' cannot be empty (minLength: " + str(min-length) + ")")
  }
  ()
}

/// Validates that an array meets minimum length requirements.
/// - arr (array): The array to validate.
/// - min-items (int): Minimum required number of items.
/// - path (str): Property path for error messages.
/// -> array: Empty array if valid, array with error message if invalid.
#let validate-array-length(arr, min-items, path) = {
  if arr.len() < min-items {
    return ("Property '" + path + "' must contain at least " + str(min-items) + " item" + (if min-items > 1 { "s" } else { "" }))
  }
  ()
}

/// Validates that a value is either null or an array.
/// - value (any): The value to check.
/// - path (str): Property path for error messages.
/// -> array: Empty array if valid, array with error message if invalid.
#let validate-nullable-array(value, path) = {
  if value != none and type(value) != array {
    return ("Property '" + path + "' must be an array or null")
  }
  ()
}

/// Validates an enum value against allowed options.
/// - value (str): The value to validate.
/// - allowed-values (array): Array of allowed string values.
/// - path (str): Property path for error messages.
/// -> array: Empty array if valid, array with error message if invalid.
#let validate-enum(value, allowed-values, path) = {
  if value not in allowed-values {
    return ("Property '" + path + "' must be one of: " + allowed-values.join(", ") + " (got '" + str(value) + "')")
  }
  ()
}

/// Generic property validator that applies multiple validation rules.
/// - value (any): The value to validate.
/// - rules (array): Array of validation rule dictionaries.
/// - path (str): Property path for error messages.
/// -> array: Array of error messages (empty if all validations pass).
#let validate-property(value, rules, path) = {
  let errors = ()
  
  for rule in rules {
    if rule.type == "type" {
      errors = errors + validate-type(value, rule.expected, path)
    } else if rule.type == "array-items" {
      if type(value) == array {
        errors = errors + validate-array-items(value, rule.expected, path)
      }
    } else if rule.type == "string-length" {
      if type(value) == str {
        errors = errors + validate-string-length(value, rule.min, path)
      }
    } else if rule.type == "array-length" {
      if type(value) == array {
        errors = errors + validate-array-length(value, rule.min, path)
      }
    } else if rule.type == "nullable-array" {
      errors = errors + validate-nullable-array(value, path)
    } else if rule.type == "enum" {
      if type(value) == str {
        errors = errors + validate-enum(value, rule.allowed, path)
      }
    }
  }
  
  errors
}

/// Validates a data object against a schema definition.
/// - data (dictionary): The data to validate.
/// - schema (dictionary): Schema definition with properties and rules.
/// -> array: Array of error messages (empty if valid).
#let validate-schema(data, schema) = {
  let errors = ()
  
  // Check for unexpected properties
  if "allowed-properties" in schema {
    for key in data.keys() {
      if key not in schema.allowed-properties {
        errors.push("Unexpected property '" + key + "' found. Only these properties are allowed: " + schema.allowed-properties.join(", "))
      }
    }
  }
  
  // Check for required properties
  if "required-properties" in schema {
    for prop in schema.required-properties {
      if prop not in data.keys() {
        errors.push("Required property '" + prop + "' is missing")
      }
    }
  }
  
  // Validate individual properties
  if "properties" in schema {
    for (prop-name, prop-rules) in schema.properties {
      if prop-name in data.keys() {
        let prop-errors = validate-property(data.at(prop-name), prop-rules, prop-name)
        errors = errors + prop-errors
      }
    }
  }
  
  errors
}

/// Complete validation function for official memorandum input.
/// - data (dictionary): The input data to validate.
/// -> array: Array of error messages (empty if valid).
#let validate-memo-input(data) = {
  let errors = validate-schema(data, official-memorandum-schema)
  
  // Handle special validation for references array items (if not null)
  if "references" in data.keys() and data.references != none {
    if type(data.references) == array {
      let ref-errors = validate-property(data.references, ((type: "array-items", expected: str),), "references")
      errors = errors + ref-errors
    }
  }
  
  // Validate nested body object
  if "body" in data.keys() and type(data.body) == dictionary {
    let body-errors = validate-schema(data.body, official-memorandum-schema.body-schema)
    // Prefix body errors with "body." for clarity
    for error in body-errors {
      errors.push("body." + error)
    }
  }
  
  return errors
}

/// Applies default values for optional fields based on schema.
/// - data (dictionary): The input data.
/// -> dictionary: Data with defaults applied.
#let apply-memo-defaults(data) = {
  let result = data
  
  // Apply default for references if not present
  if "references" not in result.keys() {
    result.references = none
  }
  
  // Apply default for body.format if not present
  if "body" in result.keys() and type(result.body) == dictionary {
    if "format" not in result.body.keys() {
      result.body.format = "plaintext"
    }
  }
  
  return result
}
