use crate::domain::cell_spec::CellSpec;
use crate::domain::config_schema::{CellConfigSchema, ValidationError, ValidationResult};
use crate::domain::errors::CellError;
use crate::domain::errors::CellResult;

/// 配置 Schema 验证服务
pub struct ConfigSchemaValidator {
    schema: CellConfigSchema,
}

impl ConfigSchemaValidator {
    pub fn new() -> Self {
        Self {
            schema: CellConfigSchema::default(),
        }
    }

    pub fn with_schema(schema: CellConfigSchema) -> Self {
        Self { schema }
    }

    /// 验证 CellSpec 是否符合 schema
    pub fn validate(&self, spec: &CellSpec) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. 检查必需字段
        if spec.name.is_empty() {
            errors.push(ValidationError::new("name", "name is required and cannot be empty", "required"));
        }
        if spec.description.is_empty() {
            errors.push(ValidationError::new("description", "description is required", "required"));
        }
        if spec.version.is_empty() {
            errors.push(ValidationError::new("version", "version is required", "required"));
        }

        // 2. 检查版本格式
        let version_parts: Vec<&str> = spec.version.split('.').collect();
        if version_parts.len() < self.schema.min_version_parts {
            errors.push(ValidationError::new(
                "version",
                format!("version must have at least {} parts (e.g., 1.0.0)", self.schema.min_version_parts),
                "format",
            ));
        }

        // 3. 检查 owners
        if spec.owners.is_empty() {
            errors.push(ValidationError::new("owners", "at least one owner is required", "required"));
        } else if spec.owners.len() > self.schema.max_owners {
            warnings.push(ValidationError::new(
                "owners",
                format!("too many owners: {} (max: {})", spec.owners.len(), self.schema.max_owners),
                "constraint",
            ));
        }

        // 4. 检查 dependencies 数量
        if spec.dependencies.len() > self.schema.max_dependencies {
            warnings.push(ValidationError::new(
                "dependencies",
                format!("too many dependencies: {} (max: {})", spec.dependencies.len(), self.schema.max_dependencies),
                "constraint",
            ));
        }

        // 5. 检查端口和适配器
        if spec.ports.is_empty() {
            warnings.push(ValidationError::new("ports", "no ports defined", "best_practice"));
        }
        if spec.adapters.is_empty() {
            warnings.push(ValidationError::new("adapters", "no adapters defined", "best_practice"));
        }

        if errors.is_empty() {
            ValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings,
            }
        } else {
            ValidationResult {
                valid: false,
                errors,
                warnings,
            }
        }
    }

    /// 验证并返回 CellResult
    pub fn validate_or_err(&self, spec: &CellSpec) -> CellResult<ValidationResult> {
        let result = self.validate(spec);
        if result.valid {
            Ok(result)
        } else {
            let msg = result.errors.iter()
                .map(|e| format!("{}: {}", e.path, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            Err(CellError::Config(msg))
        }
    }
}

impl Default for ConfigSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cell_spec::CellSpec;

    #[test]
    fn test_valid_spec() {
        let spec = CellSpec {
            name: "test".into(),
            description: "desc".into(),
            version: "1.0.0".into(),
            cell_version: None,
            owners: vec!["alice".into()],
            ports: vec![],
            adapters: vec![],
            dependencies: vec![],
            tags: vec![],
            architecture: None,
            lint: None,
            entropy: None,
            domain: None,
        };
        let validator = ConfigSchemaValidator::new();
        let result = validator.validate(&spec);
        assert!(result.valid);
    }

    #[test]
    fn test_missing_required_fields() {
        let spec = CellSpec {
            name: "".into(),
            description: "".into(),
            version: "".into(),
            cell_version: None,
            owners: vec![],
            ports: vec![],
            adapters: vec![],
            dependencies: vec![],
            tags: vec![],
            architecture: None,
            lint: None,
            entropy: None,
            domain: None,
        };
        let validator = ConfigSchemaValidator::new();
        let result = validator.validate(&spec);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }
}
