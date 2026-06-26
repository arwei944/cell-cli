use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub base_path: String,
    pub config_values: std::collections::HashMap<String, String>,
    pub created_at: String,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Dev,
    Staging,
    Prod,
    Custom(String),
}

impl Environment {
    pub fn name(&self) -> &str {
        match self {
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Prod => "prod",
            Environment::Custom(s) => s,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Environment::Dev => "开发",
            Environment::Staging => "预发布",
            Environment::Prod => "生产",
            Environment::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub key: String,
    pub base_value: Option<String>,
    pub env_value: Option<String>,
    pub diff_type: DiffType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiffType {
    Added,
    Removed,
    Changed,
    Same,
}

impl DiffType {
    pub fn label(&self) -> &str {
        match self {
            DiffType::Added => "新增",
            DiffType::Removed => "删除",
            DiffType::Changed => "变更",
            DiffType::Same => "相同",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub base_env: String,
    pub target_env: String,
    pub diffs: Vec<ConfigDiff>,
    pub drift_count: usize,
    pub generated_at: String,
}

pub struct MultiEnvConfigService;

impl MultiEnvConfigService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_environment(&self, project_path: &str, env: &Environment) -> CellResult<EnvironmentConfig> {
        let env_name = env.name();
        let env_dir = Path::new(project_path).join(".cell/env");
        std::fs::create_dir_all(&env_dir)?;

        let env_path = env_dir.join(format!("{}.toml", env_name));
        let now = chrono::Utc::now().to_rfc3339();
        
        let config = EnvironmentConfig {
            name: env_name.to_string(),
            base_path: env_path.to_string_lossy().to_string(),
            config_values: std::collections::HashMap::new(),
            created_at: now.clone(),
            last_modified: now,
        };

        let default_content = format!(
            "# {} 环境配置\n# Cell Architecture Multi-Environment Config\n\n[entropy]\nthreshold = {}\n\n[architecture]\nstrict = {}\n",
            env.label(),
            if env_name == "prod" { "0.3" } else { "0.5" },
            if env_name == "prod" { "true" } else { "false" }
        );

        std::fs::write(&env_path, &default_content)?;
        
        let mut registry = self.load_registry(project_path)?;
        registry.push(config.clone());
        self.save_registry(project_path, &registry)?;

        Ok(config)
    }

    pub fn list_environments(&self, project_path: &str) -> CellResult<Vec<EnvironmentConfig>> {
        self.load_registry(project_path)
    }

    pub fn get_environment(&self, project_path: &str, env_name: &str) -> CellResult<EnvironmentConfig> {
        let registry = self.load_registry(project_path)?;
        registry.into_iter().find(|e| e.name == env_name)
            .ok_or_else(|| crate::domain::errors::CellError::Config(format!("Environment '{}' not found", env_name)))
    }

    pub fn set_config(&self, project_path: &str, env_name: &str, key: &str, value: &str) -> CellResult<()> {
        let env_dir = Path::new(project_path).join(".cell/env");
        let env_path = env_dir.join(format!("{}.toml", env_name));

        if !env_path.exists() {
            return Err(crate::domain::errors::CellError::Config(format!("Environment '{}' not found", env_name)));
        }

        let content = std::fs::read_to_string(&env_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        let key_line = format!("{} = {}", key, value);
        let key_prefix = key.split('.').next().unwrap_or(key);
        
        let mut found_section = false;
        let mut inserted = false;
        
        for i in 0..lines.len() {
            let line = lines[i].trim();
            if line.starts_with(&format!("[{}]", key_prefix)) {
                found_section = true;
            } else if found_section && (line.starts_with('[') || line.is_empty() || i == lines.len() - 1) {
                if !inserted {
                    lines.insert(i, key_line.clone());
                    inserted = true;
                    break;
                }
            }
        }

        if !inserted {
            lines.push(format!("[{}]", key_prefix));
            lines.push(key_line);
        }

        std::fs::write(&env_path, lines.join("\n"))?;

        let mut registry = self.load_registry(project_path)?;
        for env in &mut registry {
            if env.name == env_name {
                env.config_values.insert(key.to_string(), value.to_string());
                env.last_modified = chrono::Utc::now().to_rfc3339();
            }
        }
        self.save_registry(project_path, &registry)?;

        Ok(())
    }

    pub fn get_config(&self, project_path: &str, env_name: &str, key: &str) -> CellResult<Option<String>> {
        let env = self.get_environment(project_path, env_name)?;
        Ok(env.config_values.get(key).cloned())
    }

    pub fn diff_environments(&self, project_path: &str, base_env: &str, target_env: &str) -> CellResult<DriftReport> {
        let base = self.get_environment(project_path, base_env)?;
        let target = self.get_environment(project_path, target_env)?;

        let mut diffs = Vec::new();
        let all_keys: std::collections::HashSet<&String> = base.config_values.keys()
            .chain(target.config_values.keys())
            .collect();

        for key in all_keys {
            let base_val = base.config_values.get(key);
            let target_val = target.config_values.get(key);

            let diff_type = match (base_val, target_val) {
                (Some(_), None) => DiffType::Removed,
                (None, Some(_)) => DiffType::Added,
                (Some(a), Some(b)) if a != b => DiffType::Changed,
                _ => DiffType::Same,
            };

            diffs.push(ConfigDiff {
                key: key.clone(),
                base_value: base_val.cloned(),
                env_value: target_val.cloned(),
                diff_type,
            });
        }

        let drift_count = diffs.iter().filter(|d| d.diff_type != DiffType::Same).count();

        Ok(DriftReport {
            base_env: base_env.to_string(),
            target_env: target_env.to_string(),
            diffs,
            drift_count,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn detect_drift(&self, project_path: &str) -> CellResult<Vec<DriftReport>> {
        let registry = self.load_registry(project_path)?;
        let mut reports = Vec::new();

        if registry.len() >= 2 {
            let base_env = "dev";
            for env in &registry {
                if env.name != base_env {
                    if let Ok(report) = self.diff_environments(project_path, base_env, &env.name) {
                        reports.push(report);
                    }
                }
            }
        }

        Ok(reports)
    }

    pub fn sync_config(&self, project_path: &str, from_env: &str, to_env: &str) -> CellResult<()> {
        let from = self.get_environment(project_path, from_env)?;
        
        for (key, value) in &from.config_values {
            self.set_config(project_path, to_env, key, value)?;
        }

        Ok(())
    }

    fn registry_path(project_path: &str) -> std::path::PathBuf {
        Path::new(project_path).join(".cell/env/registry.json")
    }

    fn load_registry(&self, project_path: &str) -> CellResult<Vec<EnvironmentConfig>> {
        let path = Self::registry_path(project_path);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let registry: Vec<EnvironmentConfig> = serde_json::from_str(&content)
            .map_err(|e| crate::domain::errors::CellError::Config(format!("Invalid registry: {}", e)))?;
        Ok(registry)
    }

    fn save_registry(&self, project_path: &str, registry: &[EnvironmentConfig]) -> CellResult<()> {
        let path = Self::registry_path(project_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(registry)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for MultiEnvConfigService {
    fn default() -> Self {
        Self::new()
    }
}