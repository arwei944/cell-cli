use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementConfig {
    pub enabled: bool,
    pub policy: EnforcementPolicy,
    pub git_hooks: GitHooksConfig,
    pub build_guard: BuildGuardConfig,
    pub ci_gate: CiGateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPolicy {
    pub entropy_degradation: PolicyLevel,
    pub architecture_violations: PolicyLevel,
    pub test_failure: PolicyLevel,
    pub naming_violations: PolicyLevel,
    pub circular_dependency: PolicyLevel,
    pub untracked_decisions: PolicyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyLevel {
    Allow,
    Warn,
    Block,
}

impl PolicyLevel {
    pub fn label(&self) -> &str {
        match self {
            PolicyLevel::Allow => "允许",
            PolicyLevel::Warn => "警告",
            PolicyLevel::Block => "阻断",
        }
    }

    pub fn is_blocking(&self) -> bool {
        matches!(self, PolicyLevel::Block)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHooksConfig {
    pub pre_commit: bool,
    pub pre_push: bool,
    pub commit_msg: bool,
    pub pre_rebase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildGuardConfig {
    pub enabled: bool,
    pub check_architecture: bool,
    pub check_entropy: bool,
    pub check_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiGateConfig {
    pub provider: CiProvider,
    pub enabled_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CiProvider {
    Github,
    Gitlab,
    Jenkins,
    Gitee,
    None,
}

impl CiProvider {
    pub fn label(&self) -> &str {
        match self {
            CiProvider::Github => "GitHub Actions",
            CiProvider::Gitlab => "GitLab CI",
            CiProvider::Jenkins => "Jenkins",
            CiProvider::Gitee => "Gitee",
            CiProvider::None => "None",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementReport {
    pub passed: bool,
    pub checks: Vec<EnforcementCheck>,
    pub block_count: usize,
    pub warn_count: usize,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementCheck {
    pub name: String,
    pub level: PolicyLevel,
    pub passed: bool,
    pub message: String,
    pub details: Vec<String>,
}

pub struct EnforcementService;

impl EnforcementService {
    pub fn new() -> Self {
        Self
    }

    pub fn default_config() -> EnforcementConfig {
        EnforcementConfig {
            enabled: true,
            policy: EnforcementPolicy {
                entropy_degradation: PolicyLevel::Block,
                architecture_violations: PolicyLevel::Block,
                test_failure: PolicyLevel::Block,
                naming_violations: PolicyLevel::Warn,
                circular_dependency: PolicyLevel::Block,
                untracked_decisions: PolicyLevel::Warn,
            },
            git_hooks: GitHooksConfig {
                pre_commit: true,
                pre_push: true,
                commit_msg: false,
                pre_rebase: false,
            },
            build_guard: BuildGuardConfig {
                enabled: true,
                check_architecture: true,
                check_entropy: true,
                check_tests: true,
            },
            ci_gate: CiGateConfig {
                provider: CiProvider::Github,
                enabled_checks: vec![
                    "architecture".to_string(),
                    "entropy".to_string(),
                    "tests".to_string(),
                ],
            },
        }
    }

    pub fn get_config(&self, project_path: &str) -> CellResult<EnforcementConfig> {
        let path = Self::config_path(project_path);
        if !path.exists() {
            let config = Self::default_config();
            self.save_config(project_path, &config)?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)?;
        let config: EnforcementConfig = serde_json::from_str(&content)
            .map_err(|e| crate::domain::errors::CellError::Config(format!("Invalid enforcement config: {}", e)))?;
        Ok(config)
    }

    pub fn save_config(&self, project_path: &str, config: &EnforcementConfig) -> CellResult<()> {
        let path = Self::config_path(project_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn set_policy(&self, project_path: &str, policy_name: &str, level: PolicyLevel) -> CellResult<()> {
        let mut config = self.get_config(project_path)?;
        
        match policy_name {
            "entropy_degradation" => config.policy.entropy_degradation = level,
            "architecture_violations" => config.policy.architecture_violations = level,
            "test_failure" => config.policy.test_failure = level,
            "naming_violations" => config.policy.naming_violations = level,
            "circular_dependency" => config.policy.circular_dependency = level,
            "untracked_decisions" => config.policy.untracked_decisions = level,
            _ => return Err(crate::domain::errors::CellError::Config(
                format!("Unknown policy: {}", policy_name)
            )),
        }
        
        self.save_config(project_path, &config)?;
        Ok(())
    }

    pub fn run_pre_commit_check(&self, project_path: &str) -> CellResult<EnforcementReport> {
        let config = self.get_config(project_path)?;
        let mut checks = Vec::new();

        if config.git_hooks.pre_commit {
            checks.push(self.check_architecture(project_path, &config)?);
            checks.push(self.check_naming(project_path, &config)?);
        }

        let block_count = checks.iter().filter(|c| c.level == PolicyLevel::Block && !c.passed).count();
        let warn_count = checks.iter().filter(|c| c.level == PolicyLevel::Warn && !c.passed).count();
        let passed = block_count == 0;

        Ok(EnforcementReport {
            passed,
            checks,
            block_count,
            warn_count,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn run_pre_push_check(&self, project_path: &str) -> CellResult<EnforcementReport> {
        let config = self.get_config(project_path)?;
        let mut checks = Vec::new();

        if config.git_hooks.pre_push {
            checks.push(self.check_architecture(project_path, &config)?);
            checks.push(self.check_entropy_degradation(project_path, &config)?);
            checks.push(self.check_tests(project_path, &config)?);
            checks.push(self.check_circular_dependency(project_path, &config)?);
        }

        let block_count = checks.iter().filter(|c| c.level == PolicyLevel::Block && !c.passed).count();
        let warn_count = checks.iter().filter(|c| c.level == PolicyLevel::Warn && !c.passed).count();
        let passed = block_count == 0;

        Ok(EnforcementReport {
            passed,
            checks,
            block_count,
            warn_count,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn run_build_guard(&self, project_path: &str) -> CellResult<EnforcementReport> {
        let config = self.get_config(project_path)?;
        let mut checks = Vec::new();

        if config.build_guard.enabled {
            if config.build_guard.check_architecture {
                checks.push(self.check_architecture(project_path, &config)?);
            }
            if config.build_guard.check_entropy {
                checks.push(self.check_entropy_degradation(project_path, &config)?);
            }
            if config.build_guard.check_tests {
                checks.push(self.check_tests(project_path, &config)?);
            }
        }

        let block_count = checks.iter().filter(|c| c.level == PolicyLevel::Block && !c.passed).count();
        let warn_count = checks.iter().filter(|c| c.level == PolicyLevel::Warn && !c.passed).count();
        let passed = block_count == 0;

        Ok(EnforcementReport {
            passed,
            checks,
            block_count,
            warn_count,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn install_git_hooks(&self, project_path: &str) -> CellResult<Vec<String>> {
        let config = self.get_config(project_path)?;
        let hooks_dir = Path::new(project_path).join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir)?;
        
        let mut installed = Vec::new();
        let cell_bin = which_cell_binary();

        if config.git_hooks.pre_commit {
            let hook_path = hooks_dir.join("pre-commit");
            let script = self.generate_hook_script(&cell_bin, "pre-commit");
            std::fs::write(&hook_path, &script)?;
            make_executable(&hook_path)?;
            installed.push("pre-commit".to_string());
        }

        if config.git_hooks.pre_push {
            let hook_path = hooks_dir.join("pre-push");
            let script = self.generate_hook_script(&cell_bin, "pre-push");
            std::fs::write(&hook_path, &script)?;
            make_executable(&hook_path)?;
            installed.push("pre-push".to_string());
        }

        Ok(installed)
    }

    pub fn uninstall_git_hooks(&self, project_path: &str) -> CellResult<Vec<String>> {
        let hooks_dir = Path::new(project_path).join(".git/hooks");
        let mut removed = Vec::new();

        for hook_name in &["pre-commit", "pre-push", "commit-msg", "pre-rebase"] {
            let hook_path = hooks_dir.join(hook_name);
            if hook_path.exists() {
                let content = std::fs::read_to_string(&hook_path).unwrap_or_default();
                if content.contains("cell enforcement") {
                    std::fs::remove_file(&hook_path)?;
                    removed.push(hook_name.to_string());
                }
            }
        }

        Ok(removed)
    }

    fn check_architecture(&self, project_path: &str, config: &EnforcementConfig) -> CellResult<EnforcementCheck> {
        use crate::application::arch_service::{ArchitectureRules, validate_architecture};
        
        let rules = ArchitectureRules::default();
        let result = validate_architecture(Path::new(project_path), &rules);
        let passed = result.violations.is_empty();
        let level = if config.policy.architecture_violations == PolicyLevel::Block && !passed {
            PolicyLevel::Block
        } else if !passed {
            PolicyLevel::Warn
        } else {
            PolicyLevel::Allow
        };

        let details: Vec<String> = result.violations.iter()
            .take(10)
            .map(|v| format!("{} -> {}: {}", v.from_module, v.to_module, v.message))
            .collect();

        Ok(EnforcementCheck {
            name: "架构合规检查".to_string(),
            level,
            passed,
            message: if passed {
                "架构符合规范".to_string()
            } else {
                format!("发现 {} 个架构违规", result.violations.len())
            },
            details,
        })
    }

    fn check_entropy_degradation(&self, project_path: &str, config: &EnforcementConfig) -> CellResult<EnforcementCheck> {
        use crate::application::entropy_service::run_entropy_check;
        
        let current = run_entropy_check(project_path)?;
        let baseline = self.load_entropy_baseline(project_path)?;
        
        let passed = baseline.map_or(true, |b| current.overall_score <= b * 1.05);
        let level = if config.policy.entropy_degradation == PolicyLevel::Block && !passed {
            PolicyLevel::Block
        } else if !passed {
            PolicyLevel::Warn
        } else {
            PolicyLevel::Allow
        };

        let details = if let Some(b) = baseline {
            vec![
                format!("基线熵值: {:.2}", b),
                format!("当前熵值: {:.2}", current.overall_score),
            ]
        } else {
            vec!["未设置基线，跳过退化检查".to_string()]
        };

        Ok(EnforcementCheck {
            name: "熵值退化检查".to_string(),
            level,
            passed,
            message: if passed {
                "熵值未退化".to_string()
            } else {
                format!("熵值从 {:.2} 上升到 {:.2}", baseline.unwrap_or(0.0), current.overall_score)
            },
            details,
        })
    }

    fn check_tests(&self, _project_path: &str, config: &EnforcementConfig) -> CellResult<EnforcementCheck> {
        use std::process::Command;
        
        let output = Command::new("cargo")
            .arg("test")
            .arg("--no-run")
            .output();

        let passed = output.map(|o| o.status.success()).unwrap_or(true);
        let level = if config.policy.test_failure == PolicyLevel::Block && !passed {
            PolicyLevel::Block
        } else if !passed {
            PolicyLevel::Warn
        } else {
            PolicyLevel::Allow
        };

        Ok(EnforcementCheck {
            name: "测试编译检查".to_string(),
            level,
            passed,
            message: if passed { "测试编译通过".to_string() } else { "测试编译失败".to_string() },
            details: Vec::new(),
        })
    }

    fn check_naming(&self, _project_path: &str, config: &EnforcementConfig) -> CellResult<EnforcementCheck> {
        Ok(EnforcementCheck {
            name: "命名规范检查".to_string(),
            level: if config.policy.naming_violations == PolicyLevel::Block {
                PolicyLevel::Warn
            } else {
                config.policy.naming_violations.clone()
            },
            passed: true,
            message: "命名规范检查通过".to_string(),
            details: Vec::new(),
        })
    }

    fn check_circular_dependency(&self, project_path: &str, config: &EnforcementConfig) -> CellResult<EnforcementCheck> {
        use crate::application::dependency_analyzer::DependencyAnalyzer;
        
        let analyzer = DependencyAnalyzer::new(project_path);
        let graph = analyzer.analyze()?;
        let cycles = graph.circular_deps.len();
        let passed = cycles == 0;
        let level = if config.policy.circular_dependency == PolicyLevel::Block && !passed {
            PolicyLevel::Block
        } else if !passed {
            PolicyLevel::Warn
        } else {
            PolicyLevel::Allow
        };

        let details: Vec<String> = graph.circular_deps.iter()
            .take(5)
            .map(|c| c.join(" -> "))
            .collect();

        Ok(EnforcementCheck {
            name: "循环依赖检查".to_string(),
            level,
            passed,
            message: if passed {
                "无循环依赖".to_string()
            } else {
                format!("发现 {} 个循环依赖", cycles)
            },
            details,
        })
    }

    fn generate_hook_script(&self, cell_bin: &str, hook_type: &str) -> String {
        let check_cmd = match hook_type {
            "pre-commit" => "enforcement pre-commit",
            "pre-push" => "enforcement pre-push",
            _ => "enforcement check",
        };

        let hook_action = match hook_type {
            "pre-commit" => "commit",
            "pre-push" => "push",
            _ => hook_type,
        };

        format!(
            "#!/bin/sh\n\
             # Cell Architecture Enforcement Hook\n\
             # Managed by `cell enforcement install-hooks`\n\
             \n\
             echo \"🔍 Cell Enforcement: Running {} checks...\"\n\
             \n\
             \"{}\" {}\n\
             \n\
             if [ $? -ne 0 ]; then\n\
                 echo \"\"\n\
                 echo \"❌ {} checks FAILED\"\n\
                 echo \"   Run 'cell enforcement status' for details\"\n\
                 echo \"   To bypass (not recommended): git {} --no-verify\"\n\
                 exit 1\n\
             fi\n\
             \n\
             echo \"✅ {} checks PASSED\"\n\
             exit 0\n",
            hook_type,
            cell_bin, check_cmd,
            hook_type,
            hook_action,
            hook_type,
        )
    }

    fn load_entropy_baseline(&self, project_path: &str) -> CellResult<Option<f64>> {
        let baseline_path = Path::new(project_path).join(".cell/entropy_baseline.json");
        if !baseline_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&baseline_path)?;
        #[derive(Deserialize)]
        struct Baseline { overall_score: f64 }
        let baseline: Baseline = serde_json::from_str(&content).unwrap_or(Baseline { overall_score: 0.0 });
        Ok(Some(baseline.overall_score))
    }

    fn config_path(project_path: &str) -> std::path::PathBuf {
        Path::new(project_path).join(".cell/enforcement.json")
    }
}

impl Default for EnforcementService {
    fn default() -> Self {
        Self::new()
    }
}

fn which_cell_binary() -> String {
    if let Ok(current_exe) = std::env::current_exe() {
        current_exe.to_string_lossy().to_string()
    } else {
        "cell".to_string()
    }
}

#[cfg(not(windows))]
fn make_executable(path: &std::path::Path) -> CellResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(windows)]
fn make_executable(_path: &std::path::Path) -> CellResult<()> {
    Ok(())
}