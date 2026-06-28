use cell_domain::entropy_config::EntropyConfig;
use cell_domain::errors::{CellError, CellResult};
use std::path::{Path, PathBuf};

/// 熵值配置加载器
pub struct EntropyConfigLoader {
    config: EntropyConfig,
    config_path: Option<PathBuf>,
}

impl EntropyConfigLoader {
    /// 从默认路径加载配置（当前目录下的 entropy.yaml）
    pub fn load(root: &str) -> CellResult<Self> {
        let config_path = Path::new(root).join("entropy.yaml");
        Self::load_from_path(&config_path)
    }

    /// 从指定路径加载配置
    pub fn load_from_path(path: &Path) -> CellResult<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| CellError::Config(format!("无法读取 entropy.yaml: {e}")))?;
            let config: EntropyConfig = serde_yaml::from_str(&content)
                .map_err(|e| CellError::Config(format!("entropy.yaml 格式错误: {e}")))?;

            // 验证权重总和
            let sum = config.weights.structural
                + config.weights.complexity
                + config.weights.coupling
                + config.weights.naming
                + config.weights.test;
            if (sum - 1.0).abs() > 0.01 {
                return Err(CellError::Config(format!(
                    "权重总和必须为 1.0，当前为 {sum:.2}"
                )));
            }

            Ok(Self {
                config,
                config_path: Some(path.to_path_buf()),
            })
        } else {
            Ok(Self {
                config: EntropyConfig::default(),
                config_path: None,
            })
        }
    }

    /// 获取配置引用
    pub fn config(&self) -> &EntropyConfig {
        &self.config
    }

    /// 初始化 entropy.yaml 文件
    pub fn init_config(root: &str, force: bool) -> CellResult<String> {
        let config_path = Path::new(root).join("entropy.yaml");

        if config_path.exists() && !force {
            return Err(CellError::AlreadyExists(
                "entropy.yaml 已存在，使用 --force 覆盖".to_string()
            ));
        }

        let default_content = EntropyConfig::default_yaml();
        std::fs::write(&config_path, default_content)
            .map_err(|e| CellError::Config(format!("无法写入 entropy.yaml: {e}")))?;

        Ok(config_path.to_string_lossy().to_string())
    }

    /// 检查文件是否应被忽略
    pub fn should_ignore(&self, path: &str) -> bool {
        self.config.should_ignore_file(path)
    }

    /// 获取配置路径
    pub fn config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }

    /// 格式化配置信息
    pub fn format_config(&self) -> String {
        let mut o = String::new();
        let c = &self.config;

        o.push_str("📊 熵值配置\n");
        o.push_str(&"=".repeat(50));
        o.push_str("\n\n");

        if let Some(p) = &self.config_path {
            o.push_str(&format!("配置文件: {}\n\n", p.display()));
        } else {
            o.push_str("使用默认配置（未找到 entropy.yaml）\n\n");
        }

        o.push_str("五维权重:\n");
        o.push_str(&format!("  结构熵:   {:.0}%\n", c.weights.structural * 100.0));
        o.push_str(&format!("  复杂度熵: {:.0}%\n", c.weights.complexity * 100.0));
        o.push_str(&format!("  耦合熵:   {:.0}%\n", c.weights.coupling * 100.0));
        o.push_str(&format!("  命名熵:   {:.0}%\n", c.weights.naming * 100.0));
        o.push_str(&format!("  测试熵:   {:.0}%\n", c.weights.test * 100.0));

        o.push_str(&format!("\n总熵值阈值: {:.0}\n", c.thresholds.total));

        o.push_str("\n单维度阈值:\n");
        o.push_str(&format!("  结构:   {:.0}\n", c.thresholds.per_dimension.structural));
        o.push_str(&format!("  复杂度: {:.0}\n", c.thresholds.per_dimension.complexity));
        o.push_str(&format!("  耦合:   {:.0}\n", c.thresholds.per_dimension.coupling));
        o.push_str(&format!("  命名:   {:.0}\n", c.thresholds.per_dimension.naming));
        o.push_str(&format!("  测试:   {:.0}\n", c.thresholds.per_dimension.test));

        if !c.ignore.directories.is_empty() {
            o.push_str(&format!("\n忽略目录: {}\n", c.ignore.directories.join(", ")));
        }
        if !c.ignore.files.is_empty() {
            o.push_str(&format!("忽略文件: {}\n", c.ignore.files.join(", ")));
        }

        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_when_no_file() {
        let loader = EntropyConfigLoader::load("/nonexistent/path").unwrap();
        let config = loader.config();
        assert!((config.weights.structural - 0.25).abs() < 0.001);
        assert!(loader.config_path().is_none());
    }

    #[test]
    fn test_init_config() {
        let dir = std::env::temp_dir().join("cell_test_entropy_config");
        let _ = std::fs::create_dir_all(&dir);

        let path = EntropyConfigLoader::init_config(&dir.to_string_lossy(), false).unwrap();
        assert!(path.contains("entropy.yaml"));

        // 清理
        let _ = std::fs::remove_file(dir.join("entropy.yaml"));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_init_config_already_exists() {
        let dir = std::env::temp_dir().join("cell_test_entropy_config2");
        let _ = std::fs::create_dir_all(&dir);

        let _ = EntropyConfigLoader::init_config(&dir.to_string_lossy(), false).unwrap();
        let result = EntropyConfigLoader::init_config(&dir.to_string_lossy(), false);
        assert!(result.is_err());

        // force overwrite
        let result2 = EntropyConfigLoader::init_config(&dir.to_string_lossy(), true);
        assert!(result2.is_ok());

        // 清理
        let _ = std::fs::remove_file(dir.join("entropy.yaml"));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_load_custom_config() {
        let dir = std::env::temp_dir().join("cell_test_entropy_config3");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join("entropy.yaml");

        let yaml = r"
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
  files: []
";
        std::fs::write(&config_path, yaml).unwrap();

        let loader = EntropyConfigLoader::load(&dir.to_string_lossy()).unwrap();
        let config = loader.config();
        assert!((config.weights.structural - 0.30).abs() < 0.001);
        assert!((config.thresholds.total - 40.0).abs() < 0.001);

        // 清理
        let _ = std::fs::remove_file(&config_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn test_invalid_weights_sum() {
        let dir = std::env::temp_dir().join("cell_test_entropy_config4");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join("entropy.yaml");

        let yaml = r"
weights:
  structural: 0.50
  complexity: 0.50
  coupling: 0.50
  naming: 0.50
  test: 0.50
thresholds:
  total: 50.0
  per_dimension:
    structural: 30.0
    complexity: 30.0
    coupling: 30.0
    naming: 30.0
    test: 30.0
";
        std::fs::write(&config_path, yaml).unwrap();

        let result = EntropyConfigLoader::load(&dir.to_string_lossy());
        assert!(result.is_err());

        // 清理
        let _ = std::fs::remove_file(&config_path);
        let _ = std::fs::remove_dir(&dir);
    }
}
