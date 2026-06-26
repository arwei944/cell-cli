use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub rule: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>, rule: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            rule: rule.into(),
        }
    }
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_errors(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn format(&self) -> String {
        let mut o = String::new();
        o.push_str("🔍 Cell Config Validation\n");
        o.push_str(&"=".repeat(50));
        o.push_str("\n\n");

        if self.valid && self.warnings.is_empty() {
            o.push_str("✅ Valid\n");
            return o;
        }

        if !self.errors.is_empty() {
            o.push_str(&format!("❌ Errors ({}):\n", self.errors.len()));
            for err in &self.errors {
                o.push_str(&format!("  [{}] {}: {}\n", err.rule, err.path, err.message));
            }
            o.push_str("\n");
        }

        if !self.warnings.is_empty() {
            o.push_str(&format!("⚠️  Warnings ({}):\n", self.warnings.len()));
            for warn in &self.warnings {
                o.push_str(&format!("  [{}] {}: {}\n", warn.rule, warn.path, warn.message));
            }
        }

        o
    }
}

/// Cell 配置 Schema 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellConfigSchema {
    pub required_fields: Vec<String>,
    pub field_types: HashMap<String, String>,
    pub max_dependencies: usize,
    pub min_version_parts: usize,
    pub max_owners: usize,
}

impl Default for CellConfigSchema {
    fn default() -> Self {
        Self {
            required_fields: vec![
                "name".to_string(),
                "description".to_string(),
                "version".to_string(),
                "owners".to_string(),
                "ports".to_string(),
                "adapters".to_string(),
                "dependencies".to_string(),
            ],
            field_types: HashMap::from([
                ("name".to_string(), "string".to_string()),
                ("description".to_string(), "string".to_string()),
                ("version".to_string(), "semver".to_string()),
                ("owners".to_string(), "array[string]".to_string()),
                ("ports".to_string(), "array".to_string()),
                ("adapters".to_string(), "array".to_string()),
                ("dependencies".to_string(), "array[string]".to_string()),
            ]),
            max_dependencies: 50,
            min_version_parts: 2,
            max_owners: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_result_format() {
        let result = ValidationResult::success();
        let formatted = result.format();
        assert!(formatted.contains("✅ Valid"));
    }

    #[test]
    fn test_schema_default() {
        let schema = CellConfigSchema::default();
        assert!(schema.required_fields.contains(&"name".to_string()));
        assert!(schema.required_fields.contains(&"version".to_string()));
    }
}
