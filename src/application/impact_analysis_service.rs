use crate::domain::errors::CellResult;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub changed_files: Vec<ChangedFile>,
    pub impacted_modules: Vec<ImpactedModule>,
    pub impacted_tests: Vec<String>,
    pub risk_level: ImpactRisk,
    pub summary: ImpactSummary,
    pub suggested_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub module: String,
    pub layer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactedModule {
    pub name: String,
    pub layer: String,
    pub impact_reason: String,
    pub impact_level: ImpactLevel,
    pub dependent_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub total_changed_files: usize,
    pub total_impacted_modules: usize,
    pub total_lines_changed: usize,
    pub layers_affected: Vec<String>,
    pub cross_layer_changes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactRisk {
    Low,
    Medium,
    High,
}

pub struct ImpactAnalysisService {
    root: String,
}

impl ImpactAnalysisService {
    pub fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
        }
    }

    pub fn analyze(&self, base_ref: Option<&str>) -> CellResult<ImpactAnalysis> {
        let _root_path = Path::new(&self.root);

        let changed_files = self.get_changed_files(base_ref)?;

        let mut modules_set: HashSet<String> = HashSet::new();
        let mut layers_set: HashSet<String> = HashSet::new();
        let mut total_lines = 0;

        for f in &changed_files {
            modules_set.insert(f.module.clone());
            layers_set.insert(f.layer.clone());
            total_lines += f.lines_added + f.lines_deleted;
        }

        let impacted_modules = self.analyze_impact(&changed_files, &modules_set)?;

        let impacted_tests = self.find_impacted_tests(&changed_files, &impacted_modules);

        let suggested_tests = self.suggest_tests(&changed_files, &impacted_modules);

        let cross_layer_changes = layers_set.len() > 1;

        let risk_level = self.assess_risk(&changed_files, &impacted_modules, cross_layer_changes);

        let summary = ImpactSummary {
            total_changed_files: changed_files.len(),
            total_impacted_modules: impacted_modules.len(),
            total_lines_changed: total_lines,
            layers_affected: layers_set.into_iter().collect(),
            cross_layer_changes,
        };

        Ok(ImpactAnalysis {
            changed_files,
            impacted_modules,
            impacted_tests,
            risk_level,
            summary,
            suggested_tests,
        })
    }

    fn get_changed_files(&self, base_ref: Option<&str>) -> CellResult<Vec<ChangedFile>> {
        let _root_path = Path::new(&self.root);
        let mut files = Vec::new();

        let diff_output = if let Some(reference) = base_ref {
            self.run_git(&["diff", "--name-status", reference])?
        } else {
            let staged = self.run_git(&["diff", "--cached", "--name-status"])?;
            let unstaged = self.run_git(&["diff", "--name-status"])?;
            format!("{}\n{}", staged, unstaged)
        };

        for line in diff_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let change_type = match parts[0].chars().next().unwrap_or(' ') {
                'A' => ChangeType::Added,
                'M' => ChangeType::Modified,
                'D' => ChangeType::Deleted,
                'R' => ChangeType::Renamed,
                _ => ChangeType::Modified,
            };

            let file_path = if parts.len() >= 3 {
                parts[2].to_string()
            } else {
                parts[1].to_string()
            };

            if !file_path.ends_with(".rs") {
                continue;
            }

            let module = self.get_module_name(&file_path);
            let layer = self.detect_layer(&file_path);

            let (added, deleted) = self.get_line_stats(&file_path, base_ref)?;

            files.push(ChangedFile {
                path: file_path,
                change_type,
                lines_added: added,
                lines_deleted: deleted,
                module,
                layer,
            });
        }

        Ok(files)
    }

    fn get_line_stats(&self, file_path: &str, base_ref: Option<&str>) -> CellResult<(usize, usize)> {
        let output = if let Some(reference) = base_ref {
            self.run_git(&["diff", "--numstat", reference, "--", file_path])?
        } else {
            self.run_git(&["diff", "--numstat", "--", file_path])?
        };

        if let Some(line) = output.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let added = parts[0].parse::<usize>().unwrap_or(0);
                let deleted = parts[1].parse::<usize>().unwrap_or(0);
                return Ok((added, deleted));
            }
        }

        Ok((0, 0))
    }

    fn analyze_impact(
        &self,
        _changed_files: &[ChangedFile],
        changed_modules: &HashSet<String>,
    ) -> CellResult<Vec<ImpactedModule>> {
        let mut impacted: HashMap<String, ImpactedModule> = HashMap::new();
        let root_path = Path::new(&self.root);
        let src_dir = root_path.join("src");

        if !src_dir.exists() {
            return Ok(Vec::new());
        }

        let mut module_dependents: HashMap<String, Vec<String>> = HashMap::new();

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let content = std::fs::read_to_string(path).unwrap_or_default();
            let file_module = self.get_module_name_from_path(path, &src_dir);

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("use crate::") {
                    let dep_path = trimmed.trim_start_matches("use crate::")
                        .split(';')
                        .next()
                        .unwrap_or("");

                    let parts: Vec<&str> = dep_path.split("::").collect();
                    if parts.len() >= 2 {
                        let dep_module = format!("{}::{}", parts[0], parts[1]);
                        module_dependents
                            .entry(dep_module)
                            .or_insert_with(Vec::new)
                            .push(file_module.clone());
                    }
                }
            }
        }

        for module in changed_modules {
            let layer = self.get_module_layer(module);
            impacted.insert(module.clone(), ImpactedModule {
                name: module.clone(),
                layer: layer.clone(),
                impact_reason: "直接修改".to_string(),
                impact_level: ImpactLevel::High,
                dependent_files: Vec::new(),
            });

            if let Some(dependents) = module_dependents.get(module) {
                for dep in dependents {
                    if !impacted.contains_key(dep) {
                        let dep_layer = self.get_module_layer(dep);
                        let impact_level = if layer == "domain" {
                            ImpactLevel::High
                        } else if dep_layer == "interfaces" {
                            ImpactLevel::Medium
                        } else {
                            ImpactLevel::Medium
                        };

                        impacted.insert(dep.clone(), ImpactedModule {
                            name: dep.clone(),
                            layer: dep_layer,
                            impact_reason: format!("依赖被修改的模块 {}", module),
                            impact_level,
                            dependent_files: Vec::new(),
                        });
                    }
                }
            }
        }

        let mut result: Vec<ImpactedModule> = impacted.into_values().collect();
        result.sort_by(|a, b| b.impact_level.partial_cmp(&a.impact_level).unwrap_or(std::cmp::Ordering::Equal));

        Ok(result)
    }

    fn find_impacted_tests(
        &self,
        _changed_files: &[ChangedFile],
        impacted_modules: &[ImpactedModule],
    ) -> Vec<String> {
        let mut tests = Vec::new();
        let root_path = Path::new(&self.root);
        let src_dir = root_path.join("src");

        if !src_dir.exists() {
            return tests;
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let rel_path = path.strip_prefix(root_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let has_tests = file_name.contains("test") || file_name == "tests.rs" || {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                content.contains("#[test]") || content.contains("#[cfg(test)]")
            };

            if has_tests {
                let module = self.get_module_name_from_path(path, &src_dir);
                for m in impacted_modules {
                    if module == m.name || module.starts_with(&format!("{}::", m.name)) {
                        tests.push(rel_path.clone());
                        break;
                    }
                }
            }
        }

        tests.sort();
        tests.dedup();
        tests
    }

    fn suggest_tests(
        &self,
        _changed_files: &[ChangedFile],
        impacted_modules: &[ImpactedModule],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        for m in impacted_modules {
            match m.impact_level {
                ImpactLevel::Critical | ImpactLevel::High => {
                    suggestions.push(format!("cargo test -p {} -- --nocapture", m.name));
                }
                ImpactLevel::Medium => {
                    suggestions.push(format!("cargo test {}::", m.name));
                }
                _ => {}
            }
        }

        suggestions.push("cargo test --lib".to_string());
        suggestions
    }

    fn assess_risk(
        &self,
        changed_files: &[ChangedFile],
        impacted_modules: &[ImpactedModule],
        cross_layer: bool,
    ) -> ImpactRisk {
        let mut score = 0;

        let total_lines: usize = changed_files.iter()
            .map(|f| f.lines_added + f.lines_deleted)
            .sum();

        if total_lines > 500 {
            score += 3;
        } else if total_lines > 200 {
            score += 2;
        } else if total_lines > 50 {
            score += 1;
        }

        if changed_files.len() > 10 {
            score += 2;
        } else if changed_files.len() > 5 {
            score += 1;
        }

        if cross_layer {
            score += 2;
        }

        let has_domain_changes = changed_files.iter().any(|f| f.layer == "domain");
        if has_domain_changes {
            score += 2;
        }

        let high_impact_count = impacted_modules.iter()
            .filter(|m| m.impact_level >= ImpactLevel::High)
            .count();

        if high_impact_count > 5 {
            score += 2;
        } else if high_impact_count > 2 {
            score += 1;
        }

        if score >= 6 {
            ImpactRisk::High
        } else if score >= 3 {
            ImpactRisk::Medium
        } else {
            ImpactRisk::Low
        }
    }

    fn run_git(&self, args: &[&str]) -> CellResult<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| crate::domain::errors::CellError::Io(e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn get_module_name(&self, path: &str) -> String {
        let path = path.replace('\\', "/");
        let path = path.trim_start_matches("src/");
        let parts: Vec<&str> = path.split('/').collect();

        if parts.is_empty() {
            return "root".to_string();
        }

        if parts.len() == 1 {
            if parts[0] == "main.rs" || parts[0] == "lib.rs" {
                return "root".to_string();
            }
            return parts[0].trim_end_matches(".rs").to_string();
        }

        let top = parts[0];
        let second = parts.get(1).copied().unwrap_or("");
        if second.ends_with(".rs") {
            format!("{}::{}", top, second.trim_end_matches(".rs"))
        } else {
            format!("{}::{}", top, second)
        }
    }

    fn get_module_name_from_path(&self, path: &std::path::Path, src_dir: &Path) -> String {
        let rel = path.strip_prefix(src_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        self.get_module_name(&rel)
    }

    fn detect_layer(&self, path: &str) -> String {
        let path = path.replace('\\', "/");
        if path.starts_with("src/domain/") || path.contains("/domain/") || path.starts_with("domain/") {
            "domain".to_string()
        } else if path.starts_with("src/application/") || path.contains("/application/") || path.starts_with("application/") {
            "application".to_string()
        } else if path.starts_with("src/adapters/") || path.contains("/adapters/") || path.starts_with("adapters/") {
            "adapters".to_string()
        } else if path.starts_with("src/interfaces/") || path.contains("/interfaces/") || path.starts_with("interfaces/") {
            "interfaces".to_string()
        } else {
            "other".to_string()
        }
    }

    fn get_module_layer(&self, module: &str) -> String {
        let lower = module.to_lowercase();
        if lower.starts_with("domain") {
            "domain".to_string()
        } else if lower.starts_with("application") {
            "application".to_string()
        } else if lower.starts_with("adapters") {
            "adapters".to_string()
        } else if lower.starts_with("interfaces") {
            "interfaces".to_string()
        } else {
            "other".to_string()
        }
    }

    pub fn format_report(&self, analysis: &ImpactAnalysis) -> String {
        let mut output = String::new();

        let risk_icon = match analysis.risk_level {
            ImpactRisk::High => "🔴",
            ImpactRisk::Medium => "🟡",
            ImpactRisk::Low => "🟢",
        };

        output.push_str("\n🔍 代码变更影响分析\n\n");
        output.push_str(&format!("  风险等级: {} {:?}\n\n", risk_icon, analysis.risk_level));

        output.push_str("  ┌─────────────────────────────────────────────────────┐\n");
        output.push_str("  │                   变更概览                          │\n");
        output.push_str("  ├─────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("  │  变更文件数:  {:<36}│\n", analysis.summary.total_changed_files));
        output.push_str(&format!("  │  影响模块数:  {:<36}│\n", analysis.summary.total_impacted_modules));
        output.push_str(&format!("  │  变更代码行数: {:<35}│\n", analysis.summary.total_lines_changed));
        output.push_str(&format!("  │  受影响层级:  {:<36}│\n", analysis.summary.layers_affected.join(", ")));
        output.push_str(&format!("  │  跨层变更:    {:<36}│\n", if analysis.summary.cross_layer_changes { "是 ⚠️" } else { "否" }));
        output.push_str("  └─────────────────────────────────────────────────────┘\n\n");

        if !analysis.changed_files.is_empty() {
            output.push_str("  📝 变更文件列表\n\n");
            for f in &analysis.changed_files {
                let type_icon = match f.change_type {
                    ChangeType::Added => "➕",
                    ChangeType::Modified => "✏️",
                    ChangeType::Deleted => "🗑️",
                    ChangeType::Renamed => "📝",
                };
                output.push_str(&format!(
                    "     {} {:<40} (+{}/-{}) [{}]\n",
                    type_icon,
                    f.path,
                    f.lines_added,
                    f.lines_deleted,
                    f.layer
                ));
            }
            output.push('\n');
        }

        if !analysis.impacted_modules.is_empty() {
            output.push_str("  📊 受影响模块\n\n");
            output.push_str("  │ 模块                        │ 层级        │ 影响等级 │ 原因\n");
            output.push_str("  ├─────────────────────────────┼─────────────┼──────────┼────────\n");
            for m in &analysis.impacted_modules {
                let level_icon = match m.impact_level {
                    ImpactLevel::Critical => "🔴 严重",
                    ImpactLevel::High => "🟠 高",
                    ImpactLevel::Medium => "🟡 中",
                    ImpactLevel::Low => "🟢 低",
                };
                output.push_str(&format!(
                    "  │ {:<27} │ {:<11} │ {:<8} │ {}\n",
                    m.name,
                    m.layer,
                    level_icon,
                    m.impact_reason
                ));
            }
            output.push('\n');
        }

        if !analysis.impacted_tests.is_empty() {
            output.push_str(&format!("  ✅ 相关测试文件 ({})\n\n", analysis.impacted_tests.len()));
            for t in &analysis.impacted_tests {
                output.push_str(&format!("     • {}\n", t));
            }
            output.push('\n');
        }

        if !analysis.suggested_tests.is_empty() {
            output.push_str("  💡 建议运行的测试\n\n");
            for t in analysis.suggested_tests.iter().take(5) {
                output.push_str(&format!("     $ {}\n", t));
            }
            output.push('\n');
        }

        output
    }
}
