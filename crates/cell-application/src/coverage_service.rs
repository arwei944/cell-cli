use cell_domain::errors::CellResult;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_files: usize,
    pub total_lines: usize,
    pub tested_files: usize,
    pub total_test_files: usize,
    pub total_test_functions: usize,
    pub modules: HashMap<String, ModuleCoverage>,
    pub overall_coverage: f64,
    pub uncovered_files: Vec<String>,
    pub low_coverage_files: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCoverage {
    pub name: String,
    pub layer: String,
    pub file_count: usize,
    pub test_file_count: usize,
    pub test_function_count: usize,
    pub lines_of_code: usize,
    pub test_lines: usize,
    pub coverage_ratio: f64,
    pub files: Vec<FileCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub path: String,
    pub lines: usize,
    pub has_tests: bool,
    pub test_file: Option<String>,
    pub test_functions: usize,
}

pub struct CoverageService {
    root: String,
}

impl CoverageService {
    pub fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
        }
    }

    pub fn analyze(&self) -> CellResult<CoverageReport> {
        let root_path = Path::new(&self.root);
        let src_dir = root_path.join("src");

        if !src_dir.exists() {
            return Err(cell_domain::errors::CellError::NotFound(
                "src directory not found".to_string(),
            ));
        }

        let mut source_files: Vec<String> = Vec::new();
        let mut test_files: Vec<String> = Vec::new();

        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = path
                .strip_prefix(root_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if file_name.contains("test") || file_name == "tests.rs" {
                test_files.push(rel_path);
            } else {
                source_files.push(rel_path);
            }
        }

        let mut modules: HashMap<String, ModuleCoverage> = HashMap::new();
        let mut total_test_functions = 0;
        let mut _total_test_lines = 0;
        let mut total_lines = 0;

        for file_path in &source_files {
            let full_path = root_path.join(file_path);
            let content = std::fs::read_to_string(&full_path).unwrap_or_default();
            let line_count = content.lines().count();
            total_lines += line_count;

            let module_name = self.get_module_name(file_path);
            let layer = self.detect_layer(file_path);

            let module = modules.entry(module_name.clone()).or_insert(ModuleCoverage {
                name: module_name.clone(),
                layer,
                file_count: 0,
                test_file_count: 0,
                test_function_count: 0,
                lines_of_code: 0,
                test_lines: 0,
                coverage_ratio: 0.0,
                files: Vec::new(),
            });

            module.file_count += 1;
            module.lines_of_code += line_count;

            let test_file = self.find_test_file(file_path, &test_files);
            let (has_tests, test_func_count) = if let Some(ref tf) = test_file {
                let tf_path = root_path.join(tf);
                let tf_content = std::fs::read_to_string(&tf_path).unwrap_or_default();
                let func_count = self.count_test_functions(&tf_content);
                (true, func_count)
            } else {
                let inline_tests = self.count_test_functions(&content);
                (inline_tests > 0, inline_tests)
            };

            module.files.push(FileCoverage {
                path: file_path.clone(),
                lines: line_count,
                has_tests,
                test_file: test_file.clone(),
                test_functions: test_func_count,
            });
        }

        for test_path in &test_files {
            let full_path = root_path.join(test_path);
            let content = std::fs::read_to_string(&full_path).unwrap_or_default();
            let line_count = content.lines().count();
            let func_count = self.count_test_functions(&content);
            total_test_functions += func_count;
            _total_test_lines += line_count;

            let module_name = self.get_module_name(test_path);
            if let Some(module) = modules.get_mut(&module_name) {
                module.test_file_count += 1;
                module.test_function_count += func_count;
                module.test_lines += line_count;
            }
        }

        for module in modules.values_mut() {
            let test_func_per_file = if module.file_count > 0 {
                module.test_function_count as f64 / module.file_count as f64
            } else {
                0.0
            };
            let files_with_tests = module.files.iter().filter(|f| f.has_tests).count() as f64;
            let file_coverage = if module.file_count > 0 {
                files_with_tests / module.file_count as f64
            } else {
                0.0
            };
            module.coverage_ratio = test_func_per_file.min(1.0).mul_add(0.4, file_coverage * 0.6) * 100.0;
        }

        let tested_files = modules.values()
            .map(|m| m.files.iter().filter(|f| f.has_tests).count())
            .sum::<usize>();

        let overall_coverage = if source_files.is_empty() {
            0.0
        } else {
            tested_files as f64 / source_files.len() as f64 * 100.0
        };

        let mut uncovered_files: Vec<String> = modules.values()
            .flat_map(|m| m.files.iter().filter(|f| !f.has_tests).map(|f| f.path.clone()))
            .collect();
        uncovered_files.sort();

        let mut low_coverage_files: Vec<(String, f64)> = modules.values()
            .flat_map(|m| m.files.iter().filter(|f| f.test_functions < 2 && f.lines > 50).map(|f| (f.path.clone(), f.test_functions as f64)))
            .collect();
        low_coverage_files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(CoverageReport {
            total_files: source_files.len(),
            total_lines,
            tested_files,
            total_test_files: test_files.len(),
            total_test_functions,
            modules,
            overall_coverage,
            uncovered_files,
            low_coverage_files,
        })
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
            format!("{top}::{second}")
        }
    }

    fn detect_layer(&self, path: &str) -> String {
        let path = path.replace('\\', "/");
        if path.starts_with("domain/") || path.contains("/domain/") {
            "domain".to_string()
        } else if path.starts_with("application/") || path.contains("/application/") {
            "application".to_string()
        } else if path.starts_with("adapters/") || path.contains("/adapters/") {
            "adapters".to_string()
        } else if path.starts_with("interfaces/") || path.contains("/interfaces/") {
            "interfaces".to_string()
        } else {
            "other".to_string()
        }
    }

    fn find_test_file(&self, source_file: &str, test_files: &[String]) -> Option<String> {
        let stem = source_file
            .trim_end_matches(".rs")
            .replace("src/", "");

        for tf in test_files {
            if tf.contains(&stem) || tf.ends_with(&format!("{}.rs", stem.split('/').next_back().unwrap_or(""))) {
                return Some(tf.clone());
            }
        }

        None
    }

    fn count_test_functions(&self, content: &str) -> usize {
        let mut count = 0;
        let mut in_test_module = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("#[test]") {
                in_test_module = true;
            }
            if in_test_module && trimmed.starts_with("fn ") && trimmed.contains("test") {
                count += 1;
            }
            if in_test_module && trimmed.starts_with("#[test]") {
                count += 1;
            }
        }

        count
    }

    pub fn format_report(&self, report: &CoverageReport) -> String {
        let mut output = String::new();

        output.push_str("\n📊 测试覆盖率报告\n\n");
        output.push_str(&format!("  总体覆盖率: {:.1}%\n\n", report.overall_coverage));

        output.push_str("  ┌─────────────────────────────────────────────────────┐\n");
        output.push_str("  │                   概览统计                          │\n");
        output.push_str("  ├─────────────────────────────────────────────────────┤\n");
        output.push_str(&format!("  │  源文件总数:  {:<36}│\n", report.total_files));
        output.push_str(&format!("  │  有测试文件:  {:<36}│\n", report.tested_files));
        output.push_str(&format!("  │  测试文件数:  {:<36}│\n", report.total_test_files));
        output.push_str(&format!("  │  测试函数数:  {:<36}│\n", report.total_test_functions));
        output.push_str(&format!("  │  代码总行数:  {:<36}│\n", report.total_lines));
        output.push_str("  └─────────────────────────────────────────────────────┘\n\n");

        output.push_str("  📁 各模块覆盖率\n\n");

        let mut module_list: Vec<&ModuleCoverage> = report.modules.values().collect();
        module_list.sort_by(|a, b| b.coverage_ratio.partial_cmp(&a.coverage_ratio).unwrap_or(std::cmp::Ordering::Equal));

        output.push_str("  │ 模块                        │ 层级        │ 覆盖率 │ 文件 │ 测试函数 │\n");
        output.push_str("  ├─────────────────────────────┼─────────────┼────────┼──────┼──────────┤\n");

        for m in &module_list {
            let bar_len = (m.coverage_ratio / 10.0) as usize;
            let bar = format!("{}{}", "█".repeat(bar_len), "░".repeat(10 - bar_len));
            output.push_str(&format!(
                "  │ {:<27} │ {:<11} │ {} {:>4.1}% │ {:>4} │ {:>8} │\n",
                m.name,
                m.layer,
                bar,
                m.coverage_ratio,
                m.file_count,
                m.test_function_count
            ));
        }
        output.push('\n');

        if !report.uncovered_files.is_empty() {
            output.push_str(&format!("  ⚠️  无测试的文件 ({}/{})\n\n", report.uncovered_files.len(), report.total_files));
            for f in &report.uncovered_files {
                output.push_str(&format!("     • {f}\n"));
            }
            output.push('\n');
        }

        if !report.low_coverage_files.is_empty() {
            output.push_str("  💡 建议补充测试的文件（代码多但测试少）\n\n");
            for (f, _) in report.low_coverage_files.iter().take(10) {
                output.push_str(&format!("     • {f}\n"));
            }
            output.push('\n');
        }

        output
    }
}
