use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureRules {
    pub layers: Vec<LayerRule>,
    pub dependency_direction: DependencyDirection,
    pub banned_dependencies: Vec<BannedDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerRule {
    pub name: String,
    pub path_pattern: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyDirection {
    InwardOnly,
    OutwardOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedDependency {
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub layer_stats: HashMap<String, LayerStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub from_module: String,
    pub to_module: String,
    pub file: String,
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayerStats {
    pub file_count: usize,
    pub internal_deps: usize,
    pub external_deps: usize,
    pub violations: usize,
}

impl Default for ArchitectureRules {
    fn default() -> Self {
        ArchitectureRules {
            layers: vec![
                LayerRule {
                    name: "domain".to_string(),
                    path_pattern: "src/domain".to_string(),
                    description: "领域内核层，纯业务逻辑，无外部依赖".to_string(),
                },
                LayerRule {
                    name: "application".to_string(),
                    path_pattern: "src/application".to_string(),
                    description: "应用服务层，编排业务流程，依赖domain".to_string(),
                },
                LayerRule {
                    name: "adapters".to_string(),
                    path_pattern: "src/adapters".to_string(),
                    description: "适配器层，实现Port接口，依赖application/domain".to_string(),
                },
                LayerRule {
                    name: "interfaces".to_string(),
                    path_pattern: "src/interfaces".to_string(),
                    description: "接口层，CLI/API等入口，依赖application".to_string(),
                },
            ],
            dependency_direction: DependencyDirection::InwardOnly,
            banned_dependencies: vec![
                BannedDependency {
                    from: "domain".to_string(),
                    to: "application".to_string(),
                    reason: "领域层不能依赖应用层".to_string(),
                },
                BannedDependency {
                    from: "domain".to_string(),
                    to: "adapters".to_string(),
                    reason: "领域层不能依赖适配器层".to_string(),
                },
                BannedDependency {
                    from: "domain".to_string(),
                    to: "interfaces".to_string(),
                    reason: "领域层不能依赖接口层".to_string(),
                },
                BannedDependency {
                    from: "application".to_string(),
                    to: "interfaces".to_string(),
                    reason: "应用层不能依赖接口层".to_string(),
                },
                BannedDependency {
                    from: "application".to_string(),
                    to: "adapters".to_string(),
                    reason: "应用层不能直接依赖适配器，必须通过Port".to_string(),
                },
                BannedDependency {
                    from: "adapters".to_string(),
                    to: "interfaces".to_string(),
                    reason: "适配器层不能依赖接口层".to_string(),
                },
            ],
        }
    }
}

pub fn validate_architecture(root: &Path, rules: &ArchitectureRules) -> ValidationResult {
    let mut violations = Vec::new();
    let mut layer_stats: HashMap<String, LayerStats> = HashMap::new();

    for layer in &rules.layers {
        layer_stats.insert(layer.name.clone(), LayerStats::default());
    }

    let module_deps = collect_module_dependencies(root);

    for (module_path, deps) in &module_deps {
        let from_layer = detect_layer(module_path, rules);

        if let Some(ref from_layer_name) = from_layer
            && let Some(stats) = layer_stats.get_mut(from_layer_name)
        {
            stats.file_count += 1;
        }

        for (dep_path, dep_info) in deps {
            let to_layer = detect_layer(dep_path, rules);

            if let (Some(from_name), Some(to_name)) = (&from_layer, &to_layer) {
                if let Some(stats) = layer_stats.get_mut(from_name) {
                    if from_name == to_name {
                        stats.internal_deps += 1;
                    } else {
                        stats.external_deps += 1;
                    }
                }

                for banned in &rules.banned_dependencies {
                    if banned.from == *from_name && banned.to == *to_name {
                        if let Some(stats) = layer_stats.get_mut(from_name) {
                            stats.violations += 1;
                        }
                        violations.push(Violation {
                            rule: "layer_dependency".to_string(),
                            from_module: module_path.clone(),
                            to_module: dep_path.clone(),
                            file: dep_info.file.clone(),
                            line: dep_info.line,
                            severity: Severity::Error,
                            message: format!("{} → {}: {}", from_name, to_name, banned.reason),
                        });
                    }
                }
            }
        }
    }

    let error_count = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();

    ValidationResult {
        passed: error_count == 0,
        violations,
        layer_stats,
    }
}

fn detect_layer(module_path: &str, rules: &ArchitectureRules) -> Option<String> {
    for layer in &rules.layers {
        let pattern = layer.path_pattern.replace("src/", "");
        if module_path.starts_with(&format!("{}::", pattern))
            || module_path == pattern
            || module_path.starts_with(&format!("crate::{}::", pattern))
        {
            return Some(layer.name.clone());
        }
    }
    None
}

#[derive(Debug, Clone)]
struct DepInfo {
    file: String,
    line: usize,
}

fn collect_module_dependencies(root: &Path) -> HashMap<String, HashMap<String, DepInfo>> {
    let mut result = HashMap::new();
    let src_path = root.join("src");

    if !src_path.exists() {
        return result;
    }

    collect_rs_files(&src_path, &src_path, &mut result);
    result
}

fn collect_rs_files(
    dir: &Path,
    base: &Path,
    result: &mut HashMap<String, HashMap<String, DepInfo>>,
) {
    use std::fs;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, base, result);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let module_path = path_to_module(&path, base);
                let deps = parse_file_deps(&path);
                result.insert(module_path, deps);
            }
        }
    }
}

fn path_to_module(path: &Path, base: &Path) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let without_ext = rel_str.trim_end_matches(".rs");

    let mod_path = if without_ext.ends_with("/mod") {
        without_ext.trim_end_matches("/mod").to_string()
    } else if without_ext == "mod" {
        "lib".to_string()
    } else {
        without_ext.to_string()
    };

    mod_path.replace('/', "::").replace("lib::", "")
}

fn parse_file_deps(path: &Path) -> HashMap<String, DepInfo> {
    let mut deps = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("use crate::") || trimmed.starts_with("pub use crate::") {
                let rest = trimmed
                    .trim_start_matches("pub use ")
                    .trim_start_matches("use ");

                let dep_module = rest
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .split(" as ")
                    .next()
                    .unwrap_or("")
                    .trim_end_matches("::*")
                    .trim_end_matches('{')
                    .trim();

                let clean = dep_module.trim_end_matches("::");

                if !clean.is_empty() && clean != "crate" {
                    deps.insert(
                        clean.to_string(),
                        DepInfo {
                            file: path.to_string_lossy().to_string(),
                            line: line_num + 1,
                        },
                    );
                }
            }

            if trimmed.starts_with("use super::") {
                let dep_module = trimmed
                    .trim_start_matches("use ")
                    .split(';')
                    .next()
                    .unwrap_or("");

                deps.insert(
                    format!("super::{}", dep_module),
                    DepInfo {
                        file: path.to_string_lossy().to_string(),
                        line: line_num + 1,
                    },
                );
            }
        }
    }

    deps
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");

        fs::create_dir_all(src.join("domain")).unwrap();
        fs::create_dir_all(src.join("application")).unwrap();
        fs::create_dir_all(src.join("adapters")).unwrap();
        fs::create_dir_all(src.join("interfaces")).unwrap();

        fs::write(
            src.join("domain").join("mod.rs"),
            r#"
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
"#,
        )
        .unwrap();

        fs::write(
            src.join("application").join("mod.rs"),
            r#"
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
"#,
        )
        .unwrap();

        fs::write(
            src.join("adapters").join("mod.rs"),
            r#"
use crate::application::Service;
pub struct Adapter;
impl Adapter {
    pub fn run() { Service::do_work(); }
}
"#,
        )
        .unwrap();

        fs::write(
            src.join("interfaces").join("cli.rs"),
            r#"
use crate::application::Service;
pub fn run() { Service::do_work(); }
"#,
        )
        .unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"
pub mod domain;
pub mod application;
pub mod adapters;
pub mod interfaces;
"#,
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_valid_architecture_passes() {
        let dir = setup_test_project();
        let rules = ArchitectureRules::default();
        let result = validate_architecture(dir.path(), &rules);
        assert!(
            result.passed,
            "Expected valid arch to pass: {:?}",
            result.violations
        );
    }

    #[test]
    fn test_domain_depends_on_application_fails() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r#"
use crate::application::Service;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
"#,
        )
        .unwrap();

        let rules = ArchitectureRules::default();
        let result = validate_architecture(dir.path(), &rules);
        assert!(!result.passed);
        assert!(
            result
                .violations
                .iter()
                .any(|v| v.from_module.contains("domain"))
        );
    }

    #[test]
    fn test_layer_stats_collected() {
        let dir = setup_test_project();
        let rules = ArchitectureRules::default();
        let result = validate_architecture(dir.path(), &rules);

        assert!(result.layer_stats.contains_key("domain"));
        assert!(result.layer_stats.contains_key("application"));
        assert_eq!(result.layer_stats.get("domain").unwrap().file_count, 1);
    }
}
