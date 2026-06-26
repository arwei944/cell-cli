use crate::application::entropy_service;
use crate::domain::errors::CellResult;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn find_cargo() -> String {
    if let Ok(path) = std::env::var("CARGO") {
        return path;
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        let candidate = format!("{}\\.cargo\\bin\\cargo.exe", home);
        if std::path::Path::new(&candidate).exists() {
            return candidate;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let candidate = format!("{}/.cargo/bin/cargo", home);
        if std::path::Path::new(&candidate).exists() {
            return candidate;
        }
    }
    "cargo".to_string()
}

pub struct FastVerifyService;

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub passed: bool,
    pub duration_ms: u64,
    pub checks: Vec<VerifyCheck>,
}

#[derive(Debug, Clone)]
pub struct VerifyCheck {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub details: Option<String>,
}

impl FastVerifyService {
    pub fn new() -> Self {
        Self
    }

    pub fn check_compilation(&self, project_path: &str) -> CellResult<()> {
        let check = self.run_cargo_check(project_path)?;
        if check.passed {
            Ok(())
        } else {
            Err(crate::domain::errors::CellError::Other(
                check.details.unwrap_or_else(|| "编译失败".to_string())
            ))
        }
    }

    pub fn check_tests(&self, project_path: &str, deep: bool) -> CellResult<()> {
        let check = if deep {
            self.run_all_tests(project_path)?
        } else {
            self.run_fast_tests(project_path)?
        };
        if check.passed {
            Ok(())
        } else {
            Err(crate::domain::errors::CellError::Other(
                check.details.unwrap_or_else(|| "测试失败".to_string())
            ))
        }
    }

    pub fn check_architecture(&self, project_path: &str) -> CellResult<()> {
        let check = self.check_architecture_rules(project_path)?;
        if check.passed {
            Ok(())
        } else {
            Err(crate::domain::errors::CellError::Other(
                check.details.unwrap_or_else(|| "架构验证失败".to_string())
            ))
        }
    }

    pub fn check_entropy_gate(&self, project_path: &str, threshold: f64) -> CellResult<()> {
        let check = self.run_entropy_check(project_path, threshold)?;
        if check.passed {
            Ok(())
        } else {
            Err(crate::domain::errors::CellError::Other(
                check.details.unwrap_or_else(|| format!("熵值超过阈值 {}", threshold))
            ))
        }
    }

    pub fn quick_check(&self, project_path: &str) -> CellResult<VerifyResult> {
        let start = Instant::now();
        let mut checks = Vec::new();

        checks.push(self.run_cargo_check(project_path)?);
        checks.push(self.run_fast_tests(project_path)?);
        checks.push(self.check_architecture_rules(project_path)?);

        let all_passed = checks.iter().all(|c| c.passed);
        let duration = start.elapsed().as_millis() as u64;

        Ok(VerifyResult {
            passed: all_passed,
            duration_ms: duration,
            checks,
        })
    }

    pub fn deep_check(&self, project_path: &str) -> CellResult<VerifyResult> {
        let start = Instant::now();
        let mut checks = Vec::new();

        checks.push(self.run_cargo_check(project_path)?);
        checks.push(self.run_all_tests(project_path)?);
        checks.push(self.run_entropy_check(project_path, 60.0)?);
        checks.push(self.check_architecture_rules(project_path)?);

        let all_passed = checks.iter().all(|c| c.passed);
        let duration = start.elapsed().as_millis() as u64;

        Ok(VerifyResult {
            passed: all_passed,
            duration_ms: duration,
            checks,
        })
    }

    fn run_cargo_check(&self, project_path: &str) -> CellResult<VerifyCheck> {
        let start = Instant::now();
        let cargo = find_cargo();
        let output = Command::new(&cargo)
            .arg("check")
            .current_dir(project_path)
            .output()
            .map_err(|e| crate::domain::errors::CellError::Other(format!("cargo check失败: {}", e)))?;

        let duration = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let details = if passed {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        Ok(VerifyCheck {
            name: "编译检查 (cargo check)".to_string(),
            passed,
            duration_ms: duration,
            details,
        })
    }

    fn run_fast_tests(&self, project_path: &str) -> CellResult<VerifyCheck> {
        let start = Instant::now();
        let cargo = find_cargo();
        let output = Command::new(&cargo)
            .args(["test", "--lib", "--", "--test-threads=4"])
            .current_dir(project_path)
            .output()
            .map_err(|e| crate::domain::errors::CellError::Other(format!("测试执行失败: {}", e)))?;

        let duration = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let details = if passed {
            None
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Some(format!("{}\n{}", stderr, stdout))
        };

        Ok(VerifyCheck {
            name: "快速单元测试".to_string(),
            passed,
            duration_ms: duration,
            details,
        })
    }

    fn run_all_tests(&self, project_path: &str) -> CellResult<VerifyCheck> {
        let start = Instant::now();
        let cargo = find_cargo();
        let output = Command::new(&cargo)
            .args(["test", "--", "--test-threads=4"])
            .current_dir(project_path)
            .output()
            .map_err(|e| crate::domain::errors::CellError::Other(format!("测试执行失败: {}", e)))?;

        let duration = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let details = if passed {
            None
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Some(format!("{}\n{}", stderr, stdout))
        };

        Ok(VerifyCheck {
            name: "完整测试套件".to_string(),
            passed,
            duration_ms: duration,
            details,
        })
    }

    fn check_architecture_rules(&self, project_path: &str) -> CellResult<VerifyCheck> {
        let start = Instant::now();
        let src_dir = Path::new(project_path).join("src");

        let mut violations = Vec::new();

        if src_dir.join("domain").exists() {
            if let Ok(entries) = std::fs::read_dir(&src_dir.join("domain")) {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if content.contains("use crate::application") 
                            || content.contains("use crate::adapters") 
                            || content.contains("use crate::interfaces") {
                            violations.push(format!(
                                "领域层依赖了外层: {}",
                                entry.file_name().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }

        if src_dir.join("application").exists() {
            if let Ok(entries) = std::fs::read_dir(&src_dir.join("application")) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name == "fast_verify_service.rs" {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let has_adapter_dep = content.lines()
                            .filter(|l| !l.trim_start().starts_with("//"))
                            .any(|l| l.contains("use crate::adapters::"));
                        let has_interface_dep = content.lines()
                            .filter(|l| !l.trim_start().starts_with("//"))
                            .any(|l| l.contains("use crate::interfaces::"));
                        
                        if has_adapter_dep || has_interface_dep {
                            violations.push(format!(
                                "应用层依赖了外层: {}",
                                entry.file_name().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        let passed = violations.is_empty();
        let details = if passed {
            None
        } else {
            Some(violations.join("\n"))
        };

        Ok(VerifyCheck {
            name: "架构分层规则检查".to_string(),
            passed,
            duration_ms: duration,
            details,
        })
    }

    fn run_entropy_check(&self, project_path: &str, threshold: f64) -> CellResult<VerifyCheck> {
        let start = Instant::now();

        let result = entropy_service::run_entropy_check(project_path);

        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(report) => {
                let passed = report.overall_score < threshold;
                let details = if passed {
                    None
                } else {
                    Some(format!(
                        "熵值 {} 超过阈值 {} (等级: {})",
                        report.overall_score,
                        threshold,
                        report.grade.label()
                    ))
                };

                Ok(VerifyCheck {
                    name: "架构熵值门禁".to_string(),
                    passed,
                    duration_ms: duration,
                    details,
                })
            }
            Err(e) => Ok(VerifyCheck {
                name: "架构熵值门禁".to_string(),
                passed: false,
                duration_ms: duration,
                details: Some(format!("熵值检查失败: {}", e)),
            }),
        }
    }

    pub fn format_result(&self, result: &VerifyResult) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n{} 快速验证结果 (耗时: {}ms)\n\n",
            if result.passed { "✅" } else { "❌" },
            result.duration_ms
        ));

        for check in &result.checks {
            let icon = if check.passed { "✅" } else { "❌" };
            output.push_str(&format!(
                "  {} {} ({}ms)\n",
                icon, check.name, check.duration_ms
            ));
            if let Some(details) = &check.details {
                let lines: Vec<&str> = details.lines().take(10).collect();
                output.push_str(&format!("     {}\n", lines.join("\n     ")));
            }
        }

        output.push('\n');
        output
    }
}

impl Default for FastVerifyService {
    fn default() -> Self {
        Self::new()
    }
}
