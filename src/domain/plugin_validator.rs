use crate::domain::plugin_system::{Permission, PluginId, PluginManifest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl ValidationSeverity {
    pub fn label(&self) -> &str {
        match self {
            ValidationSeverity::Info => "info",
            ValidationSeverity::Warning => "warning",
            ValidationSeverity::Error => "error",
            ValidationSeverity::Critical => "critical",
        }
    }

    pub fn is_error_or_above(&self) -> bool {
        matches!(self, ValidationSeverity::Error | ValidationSeverity::Critical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationType {
    Manifest,
    Permission,
    Dependency,
    VersionCompatibility,
    NamingConvention,
    Security,
}

impl ValidationType {
    pub fn label(&self) -> &str {
        match self {
            ValidationType::Manifest => "manifest",
            ValidationType::Permission => "permission",
            ValidationType::Dependency => "dependency",
            ValidationType::VersionCompatibility => "version_compatibility",
            ValidationType::NamingConvention => "naming_convention",
            ValidationType::Security => "security",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFinding {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: ValidationSeverity,
    pub validation_type: ValidationType,
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub plugin_id: PluginId,
    pub plugin_name: String,
    pub findings: Vec<ValidationFinding>,
    pub validated_at: DateTime<Utc>,
    pub duration_ms: u64,
}

impl ValidationResult {
    pub fn new(plugin_id: PluginId, plugin_name: String) -> Self {
        Self {
            plugin_id,
            plugin_name,
            findings: Vec::new(),
            validated_at: Utc::now(),
            duration_ms: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.findings.iter().any(|f| f.severity.is_error_or_above())
    }

    pub fn has_critical(&self) -> bool {
        self.findings.iter().any(|f| f.severity == ValidationSeverity::Critical)
    }

    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == ValidationSeverity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(|f| f.severity == ValidationSeverity::Warning)
    }

    pub fn filter_by_severity(&self, min_severity: ValidationSeverity) -> Vec<&ValidationFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity >= min_severity)
            .collect()
    }

    pub fn count_by_severity(&self) -> HashMap<ValidationSeverity, usize> {
        let mut counts = HashMap::new();
        for finding in &self.findings {
            *counts.entry(finding.severity).or_insert(0) += 1;
        }
        counts
    }

    pub fn add_finding(&mut self, finding: ValidationFinding) {
        self.findings.push(finding);
    }
}

pub type CheckFn = fn(&PluginManifest) -> Option<ValidationFinding>;

pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub validation_type: ValidationType,
    check: Box<CheckFn>,
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationRule")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("severity", &self.severity)
            .field("validation_type", &self.validation_type)
            .finish()
    }
}

impl ValidationRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        severity: ValidationSeverity,
        validation_type: ValidationType,
        check: CheckFn,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            severity,
            validation_type,
            check: Box::new(check),
        }
    }

    pub fn check(&self, manifest: &PluginManifest) -> Option<ValidationFinding> {
        let mut finding = (self.check)(manifest)?;
        if finding.rule_id.is_empty() {
            finding.rule_id = self.id.clone();
            finding.rule_name = self.name.clone();
            finding.severity = self.severity;
            finding.validation_type = self.validation_type.clone();
        }
        Some(finding)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_plugins: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<ValidationResult>,
    pub generated_at: DateTime<Utc>,
}

impl ValidationReport {
    pub fn new(results: Vec<ValidationResult>) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.is_valid()).count();
        let failed = total - passed;
        Self {
            total_plugins: total,
            passed,
            failed,
            results,
            generated_at: Utc::now(),
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    pub fn filter_results_by_severity(&self, min_severity: ValidationSeverity) -> Vec<&ValidationResult> {
        self.results
            .iter()
            .filter(|r| r.findings.iter().any(|f| f.severity >= min_severity))
            .collect()
    }

    pub fn total_findings_by_severity(&self) -> HashMap<ValidationSeverity, usize> {
        let mut totals = HashMap::new();
        for result in &self.results {
            for (sev, count) in result.count_by_severity() {
                *totals.entry(sev).or_insert(0) += count;
            }
        }
        totals
    }
}

pub struct PluginValidator {
    rules: Vec<ValidationRule>,
    host_version: String,
}

impl PluginValidator {
    pub fn new(host_version: impl Into<String>) -> Self {
        Self {
            rules: Vec::new(),
            host_version: host_version.into(),
        }
    }

    pub fn with_default_rules(host_version: impl Into<String>) -> Self {
        let mut validator = Self::new(host_version);
        for rule in builtin_rules() {
            validator.register_rule(rule);
        }
        validator
    }

    pub fn register_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    pub fn rules(&self) -> &[ValidationRule] {
        &self.rules
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn validate(&self, manifest: &PluginManifest) -> ValidationResult {
        let start = Utc::now();
        let mut result = ValidationResult::new(manifest.id.clone(), manifest.name.clone());

        for rule in &self.rules {
            if let Some(finding) = rule.check(manifest) {
                result.add_finding(finding);
            }
        }

        if let Some(finding) = self.check_host_version_compatibility(manifest) {
            result.add_finding(finding);
        }

        let duration = (Utc::now() - start).num_milliseconds().max(0) as u64;
        result.duration_ms = duration;
        result
    }

    fn check_host_version_compatibility(
        &self,
        manifest: &PluginManifest,
    ) -> Option<ValidationFinding> {
        let host_parts: Vec<u64> = self
            .host_version
            .split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        let min_parts: Vec<u64> = manifest
            .min_host_version
            .split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        let mut compatible = true;
        for i in 0..3 {
            let host = host_parts.get(i).copied().unwrap_or(0);
            let min = min_parts.get(i).copied().unwrap_or(0);
            if host > min {
                break;
            } else if host < min {
                compatible = false;
                break;
            }
        }

        if !compatible {
            Some(ValidationFinding {
                rule_id: "V002".to_string(),
                rule_name: "host_version_compatible".to_string(),
                severity: ValidationSeverity::Error,
                validation_type: ValidationType::VersionCompatibility,
                message: format!(
                    "Plugin requires host version {}, current is {}",
                    manifest.min_host_version, self.host_version
                ),
                field: Some("min_host_version".to_string()),
                suggestion: Some("Upgrade host or use a compatible plugin version".to_string()),
            })
        } else {
            None
        }
    }

    pub fn validate_with_severity_filter(
        &self,
        manifest: &PluginManifest,
        min_severity: ValidationSeverity,
    ) -> ValidationResult {
        let full_result = self.validate(manifest);
        let mut filtered_result =
            ValidationResult::new(full_result.plugin_id.clone(), full_result.plugin_name.clone());
        filtered_result.validated_at = full_result.validated_at;
        filtered_result.duration_ms = full_result.duration_ms;

        for finding in full_result.findings {
            if finding.severity >= min_severity {
                filtered_result.add_finding(finding);
            }
        }

        filtered_result
    }

    pub fn validate_all(&self, manifests: &[PluginManifest]) -> ValidationReport {
        let results: Vec<ValidationResult> =
            manifests.iter().map(|m| self.validate(m)).collect();
        ValidationReport::new(results)
    }

    pub fn generate_report(&self, manifests: &[PluginManifest]) -> ValidationReport {
        self.validate_all(manifests)
    }
}

pub fn builtin_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::new(
            "M001",
            "plugin_id_not_empty",
            "Plugin ID must not be empty",
            ValidationSeverity::Critical,
            ValidationType::Manifest,
            |m| {
                if m.id.0.is_empty() {
                    Some(ValidationFinding {
                        rule_id: "M001".to_string(),
                        rule_name: "plugin_id_not_empty".to_string(),
                        severity: ValidationSeverity::Critical,
                        validation_type: ValidationType::Manifest,
                        message: "Plugin ID is empty".to_string(),
                        field: Some("id".to_string()),
                        suggestion: Some("Provide a non-empty plugin ID".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "M002",
            "plugin_name_not_empty",
            "Plugin name must not be empty",
            ValidationSeverity::Error,
            ValidationType::Manifest,
            |m| {
                if m.name.is_empty() {
                    Some(ValidationFinding {
                        rule_id: "M002".to_string(),
                        rule_name: "plugin_name_not_empty".to_string(),
                        severity: ValidationSeverity::Error,
                        validation_type: ValidationType::Manifest,
                        message: "Plugin name is empty".to_string(),
                        field: Some("name".to_string()),
                        suggestion: Some("Provide a non-empty plugin name".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "V001",
            "version_format_valid",
            "Plugin version must be valid semantic version format (x.y.z)",
            ValidationSeverity::Error,
            ValidationType::VersionCompatibility,
            |m| {
                let parts: Vec<&str> = m.version.split('.').collect();
                let is_valid = parts.len() == 3
                    && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
                if !is_valid {
                    Some(ValidationFinding {
                        rule_id: "V001".to_string(),
                        rule_name: "version_format_valid".to_string(),
                        severity: ValidationSeverity::Error,
                        validation_type: ValidationType::VersionCompatibility,
                        message: format!("Version '{}' is not valid semantic version format", m.version),
                        field: Some("version".to_string()),
                        suggestion: Some("Use semantic version format: x.y.z (e.g., 1.0.0)".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "N001",
            "plugin_id_lowercase_snake_case",
            "Plugin ID should use lowercase with hyphens",
            ValidationSeverity::Warning,
            ValidationType::NamingConvention,
            |m| {
                let valid = m
                    .id
                    .0
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
                if !valid {
                    Some(ValidationFinding {
                        rule_id: "N001".to_string(),
                        rule_name: "plugin_id_lowercase_snake_case".to_string(),
                        severity: ValidationSeverity::Warning,
                        validation_type: ValidationType::NamingConvention,
                        message: "Plugin ID should contain only lowercase letters, digits, and hyphens".to_string(),
                        field: Some("id".to_string()),
                        suggestion: Some("Use lowercase letters, digits, and hyphens (e.g., my-plugin-1)".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "P001",
            "no_unnecessary_all_permission",
            "Avoid using All permission unless absolutely necessary",
            ValidationSeverity::Warning,
            ValidationType::Permission,
            |m| {
                if m.permissions.contains(&Permission::All) {
                    Some(ValidationFinding {
                        rule_id: "P001".to_string(),
                        rule_name: "no_unnecessary_all_permission".to_string(),
                        severity: ValidationSeverity::Warning,
                        validation_type: ValidationType::Permission,
                        message: "Plugin requests 'All' permission which grants full access".to_string(),
                        field: Some("permissions".to_string()),
                        suggestion: Some(
                            "Request only the specific permissions needed by the plugin".to_string(),
                        ),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "D001",
            "no_self_dependency",
            "Plugin must not depend on itself",
            ValidationSeverity::Error,
            ValidationType::Dependency,
            |m| {
                if m.dependencies.contains(&m.id) {
                    Some(ValidationFinding {
                        rule_id: "D001".to_string(),
                        rule_name: "no_self_dependency".to_string(),
                        severity: ValidationSeverity::Error,
                        validation_type: ValidationType::Dependency,
                        message: "Plugin depends on itself".to_string(),
                        field: Some("dependencies".to_string()),
                        suggestion: Some("Remove the self-reference from dependencies".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "S001",
            "entry_point_has_valid_extension",
            "Entry point should have a valid file extension",
            ValidationSeverity::Info,
            ValidationType::Security,
            |m| {
                let has_valid_ext = m.entry_point.ends_with(".wasm")
                    || m.entry_point.ends_with(".js")
                    || m.entry_point.ends_with(".ts");
                if !has_valid_ext {
                    Some(ValidationFinding {
                        rule_id: "S001".to_string(),
                        rule_name: "entry_point_has_valid_extension".to_string(),
                        severity: ValidationSeverity::Info,
                        validation_type: ValidationType::Security,
                        message: format!(
                            "Entry point '{}' does not have a recognized extension",
                            m.entry_point
                        ),
                        field: Some("entry_point".to_string()),
                        suggestion: Some("Use .wasm, .js, or .ts extension for entry point".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
        ValidationRule::new(
            "M003",
            "description_not_empty",
            "Plugin should have a description",
            ValidationSeverity::Info,
            ValidationType::Manifest,
            |m| {
                if m.description.is_empty() {
                    Some(ValidationFinding {
                        rule_id: "M003".to_string(),
                        rule_name: "description_not_empty".to_string(),
                        severity: ValidationSeverity::Info,
                        validation_type: ValidationType::Manifest,
                        message: "Plugin has no description".to_string(),
                        field: Some("description".to_string()),
                        suggestion: Some("Add a brief description of what the plugin does".to_string()),
                    })
                } else {
                    None
                }
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plugin_system::PluginKind;

    fn create_test_manifest(id: &str, name: &str) -> PluginManifest {
        PluginManifest {
            id: PluginId(id.to_string()),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Test plugin {}", name),
            author: "Test Author".to_string(),
            kind: PluginKind::Analyzer,
            min_host_version: "0.1.0".to_string(),
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            entry_point: "plugin.wasm".to_string(),
            permissions: vec![Permission::ReadFileSystem],
            tags: vec!["test".to_string()],
            homepage: None,
            repository: None,
            license: None,
        }
    }

    #[test]
    fn test_validation_severity_ordering() {
        assert!(ValidationSeverity::Info < ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning < ValidationSeverity::Error);
        assert!(ValidationSeverity::Error < ValidationSeverity::Critical);
        assert!(ValidationSeverity::Critical >= ValidationSeverity::Error);
    }

    #[test]
    fn test_validation_severity_is_error_or_above() {
        assert!(!ValidationSeverity::Info.is_error_or_above());
        assert!(!ValidationSeverity::Warning.is_error_or_above());
        assert!(ValidationSeverity::Error.is_error_or_above());
        assert!(ValidationSeverity::Critical.is_error_or_above());
    }

    #[test]
    fn test_validation_result_is_valid() {
        let mut result = ValidationResult::new(PluginId("test".into()), "Test".into());
        assert!(result.is_valid());

        result.add_finding(ValidationFinding {
            rule_id: "R001".into(),
            rule_name: "test".into(),
            severity: ValidationSeverity::Warning,
            validation_type: ValidationType::Manifest,
            message: "test".into(),
            field: None,
            suggestion: None,
        });
        assert!(result.is_valid());

        result.add_finding(ValidationFinding {
            rule_id: "R002".into(),
            rule_name: "test".into(),
            severity: ValidationSeverity::Error,
            validation_type: ValidationType::Manifest,
            message: "test".into(),
            field: None,
            suggestion: None,
        });
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validation_result_filter_by_severity() {
        let mut result = ValidationResult::new(PluginId("test".into()), "Test".into());
        result.add_finding(ValidationFinding {
            rule_id: "R1".into(),
            rule_name: "info_rule".into(),
            severity: ValidationSeverity::Info,
            validation_type: ValidationType::Manifest,
            message: "info".into(),
            field: None,
            suggestion: None,
        });
        result.add_finding(ValidationFinding {
            rule_id: "R2".into(),
            rule_name: "warn_rule".into(),
            severity: ValidationSeverity::Warning,
            validation_type: ValidationType::Manifest,
            message: "warning".into(),
            field: None,
            suggestion: None,
        });
        result.add_finding(ValidationFinding {
            rule_id: "R3".into(),
            rule_name: "err_rule".into(),
            severity: ValidationSeverity::Error,
            validation_type: ValidationType::Manifest,
            message: "error".into(),
            field: None,
            suggestion: None,
        });

        assert_eq!(result.filter_by_severity(ValidationSeverity::Info).len(), 3);
        assert_eq!(result.filter_by_severity(ValidationSeverity::Warning).len(), 2);
        assert_eq!(result.filter_by_severity(ValidationSeverity::Error).len(), 1);
        assert_eq!(result.filter_by_severity(ValidationSeverity::Critical).len(), 0);
    }

    #[test]
    fn test_validation_result_count_by_severity() {
        let mut result = ValidationResult::new(PluginId("test".into()), "Test".into());
        result.add_finding(ValidationFinding {
            rule_id: "R1".into(),
            rule_name: "info_rule".into(),
            severity: ValidationSeverity::Info,
            validation_type: ValidationType::Manifest,
            message: "info".into(),
            field: None,
            suggestion: None,
        });
        result.add_finding(ValidationFinding {
            rule_id: "R2".into(),
            rule_name: "info_rule2".into(),
            severity: ValidationSeverity::Info,
            validation_type: ValidationType::Manifest,
            message: "info2".into(),
            field: None,
            suggestion: None,
        });
        result.add_finding(ValidationFinding {
            rule_id: "R3".into(),
            rule_name: "err_rule".into(),
            severity: ValidationSeverity::Error,
            validation_type: ValidationType::Manifest,
            message: "error".into(),
            field: None,
            suggestion: None,
        });

        let counts = result.count_by_severity();
        assert_eq!(*counts.get(&ValidationSeverity::Info).unwrap(), 2);
        assert_eq!(*counts.get(&ValidationSeverity::Error).unwrap(), 1);
        assert!(!counts.contains_key(&ValidationSeverity::Warning));
    }

    #[test]
    fn test_single_rule_validation_pass() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let manifest = create_test_manifest("my-plugin", "My Plugin");
        let result = validator.validate(&manifest);
        assert!(result.is_valid());
    }

    #[test]
    fn test_single_rule_validation_fail() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let mut manifest = create_test_manifest("", "My Plugin");
        manifest.id = PluginId("".into());
        let result = validator.validate(&manifest);
        assert!(!result.is_valid());
        assert!(result.has_critical());
    }

    #[test]
    fn test_custom_rule() {
        let mut validator = PluginValidator::new("1.0.0");
        validator.register_rule(ValidationRule::new(
            "CUSTOM001",
            "author_not_empty",
            "Author field must not be empty",
            ValidationSeverity::Warning,
            ValidationType::Manifest,
            |m| {
                if m.author.is_empty() {
                    Some(ValidationFinding {
                        rule_id: "CUSTOM001".into(),
                        rule_name: "author_not_empty".into(),
                        severity: ValidationSeverity::Warning,
                        validation_type: ValidationType::Manifest,
                        message: "Author is empty".into(),
                        field: Some("author".into()),
                        suggestion: Some("Provide author information".into()),
                    })
                } else {
                    None
                }
            },
        ));

        let mut manifest = create_test_manifest("test", "Test");
        manifest.author = "".into();
        let result = validator.validate(&manifest);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].rule_id, "CUSTOM001");
    }

    #[test]
    fn test_multiple_rules_validation() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let mut manifest = create_test_manifest("", "");
        manifest.version = "invalid".into();
        let result = validator.validate(&manifest);

        let ids: Vec<&str> = result.findings.iter().map(|f| f.rule_id.as_str()).collect();
        assert!(ids.contains(&"M001"));
        assert!(ids.contains(&"M002"));
        assert!(ids.contains(&"V001"));
        assert!(!result.is_valid());
    }

    #[test]
    fn test_validate_with_severity_filter() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let mut manifest = create_test_manifest("", "");
        manifest.description = "".into();

        let full_result = validator.validate(&manifest);
        assert!(full_result.findings.len() >= 2);

        let filtered = validator.validate_with_severity_filter(&manifest, ValidationSeverity::Error);
        let all_error_or_above = filtered.findings.iter().all(|f| f.severity.is_error_or_above());
        assert!(all_error_or_above);
    }

    #[test]
    fn test_builtin_rules_count() {
        let rules = builtin_rules();
        assert!(rules.len() >= 8);
    }

    #[test]
    fn test_builtin_rules_have_unique_ids() {
        let rules = builtin_rules();
        let mut ids = std::collections::HashSet::new();
        for rule in &rules {
            assert!(ids.insert(&rule.id), "Duplicate rule ID: {}", rule.id);
        }
        assert_eq!(ids.len(), rules.len());
    }

    #[test]
    fn test_validation_report() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let mut manifests = vec![
            create_test_manifest("plugin-1", "Plugin 1"),
            create_test_manifest("plugin-2", "Plugin 2"),
        ];
        manifests[1].id = PluginId("".into());

        let report = validator.generate_report(&manifests);
        assert_eq!(report.total_plugins, 2);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 1);
        assert!(!report.all_passed());
    }

    #[test]
    fn test_validation_report_all_passed() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let manifests = vec![
            create_test_manifest("plugin-1", "Plugin 1"),
            create_test_manifest("plugin-2", "Plugin 2"),
        ];

        let report = validator.generate_report(&manifests);
        assert_eq!(report.total_plugins, 2);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 0);
        assert!(report.all_passed());
    }

    #[test]
    fn test_validation_report_total_findings_by_severity() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let mut manifests = vec![create_test_manifest("", "")];
        manifests[0].version = "bad".into();

        let report = validator.generate_report(&manifests);
        let totals = report.total_findings_by_severity();

        assert!(totals.contains_key(&ValidationSeverity::Critical));
        assert!(totals.contains_key(&ValidationSeverity::Error));
    }

    #[test]
    fn test_validation_report_filter_by_severity() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        let mut m1 = create_test_manifest("ok-plugin", "OK Plugin");
        m1.description = "".into();

        let mut m2 = create_test_manifest("", "Bad Plugin");
        m2.version = "invalid".into();

        let manifests = vec![m1, m2];
        let report = validator.generate_report(&manifests);

        let critical_results = report.filter_results_by_severity(ValidationSeverity::Critical);
        assert_eq!(critical_results.len(), 1);
    }

    #[test]
    fn test_validator_with_default_rules() {
        let validator = PluginValidator::with_default_rules("1.0.0");
        assert!(validator.rule_count() >= 8);
    }

    #[test]
    fn test_rule_has_all_fields() {
        let rules = builtin_rules();
        for rule in &rules {
            assert!(!rule.id.is_empty());
            assert!(!rule.name.is_empty());
            assert!(!rule.description.is_empty());
        }
    }

    #[test]
    fn test_version_format_rule() {
        let validator = PluginValidator::with_default_rules("1.0.0");

        let good = create_test_manifest("test", "Test");
        assert!(validator.validate(&good).is_valid());

        let mut bad1 = create_test_manifest("test", "Test");
        bad1.version = "1.0".into();
        assert!(!validator.validate(&bad1).is_valid());

        let mut bad2 = create_test_manifest("test", "Test");
        bad2.version = "v1.0.0".into();
        assert!(!validator.validate(&bad2).is_valid());
    }

    #[test]
    fn test_host_version_compatibility() {
        let validator = PluginValidator::with_default_rules("1.5.0");

        let compatible = create_test_manifest("test", "Test");
        let result = validator.validate(&compatible);
        assert!(result.is_valid());

        let exact = create_test_manifest("test", "Test");
        let result2 = validator.validate(&exact);
        assert!(result2.is_valid());

        let mut incompatible = create_test_manifest("test", "Test");
        incompatible.min_host_version = "2.0.0".into();
        let result3 = validator.validate(&incompatible);
        assert!(!result3.is_valid());
        let has_v002 = result3.findings.iter().any(|f| f.rule_id == "V002");
        assert!(has_v002);
    }

    #[test]
    fn test_self_dependency_rule() {
        let validator = PluginValidator::with_default_rules("1.0.0");

        let mut manifest = create_test_manifest("my-plugin", "My Plugin");
        manifest.dependencies = vec![PluginId("my-plugin".into())];
        let result = validator.validate(&manifest);

        let has_self_dep = result.findings.iter().any(|f| f.rule_id == "D001");
        assert!(has_self_dep);
    }

    #[test]
    fn test_all_permission_warning() {
        let validator = PluginValidator::with_default_rules("1.0.0");

        let mut manifest = create_test_manifest("my-plugin", "My Plugin");
        manifest.permissions = vec![Permission::All];
        let result = validator.validate(&manifest);

        let has_all_perm_warning = result.findings.iter().any(|f| f.rule_id == "P001");
        assert!(has_all_perm_warning);
    }

    #[test]
    fn test_naming_convention_rule() {
        let validator = PluginValidator::with_default_rules("1.0.0");

        let bad = create_test_manifest("My_Plugin", "My Plugin");
        let result = validator.validate(&bad);
        let has_naming = result.findings.iter().any(|f| f.rule_id == "N001");
        assert!(has_naming);

        let good = create_test_manifest("my-plugin-123", "My Plugin");
        let result2 = validator.validate(&good);
        let has_naming2 = result2.findings.iter().any(|f| f.rule_id == "N001");
        assert!(!has_naming2);
    }
}
