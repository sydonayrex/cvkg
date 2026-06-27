//! Form validation framework with schema-based rules.
//!
//! Provides `ValidationRule`, `FormValidator`, and `ValidationError` types
//! for client-side form validation with clear error messages.

use std::collections::HashMap;
use std::fmt;

/// A single validation rule for a field.
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// Field must not be empty.
    Required,
    /// Minimum string length.
    MinLength(usize),
    /// Maximum string length.
    MaxLength(usize),
    /// Email format validation (simple check).
    Email,
    /// Numeric range validation.
    Range(f64, f64),
    /// Must match one of the allowed values.
    OneOf(Vec<String>),
}

impl ValidationRule {
    /// Validate a value against this rule.
    pub fn validate(&self, field_name: &str, value: &str) -> Result<(), ValidationError> {
        match self {
            ValidationRule::Required => {
                if value.trim().is_empty() {
                    Err(ValidationError::new(field_name, "is required"))
                } else {
                    Ok(())
                }
            }
            ValidationRule::MinLength(min) => {
                if value.len() < *min {
                    Err(ValidationError::new(
                        field_name,
                        &format!("must be at least {} characters", min),
                    ))
                } else {
                    Ok(())
                }
            }
            ValidationRule::MaxLength(max) => {
                if value.len() > *max {
                    Err(ValidationError::new(
                        field_name,
                        &format!("must be at most {} characters", max),
                    ))
                } else {
                    Ok(())
                }
            }
            ValidationRule::Email => {
                // Simple email validation without regex
                let has_at = value.contains('@');
                let has_domain = value.contains('.');
                let no_spaces = !value.contains(' ');
                let valid = has_at && has_domain && no_spaces && value.len() >= 5;
                if !valid {
                    Err(ValidationError::new(
                        field_name,
                        "must be a valid email address",
                    ))
                } else {
                    Ok(())
                }
            }
            ValidationRule::Range(min, max) => match value.parse::<f64>() {
                Ok(n) if n >= *min && n <= *max => Ok(()),
                Ok(_) => Err(ValidationError::new(
                    field_name,
                    &format!("must be between {} and {}", min, max),
                )),
                Err(_) => Err(ValidationError::new(field_name, "must be a number")),
            },
            ValidationRule::OneOf(options) => {
                if !options.contains(&value.to_string()) {
                    Err(ValidationError::new(
                        field_name,
                        &format!("must be one of: {}", options.join(", ")),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// A validation error for a specific field.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Schema definition for form validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationSchema {
    rules: HashMap<String, Vec<ValidationRule>>,
}

impl ValidationSchema {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add rules for a field.
    pub fn field(mut self, name: &str, rules: Vec<ValidationRule>) -> Self {
        self.rules.insert(name.to_string(), rules);
        self
    }

    /// Validate all fields in a form data map.
    pub fn validate(&self, data: &HashMap<String, String>) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (field_name, rules) in &self.rules {
            let value = data.get(field_name).unwrap_or(&String::new()).clone();
            for rule in rules {
                if let Err(err) = rule.validate(field_name, &value) {
                    errors.push(err);
                }
            }
        }

        errors
    }

    /// Validate a single field.
    pub fn validate_field(&self, field_name: &str, value: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        if let Some(rules) = self.rules.get(field_name) {
            for rule in rules {
                if let Err(err) = rule.validate(field_name, value) {
                    errors.push(err);
                }
            }
        }
        errors
    }
}

/// Convenience builder for common validation patterns.
pub struct FieldValidator;

impl FieldValidator {
    pub fn required() -> Vec<ValidationRule> {
        vec![ValidationRule::Required]
    }

    pub fn email() -> Vec<ValidationRule> {
        vec![ValidationRule::Required, ValidationRule::Email]
    }

    pub fn password(min_len: usize) -> Vec<ValidationRule> {
        vec![ValidationRule::Required, ValidationRule::MinLength(min_len)]
    }

    pub fn text(min: usize, max: usize) -> Vec<ValidationRule> {
        vec![
            ValidationRule::Required,
            ValidationRule::MinLength(min),
            ValidationRule::MaxLength(max),
        ]
    }

    pub fn number(min: f64, max: f64) -> Vec<ValidationRule> {
        vec![ValidationRule::Range(min, max)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_rejects_empty() {
        let rule = ValidationRule::Required;
        assert!(rule.validate("name", "").is_err());
        assert!(rule.validate("name", "  ").is_err());
        assert!(rule.validate("name", "Alice").is_ok());
    }

    #[test]
    fn min_length_validates() {
        let rule = ValidationRule::MinLength(3);
        assert!(rule.validate("name", "ab").is_err());
        assert!(rule.validate("name", "abc").is_ok());
        assert!(rule.validate("name", "Alice").is_ok());
    }

    #[test]
    fn max_length_validates() {
        let rule = ValidationRule::MaxLength(5);
        assert!(rule.validate("code", "abcdef").is_err());
        assert!(rule.validate("code", "abc").is_ok());
    }

    #[test]
    fn email_validates() {
        let rule = ValidationRule::Email;
        assert!(rule.validate("email", "not-an-email").is_err());
        assert!(rule.validate("email", "user@").is_err());
        assert!(rule.validate("email", "u@c").is_err()); // too short
        assert!(rule.validate("email", "user@example.com").is_ok());
        assert!(rule.validate("email", "a@b.c").is_ok());
    }

    #[test]
    fn schema_validates_all_fields() {
        let schema = ValidationSchema::new()
            .field("name", FieldValidator::required())
            .field("email", FieldValidator::email())
            .field("password", FieldValidator::password(8));

        let mut data = HashMap::new();
        data.insert("name".to_string(), "".to_string());
        data.insert("email".to_string(), "bad".to_string());
        data.insert("password".to_string(), "short".to_string());

        let errors = schema.validate(&data);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn schema_accepts_valid_data() {
        let schema = ValidationSchema::new()
            .field("name", FieldValidator::required())
            .field("email", FieldValidator::email());

        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());
        data.insert("email".to_string(), "alice@example.com".to_string());

        let errors = schema.validate(&data);
        assert!(errors.is_empty());
    }

    #[test]
    fn range_validates_numbers() {
        let rule = ValidationRule::Range(0.0, 100.0);
        assert!(rule.validate("score", "50").is_ok());
        assert!(rule.validate("score", "150").is_err());
        assert!(rule.validate("score", "abc").is_err());
    }

    #[test]
    fn one_of_validates_options() {
        let rule = ValidationRule::OneOf(vec![
            "red".to_string(),
            "green".to_string(),
            "blue".to_string(),
        ]);
        assert!(rule.validate("color", "red").is_ok());
        assert!(rule.validate("color", "purple").is_err());
    }

    #[test]
    fn single_field_validation() {
        let schema = ValidationSchema::new().field("email", FieldValidator::email());

        let errors = schema.validate_field("email", "bad");
        assert!(!errors.is_empty());

        let errors = schema.validate_field("email", "good@example.com");
        assert!(errors.is_empty());
    }
}
