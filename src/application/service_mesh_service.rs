use crate::domain::errors::CellResult;
use crate::domain::service_mesh::MeshGenerator;
use chrono::{DateTime, Local};
use difflib::unified_diff;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IstioConfig {
    pub name: String,
    pub namespace: String,
    pub versions: Vec<String>,
    pub gateway_name: Option<String>,
    pub yaml: String,
    pub generated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub kind: Option<String>,
    pub name: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub added_lines: usize,
    pub removed_lines: usize,
    pub modified_lines: usize,
    pub diff: String,
}

pub struct ServiceMeshService;

impl ServiceMeshService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_istio_config(&self, name: &str) -> CellResult<IstioConfig> {
        self.generate_istio_config_with_options(name, "default", &["v1"], None)
    }

    pub fn generate_istio_config_with_options(
        &self,
        name: &str,
        namespace: &str,
        versions: &[&str],
        gateway_name: Option<&str>,
    ) -> CellResult<IstioConfig> {
        let versions_str: Vec<String> = versions.iter().map(|s| s.to_string()).collect();
        let generator = MeshGenerator::new();
        let config = generator.generate_complete_config(name, namespace, &versions_str, gateway_name.map(|s| s.to_string()));
        let yaml = config.to_yaml();
        
        Ok(IstioConfig {
            name: name.to_string(),
            namespace: namespace.to_string(),
            versions: versions_str,
            gateway_name: gateway_name.map(|s| s.to_string()),
            yaml,
            generated_at: Local::now(),
        })
    }

    pub fn validate_istio_config(&self, path: &str) -> CellResult<ValidationResult> {
        let content = fs::read_to_string(path)?;
        self.validate_istio_config_content(&content)
    }

    pub fn validate_istio_config_content(&self, content: &str) -> CellResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut kind: Option<String> = None;
        let mut name: Option<String> = None;
        let mut namespace: Option<String> = None;

        if content.trim().is_empty() {
            errors.push("配置内容为空".to_string());
            return Ok(ValidationResult { valid: false, errors, warnings, kind, name, namespace });
        }

        let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| {
                errors.push(format!("YAML 解析错误: {}", e));
                crate::domain::errors::CellError::Yaml(e)
            })?;

        let docs: Vec<serde_yaml::Value> = match yaml_value {
            serde_yaml::Value::Sequence(seq) => seq,
            _ => vec![yaml_value],
        };

        for doc in docs {
            if let serde_yaml::Value::Mapping(m) = doc {
                for (k, v) in m {
                    let key = k.as_str().unwrap_or("");
                    match key {
                        "kind" => {
                            kind = v.as_str().map(|s| s.to_string());
                            let k = kind.as_deref().unwrap_or("");
                            let valid_kinds = ["VirtualService", "DestinationRule", "Gateway", "Sidecar", "ServiceEntry"];
                            if !valid_kinds.contains(&k) {
                                warnings.push(format!("未知的 Istio 资源类型: {}", k));
                            }
                        }
                        "metadata" => {
                            if let serde_yaml::Value::Mapping(meta) = v {
                                for (mk, mv) in meta {
                                    let mkey = mk.as_str().unwrap_or("");
                                    match mkey {
                                        "name" => name = mv.as_str().map(|s| s.to_string()),
                                        "namespace" => namespace = mv.as_str().map(|s| s.to_string()),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "spec" => {
                            if let serde_yaml::Value::Null = v {
                                errors.push("spec 字段为空".to_string());
                            }
                        }
                        _ => {}
                    }
                }

                if kind.is_none() {
                    errors.push("缺少 kind 字段".to_string());
                }
                if name.is_none() {
                    warnings.push("缺少 metadata.name 字段".to_string());
                }
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            kind,
            name,
            namespace,
        })
    }

    pub fn diff_configs(&self, old_path: &str, new_path: &str) -> CellResult<ConfigDiff> {
        let old_content = fs::read_to_string(old_path)?;
        let new_content = fs::read_to_string(new_path)?;
        self.diff_configs_content(&old_content, &new_path, &new_content, &old_path)
    }

    pub fn diff_configs_content(&self, old: &str, old_name: &str, new: &str, new_name: &str) -> CellResult<ConfigDiff> {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();
        
        let diff = unified_diff(
            &old_lines,
            &new_lines,
            old_name,
            new_name,
            "",
            "",
            3,
        );
        
        let diff_str = diff.join("\n");
        
        let mut added_lines = 0;
        let mut removed_lines = 0;
        let mut modified_lines = 0;
        
        for line in diff_str.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                added_lines += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                removed_lines += 1;
            } else if line.starts_with('!') {
                modified_lines += 1;
            }
        }
        
        Ok(ConfigDiff {
            added_lines,
            removed_lines,
            modified_lines,
            diff: diff_str,
        })
    }

    pub fn format_validation_result(&self, result: &ValidationResult) -> String {
        let mut output = String::new();
        
        if result.valid {
            output.push_str("\n✅ 配置验证通过\n");
        } else {
            output.push_str("\n❌ 配置验证失败\n");
        }
        
        if let Some(kind) = &result.kind {
            output.push_str(&format!("资源类型: {}\n", kind));
        }
        if let Some(name) = &result.name {
            output.push_str(&format!("名称: {}\n", name));
        }
        if let Some(namespace) = &result.namespace {
            output.push_str(&format!("命名空间: {}\n", namespace));
        }
        
        if !result.errors.is_empty() {
            output.push_str(&format!("\n错误 ({})：\n", result.errors.len()));
            for (i, e) in result.errors.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, e));
            }
        }
        
        if !result.warnings.is_empty() {
            output.push_str(&format!("\n警告 ({})：\n", result.warnings.len()));
            for (i, w) in result.warnings.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, w));
            }
        }
        
        output
    }

    pub fn format_diff(&self, diff: &ConfigDiff) -> String {
        let mut output = String::new();
        
        output.push_str("\n📊 配置差异分析\n");
        output.push_str(&format!("添加行数: {}\n", diff.added_lines));
        output.push_str(&format!("删除行数: {}\n", diff.removed_lines));
        output.push_str(&format!("修改行数: {}\n", diff.modified_lines));
        
        if !diff.diff.is_empty() {
            output.push_str("\n差异详情:\n");
            output.push_str("───────────────────────────────────────────────\n");
            output.push_str(&diff.diff);
            output.push_str("\n───────────────────────────────────────────────\n");
        }
        
        output
    }
}

impl Default for ServiceMeshService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_istio_config_default() {
        let service = ServiceMeshService::new();
        let config = service.generate_istio_config("test-service").unwrap();
        
        assert_eq!(config.name, "test-service");
        assert_eq!(config.namespace, "default");
        assert_eq!(config.versions, vec!["v1"]);
        assert!(config.yaml.contains("VirtualService"));
        assert!(config.yaml.contains("DestinationRule"));
        assert!(config.yaml.contains("Sidecar"));
    }

    #[test]
    fn test_generate_istio_config_with_multiple_versions() {
        let service = ServiceMeshService::new();
        let config = service.generate_istio_config_with_options("myapp", "prod", &["v1", "v2", "v3"], Some("myapp-gw")).unwrap();
        
        assert_eq!(config.name, "myapp");
        assert_eq!(config.namespace, "prod");
        assert_eq!(config.versions, vec!["v1", "v2", "v3"]);
        assert_eq!(config.gateway_name, Some("myapp-gw".to_string()));
        assert!(config.yaml.contains("Gateway"));
    }

    #[test]
    fn test_validate_valid_config() {
        let service = ServiceMeshService::new();
        let valid_yaml = r#"
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: test-vs
  namespace: default
spec:
  hosts:
    - test-service.default.svc.cluster.local
  http:
    - route:
        - destination:
            host: test-service.default.svc.cluster.local
            weight: 100
"#;
        
        let result = service.validate_istio_config_content(valid_yaml).unwrap();
        
        assert!(result.valid);
        assert_eq!(result.kind, Some("VirtualService".to_string()));
        assert_eq!(result.name, Some("test-vs".to_string()));
        assert_eq!(result.namespace, Some("default".to_string()));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_config() {
        let service = ServiceMeshService::new();
        let invalid_yaml = r#"
apiVersion: networking.istio.io/v1alpha3
kind: InvalidKind
metadata:
  name: test-vs
spec: null
"#;
        
        let result = service.validate_istio_config_content(invalid_yaml).unwrap();
        
        assert!(!result.valid);
        assert!(result.warnings.contains(&"未知的 Istio 资源类型: InvalidKind".to_string()));
        assert!(result.errors.contains(&"spec 字段为空".to_string()));
    }

    #[test]
    fn test_diff_configs() {
        let service = ServiceMeshService::new();
        let old_content = "apiVersion: v1\nkind: Service\nmetadata:\n  name: test\n";
        let new_content = "apiVersion: v1\nkind: Service\nmetadata:\n  name: test\n  namespace: default\n";
        
        let diff = service.diff_configs_content(old_content, "old.yaml", new_content, "new.yaml").unwrap();
        
        assert_eq!(diff.added_lines, 1);
        assert_eq!(diff.removed_lines, 0);
        assert!(diff.diff.contains("+  namespace: default"));
    }
}