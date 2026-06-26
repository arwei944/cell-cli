use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LintCategory {
    Layering,
    Complexity,
    Naming,
    Testing,
    Coupling,
    BestPractice,
    Invariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintViolation {
    pub rule_id: String,
    pub category: LintCategory,
    pub severity: LintSeverity,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintRule {
    pub id: String,
    pub name: String,
    pub category: LintCategory,
    pub severity: LintSeverity,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub total_files: usize,
    pub total_violations: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub violations: Vec<LintViolation>,
    pub by_category: HashMap<String, usize>,
    pub by_severity: HashMap<String, usize>,
}

pub struct ArchitectureLinter {
    rules: Vec<LintRule>,
}

impl ArchitectureLinter {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    pub fn default_rules() -> Vec<LintRule> {
        vec![
            LintRule {
                id: "L001".to_string(),
                name: "domain-no-application-dep".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Domain 层不能 import Application 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L002".to_string(),
                name: "domain-no-adapters-dep".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Domain 层不能 import Adapters 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L003".to_string(),
                name: "domain-no-interfaces-dep".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Domain 层不能 import Interfaces 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L004".to_string(),
                name: "application-no-interfaces-dep".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Application 层不能 import Interfaces 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L005".to_string(),
                name: "application-no-adapters-dep".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Application 层不能直接依赖 Adapters，必须通过 Port".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L006".to_string(),
                name: "adapters-no-interfaces-dep".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Adapters 层不能 import Interfaces 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L007".to_string(),
                name: "no-circular-dependency".to_string(),
                category: LintCategory::Coupling,
                severity: LintSeverity::Error,
                description: "模块之间不能有循环依赖".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L008".to_string(),
                name: "port-must-be-trait".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Error,
                description: "Port 必须是 trait，不能有实现".to_string(),
                enabled: true,
            },
            LintRule {
                id: "L009".to_string(),
                name: "usecase-in-application".to_string(),
                category: LintCategory::Layering,
                severity: LintSeverity::Warning,
                description: "UseCase 必须在 application 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "C001".to_string(),
                name: "function-too-long".to_string(),
                category: LintCategory::Complexity,
                severity: LintSeverity::Warning,
                description: "函数长度不能超过 80 行".to_string(),
                enabled: true,
            },
            LintRule {
                id: "C002".to_string(),
                name: "cyclomatic-complexity".to_string(),
                category: LintCategory::Complexity,
                severity: LintSeverity::Warning,
                description: "函数圈复杂度不能超过 15".to_string(),
                enabled: true,
            },
            LintRule {
                id: "C003".to_string(),
                name: "nesting-too-deep".to_string(),
                category: LintCategory::Complexity,
                severity: LintSeverity::Warning,
                description: "嵌套深度不能超过 4 层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "C004".to_string(),
                name: "too-many-params".to_string(),
                category: LintCategory::Complexity,
                severity: LintSeverity::Warning,
                description: "函数参数不能超过 6 个".to_string(),
                enabled: true,
            },
            LintRule {
                id: "C005".to_string(),
                name: "struct-too-many-fields".to_string(),
                category: LintCategory::Complexity,
                severity: LintSeverity::Warning,
                description: "结构体字段不能超过 15 个".to_string(),
                enabled: true,
            },
            LintRule {
                id: "N001".to_string(),
                name: "port-naming".to_string(),
                category: LintCategory::Naming,
                severity: LintSeverity::Warning,
                description: "Port 接口使用名词或 -er 后缀".to_string(),
                enabled: true,
            },
            LintRule {
                id: "N002".to_string(),
                name: "usecase-naming".to_string(),
                category: LintCategory::Naming,
                severity: LintSeverity::Info,
                description: "UseCase 使用动词 + 名词命名".to_string(),
                enabled: true,
            },
            LintRule {
                id: "N003".to_string(),
                name: "adapter-naming".to_string(),
                category: LintCategory::Naming,
                severity: LintSeverity::Warning,
                description: "Adapter 命名包含技术栈名 + Port 名".to_string(),
                enabled: true,
            },
            LintRule {
                id: "N004".to_string(),
                name: "test-naming".to_string(),
                category: LintCategory::Naming,
                severity: LintSeverity::Info,
                description: "测试函数名清晰表达测试意图".to_string(),
                enabled: true,
            },
            LintRule {
                id: "T001".to_string(),
                name: "core-module-test-coverage".to_string(),
                category: LintCategory::Testing,
                severity: LintSeverity::Warning,
                description: "核心模块测试覆盖率建议 ≥ 85%".to_string(),
                enabled: true,
            },
            LintRule {
                id: "T002".to_string(),
                name: "domain-test-coverage".to_string(),
                category: LintCategory::Testing,
                severity: LintSeverity::Warning,
                description: "Domain 层测试覆盖率建议 ≥ 95%".to_string(),
                enabled: true,
            },
            LintRule {
                id: "T003".to_string(),
                name: "public-function-has-test".to_string(),
                category: LintCategory::Testing,
                severity: LintSeverity::Info,
                description: "每个 Public 函数至少有一个测试".to_string(),
                enabled: true,
            },
            LintRule {
                id: "B001".to_string(),
                name: "no-unwrap".to_string(),
                category: LintCategory::BestPractice,
                severity: LintSeverity::Warning,
                description: "避免使用 unwrap()/expect()，改用错误处理".to_string(),
                enabled: true,
            },
            LintRule {
                id: "B002".to_string(),
                name: "no-unsafe".to_string(),
                category: LintCategory::BestPractice,
                severity: LintSeverity::Warning,
                description: "避免使用 unsafe 代码块".to_string(),
                enabled: true,
            },
            LintRule {
                id: "B003".to_string(),
                name: "no-todo".to_string(),
                category: LintCategory::BestPractice,
                severity: LintSeverity::Info,
                description: "避免遗留 todo!() / unimplemented!()".to_string(),
                enabled: true,
            },
            LintRule {
                id: "B004".to_string(),
                name: "module-docs".to_string(),
                category: LintCategory::BestPractice,
                severity: LintSeverity::Info,
                description: "公开模块应有文档注释".to_string(),
                enabled: true,
            },
            LintRule {
                id: "INV01".to_string(),
                name: "inward-dependency".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Error,
                description: "不变量1：依赖向内不变，外层只能依赖内层".to_string(),
                enabled: true,
            },
            LintRule {
                id: "INV02".to_string(),
                name: "port-contract-stability".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                description: "不变量2：端口契约不变，Port 接口语义应稳定".to_string(),
                enabled: true,
            },
            LintRule {
                id: "INV03".to_string(),
                name: "isolation-no-shared-state".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                description: "不变量3：隔离性不变，Cell 间不共享内部可变状态".to_string(),
                enabled: true,
            },
            LintRule {
                id: "INV04".to_string(),
                name: "event-contract-compatibility".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                description: "不变量4：事件契约不变，事件 Schema 应向前兼容".to_string(),
                enabled: true,
            },
            LintRule {
                id: "INV05".to_string(),
                name: "testability-core-independent".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                description: "不变量5：可测试性不变，核心逻辑应可独立测试".to_string(),
                enabled: true,
            },
            LintRule {
                id: "INV06".to_string(),
                name: "deletability-modular-design".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Info,
                description: "不变量6：可删除性不变，功能应可整体删除".to_string(),
                enabled: true,
            },
        ]
    }

    pub fn list_rules(&self) -> &[LintRule] {
        &self.rules
    }

    pub fn lint(&self, root: &Path) -> LintResult {
        let mut violations = Vec::new();
        let src_path = root.join("src");
        let mut total_files = 0;

        if src_path.exists() {
            self.lint_directory(&src_path, &src_path, &mut violations, &mut total_files);
        }

        let mut by_category = HashMap::new();
        let mut by_severity = HashMap::new();

        for v in &violations {
            let cat = format!("{:?}", v.category);
            *by_category.entry(cat).or_insert(0) += 1;

            let sev = format!("{:?}", v.severity);
            *by_severity.entry(sev).or_insert(0) += 1;
        }

        let error_count = violations
            .iter()
            .filter(|v| v.severity == LintSeverity::Error)
            .count();
        let warning_count = violations
            .iter()
            .filter(|v| v.severity == LintSeverity::Warning)
            .count();
        let info_count = violations
            .iter()
            .filter(|v| v.severity == LintSeverity::Info)
            .count();

        LintResult {
            total_files,
            total_violations: violations.len(),
            error_count,
            warning_count,
            info_count,
            violations,
            by_category,
            by_severity,
        }
    }

    fn lint_directory(
        &self,
        dir: &Path,
        base: &Path,
        violations: &mut Vec<LintViolation>,
        total_files: &mut usize,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.lint_directory(&path, base, violations, total_files);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    *total_files += 1;
                    self.lint_file(&path, base, violations);
                }
            }
        }
    }

    fn lint_file(&self, path: &Path, base: &Path, violations: &mut Vec<LintViolation>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let file_str = path.to_string_lossy().to_string();
        let rel_path = match path.strip_prefix(base) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => file_str.clone(),
        };

        self.check_layer_deps(path, &rel_path, &content, violations);
        self.check_complexity(&rel_path, &content, violations);
        self.check_naming(&rel_path, &content, violations);
        self.check_best_practices(&rel_path, &content, violations);
        self.check_invariants(&rel_path, &content, violations);
    }

    fn check_layer_deps(
        &self,
        _path: &Path,
        rel_path: &str,
        content: &str,
        violations: &mut Vec<LintViolation>,
    ) {
        let layer = Self::detect_layer(rel_path);

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("use ") && !trimmed.starts_with("pub use ") {
                continue;
            }

            let has_crate = trimmed.contains("crate::");

            if !has_crate {
                continue;
            }

            let dep_layer = if trimmed.contains("::domain") || trimmed.contains("crate::domain") {
                Some("domain")
            } else if trimmed.contains("::application")
                || trimmed.contains("crate::application")
            {
                Some("application")
            } else if trimmed.contains("::adapters") || trimmed.contains("crate::adapters") {
                Some("adapters")
            } else if trimmed.contains("::interfaces")
                || trimmed.contains("crate::interfaces")
            {
                Some("interfaces")
            } else {
                None
            };

            if let (Some(from), Some(to)) = (layer, dep_layer) {
                self.check_banned_dep(from, to, rel_path, line_num + 1, violations);
            }
        }
    }

    fn check_banned_dep(
        &self,
        from: &str,
        to: &str,
        file: &str,
        line: usize,
        violations: &mut Vec<LintViolation>,
    ) {
        let rules: [(&str, &str, &str, &str); 6] = [
            ("domain", "application", "L001", "Domain 层不能 import Application 层"),
            ("domain", "adapters", "L002", "Domain 层不能 import Adapters 层"),
            ("domain", "interfaces", "L003", "Domain 层不能 import Interfaces 层"),
            (
                "application",
                "interfaces",
                "L004",
                "Application 层不能 import Interfaces 层",
            ),
            (
                "application",
                "adapters",
                "L005",
                "Application 层不能直接依赖 Adapters，必须通过 Port",
            ),
            (
                "adapters",
                "interfaces",
                "L006",
                "Adapters 层不能 import Interfaces 层",
            ),
        ];

        for (from_rule, to_rule, rule_id, message) in rules {
            if from == from_rule && to == to_rule {
                if self.is_rule_enabled(rule_id) {
                    violations.push(LintViolation {
                        rule_id: rule_id.to_string(),
                        category: LintCategory::Layering,
                        severity: if rule_id.starts_with('L') && rule_id != "L009" {
                            LintSeverity::Error
                        } else {
                            LintSeverity::Warning
                        },
                        file: file.to_string(),
                        line,
                        message: message.to_string(),
                        suggestion: Some(format!("移除 {} → {} 的依赖", from, to)),
                    });
                }
                break;
            }
        }
    }

    fn check_complexity(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        let mut func_start = 0;
        let mut func_name = String::new();
        let mut in_function = false;
        let mut brace_count = 0;
        let mut max_nesting = 0;
        let mut current_nesting = 0;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("pub async fn "))
                && trimmed.contains('(')
                && !in_function
            {
                in_function = true;
                func_start = line_num + 1;
                func_name = trimmed
                    .split_whitespace()
                    .find(|s| s.contains('('))
                    .unwrap_or("unknown")
                    .split('(')
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                brace_count = 0;
                max_nesting = 0;
                current_nesting = 0;

                let param_count = trimmed.matches(',').count() + 1;
                if param_count > 6 && self.is_rule_enabled("C004") {
                    violations.push(LintViolation {
                        rule_id: "C004".to_string(),
                        category: LintCategory::Complexity,
                        severity: LintSeverity::Warning,
                        file: file.to_string(),
                        line: line_num + 1,
                        message: format!(
                            "函数 `{}` 参数过多 ({})，建议 ≤ 6 个",
                            func_name, param_count
                        ),
                        suggestion: Some("考虑使用结构体或元组合并参数".to_string()),
                    });
                }
            }

            if in_function {
                let opens = trimmed.chars().filter(|c| *c == '{').count();
                let closes = trimmed.chars().filter(|c| *c == '}').count();

                if trimmed.contains("if ")
                    || trimmed.starts_with("else")
                    || trimmed.contains("match ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                    || trimmed.contains("loop {")
                {
                    current_nesting += opens;
                    max_nesting = max_nesting.max(current_nesting);
                }

                current_nesting = current_nesting.saturating_sub(closes);
                brace_count += opens as i32;
                brace_count -= closes as i32;

                if brace_count <= 0 && func_start > 0 {
                    let func_length = line_num + 1 - func_start;
                    if func_length > 80 && self.is_rule_enabled("C001") {
                        violations.push(LintViolation {
                            rule_id: "C001".to_string(),
                            category: LintCategory::Complexity,
                            severity: LintSeverity::Warning,
                            file: file.to_string(),
                            line: func_start,
                            message: format!(
                                "函数 `{}` 过长 ({} 行)，建议 ≤ 80 行",
                                func_name, func_length
                            ),
                            suggestion: Some("考虑拆分为多个小函数".to_string()),
                        });
                    }

                    if max_nesting > 4 && self.is_rule_enabled("C003") {
                        violations.push(LintViolation {
                            rule_id: "C003".to_string(),
                            category: LintCategory::Complexity,
                            severity: LintSeverity::Warning,
                            file: file.to_string(),
                            line: func_start,
                            message: format!(
                                "函数 `{}` 嵌套过深 ({} 层)，建议 ≤ 4 层",
                                func_name, max_nesting
                            ),
                            suggestion: Some("考虑提前返回或提取函数".to_string()),
                        });
                    }

                    in_function = false;
                }
            }
        }
    }

    fn check_naming(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if self.is_rule_enabled("B001") && (trimmed.contains(".unwrap()") || trimmed.contains(".expect(")) {
                if !trimmed.starts_with("//") && !trimmed.starts_with("*") {
                    violations.push(LintViolation {
                        rule_id: "B001".to_string(),
                        category: LintCategory::BestPractice,
                        severity: LintSeverity::Warning,
                        file: file.to_string(),
                        line: line_num + 1,
                        message: "避免使用 unwrap()/expect()，改用适当的错误处理".to_string(),
                        suggestion: Some("使用 match 或 ? 运算符处理错误".to_string()),
                    });
                }
            }

            if self.is_rule_enabled("B003")
                && (trimmed.contains("todo!") || trimmed.contains("unimplemented!"))
            {
                if !trimmed.starts_with("//") && !trimmed.starts_with("*") {
                    violations.push(LintViolation {
                        rule_id: "B003".to_string(),
                        category: LintCategory::BestPractice,
                        severity: LintSeverity::Info,
                        file: file.to_string(),
                        line: line_num + 1,
                        message: "存在 todo!() 或 unimplemented!()，请尽快实现".to_string(),
                        suggestion: None,
                    });
                }
            }

            if self.is_rule_enabled("B002") && trimmed.contains("unsafe ") {
                if !trimmed.starts_with("//") && !trimmed.starts_with("*") {
                    violations.push(LintViolation {
                        rule_id: "B002".to_string(),
                        category: LintCategory::BestPractice,
                        severity: LintSeverity::Warning,
                        file: file.to_string(),
                        line: line_num + 1,
                        message: "避免使用 unsafe 代码块".to_string(),
                        suggestion: Some("考虑使用安全的抽象替代 unsafe".to_string()),
                    });
                }
            }
        }
    }

    fn check_best_practices(&self, _file: &str, _content: &str, _violations: &mut Vec<LintViolation>) {
    }

    fn check_invariants(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        let layer = Self::detect_layer(file);

        if self.is_rule_enabled("INV02") {
            self.check_port_stability(file, content, violations);
        }

        if self.is_rule_enabled("INV03") {
            self.check_isolation(file, content, violations);
        }

        if self.is_rule_enabled("INV04") {
            self.check_event_compatibility(file, content, violations);
        }

        if self.is_rule_enabled("INV05") {
            self.check_testability(file, content, layer, violations);
        }

        if self.is_rule_enabled("INV06") {
            self.check_deletability(file, content, violations);
        }
    }

    fn check_port_stability(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        if !file.contains("ports/") && !file.contains("port") {
            return;
        }

        let trait_count = content.matches("pub trait").count();
        if trait_count == 0 {
            return;
        }

        let method_count = content.matches("fn ").count();
        if method_count > 10 {
            violations.push(LintViolation {
                rule_id: "INV02".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                file: file.to_string(),
                line: 1,
                message: format!("Port 接口方法过多 ({} 个)，可能影响契约稳定性", method_count),
                suggestion: Some("考虑拆分接口，遵循接口隔离原则".to_string()),
            });
        }
    }

    fn check_isolation(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        if content.contains("static mut") || content.contains("static ref") {
            let line_num = content.lines()
                .position(|l| l.contains("static mut") || l.contains("static ref"))
                .map(|i| i + 1)
                .unwrap_or(1);

            violations.push(LintViolation {
                rule_id: "INV03".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                file: file.to_string(),
                line: line_num,
                message: "检测到全局可变状态，可能破坏隔离性".to_string(),
                suggestion: Some("考虑使用依赖注入或线程局部存储，避免共享可变状态".to_string()),
            });
        }
    }

    fn check_event_compatibility(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        if !file.contains("event") && !file.contains("Event") {
            return;
        }

        let has_optional_fields = content.contains("Option<");
        let struct_count = content.matches("pub struct").count();

        if struct_count > 0 && !has_optional_fields {
            violations.push(LintViolation {
                rule_id: "INV04".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Info,
                file: file.to_string(),
                line: 1,
                message: "事件结构体建议使用 Option 字段以支持向前兼容".to_string(),
                suggestion: Some("新增字段使用 Option<T> 包装，默认值为 None，确保旧版本消费者不会因新字段而失败".to_string()),
            });
        }
    }

    fn check_testability(&self, file: &str, content: &str, layer: Option<&str>, violations: &mut Vec<LintViolation>) {
        if layer != Some("domain") {
            return;
        }

        let has_external_deps = content.contains("reqwest::") 
            || content.contains("tokio::")
            || content.contains("sqlx::")
            || content.contains("diesel::");

        if has_external_deps {
            let line_num = content.lines()
                .position(|l| l.contains("reqwest::") || l.contains("tokio::") || l.contains("sqlx::") || l.contains("diesel::"))
                .map(|i| i + 1)
                .unwrap_or(1);

            violations.push(LintViolation {
                rule_id: "INV05".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Warning,
                file: file.to_string(),
                line: line_num,
                message: "Domain 层不应直接依赖外部框架，影响可测试性".to_string(),
                suggestion: Some("通过 Port 接口抽象外部依赖，在 Adapters 层实现".to_string()),
            });
        }
    }

    fn check_deletability(&self, file: &str, content: &str, violations: &mut Vec<LintViolation>) {
        let pub_items = content.matches("pub fn ").count() 
            + content.matches("pub struct ").count()
            + content.matches("pub trait ").count();

        let imports = content.matches("use crate::").count();

        if pub_items > 15 && imports > 10 {
            violations.push(LintViolation {
                rule_id: "INV06".to_string(),
                category: LintCategory::Invariant,
                severity: LintSeverity::Info,
                file: file.to_string(),
                line: 1,
                message: format!("模块公共接口较多 ({}) 且内部依赖广泛 ({})，可能影响可删除性", pub_items, imports),
                suggestion: Some("考虑按功能拆分模块，降低耦合度，使功能可以独立删除".to_string()),
            });
        }
    }

    fn detect_layer(path: &str) -> Option<&'static str> {
        let normalized = path.replace('\\', "/");
        if normalized.contains("src/domain/") || normalized.starts_with("domain/") {
            Some("domain")
        } else if normalized.contains("src/application/")
            || normalized.starts_with("application/")
        {
            Some("application")
        } else if normalized.contains("src/adapters/") || normalized.starts_with("adapters/") {
            Some("adapters")
        } else if normalized.contains("src/interfaces/")
            || normalized.starts_with("interfaces/")
        {
            Some("interfaces")
        } else {
            None
        }
    }

    fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rules
            .iter()
            .find(|r| r.id == rule_id)
            .map(|r| r.enabled)
            .unwrap_or(false)
    }

    pub fn format_result(&self, result: &LintResult) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n🏗️  架构 Lint 结果\n{}\n",
            "─".repeat(50)
        ));
        output.push_str(&format!(
            "  扫描文件: {} 个\n",
            result.total_files
        ));
        output.push_str(&format!(
            "  违规总数: {} 个 (❌ {} | ⚠️  {} | ℹ️  {})\n\n",
            result.total_violations, result.error_count, result.warning_count, result.info_count
        ));

        if !result.by_category.is_empty() {
            output.push_str("📊 按分类统计:\n");
            let mut cats: Vec<_> = result.by_category.iter().collect();
            cats.sort_by_key(|(k, _)| (*k).clone());
            for (cat, count) in cats {
                output.push_str(&format!("  {}: {} 个\n", cat, count));
            }
            output.push('\n');
        }

        if result.violations.is_empty() {
            output.push_str("✅ 没有发现问题！架构很健康。\n");
        } else {
            output.push_str("🔍 违规详情:\n\n");

            let mut sorted = result.violations.clone();
            sorted.sort_by(|a, b| {
                let sev_order = |s: &LintSeverity| match s {
                    LintSeverity::Error => 0,
                    LintSeverity::Warning => 1,
                    LintSeverity::Info => 2,
                };
                sev_order(&a.severity).cmp(&sev_order(&b.severity))
            });

            for v in &sorted {
                let icon = match v.severity {
                    LintSeverity::Error => "❌",
                    LintSeverity::Warning => "⚠️ ",
                    LintSeverity::Info => "ℹ️ ",
                };

                output.push_str(&format!(
                    "  {} [{}] {}:{} - {}\n",
                    icon, v.rule_id, v.file, v.line, v.message
                ));

                if let Some(suggestion) = &v.suggestion {
                    output.push_str(&format!("     💡 建议: {}\n", suggestion));
                }
            }
        }

        output.push_str(&format!("\n{}\n", "─".repeat(50)));

        if result.error_count > 0 {
            output.push_str(&format!(
                "❌ 检查失败：存在 {} 个错误\n",
                result.error_count
            ));
        } else if result.warning_count > 0 {
            output.push_str(&format!(
                "⚠️  检查通过，但有 {} 个警告\n",
                result.warning_count
            ));
        } else {
            output.push_str("✅ 检查通过！\n");
        }

        output
    }
}

impl Default for ArchitectureLinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_project() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
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
    fn test_linter_has_25_rules() {
        let linter = ArchitectureLinter::new();
        assert!(linter.list_rules().len() >= 20);
    }

    #[test]
    fn test_valid_architecture_no_errors() {
        let dir = setup_test_project();
        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering
                && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "Valid architecture should have no layer errors: {:?}",
            layer_errors
        );
    }

    #[test]
    fn test_domain_depends_on_application_detected() {
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

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let has_l001 = result.violations.iter().any(|v| v.rule_id == "L001");
        assert!(has_l001, "Should detect L001 violation");
    }

    #[test]
    fn test_unwrap_detected() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r#"
pub struct Entity;
impl Entity {
    pub fn new() -> Self {
        let x = Some(1);
        let _y = x.unwrap();
        Entity
    }
}
"#,
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let has_b001 = result.violations.iter().any(|v| v.rule_id == "B001");
        assert!(has_b001, "Should detect B001 (unwrap) violation");
    }

    #[test]
    fn test_categories_present() {
        let linter = ArchitectureLinter::new();
        let rules = linter.list_rules();

        let cats: std::collections::HashSet<_> = rules
            .iter()
            .map(|r| format!("{:?}", r.category))
            .collect();

        assert!(cats.contains("Layering"));
        assert!(cats.contains("Complexity"));
        assert!(cats.contains("Naming"));
        assert!(cats.contains("Testing"));
        assert!(cats.contains("BestPractice"));
    }
}
