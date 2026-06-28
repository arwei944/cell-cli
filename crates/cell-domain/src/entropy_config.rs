use serde::{Deserialize, Serialize};
use crate::entropy::DimensionWeights;

/// entropy.yaml 配置文件的数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct EntropyConfig {
    /// 五维权重配置
    #[serde(default = "default_weights")]
    pub weights: DimensionWeightsConfig,

    /// 阈值配置
    #[serde(default = "default_thresholds")]
    pub thresholds: ThresholdConfig,

    /// 忽略配置
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionWeightsConfig {
    #[serde(default = "default_structural")]
    pub structural: f64,
    #[serde(default = "default_complexity")]
    pub complexity: f64,
    #[serde(default = "default_coupling")]
    pub coupling: f64,
    #[serde(default = "default_naming")]
    pub naming: f64,
    #[serde(default = "default_test")]
    pub test: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// 总熵值阈值（0-100）
    #[serde(default = "default_total_threshold")]
    pub total: f64,

    /// 单维度阈值
    #[serde(default)]
    pub per_dimension: PerDimensionThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerDimensionThreshold {
    #[serde(default = "default_structural_threshold")]
    pub structural: f64,
    #[serde(default = "default_complexity_threshold")]
    pub complexity: f64,
    #[serde(default = "default_coupling_threshold")]
    pub coupling: f64,
    #[serde(default = "default_naming_threshold")]
    pub naming: f64,
    #[serde(default = "default_test_threshold")]
    pub test: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    /// 忽略的文件模式（glob）
    #[serde(default)]
    pub files: Vec<String>,

    /// 忽略的目录
    #[serde(default)]
    pub directories: Vec<String>,
}

// ---- 默认值函数 ----

fn default_weights() -> DimensionWeightsConfig {
    DimensionWeightsConfig {
        structural: 0.25,
        complexity: 0.25,
        coupling: 0.20,
        naming: 0.15,
        test: 0.15,
    }
}

fn default_thresholds() -> ThresholdConfig {
    ThresholdConfig {
        total: 50.0,
        per_dimension: PerDimensionThreshold::default(),
    }
}

fn default_structural() -> f64 { 0.25 }
fn default_complexity() -> f64 { 0.25 }
fn default_coupling() -> f64 { 0.20 }
fn default_naming() -> f64 { 0.15 }
fn default_test() -> f64 { 0.15 }

fn default_total_threshold() -> f64 { 50.0 }
fn default_structural_threshold() -> f64 { 30.0 }
fn default_complexity_threshold() -> f64 { 30.0 }
fn default_coupling_threshold() -> f64 { 30.0 }
fn default_naming_threshold() -> f64 { 30.0 }
fn default_test_threshold() -> f64 { 30.0 }


impl Default for DimensionWeightsConfig {
    fn default() -> Self {
        default_weights()
    }
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        default_thresholds()
    }
}

impl Default for PerDimensionThreshold {
    fn default() -> Self {
        Self {
            structural: 30.0,
            complexity: 30.0,
            coupling: 30.0,
            naming: 30.0,
            test: 30.0,
        }
    }
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            files: vec![],
            directories: vec!["target".to_string(), ".git".to_string(), "node_modules".to_string()],
        }
    }
}

impl EntropyConfig {
    /// 转换为 domain 层的 `DimensionWeights`
    pub fn to_dimension_weights(&self) -> DimensionWeights {
        DimensionWeights {
            structural: self.weights.structural,
            complexity: self.weights.complexity,
            coupling: self.weights.coupling,
            naming: self.weights.naming,
            test: self.weights.test,
        }
    }

    /// 检查文件是否应该被忽略
    pub fn should_ignore_file(&self, path: &str) -> bool {
        for dir in &self.ignore.directories {
            if path.starts_with(dir) || path.contains(&format!("/{dir}/")) || path.contains(&format!("\\{dir}\\")) {
                return true;
            }
        }
        for pattern in &self.ignore.files {
            if path.contains(pattern) {
                return true;
            }
        }
        false
    }

    /// 生成默认的 entropy.yaml 内容
    pub fn default_yaml() -> &'static str {
        r"# Cell Architecture 熵值配置文件
# 修改此文件可自定义熵值计算的权重、阈值和忽略规则

# 五维权重配置（总和应为 1.0）
weights:
  structural: 0.25   # 结构熵 - 文件大小分布、模块均匀度
  complexity: 0.25   # 复杂度熵 - 圈复杂度、嵌套深度
  coupling: 0.20     # 耦合熵 - 入度/出度、循环依赖
  naming: 0.15       # 命名熵 - 命名一致性、缩写使用率
  test: 0.15         # 测试熵 - 测试覆盖率、断言密度

# 阈值配置
thresholds:
  total: 50.0        # 总熵值阈值（0-100），超过则门禁失败
  per_dimension:     # 单维度阈值
    structural: 30.0
    complexity: 30.0
    coupling: 30.0
    naming: 30.0
    test: 30.0

# 忽略配置
ignore:
  # 忽略的目录
  directories:
    - target
    - .git
    - node_modules
  # 忽略的文件模式
  files: []
"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EntropyConfig::default();
        assert!((config.weights.structural - 0.25).abs() < 0.001);
        assert!((config.weights.complexity - 0.25).abs() < 0.001);
        assert!((config.weights.coupling - 0.20).abs() < 0.001);
        assert!((config.weights.naming - 0.15).abs() < 0.001);
        assert!((config.weights.test - 0.15).abs() < 0.001);
        assert!((config.thresholds.total - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let config = EntropyConfig::default();
        let sum = config.weights.structural
            + config.weights.complexity
            + config.weights.coupling
            + config.weights.naming
            + config.weights.test;
        assert!((sum - 1.0).abs() < 0.001, "weights should sum to 1.0, got {sum}");
    }

    #[test]
    fn test_to_dimension_weights() {
        let config = EntropyConfig::default();
        let dw = config.to_dimension_weights();
        assert!((dw.structural - 0.25).abs() < 0.001);
        assert!((dw.complexity - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_should_ignore_file() {
        let config = EntropyConfig::default();
        assert!(config.should_ignore_file("target/debug/build/foo.rs"));
        assert!(config.should_ignore_file(".git/config"));
        assert!(config.should_ignore_file("node_modules/pkg/index.js"));
        assert!(!config.should_ignore_file("src/main.rs"));
        assert!(!config.should_ignore_file("src/domain/entropy.rs"));
    }

    #[test]
    fn test_custom_config_parse() {
        let yaml = r#"
weights:
  structural: 0.30
  complexity: 0.20
  coupling: 0.20
  naming: 0.15
  test: 0.15
thresholds:
  total: 40.0
  per_dimension:
    structural: 25.0
    complexity: 25.0
    coupling: 25.0
    naming: 25.0
    test: 25.0
ignore:
  directories:
    - target
    - vendor
  files:
    - "generated.rs"
"#;
        let config: EntropyConfig = serde_yaml::from_str(yaml).unwrap();
        assert!((config.weights.structural - 0.30).abs() < 0.001);
        assert!((config.thresholds.total - 40.0).abs() < 0.001);
        assert_eq!(config.ignore.directories.len(), 2);
        assert!(config.ignore.directories.contains(&"vendor".to_string()));
    }

    #[test]
    fn test_default_yaml_is_valid() {
        let yaml = EntropyConfig::default_yaml();
        let config: EntropyConfig = serde_yaml::from_str(yaml).unwrap();
        assert!((config.weights.structural - 0.25).abs() < 0.001);
    }
}
