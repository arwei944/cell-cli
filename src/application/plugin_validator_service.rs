use crate::domain::errors::CellResult;
use crate::domain::plugin_system::PluginManifest;
use crate::domain::plugin_validator::{
    PluginValidator, ValidationReport, ValidationResult, ValidationSeverity,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScanResult {
    pub path: String,
    pub found_plugins: Vec<PluginManifest>,
    pub scanned_at: DateTime<Utc>,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuditResult {
    pub report: ValidationReport,
    pub summary: AuditSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_findings: usize,
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct PluginValidatorService {
    validator: PluginValidator,
}

impl PluginValidatorService {
    pub fn new() -> Self {
        let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
        Self {
            validator: PluginValidator::with_default_rules(version),
        }
    }

    pub fn validate_plugin(&self, path: &str) -> CellResult<ValidationResult> {
        let manifest = Self::load_manifest(path)?;
        Ok(self.validator.validate(&manifest))
    }

    pub fn scan_plugin(&self, path: &str) -> CellResult<PluginScanResult> {
        let start = Utc::now();
        let path_buf = Path::new(path);

        let mut manifests = Vec::new();

        if path_buf.is_file() {
            if path_buf.extension().and_then(|e| e.to_str()) == Some("json") {
                let manifest = Self::load_manifest(path)?;
                manifests.push(manifest);
            }
        } else if path_buf.is_dir() {
            for entry in walkdir::WalkDir::new(path_buf)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let file_path = entry.path();
                if file_path.is_file()
                    && file_path.file_name().and_then(|n| n.to_str())
                        == Some("plugin.json")
                {
                    if let Ok(manifest) = Self::load_manifest(file_path.to_str().unwrap()) {
                        manifests.push(manifest);
                    }
                }
            }
        }

        let duration = (Utc::now() - start).num_milliseconds().max(0) as u64;

        Ok(PluginScanResult {
            path: path.to_string(),
            found_plugins: manifests,
            scanned_at: start,
            scan_duration_ms: duration,
        })
    }

    pub fn audit_plugin(&self, path: &str) -> CellResult<PluginAuditResult> {
        let scan_result = self.scan_plugin(path)?;

        if scan_result.found_plugins.is_empty() {
            return Ok(PluginAuditResult {
                report: ValidationReport::new(Vec::new()),
                summary: AuditSummary {
                    critical_count: 0,
                    error_count: 0,
                    warning_count: 0,
                    info_count: 0,
                    total_findings: 0,
                    risk_level: RiskLevel::Low,
                    recommendations: vec!["No plugins found to audit".to_string()],
                },
            });
        }

        let report = self.validator.generate_report(&scan_result.found_plugins);
        let summary = self.generate_audit_summary(&report);

        Ok(PluginAuditResult { report, summary })
    }

    fn load_manifest(path: &str) -> CellResult<PluginManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest: PluginManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    fn generate_audit_summary(&self, report: &ValidationReport) -> AuditSummary {
        let totals = report.total_findings_by_severity();

        let critical_count = *totals.get(&ValidationSeverity::Critical).unwrap_or(&0);
        let error_count = *totals.get(&ValidationSeverity::Error).unwrap_or(&0);
        let warning_count = *totals.get(&ValidationSeverity::Warning).unwrap_or(&0);
        let info_count = *totals.get(&ValidationSeverity::Info).unwrap_or(&0);
        let total_findings = critical_count + error_count + warning_count + info_count;

        let risk_level = match (critical_count, error_count) {
            (c, _) if c > 0 => RiskLevel::Critical,
            (_, e) if e >= 3 => RiskLevel::High,
            (_, e) if e > 0 => RiskLevel::Medium,
            (_, _) => RiskLevel::Low,
        };

        let mut recommendations = Vec::new();

        if critical_count > 0 {
            recommendations.push(format!("Fix {} Critical issues", critical_count));
        }
        if error_count > 0 {
            recommendations.push(format!("Fix {} Error issues", error_count));
        }
        if warning_count > 0 {
            recommendations.push(format!("Review {} Warning issues", warning_count));
        }

        if recommendations.is_empty() {
            recommendations.push("All plugins passed validation".to_string());
        }

        AuditSummary {
            critical_count,
            error_count,
            warning_count,
            info_count,
            total_findings,
            risk_level,
            recommendations,
        }
    }

    pub fn format_validation_result(&self, result: &ValidationResult) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\nPlugin Validation Result: {} ({})\n",
            result.plugin_name, result.plugin_id.0
        ));
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");

        if result.is_valid() {
            output.push_str("Validation passed\n");
            output.push_str(&format!("Duration: {}ms\n", result.duration_ms));
            return output;
        }

        let counts = result.count_by_severity();
        output.push_str(&format!(
            "Critical: {}\n",
            counts.get(&ValidationSeverity::Critical).unwrap_or(&0)
        ));
        output.push_str(&format!(
            "Error: {}\n",
            counts.get(&ValidationSeverity::Error).unwrap_or(&0)
        ));
        output.push_str(&format!(
            "Warning: {}\n",
            counts.get(&ValidationSeverity::Warning).unwrap_or(&0)
        ));
        output.push_str(&format!(
            "Info: {}\n",
            counts.get(&ValidationSeverity::Info).unwrap_or(&0)
        ));

        output.push_str("\nFindings:\n");
        output.push_str("──────────────────────────────────────────────────────────────\n\n");

        for finding in &result.findings {
            let severity_icon = match finding.severity {
                ValidationSeverity::Critical => "Critical",
                ValidationSeverity::Error => "Error",
                ValidationSeverity::Warning => "Warning",
                ValidationSeverity::Info => "Info",
            };

            output.push_str(&format!(
                "[{}] {} - {}\n",
                severity_icon, finding.rule_id, finding.message
            ));

            if let Some(field) = &finding.field {
                output.push_str(&format!("  Field: {}\n", field));
            }

            if let Some(suggestion) = &finding.suggestion {
                output.push_str(&format!("  Suggestion: {}\n", suggestion));
            }

            output.push_str("\n");
        }

        output.push_str(&format!("Duration: {}ms\n", result.duration_ms));

        output
    }

    pub fn format_scan_result(&self, result: &PluginScanResult) -> String {
        let mut output = String::new();

        output.push_str(&format!("\nPlugin Scan Result: {}\n", result.path));
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!(
            "Found {} plugins\n",
            result.found_plugins.len()
        ));
        output.push_str(&format!("Scan duration: {}ms\n", result.scan_duration_ms));

        if !result.found_plugins.is_empty() {
            output.push_str("\nPlugins:\n");
            output.push_str("──────────────────────────────────────────────────────────────\n\n");

            for (i, manifest) in result.found_plugins.iter().enumerate() {
                let kind_label = manifest.kind.label();
                output.push_str(&format!(
                    "{}. {} ({})\n",
                    i + 1, manifest.name, kind_label
                ));
                output.push_str(&format!("  ID: {}\n", manifest.id.0));
                output.push_str(&format!("  Version: {}\n", manifest.version));
                output.push_str(&format!("  Author: {}\n", manifest.author));

                if !manifest.description.is_empty() {
                    output.push_str(&format!("  Description: {}\n", manifest.description));
                }

                output.push_str("\n");
            }
        }

        output
    }

    pub fn format_audit_result(&self, result: &PluginAuditResult) -> String {
        let mut output = String::new();

        output.push_str("\nPlugin Audit Report\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!(
            "Audited plugins: {} (passed: {}, failed: {})\n",
            result.report.total_plugins, result.report.passed, result.report.failed
        ));

        let risk_label = match result.summary.risk_level {
            RiskLevel::Critical => "Critical",
            RiskLevel::High => "High",
            RiskLevel::Medium => "Medium",
            RiskLevel::Low => "Low",
        };

        output.push_str(&format!("Risk Level: {}\n", risk_label));

        output.push_str(&format!(
            "\nStatistics:\n  Critical: {}\n  Error: {}\n  Warning: {}\n  Info: {}\n  Total: {}\n",
            result.summary.critical_count,
            result.summary.error_count,
            result.summary.warning_count,
            result.summary.info_count,
            result.summary.total_findings
        ));

        output.push_str("\nRecommendations:\n");
        for (i, rec) in result.summary.recommendations.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, rec));
        }

        if !result.report.results.is_empty() {
            output.push_str("\nDetailed Results:\n");
            output.push_str("──────────────────────────────────────────────────────────────\n\n");

            for result_item in &result.report.results {
                let status = if result_item.is_valid() { "PASS" } else { "FAIL" };
                output.push_str(&format!(
                    "[{}] {} ({})\n",
                    status, result_item.plugin_name, result_item.plugin_id.0
                ));

                if !result_item.findings.is_empty() {
                    for finding in &result_item.findings {
                        let severity_label = match finding.severity {
                            ValidationSeverity::Critical => "Critical",
                            ValidationSeverity::Error => "Error",
                            ValidationSeverity::Warning => "Warning",
                            ValidationSeverity::Info => "Info",
                        };
                        output.push_str(&format!(
                            "  [{}] {} - {}\n",
                            severity_label, finding.rule_id, finding.message
                        ));
                    }
                }

                output.push_str("\n");
            }
        }

        output
    }
}

impl Default for PluginValidatorService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plugin_system::{Permission, PluginId, PluginKind, PluginManifest};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

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

    fn write_manifest_to_file(path: &Path, manifest: &PluginManifest) {
        let content = serde_json::to_string_pretty(manifest).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_validate_plugin_valid() {
        let dir = tempdir().unwrap();
        let manifest = create_test_manifest("valid-plugin", "Valid Plugin");
        let manifest_path = dir.path().join("plugin.json");
        write_manifest_to_file(&manifest_path, &manifest);

        let service = PluginValidatorService::new();
        let result = service.validate_plugin(manifest_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(result.unwrap().is_valid());
    }

    #[test]
    fn test_validate_plugin_invalid() {
        let dir = tempdir().unwrap();
        let mut manifest = create_test_manifest("", "Invalid Plugin");
        manifest.id = PluginId("".into());
        let manifest_path = dir.path().join("plugin.json");
        write_manifest_to_file(&manifest_path, &manifest);

        let service = PluginValidatorService::new();
        let result = service.validate_plugin(manifest_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(!result.unwrap().is_valid());
    }

    #[test]
    fn test_scan_plugin_directory() {
        let dir = tempdir().unwrap();

        let manifest1 = create_test_manifest("plugin-1", "Plugin 1");
        let subdir1 = dir.path().join("subdir1");
        std::fs::create_dir(&subdir1).unwrap();
        write_manifest_to_file(&subdir1.join("plugin.json"), &manifest1);

        let manifest2 = create_test_manifest("plugin-2", "Plugin 2");
        let subdir2 = dir.path().join("subdir2");
        std::fs::create_dir(&subdir2).unwrap();
        write_manifest_to_file(&subdir2.join("plugin.json"), &manifest2);

        let service = PluginValidatorService::new();
        let result = service.scan_plugin(dir.path().to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().found_plugins.len(), 2);
    }

    #[test]
    fn test_scan_plugin_file() {
        let dir = tempdir().unwrap();
        let manifest = create_test_manifest("single-plugin", "Single Plugin");
        let manifest_path = dir.path().join("plugin.json");
        write_manifest_to_file(&manifest_path, &manifest);

        let service = PluginValidatorService::new();
        let result = service.scan_plugin(manifest_path.to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().found_plugins.len(), 1);
    }

    #[test]
    fn test_audit_plugin() {
        let dir = tempdir().unwrap();

        let mut bad_manifest = create_test_manifest("", "Bad Plugin");
        bad_manifest.id = PluginId("".into());
        bad_manifest.version = "invalid".into();
        write_manifest_to_file(&dir.path().join("bad_plugin.json"), &bad_manifest);

        let good_manifest = create_test_manifest("good-plugin", "Good Plugin");
        write_manifest_to_file(&dir.path().join("good_plugin.json"), &good_manifest);

        let service = PluginValidatorService::new();
        let result = service.audit_plugin(dir.path().to_str().unwrap());

        assert!(result.is_ok());
        let audit_result = result.unwrap();
        assert_eq!(audit_result.report.total_plugins, 2);
        assert_eq!(audit_result.report.passed, 1);
        assert_eq!(audit_result.report.failed, 1);
    }

    #[test]
    fn test_format_validation_result() {
        let service = PluginValidatorService::new();
        let manifest = create_test_manifest("test-plugin", "Test Plugin");

        let result = service.validator.validate(&manifest);
        let formatted = service.format_validation_result(&result);

        assert!(formatted.contains("Plugin Validation Result"));
        assert!(formatted.contains("Validation passed"));
    }

    #[test]
    fn test_format_scan_result() {
        let service = PluginValidatorService::new();
        let result = PluginScanResult {
            path: "/test/path".to_string(),
            found_plugins: vec![create_test_manifest("test", "Test")],
            scanned_at: Utc::now(),
            scan_duration_ms: 10,
        };

        let formatted = service.format_scan_result(&result);
        assert!(formatted.contains("Plugin Scan Result"));
        assert!(formatted.contains("Found 1 plugins"));
    }

    #[test]
    fn test_format_audit_result() {
        let service = PluginValidatorService::new();
        let report = ValidationReport::new(Vec::new());
        let summary = service.generate_audit_summary(&report);

        let result = PluginAuditResult { report, summary };
        let formatted = service.format_audit_result(&result);

        assert!(formatted.contains("Plugin Audit Report"));
    }
}
