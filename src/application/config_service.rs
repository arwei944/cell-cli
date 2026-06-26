use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub architecture: ArchitectureConfig,
    #[serde(default)]
    pub entropy: EntropyConfig,
    #[serde(default)]
    pub quality: QualityConfig,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub language: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            version: default_version(),
            description: String::new(),
            language: "rust".to_string(),
        }
    }
}

fn default_name() -> String {
    "cell-project".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureConfig {
    #[serde(default = "default_layers")]
    pub layers: Vec<String>,
    #[serde(default = "default_strict_mode")]
    pub strict_mode: bool,
    #[serde(default)]
    pub allowed_cross_layer: Vec<String>,
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        Self {
            layers: default_layers(),
            strict_mode: default_strict_mode(),
            allowed_cross_layer: Vec::new(),
        }
    }
}

fn default_layers() -> Vec<String> {
    vec![
        "domain".to_string(),
        "application".to_string(),
        "adapters".to_string(),
        "interfaces".to_string(),
    ]
}

fn default_strict_mode() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyConfig {
    #[serde(default = "default_entropy_threshold")]
    pub warning_threshold: f64,
    #[serde(default = "default_entropy_critical")]
    pub critical_threshold: f64,
    #[serde(default = "default_entropy_history")]
    pub keep_history_days: u32,
    #[serde(default)]
    pub dimension_weights: DimensionWeightsConfig,
    #[serde(default)]
    pub dimension_thresholds: DimensionThresholdsConfig,
    #[serde(default)]
    pub exclude_paths: Vec<String>,
    #[serde(default)]
    pub complexity: ComplexityThresholdsConfig,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            warning_threshold: default_entropy_threshold(),
            critical_threshold: default_entropy_critical(),
            keep_history_days: default_entropy_history(),
            dimension_weights: DimensionWeightsConfig::default(),
            dimension_thresholds: DimensionThresholdsConfig::default(),
            exclude_paths: Vec::new(),
            complexity: ComplexityThresholdsConfig::default(),
        }
    }
}

fn default_entropy_threshold() -> f64 {
    0.7
}

fn default_entropy_critical() -> f64 {
    0.85
}

fn default_entropy_history() -> u32 {
    90
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionWeightsConfig {
    #[serde(default = "default_structural_weight")]
    pub structural: f64,
    #[serde(default = "default_complexity_weight")]
    pub complexity: f64,
    #[serde(default = "default_coupling_weight")]
    pub coupling: f64,
    #[serde(default = "default_naming_weight")]
    pub naming: f64,
    #[serde(default = "default_test_weight")]
    pub test: f64,
}

impl Default for DimensionWeightsConfig {
    fn default() -> Self {
        Self {
            structural: default_structural_weight(),
            complexity: default_complexity_weight(),
            coupling: default_coupling_weight(),
            naming: default_naming_weight(),
            test: default_test_weight(),
        }
    }
}

fn default_structural_weight() -> f64 { 0.25 }
fn default_complexity_weight() -> f64 { 0.25 }
fn default_coupling_weight() -> f64 { 0.20 }
fn default_naming_weight() -> f64 { 0.15 }
fn default_test_weight() -> f64 { 0.15 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionThresholdsConfig {
    pub structural_warning: Option<f64>,
    pub structural_critical: Option<f64>,
    pub complexity_warning: Option<f64>,
    pub complexity_critical: Option<f64>,
    pub coupling_warning: Option<f64>,
    pub coupling_critical: Option<f64>,
    pub naming_warning: Option<f64>,
    pub naming_critical: Option<f64>,
    pub test_warning: Option<f64>,
    pub test_critical: Option<f64>,
}

impl Default for DimensionThresholdsConfig {
    fn default() -> Self {
        Self {
            structural_warning: None,
            structural_critical: None,
            complexity_warning: None,
            complexity_critical: None,
            coupling_warning: None,
            coupling_critical: None,
            naming_warning: None,
            naming_critical: None,
            test_warning: None,
            test_critical: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityThresholdsConfig {
    #[serde(default = "default_entropy_max_function_lines")]
    pub max_function_lines: usize,
    #[serde(default = "default_entropy_max_file_lines")]
    pub max_file_lines: usize,
    #[serde(default = "default_entropy_max_nesting_depth")]
    pub max_nesting_depth: usize,
    #[serde(default = "default_entropy_max_cyclomatic_complexity")]
    pub max_cyclomatic_complexity: usize,
}

impl Default for ComplexityThresholdsConfig {
    fn default() -> Self {
        Self {
            max_function_lines: default_entropy_max_function_lines(),
            max_file_lines: default_entropy_max_file_lines(),
            max_nesting_depth: default_entropy_max_nesting_depth(),
            max_cyclomatic_complexity: default_entropy_max_cyclomatic_complexity(),
        }
    }
}

fn default_entropy_max_nesting_depth() -> usize { 5 }
fn default_entropy_max_cyclomatic_complexity() -> usize { 10 }
fn default_entropy_max_function_lines() -> usize { 50 }
fn default_entropy_max_file_lines() -> usize { 500 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    #[serde(default = "default_min_test_coverage")]
    pub min_test_coverage: f64,
    #[serde(default = "default_max_file_lines")]
    pub max_file_lines: usize,
    #[serde(default = "default_max_function_lines")]
    pub max_function_lines: usize,
    #[serde(default = "default_max_cyclomatic_complexity")]
    pub max_cyclomatic_complexity: usize,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            min_test_coverage: default_min_test_coverage(),
            max_file_lines: default_max_file_lines(),
            max_function_lines: default_max_function_lines(),
            max_cyclomatic_complexity: default_max_cyclomatic_complexity(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CellConfig {
    pub fn validate(&self) -> ConfigValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if self.project.name.is_empty() {
            errors.push("project.name 不能为空".to_string());
        }

        if self.project.version.is_empty() {
            errors.push("project.version 不能为空".to_string());
        } else if !semver_like(&self.project.version) {
            warnings.push("project.version 建议使用语义化版本格式 (如 0.1.0)".to_string());
        }

        if self.entropy.warning_threshold < 0.0 || self.entropy.warning_threshold > 1.0 {
            errors.push("entropy.warning_threshold 必须在 0.0 到 1.0 之间".to_string());
        }

        if self.entropy.critical_threshold < 0.0 || self.entropy.critical_threshold > 1.0 {
            errors.push("entropy.critical_threshold 必须在 0.0 到 1.0 之间".to_string());
        }

        if self.entropy.warning_threshold >= self.entropy.critical_threshold {
            errors.push("entropy.warning_threshold 必须小于 entropy.critical_threshold".to_string());
        }

        if self.quality.min_test_coverage < 0.0 || self.quality.min_test_coverage > 100.0 {
            errors.push("quality.min_test_coverage 必须在 0 到 100 之间".to_string());
        }

        if self.quality.max_file_lines == 0 {
            errors.push("quality.max_file_lines 不能为 0".to_string());
        }

        if self.quality.max_function_lines == 0 {
            errors.push("quality.max_function_lines 不能为 0".to_string());
        }

        if self.architecture.layers.is_empty() {
            warnings.push("architecture.layers 为空，分层检查将被跳过".to_string());
        }

        ConfigValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

fn semver_like(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

fn default_min_test_coverage() -> f64 {
    50.0
}

fn default_max_file_lines() -> usize {
    500
}

fn default_max_function_lines() -> usize {
    50
}

fn default_max_cyclomatic_complexity() -> usize {
    10
}

pub struct ConfigService {
    root: String,
}

impl ConfigService {
    pub fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
        }
    }

    pub fn config_path(&self) -> String {
        Path::new(&self.root).join("cell.toml").to_string_lossy().to_string()
    }

    pub fn load(&self) -> CellResult<CellConfig> {
        let path = self.config_path();
        let path = Path::new(&path);

        if !path.exists() {
            return Ok(CellConfig::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::domain::errors::CellError::Io(e))?;

        let config: CellConfig = toml::from_str(&content)
            .map_err(|e| crate::domain::errors::CellError::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn save(&self, config: &CellConfig) -> CellResult<()> {
        let path = self.config_path();
        let content = toml::to_string_pretty(config)
            .map_err(|e| crate::domain::errors::CellError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&path, content)
            .map_err(|e| crate::domain::errors::CellError::Io(e))?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> CellResult<Option<String>> {
        let config = self.load()?;
        Ok(self.get_value(&config, key))
    }

    pub fn set(&self, key: &str, value: &str) -> CellResult<()> {
        let mut config = self.load()?;
        self.set_value(&mut config, key, value)?;
        self.save(&config)
    }

    fn get_value(&self, config: &CellConfig, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "project" => self.get_nested_value(&config.project, &parts[1..]),
            "architecture" => self.get_nested_value(&config.architecture, &parts[1..]),
            "entropy" => self.get_nested_value(&config.entropy, &parts[1..]),
            "quality" => self.get_nested_value(&config.quality, &parts[1..]),
            "custom" => {
                if parts.len() > 1 {
                    config.custom.get(parts[1]).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn get_nested_value<T: serde::Serialize>(&self, obj: &T, path: &[&str]) -> Option<String> {
        if path.is_empty() {
            return None;
        }

        let value = serde_json::to_value(obj).ok()?;
        let mut current = &value;

        for key in path {
            current = current.get(*key)?;
        }

        match current {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            serde_json::Value::Array(arr) => Some(format!("[{}]", arr.iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
                .join(", "))),
            _ => None,
        }
    }

    fn set_value(&self, config: &mut CellConfig, key: &str, value: &str) -> CellResult<()> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() < 2 {
            return Err(crate::domain::errors::CellError::Config(
                "Invalid key format. Use: section.key".to_string()
            ));
        }

        let section = parts[0];
        let field = parts[1];

        let mut config_value = serde_json::to_value(&*config)
            .map_err(|e| crate::domain::errors::CellError::Config(format!("{}", e)))?;

        if let Some(obj) = config_value.get_mut(section).and_then(|v| v.as_object_mut()) {
            obj.insert(field.to_string(), serde_json::Value::String(value.to_string()));

            *config = serde_json::from_value(config_value)
                .map_err(|e| crate::domain::errors::CellError::Config(format!("Invalid value: {}", e)))?;

            return Ok(());
        }

        Err(crate::domain::errors::CellError::Config(
            format!("Unknown section: {}", section)
        ))
    }

    pub fn format_show(&self, config: &CellConfig) -> String {
        let mut output = String::new();

        output.push_str("\n⚙️  项目配置\n\n");

        output.push_str("  ┌─────────────────────────────────────────────────────┐\n");
        output.push_str("  │  项目信息                                           │\n");
        output.push_str("  ├─────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("  │  名称:       {:<37}│\n", config.project.name));
        output.push_str(&format!("  │  版本:       {:<37}│\n", config.project.version));
        output.push_str(&format!("  │  语言:       {:<37}│\n", config.project.language));
        let desc = if config.project.description.is_empty() { "(未设置)" } else { &config.project.description };
        output.push_str(&format!("  │  描述:       {:<37}│\n", desc));
        output.push_str("  └─────────────────────────────────────────────────────┘\n\n");

        output.push_str("  ┌─────────────────────────────────────────────────────┐\n");
        output.push_str("  │  架构规则                                           │\n");
        output.push_str("  ├─────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("  │  分层:       {:<37}│\n", config.architecture.layers.join(" → ")));
        output.push_str(&format!("  │  严格模式:   {:<37}│\n", if config.architecture.strict_mode { "开启" } else { "关闭" }));
        output.push_str("  └─────────────────────────────────────────────────────┘\n\n");

        output.push_str("  ┌─────────────────────────────────────────────────────┐\n");
        output.push_str("  │  熵值阈值                                           │\n");
        output.push_str("  ├─────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("  │  警告阈值:   {:<37.2}│\n", config.entropy.warning_threshold));
        output.push_str(&format!("  │  严重阈值:   {:<37.2}│\n", config.entropy.critical_threshold));
        output.push_str(&format!("  │  历史保留:   {:<33}天  │\n", config.entropy.keep_history_days));
        output.push_str("  └─────────────────────────────────────────────────────┘\n\n");

        output.push_str("  ┌─────────────────────────────────────────────────────┐\n");
        output.push_str("  │  代码质量                                           │\n");
        output.push_str("  ├─────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("  │  最低测试覆盖率: {:<33.1}% │\n", config.quality.min_test_coverage));
        output.push_str(&format!("  │  最大文件行数: {:<33}行 │\n", config.quality.max_file_lines));
        output.push_str(&format!("  │  最大函数行数: {:<33}行 │\n", config.quality.max_function_lines));
        output.push_str(&format!("  │  最大圈复杂度: {:<33}   │\n", config.quality.max_cyclomatic_complexity));
        output.push_str("  └─────────────────────────────────────────────────────┘\n\n");

        if !config.custom.is_empty() {
            output.push_str("  自定义配置:\n");
            for (k, v) in &config.custom {
                output.push_str(&format!("    {} = {}\n", k, v));
            }
            output.push('\n');
        }

        output.push_str(&format!("  📄 配置文件: {}\n", self.config_path()));

        output
    }
}
