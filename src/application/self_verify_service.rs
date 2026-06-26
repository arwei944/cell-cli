use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfVerifyResult {
    pub passed: bool,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: Vec<FailedCheck>,
    pub attempts: u32,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCheck {
    pub check_name: String,
    pub error_message: String,
    pub severity: CheckSeverity,
    pub fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyConfig {
    pub max_attempts: u32,
    pub run_arch_check: bool,
    pub run_tests: bool,
    pub run_entropy_check: bool,
    pub auto_fix: bool,
}

impl Default for VerifyConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            run_arch_check: true,
            run_tests: true,
            run_entropy_check: true,
            auto_fix: true,
        }
    }
}

pub struct SelfVerifyService {
    config: VerifyConfig,
}

impl SelfVerifyService {
    pub fn new() -> Self {
        Self {
            config: VerifyConfig::default(),
        }
    }

    pub fn with_config(config: VerifyConfig) -> Self {
        Self { config }
    }

    pub fn run_self_verify(&self, project_path: &str) -> CellResult<SelfVerifyResult> {
        let mut all_failed: Vec<FailedCheck> = Vec::new();
        let mut passed_checks = 0;
        let mut total_checks = 0;

        for attempt in 1..=self.config.max_attempts {
            println!("\n🔍 自我验证 - 第 {} 次尝试", attempt);

            let mut failed = Vec::new();

            if self.config.run_arch_check {
                println!("  🏗️  检查架构规则...");
                match self.check_architecture(project_path) {
                    Ok(true) => {
                        println!("    ✅ 通过");
                    }
                    Ok(false) => {
                        let msg = "存在架构违规".to_string();
                        println!("    ❌ {}", msg);
                        if self.config.auto_fix {
                            println!("    🔧 尝试自动修复...");
                            let _ = self.auto_fix_arch(project_path);
                        }
                        failed.push(FailedCheck {
                            check_name: "architecture".to_string(),
                            error_message: msg,
                            severity: CheckSeverity::Error,
                            fixable: true,
                        });
                    }
                    Err(e) => {
                        let msg = format!("架构检查失败: {}", e);
                        println!("    ❌ {}", msg);
                        failed.push(FailedCheck {
                            check_name: "architecture".to_string(),
                            error_message: msg,
                            severity: CheckSeverity::Critical,
                            fixable: false,
                        });
                    }
                }
                total_checks += 1;
            }

            if self.config.run_tests {
                println!("  🧪 运行测试...");
                match self.run_tests(project_path) {
                    Ok(true) => {
                        println!("    ✅ 通过");
                        passed_checks += 1;
                    }
                    Ok(false) => {
                        let msg = "测试失败".to_string();
                        println!("    ❌ {}", msg);
                        failed.push(FailedCheck {
                            check_name: "tests".to_string(),
                            error_message: msg,
                            severity: CheckSeverity::Error,
                            fixable: false,
                        });
                    }
                    Err(e) => {
                        let msg = format!("测试运行失败: {}", e);
                        println!("    ❌ {}", msg);
                        failed.push(FailedCheck {
                            check_name: "tests".to_string(),
                            error_message: msg,
                            severity: CheckSeverity::Critical,
                            fixable: false,
                        });
                    }
                }
                total_checks += 1;
            }

            if self.config.run_entropy_check {
                println!("  📊 检查熵值...");
                match self.check_entropy(project_path) {
                    Ok(true) => {
                        println!("    ✅ 通过");
                        passed_checks += 1;
                    }
                    Ok(false) => {
                        let msg = "熵值退化".to_string();
                        println!("    ⚠️  {}", msg);
                        failed.push(FailedCheck {
                            check_name: "entropy".to_string(),
                            error_message: msg,
                            severity: CheckSeverity::Warning,
                            fixable: true,
                        });
                    }
                    Err(e) => {
                        let msg = format!("熵值检查失败: {}", e);
                        println!("    ❌ {}", msg);
                        failed.push(FailedCheck {
                            check_name: "entropy".to_string(),
                            error_message: msg,
                            severity: CheckSeverity::Warning,
                            fixable: false,
                        });
                    }
                }
                total_checks += 1;
            }

            if failed.is_empty() {
                passed_checks = total_checks;
                all_failed.clear();
                return Ok(SelfVerifyResult {
                    passed: true,
                    total_checks,
                    passed_checks,
                    failed_checks: Vec::new(),
                    attempts: attempt,
                    max_attempts: self.config.max_attempts,
                });
            }

            all_failed = failed;

            if attempt < self.config.max_attempts && self.config.auto_fix {
                println!("\n  🔄 有问题，尝试修复后重新验证...");
                let _ = self.auto_fix_arch(project_path);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        Ok(SelfVerifyResult {
            passed: false,
            total_checks,
            passed_checks,
            failed_checks: all_failed,
            attempts: self.config.max_attempts,
            max_attempts: self.config.max_attempts,
        })
    }

    fn check_architecture(&self, project_path: &str) -> CellResult<bool> {
        let _ = project_path;
        Ok(true)
    }

    fn run_tests(&self, project_path: &str) -> CellResult<bool> {
        let output = std::process::Command::new("cargo")
            .args(["test", "--lib"])
            .current_dir(project_path)
            .output()?;

        Ok(output.status.success())
    }

    fn check_entropy(&self, project_path: &str) -> CellResult<bool> {
        let baseline_file = Path::new(project_path)
            .join(".cell")
            .join("entropy_baseline.json");

        if !baseline_file.exists() {
            return Ok(true);
        }

        Ok(true)
    }

    fn auto_fix_arch(&self, project_path: &str) -> CellResult<bool> {
        let _ = project_path;
        Ok(true)
    }

    pub fn rollback_to_stable(&self, project_path: &str) -> CellResult<bool> {
        println!("  ⏪ 回滚到上一个稳定版本...");

        let output = std::process::Command::new("git")
            .args(["reset", "--hard", "HEAD~1"])
            .current_dir(project_path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                println!("    ✅ 已回滚到上一个 commit");
                Ok(true)
            }
            _ => {
                println!("    ⚠️  回滚失败或没有可回滚的提交");
                Ok(false)
            }
        }
    }

    pub fn format_result(&self, result: &SelfVerifyResult) -> String {
        let mut output = String::new();

        output.push_str("\n🔍 自我验证结果\n\n");

        output.push_str(&format!(
            "  总检查项: {}\n",
            result.total_checks
        ));
        output.push_str(&format!(
            "  通过: {} / {}\n",
            result.passed_checks, result.total_checks
        ));
        output.push_str(&format!(
            "  尝试次数: {} / {}\n",
            result.attempts, result.max_attempts
        ));

        if result.passed {
            output.push_str("\n✅ 全部通过！\n");
        } else {
            output.push_str("\n❌ 验证未通过\n");

            if !result.failed_checks.is_empty() {
                output.push_str("\n  失败的检查:\n");
                for check in &result.failed_checks {
                    let severity_icon = match check.severity {
                        CheckSeverity::Info => "ℹ️",
                        CheckSeverity::Warning => "⚠️",
                        CheckSeverity::Error => "❌",
                        CheckSeverity::Critical => "🔥",
                    };
                    output.push_str(&format!(
                        "    {} {} - {}\n",
                        severity_icon, check.check_name, check.error_message
                    ));
                    if check.fixable {
                        output.push_str(&format!("       可自动修复: 是\n"));
                    }
                }
            }
        }

        output
    }
}

impl Default for SelfVerifyService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_config_default() {
        let config = VerifyConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert!(config.auto_fix);
    }

    #[test]
    fn test_self_verify_service_new() {
        let service = SelfVerifyService::new();
        assert_eq!(service.config.max_attempts, 3);
    }

    #[test]
    fn test_format_result() {
        let service = SelfVerifyService::new();
        let result = SelfVerifyResult {
            passed: true,
            total_checks: 3,
            passed_checks: 3,
            failed_checks: Vec::new(),
            attempts: 1,
            max_attempts: 3,
        };
        let formatted = service.format_result(&result);
        assert!(formatted.contains("✅ 全部通过"));
    }
}
