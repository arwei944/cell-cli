use crate::application::arch_service::{ArchitectureRules, Violation};
use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub fixed: usize,
    pub skipped: usize,
    pub failed: usize,
    pub changes: Vec<FixChange>,
    pub suggestions: Vec<FixSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixChange {
    pub file: String,
    pub line: usize,
    pub old_content: String,
    pub new_content: String,
    pub description: String,
    pub success: bool,
    pub fix_type: FixType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FixType {
    ImportRemove,
    ImportComment,
    ImportRewrite,
    FileMove,
    CodeRefactor,
    TodoMark,
}

impl FixType {
    pub fn label(&self) -> &str {
        match self {
            FixType::ImportRemove => "移除导入",
            FixType::ImportComment => "注释导入",
            FixType::ImportRewrite => "重写导入",
            FixType::FileMove => "移动文件",
            FixType::CodeRefactor => "代码重构",
            FixType::TodoMark => "标记TODO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSuggestion {
    pub file: String,
    pub line: Option<usize>,
    pub title: String,
    pub description: String,
    pub severity: SuggestionSeverity,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl SuggestionSeverity {
    pub fn label(&self) -> &str {
        match self {
            SuggestionSeverity::Critical => "🔴 严重",
            SuggestionSeverity::High => "🟠 高",
            SuggestionSeverity::Medium => "🟡 中",
            SuggestionSeverity::Low => "🟢 低",
        }
    }
}

pub struct ArchitectureFixer;

impl ArchitectureFixer {
    pub fn new() -> Self {
        Self
    }

    pub fn fix(&self, project_path: &str, dry_run: bool) -> CellResult<FixResult> {
        let root = Path::new(project_path);
        let rules = ArchitectureRules::default();
        let validation = super::arch_service::validate_architecture(root, &rules);

        let mut changes = Vec::new();
        let mut fixed = 0;
        let mut skipped = 0;
        let mut failed = 0;

        let mut violations_by_file: HashMap<&str, Vec<&Violation>> = HashMap::new();
        for v in &validation.violations {
            violations_by_file
                .entry(v.file.as_str())
                .or_default()
                .push(v);
        }

        for violation in &validation.violations {
            match self.attempt_fix(violation, project_path, dry_run) {
                Ok(Some(change)) => {
                    if change.success && !dry_run {
                        fixed += 1;
                    }
                    changes.push(change);
                }
                Ok(None) => {
                    skipped += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        let suggestions = self.generate_suggestions(project_path, &validation.violations);

        Ok(FixResult {
            fixed,
            skipped,
            failed,
            changes,
            suggestions,
        })
    }

    fn attempt_fix(&self, violation: &Violation, project_path: &str, dry_run: bool) -> CellResult<Option<FixChange>> {
        match violation.rule.as_str() {
            "layer_dependency" => self.fix_layer_dependency(violation, project_path, dry_run),
            "module_naming" => self.fix_naming(violation, project_path, dry_run),
            "circular_dependency" => self.fix_circular_dependency(violation, project_path, dry_run),
            _ => Ok(None),
        }
    }

    fn fix_layer_dependency(&self, violation: &Violation, project_path: &str, dry_run: bool) -> CellResult<Option<FixChange>> {
        let file_path = Path::new(project_path).join(violation.file.trim_start_matches("./").trim_start_matches(project_path));
        let file_path = if file_path.exists() {
            file_path
        } else {
            Path::new(project_path).join(&violation.file)
        };
        
        if !file_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let line_idx = violation.line.saturating_sub(1);
        if line_idx >= lines.len() {
            return Ok(None);
        }

        let old_line = lines[line_idx].to_string();
        
        let from_layer = self.extract_layer(violation.from_module.as_str());
        let to_layer = self.extract_layer(violation.to_module.as_str());

        let (new_line, description, fix_type) = if let (Some(from), Some(to)) = (from_layer, to_layer) {
            self.generate_fix_line(&old_line, from, to, violation.file.as_str())
        } else {
            return Ok(None);
        };

        let mut change = FixChange {
            file: file_path.to_string_lossy().to_string(),
            line: violation.line,
            old_content: old_line.clone(),
            new_content: new_line.clone(),
            description,
            success: false,
            fix_type,
        };

        if !dry_run {
            let mut new_lines = lines.to_vec();
            new_lines[line_idx] = &new_line;
            let new_content = new_lines.join("\n");
            
            match std::fs::write(&file_path, new_content) {
                Ok(_) => change.success = true,
                Err(_) => change.success = false,
            }
        } else {
            change.success = true;
        }

        Ok(Some(change))
    }

    fn fix_naming(&self, violation: &Violation, project_path: &str, dry_run: bool) -> CellResult<Option<FixChange>> {
        let file_path = Path::new(project_path).join(violation.file.trim_start_matches("./"));
        if !file_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let line_idx = violation.line.saturating_sub(1);
        if line_idx >= lines.len() {
            return Ok(None);
        }

        let old_line = lines[line_idx].to_string();
        let suggested_name = self.suggest_correct_name(&violation.to_module);
        
        let (new_line, description) = if let Some(suggested) = suggested_name {
            let new_line = old_line.replace(violation.to_module.as_str(), &suggested);
            (new_line, format!("重命名为符合规范的名称: {}", suggested))
        } else {
            (
                format!("// FIXME: 命名违规 - {}", old_line.trim()),
                "标记为命名违规，需要手动修复".to_string(),
            )
        };

        let mut change = FixChange {
            file: file_path.to_string_lossy().to_string(),
            line: violation.line,
            old_content: old_line.clone(),
            new_content: new_line.clone(),
            description,
            success: false,
            fix_type: FixType::TodoMark,
        };

        if !dry_run {
            let mut new_lines = lines.to_vec();
            new_lines[line_idx] = &new_line;
            let new_content = new_lines.join("\n");
            if std::fs::write(&file_path, new_content).is_ok() {
                change.success = true;
            }
        } else {
            change.success = true;
        }

        Ok(Some(change))
    }

    fn fix_circular_dependency(&self, _violation: &Violation, _project_path: &str, _dry_run: bool) -> CellResult<Option<FixChange>> {
        Ok(None)
    }

    fn extract_layer(&self, module_path: &str) -> Option<&str> {
        let parts: Vec<&str> = module_path.split(&['/', '\\', ':'][..]).filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return None;
        }
        
        let layer_names = ["domain", "application", "adapters", "interfaces"];
        for part in &parts {
            for layer in &layer_names {
                if part == layer {
                    return Some(layer);
                }
            }
        }
        
        None
    }

    fn generate_fix_line(&self, line: &str, from_layer: &str, to_layer: &str, file: &str) -> (String, String, FixType) {
        let layer_order = ["domain", "application", "adapters", "interfaces"];
        let from_idx = layer_order.iter().position(|l| l == &from_layer).unwrap_or(0);
        let to_idx = layer_order.iter().position(|l| l == &to_layer).unwrap_or(0);

        if from_idx > to_idx {
            (
                format!("// FIXME: 架构违规 - {} 禁止依赖 {}: {}", from_layer, to_layer, line.trim()),
                format!("标记违规: {} 层 → {} 层依赖", from_layer, to_layer),
                FixType::TodoMark,
            )
        } else if line.trim().starts_with("use ") || line.trim().starts_with("pub use ") {
            (
                format!("// TODO: 需要移至正确层 - {}", line.trim()),
                format!("标记待迁移: {} → {} (文件位于: {})", from_layer, to_layer, file),
                FixType::ImportComment,
            )
        } else {
            (
                line.to_string(),
                format!("需要手动重构: {} 不能依赖 {}", from_layer, to_layer),
                FixType::CodeRefactor,
            )
        }
    }

    fn suggest_correct_name(&self, _name: &str) -> Option<String> {
        None
    }

    fn generate_suggestions(&self, project_path: &str, violations: &[Violation]) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();

        let layer_violations: HashMap<&str, usize> = violations
            .iter()
            .filter(|v| v.rule == "layer_dependency")
            .fold(HashMap::new(), |mut acc, v| {
                *acc.entry(v.from_module.as_str()).or_insert(0) += 1;
                acc
            });

        for (module, count) in layer_violations.iter() {
            if *count >= 3 {
                suggestions.push(FixSuggestion {
                    file: module.to_string(),
                    line: None,
                    title: format!("模块 '{}' 存在大量架构违规", module),
                    description: format!("该模块有 {} 个跨层依赖违规，建议重构到正确的层级", count),
                    severity: if *count >= 5 { SuggestionSeverity::Critical } else { SuggestionSeverity::High },
                    estimated_effort: "2-4 小时".to_string(),
                });
            }
        }

        let naming_violations: Vec<_> = violations.iter().filter(|v| v.rule == "module_naming").collect();
        if !naming_violations.is_empty() {
            suggestions.push(FixSuggestion {
                file: project_path.to_string(),
                line: None,
                title: format!("发现 {} 个命名违规", naming_violations.len()),
                description: "建议统一模块和文件命名规范，使用 snake_case".to_string(),
                severity: SuggestionSeverity::Medium,
                estimated_effort: "1 小时".to_string(),
            });
        }

        let circular_violations: Vec<_> = violations.iter().filter(|v| v.rule == "circular_dependency").collect();
        if !circular_violations.is_empty() {
            suggestions.push(FixSuggestion {
                file: project_path.to_string(),
                line: None,
                title: format!("检测到 {} 个循环依赖", circular_violations.len()),
                description: "循环依赖会增加系统复杂度，建议通过接口抽象解耦".to_string(),
                severity: SuggestionSeverity::High,
                estimated_effort: "4-8 小时".to_string(),
            });
        }

        suggestions.sort_by(|a, b| {
            let ord_a = match a.severity {
                SuggestionSeverity::Critical => 0,
                SuggestionSeverity::High => 1,
                SuggestionSeverity::Medium => 2,
                SuggestionSeverity::Low => 3,
            };
            let ord_b = match b.severity {
                SuggestionSeverity::Critical => 0,
                SuggestionSeverity::High => 1,
                SuggestionSeverity::Medium => 2,
                SuggestionSeverity::Low => 3,
            };
            ord_a.cmp(&ord_b)
        });

        suggestions
    }

    pub fn format_result(&self, result: &FixResult) -> String {
        let mut output = String::new();

        output.push_str("\n🔧 架构自动修复结果\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!("✅ 已修复: {} 个违规\n", result.fixed));
        output.push_str(&format!("⏭️  跳过: {} 个（无法自动修复）\n", result.skipped));
        output.push_str(&format!("❌ 失败: {} 个\n", result.failed));

        if !result.changes.is_empty() {
            output.push_str("\n📋 变更详情:\n");
            output.push_str("──────────────────────────────────────────────────────────────\n\n");

            for change in &result.changes {
                let status = if change.success { "✅" } else { "❌" };
                output.push_str(&format!("{} {}:{} [{}]\n", 
                    status, change.file, change.line, change.fix_type.label()));
                output.push_str(&format!("   {}\n", change.description));
                output.push_str(&format!("   - 原: {}\n", change.old_content.trim()));
                output.push_str(&format!("   + 新: {}\n", change.new_content.trim()));
                output.push_str("\n");
            }
        }

        if !result.suggestions.is_empty() {
            output.push_str("\n💡 改进建议:\n");
            output.push_str("──────────────────────────────────────────────────────────────\n\n");

            for (i, suggestion) in result.suggestions.iter().enumerate() {
                output.push_str(&format!("  {}. {} - {}\n", i + 1, suggestion.severity.label(), suggestion.title));
                output.push_str(&format!("     {}\n", suggestion.description));
                output.push_str(&format!("     预估工作量: {}\n", suggestion.estimated_effort));
                output.push_str("\n");
            }
        }

        output.push_str("──────────────────────────────────────────────────────────────\n");
        output.push_str("💡 提示: 部分违规需要手动修复，建议优先处理严重级别高的建议。\n");

        output
    }
}

impl Default for ArchitectureFixer {
    fn default() -> Self {
        Self::new()
    }
}
