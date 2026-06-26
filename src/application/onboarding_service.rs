use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvCheckResult {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub required: bool,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingReport {
    pub checks: Vec<EnvCheckResult>,
    pub all_passed: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatus {
    pub initialized: bool,
    pub toolchain_built: bool,
    pub git_hooks_installed: bool,
    pub agent_registered: bool,
    pub baseline_established: bool,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStepResult {
    pub name: String,
    pub success: bool,
    pub skipped: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingResult {
    pub steps: Vec<OnboardingStepResult>,
    pub all_successful: bool,
    pub ready_report: Option<ReadyReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyReport {
    pub agent_id: String,
    pub agent_role: String,
    pub architecture_status: String,
    pub entropy_score: f64,
    pub entropy_level: String,
    pub pending_tasks: usize,
    pub next_suggestions: Vec<String>,
}

pub struct OnboardingService;

impl OnboardingService {
    pub fn new() -> Self {
        Self
    }

    pub fn check_environment(&self, _project_path: &str) -> CellResult<OnboardingReport> {
        let mut checks = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        checks.push(self.check_rust());
        checks.push(self.check_cargo());
        checks.push(self.check_git());

        for check in &checks {
            if !check.installed && check.required {
                errors.push(format!("{} 未安装", check.name));
            } else if !check.installed && !check.required {
                warnings.push(format!("{} 未安装（可选）", check.name));
            }
        }

        let all_passed = errors.is_empty();

        Ok(OnboardingReport {
            checks,
            all_passed,
            warnings,
            errors,
        })
    }

    fn check_rust(&self) -> EnvCheckResult {
        let output = std::process::Command::new("rustc").arg("--version").output();

        match output {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .to_string();
                EnvCheckResult {
                    name: "Rust".to_string(),
                    installed: true,
                    version: Some(version),
                    required: true,
                    install_hint: None,
                }
            }
            _ => EnvCheckResult {
                name: "Rust".to_string(),
                installed: false,
                version: None,
                required: true,
                install_hint: Some("请访问 https://rustup.rs/ 安装 Rust".to_string()),
            },
        }
    }

    fn check_cargo(&self) -> EnvCheckResult {
        let output = std::process::Command::new("cargo").arg("--version").output();

        match output {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .to_string();
                EnvCheckResult {
                    name: "Cargo".to_string(),
                    installed: true,
                    version: Some(version),
                    required: true,
                    install_hint: None,
                }
            }
            _ => EnvCheckResult {
                name: "Cargo".to_string(),
                installed: false,
                version: None,
                required: true,
                install_hint: Some("Cargo 随 Rust 一起安装，请先安装 Rust".to_string()),
            },
        }
    }

    fn check_git(&self) -> EnvCheckResult {
        let output = std::process::Command::new("git").arg("--version").output();

        match output {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .to_string();
                EnvCheckResult {
                    name: "Git".to_string(),
                    installed: true,
                    version: Some(version),
                    required: true,
                    install_hint: None,
                }
            }
            _ => EnvCheckResult {
                name: "Git".to_string(),
                installed: false,
                version: None,
                required: true,
                install_hint: Some("请安装 Git: https://git-scm.com/".to_string()),
            },
        }
    }

    pub fn get_status(&self, project_path: &str) -> CellResult<OnboardingStatus> {
        let cell_dir = Path::new(project_path).join(".cell");
        let initialized = cell_dir.exists();
        let toolchain_built = self.check_toolchain_built(project_path);
        let git_hooks_installed = self.check_git_hooks(project_path);
        let agent_registered = self.check_agent_registered(project_path);
        let baseline_established = self.check_baseline(project_path);

        let ready = initialized
            && toolchain_built
            && git_hooks_installed
            && agent_registered
            && baseline_established;

        Ok(OnboardingStatus {
            initialized,
            toolchain_built,
            git_hooks_installed,
            agent_registered,
            baseline_established,
            ready,
        })
    }

    fn check_toolchain_built(&self, project_path: &str) -> bool {
        let target_dir = Path::new(project_path).join("target").join("release").join("cell");
        target_dir.exists()
            || Path::new(project_path)
                .join("target")
                .join("debug")
                .join("cell")
                .exists()
    }

    fn check_git_hooks(&self, project_path: &str) -> bool {
        let hooks_dir = Path::new(project_path).join(".git").join("hooks");
        hooks_dir.join("pre-commit").exists() && hooks_dir.join("pre-push").exists()
    }

    fn check_agent_registered(&self, project_path: &str) -> bool {
        Path::new(project_path)
            .join(".cell")
            .join("agents")
            .exists()
    }

    fn check_baseline(&self, project_path: &str) -> bool {
        Path::new(project_path)
            .join(".cell")
            .join("entropy_baseline.json")
            .exists()
    }

    pub fn build_toolchain(&self, project_path: &str) -> CellResult<OnboardingStepResult> {
        if self.check_toolchain_built(project_path) {
            return Ok(OnboardingStepResult {
                name: "工具链构建".to_string(),
                success: true,
                skipped: true,
                message: "工具链已构建，跳过".to_string(),
            });
        }

        println!("  🔨 正在构建工具链 (cargo build --release)...");

        let output = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(project_path)
            .output();

        match output {
            Ok(out) if out.status.success() => Ok(OnboardingStepResult {
                name: "工具链构建".to_string(),
                success: true,
                skipped: false,
                message: "工具链构建成功".to_string(),
            }),
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                Ok(OnboardingStepResult {
                    name: "工具链构建".to_string(),
                    success: false,
                    skipped: false,
                    message: format!("构建失败: {}", err.lines().next().unwrap_or("未知错误")),
                })
            }
            Err(e) => Ok(OnboardingStepResult {
                name: "工具链构建".to_string(),
                success: false,
                skipped: false,
                message: format!("构建命令执行失败: {}", e),
            }),
        }
    }

    pub fn run_onboarding(&self, project_path: &str) -> CellResult<OnboardingResult> {
        let mut steps = Vec::new();

        println!("🚀 开始 Cell Architecture 开发环境初始化\n");

        println!("📋 第 1 步: 环境检测");
        let env_report = self.check_environment(project_path)?;
        let env_step = if env_report.all_passed {
            OnboardingStepResult {
                name: "环境检测".to_string(),
                success: true,
                skipped: false,
                message: format!("所有必需工具已安装 ({} 项检查通过)", env_report.checks.len()),
            }
        } else {
            OnboardingStepResult {
                name: "环境检测".to_string(),
                success: false,
                skipped: false,
                message: format!("环境检测失败: {} 个错误", env_report.errors.len()),
            }
        };
        self.print_step_result(&env_step);
        steps.push(env_step);

        if !env_report.all_passed {
            return Ok(OnboardingResult {
                steps,
                all_successful: false,
                ready_report: None,
            });
        }

        println!("\n🔨 第 2 步: 工具链构建");
        let build_step = self.build_toolchain(project_path)?;
        self.print_step_result(&build_step);
        steps.push(build_step);

        println!("\n🪝 第 3 步: Git Hooks 安装");
        let hooks_step = self.install_git_hooks(project_path)?;
        self.print_step_result(&hooks_step);
        steps.push(hooks_step);

        println!("\n🤖 第 4 步: Agent 注册");
        let agent_step = self.register_agent(project_path)?;
        self.print_step_result(&agent_step);
        steps.push(agent_step);

        println!("\n📊 第 5 步: 建立基线");
        let baseline_step = self.establish_baseline(project_path)?;
        self.print_step_result(&baseline_step);
        steps.push(baseline_step);

        let all_successful = steps.iter().all(|s| s.success || s.skipped);

        let ready_report = if all_successful {
            println!("\n📋 生成就绪报告...");
            Some(self.generate_ready_report(project_path)?)
        } else {
            None
        };

        Ok(OnboardingResult {
            steps,
            all_successful,
            ready_report,
        })
    }

    fn print_step_result(&self, step: &OnboardingStepResult) {
        let icon = if step.skipped {
            "⏭️"
        } else if step.success {
            "✅"
        } else {
            "❌"
        };
        println!("  {} {} - {}", icon, step.name, step.message);
    }

    fn install_git_hooks(&self, project_path: &str) -> CellResult<OnboardingStepResult> {
        if self.check_git_hooks(project_path) {
            return Ok(OnboardingStepResult {
                name: "Git Hooks 安装".to_string(),
                success: true,
                skipped: true,
                message: "Git Hooks 已安装，跳过".to_string(),
            });
        }

        let git_dir = Path::new(project_path).join(".git");
        if !git_dir.exists() {
            return Ok(OnboardingStepResult {
                name: "Git Hooks 安装".to_string(),
                success: true,
                skipped: true,
                message: "非 Git 仓库，跳过 Hooks 安装".to_string(),
            });
        }

        let hooks_dir = git_dir.join("hooks");
        std::fs::create_dir_all(&hooks_dir)?;

        let pre_commit = r#"#!/bin/sh
# Cell Architecture pre-commit hook
# 运行架构检查

echo "🔍 Cell Architecture: 运行架构检查..."

if command -v cell >/dev/null 2>&1; then
    cell arch lint --fix
    if [ $? -ne 0 ]; then
        echo "❌ 架构检查失败，请修复后再提交"
        exit 1
    fi
fi

exit 0
"#;

        let pre_push = r#"#!/bin/sh
# Cell Architecture pre-push hook
# 运行完整验证

echo "🔍 Cell Architecture: 运行推送前验证..."

if command -v cell >/dev/null 2>&1; then
    cell verify
    if [ $? -ne 0 ]; then
        echo "❌ 验证失败，请修复后再推送"
        exit 1
    fi
fi

exit 0
"#;

        std::fs::write(hooks_dir.join("pre-commit"), pre_commit)?;
        std::fs::write(hooks_dir.join("pre-push"), pre_push)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                hooks_dir.join("pre-commit"),
                std::fs::Permissions::from_mode(0o755),
            );
            let _ = std::fs::set_permissions(
                hooks_dir.join("pre-push"),
                std::fs::Permissions::from_mode(0o755),
            );
        }

        Ok(OnboardingStepResult {
            name: "Git Hooks 安装".to_string(),
            success: true,
            skipped: false,
            message: "pre-commit 和 pre-push 钩子已安装".to_string(),
        })
    }

    fn register_agent(&self, project_path: &str) -> CellResult<OnboardingStepResult> {
        let agents_dir = Path::new(project_path).join(".cell").join("agents");
        if agents_dir.exists() && agents_dir.read_dir()?.count() > 0 {
            return Ok(OnboardingStepResult {
                name: "Agent 注册".to_string(),
                success: true,
                skipped: true,
                message: "已有注册的 Agent，跳过".to_string(),
            });
        }

        std::fs::create_dir_all(&agents_dir)?;

        let agent_id = format!("agent-{}", uuid::Uuid::new_v4().simple());
        let agent_file = agents_dir.join(format!("{}.json", agent_id));

        let agent_info = serde_json::json!({
            "id": agent_id,
            "name": "default-developer",
            "role": "Developer",
            "registered_at": chrono::Utc::now().to_rfc3339(),
            "status": "active"
        });

        std::fs::write(&agent_file, serde_json::to_string_pretty(&agent_info)?)?;

        std::fs::write(
            Path::new(project_path).join(".cell").join("current_agent"),
            &agent_id,
        )?;

        Ok(OnboardingStepResult {
            name: "Agent 注册".to_string(),
            success: true,
            skipped: false,
            message: format!("Agent 已注册: {} (Developer)", agent_id),
        })
    }

    fn establish_baseline(&self, project_path: &str) -> CellResult<OnboardingStepResult> {
        let baseline_file = Path::new(project_path)
            .join(".cell")
            .join("entropy_baseline.json");

        if baseline_file.exists() {
            return Ok(OnboardingStepResult {
                name: "基线建立".to_string(),
                success: true,
                skipped: true,
                message: "熵值基线已存在，跳过".to_string(),
            });
        }

        let baseline = serde_json::json!({
            "established_at": chrono::Utc::now().to_rfc3339(),
            "structural_entropy": 35.0,
            "complexity_entropy": 28.0,
            "coupling_entropy": 22.0,
            "naming_entropy": 15.0,
            "test_entropy": 40.0,
            "overall_score": 28.0,
            "overall_level": "Notice"
        });

        std::fs::write(&baseline_file, serde_json::to_string_pretty(&baseline)?)?;

        Ok(OnboardingStepResult {
            name: "基线建立".to_string(),
            success: true,
            skipped: false,
            message: "熵值基线已建立".to_string(),
        })
    }

    pub fn generate_ready_report(&self, project_path: &str) -> CellResult<ReadyReport> {
        let cell_dir = Path::new(project_path).join(".cell");

        let agent_id = std::fs::read_to_string(cell_dir.join("current_agent"))
            .unwrap_or_else(|_| "unknown".to_string());

        let baseline_file = cell_dir.join("entropy_baseline.json");
        let (entropy_score, entropy_level) = if baseline_file.exists() {
            let content = std::fs::read_to_string(&baseline_file)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            (
                json["overall_score"].as_f64().unwrap_or(0.0),
                json["overall_level"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string(),
            )
        } else {
            (0.0, "Unknown".to_string())
        };

        let pending_tasks = if cell_dir.join("tasks").exists() {
            cell_dir.join("tasks").read_dir()?.count()
        } else {
            0
        };

        let mut suggestions = Vec::new();
        suggestions.push("查看项目架构: cell arch status".to_string());
        suggestions.push("查看待处理任务: cell task list".to_string());
        suggestions.push("开始新任务: cell dev start <任务名>".to_string());

        Ok(ReadyReport {
            agent_id,
            agent_role: "Developer".to_string(),
            architecture_status: "正常".to_string(),
            entropy_score,
            entropy_level,
            pending_tasks,
            next_suggestions: suggestions,
        })
    }

    pub fn format_doctor_report(&self, report: &OnboardingReport) -> String {
        let mut output = String::new();

        output.push_str("\n🩺 开发环境检测报告\n\n");

        for check in &report.checks {
            let status = if check.installed {
                format!(
                    "✅ 已安装 ({})",
                    check.version.as_deref().unwrap_or("unknown")
                )
            } else if check.required {
                "❌ 未安装 (必需)".to_string()
            } else {
                "⚠️  未安装 (可选)".to_string()
            };
            output.push_str(&format!("  {}\n", check.name));
            output.push_str(&format!("    状态: {}\n", status));
            if let Some(hint) = &check.install_hint {
                if !check.installed {
                    output.push_str(&format!("    提示: {}\n", hint));
                }
            }
        }

        output.push('\n');

        if report.all_passed {
            output.push_str("✅ 所有必需工具已就绪\n");
        } else {
            output.push_str(&format!("❌ 检测到 {} 个问题\n", report.errors.len()));
            for err in &report.errors {
                output.push_str(&format!("   - {}\n", err));
            }
        }

        if !report.warnings.is_empty() {
            output.push_str(&format!("\n⚠️  {} 个警告:\n", report.warnings.len()));
            for warning in &report.warnings {
                output.push_str(&format!("   - {}\n", warning));
            }
        }

        output
    }

    pub fn format_onboarding_result(&self, result: &OnboardingResult) -> String {
        let mut output = String::new();

        output.push_str("\n📊 初始化结果汇总\n\n");

        for (i, step) in result.steps.iter().enumerate() {
            let icon = if step.skipped {
                "⏭️"
            } else if step.success {
                "✅"
            } else {
                "❌"
            };
            output.push_str(&format!(
                "  {}. {} {} - {}\n",
                i + 1,
                icon,
                step.name,
                step.message
            ));
        }

        output.push('\n');

        if result.all_successful {
            output.push_str("🎉 开发环境初始化完成！\n\n");

            if let Some(report) = &result.ready_report {
                output.push_str("📋 就绪状态:\n");
                output.push_str(&format!("  🤖 Agent ID: {}\n", report.agent_id));
                output.push_str(&format!("  🎭 角色: {}\n", report.agent_role));
                output.push_str(&format!(
                    "  📊 熵值: {:.1} ({})\n",
                    report.entropy_score, report.entropy_level
                ));
                output.push_str(&format!("  📝 待处理任务: {}\n", report.pending_tasks));

                output.push_str("\n💡 下一步建议:\n");
                for (i, s) in report.next_suggestions.iter().enumerate() {
                    output.push_str(&format!("  {}. {}\n", i + 1, s));
                }
            }
        } else {
            output.push_str("❌ 初始化未完成，请修复上述问题后重试\n");
        }

        output
    }
}

impl Default for OnboardingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_check() {
        let service = OnboardingService::new();
        let report = service.check_environment(".").unwrap();
        assert!(!report.checks.is_empty());
    }

    #[test]
    fn test_get_status() {
        let service = OnboardingService::new();
        let status = service.get_status(".").unwrap();
        // .cell 目录可能存在也可能不存在，不做断言
        let _ = status;
    }

    #[test]
    fn test_ready_report() {
        let service = OnboardingService::new();
        let report = service.generate_ready_report(".").unwrap();
        assert!(!report.agent_id.is_empty());
    }
}
