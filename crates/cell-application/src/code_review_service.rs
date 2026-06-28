use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub summary: ReviewSummary,
    pub issues: Vec<ReviewIssue>,
    pub score: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_files: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub major_issues: usize,
    pub minor_issues: usize,
    pub categories: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub file: String,
    pub line: Option<usize>,
    pub severity: ReviewSeverity,
    pub category: ReviewCategory,
    pub title: String,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

impl ReviewSeverity {
    pub fn label(&self) -> &str {
        match self {
            Self::Critical => "严重",
            Self::Major => "重要",
            Self::Minor => "次要",
            Self::Info => "建议",
        }
    }

    pub fn score_weight(&self) -> f64 {
        match self {
            Self::Critical => 10.0,
            Self::Major => 5.0,
            Self::Minor => 2.0,
            Self::Info => 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ReviewCategory {
    Architecture,
    Security,
    Performance,
    Maintainability,
    Testing,
    Documentation,
    Style,
}

impl ReviewCategory {
    pub fn label(&self) -> &str {
        match self {
            Self::Architecture => "架构",
            Self::Security => "安全",
            Self::Performance => "性能",
            Self::Maintainability => "可维护性",
            Self::Testing => "测试",
            Self::Documentation => "文档",
            Self::Style => "风格",
        }
    }
}

pub struct CodeReviewService;

impl CodeReviewService {
    pub fn new() -> Self {
        Self
    }

    pub fn review_code(&self, project_path: &str, deep: bool) -> CellResult<ReviewResult> {
        let mut issues = Vec::new();

        issues.extend(self.review_architecture(project_path)?);
        issues.extend(self.review_security(project_path)?);
        issues.extend(self.review_maintainability(project_path)?);
        issues.extend(self.review_testing(project_path)?);
        issues.extend(self.review_documentation(project_path)?);

        if deep {
            issues.extend(self.review_performance(project_path)?);
        }

        let summary = Self::calculate_summary(&issues);
        let score = Self::calculate_score(&issues, summary.total_files);
        let passed = score >= 70.0;

        Ok(ReviewResult {
            summary,
            issues,
            score,
            passed,
        })
    }

    fn review_architecture(&self, project_path: &str) -> CellResult<Vec<ReviewIssue>> {
        use crate::arch_service::{ArchitectureRules, validate_architecture};
        
        let rules = ArchitectureRules::default();
        let validation = validate_architecture(Path::new(project_path), &rules);
        
        let mut issues = Vec::new();
        for violation in &validation.violations {
            issues.push(ReviewIssue {
                file: violation.file.clone(),
                line: Some(violation.line),
                severity: ReviewSeverity::Major,
                category: ReviewCategory::Architecture,
                title: format!("架构违规: {}", violation.rule),
                description: violation.message.clone(),
                suggestion: Some("请遵循 Cell Architecture 分层架构原则".to_string()),
            });
        }

        Ok(issues)
    }

    fn review_security(&self, project_path: &str) -> CellResult<Vec<ReviewIssue>> {
        let mut issues = Vec::new();
        let project_path_obj = Path::new(project_path);
        let src_dir = project_path_obj.join("src");

        if !src_dir.exists() {
            return Ok(issues);
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|e| e == "rs"))
        {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                for (i, line) in content.lines().enumerate() {
                    if line.contains("unwrap()") && !line.contains("expect") {
                        issues.push(ReviewIssue {
                            file: path.strip_prefix(project_path_obj).unwrap_or(path).to_string_lossy().to_string(),
                            line: Some(i + 1),
                            severity: ReviewSeverity::Minor,
                            category: ReviewCategory::Security,
                            title: "使用 unwrap() 可能导致 panic".to_string(),
                            description: "直接使用 unwrap() 可能在运行时导致程序崩溃".to_string(),
                            suggestion: Some("考虑使用 match 或 expect() 提供更友好的错误信息".to_string()),
                        });
                    }

                    if (line.contains("TODO") || line.contains("FIXME")) && line.contains("//") {
                        issues.push(ReviewIssue {
                            file: path.strip_prefix(project_path_obj).unwrap_or(path).to_string_lossy().to_string(),
                            line: Some(i + 1),
                            severity: ReviewSeverity::Info,
                            category: ReviewCategory::Maintainability,
                            title: "存在待处理的注释".to_string(),
                            description: line.trim().to_string(),
                            suggestion: None,
                        });
                    }
                }
            }
        }

        Ok(issues)
    }

    fn review_performance(&self, project_path: &str) -> CellResult<Vec<ReviewIssue>> {
        let mut issues = Vec::new();
        let project_path_obj = Path::new(project_path);
        let src_dir = project_path_obj.join("src");

        if !src_dir.exists() {
            return Ok(issues);
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|e| e == "rs"))
        {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                for (i, line) in content.lines().enumerate() {
                    if line.contains("clone()") && !line.contains("//") {
                        issues.push(ReviewIssue {
                            file: path.strip_prefix(project_path_obj).unwrap_or(path).to_string_lossy().to_string(),
                            line: Some(i + 1),
                            severity: ReviewSeverity::Info,
                            category: ReviewCategory::Performance,
                            title: "可能存在不必要的 clone()".to_string(),
                            description: "频繁的 clone() 操作可能影响性能，请考虑使用引用".to_string(),
                            suggestion: Some("如果可能，使用 borrow 代替 clone()".to_string()),
                        });
                    }
                }
            }
        }

        Ok(issues)
    }

    fn review_maintainability(&self, project_path: &str) -> CellResult<Vec<ReviewIssue>> {
        let mut issues = Vec::new();
        let project_path_obj = Path::new(project_path);
        let src_dir = project_path_obj.join("src");

        if !src_dir.exists() {
            return Ok(issues);
        }

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|e| e == "rs"))
        {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path) {
                let line_count = content.lines().count();
                if line_count > 500 {
                    issues.push(ReviewIssue {
                        file: path.strip_prefix(project_path_obj).unwrap_or(path).to_string_lossy().to_string(),
                        line: None,
                        severity: ReviewSeverity::Minor,
                        category: ReviewCategory::Maintainability,
                        title: format!("文件过长 ({line_count} 行)"),
                        description: "文件超过 500 行，建议拆分为多个模块".to_string(),
                        suggestion: Some("考虑将文件拆分为多个模块或子模块".to_string()),
                    });
                }

                let mut function_line_count = 0;
                let mut function_start = 0;
                let mut function_name = String::new();
                
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") && (trimmed.contains('{') || content.lines().nth(i + 1).is_some_and(|l| l.trim() == "{")) {
                        function_start = i;
                        function_line_count = 1;
                        function_name = trimmed
                            .trim_start_matches("pub ")
                            .trim_start_matches("fn ")
                            .split('(')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();
                    } else if function_line_count > 0 {
                        function_line_count += 1;
                        if line.trim() == "}" {
                            if function_line_count > 80 {
                                issues.push(ReviewIssue {
                                    file: path.strip_prefix(project_path_obj).unwrap_or(path).to_string_lossy().to_string(),
                                    line: Some(function_start + 1),
                                    severity: ReviewSeverity::Minor,
                                    category: ReviewCategory::Maintainability,
                                    title: format!("函数过长: {function_name} ({function_line_count} 行)"),
                                    description: "函数超过 80 行，建议拆分".to_string(),
                                    suggestion: Some("考虑将函数拆分为多个小函数".to_string()),
                                });
                            }
                            function_line_count = 0;
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn review_testing(&self, project_path: &str) -> CellResult<Vec<ReviewIssue>> {
        let mut issues = Vec::new();
        let project_path_obj = Path::new(project_path);
        let src_dir = project_path_obj.join("src");

        if !src_dir.exists() {
            return Ok(issues);
        }

        let mut total_files = 0;
        let mut files_with_tests = 0;

        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|e| e == "rs"))
        {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            if file_name == "lib.rs" || file_name == "main.rs" || file_name == "mod.rs" {
                continue;
            }
            total_files += 1;
            
            if let Ok(content) = std::fs::read_to_string(path)
                && (content.contains("#[cfg(test)]") || content.contains("#[test]")) {
                    files_with_tests += 1;
                }
        }

        if total_files > 5 && files_with_tests < total_files / 3 {
            issues.push(ReviewIssue {
                file: "src/".to_string(),
                line: None,
                severity: ReviewSeverity::Major,
                category: ReviewCategory::Testing,
                title: "测试覆盖率不足".to_string(),
                description: format!("共 {total_files} 个源文件，仅 {files_with_tests} 个包含测试"),
                suggestion: Some("建议为核心模块添加单元测试".to_string()),
            });
        }

        Ok(issues)
    }

    fn review_documentation(&self, project_path: &str) -> CellResult<Vec<ReviewIssue>> {
        let mut issues = Vec::new();
        let project_path_obj = Path::new(project_path);
        let src_dir = project_path_obj.join("src");

        if !src_dir.exists() {
            return Ok(issues);
        }

        let lib_path = src_dir.join("lib.rs");
        if lib_path.exists()
            && let Ok(content) = std::fs::read_to_string(&lib_path)
                && !content.starts_with("//!") {
                    issues.push(ReviewIssue {
                        file: "src/lib.rs".to_string(),
                        line: None,
                        severity: ReviewSeverity::Minor,
                        category: ReviewCategory::Documentation,
                        title: "缺少库级文档".to_string(),
                        description: "lib.rs 顶部没有文档注释".to_string(),
                        suggestion: Some("建议添加 //! 开头的库级文档注释".to_string()),
                    });
                }

        Ok(issues)
    }

    fn calculate_summary(issues: &[ReviewIssue]) -> ReviewSummary {
        let mut categories = std::collections::HashMap::new();
        let mut critical = 0;
        let mut major = 0;
        let mut minor = 0;

        let mut files = std::collections::HashSet::new();
        for issue in issues {
            files.insert(issue.file.clone());
            *categories.entry(issue.category.label().to_string()).or_insert(0) += 1;
            
            match issue.severity {
                ReviewSeverity::Critical => critical += 1,
                ReviewSeverity::Major => major += 1,
                ReviewSeverity::Minor => minor += 1,
                ReviewSeverity::Info => {}
            }
        }

        ReviewSummary {
            total_files: files.len(),
            total_issues: issues.len(),
            critical_issues: critical,
            major_issues: major,
            minor_issues: minor,
            categories,
        }
    }

    fn calculate_score(issues: &[ReviewIssue], total_files: usize) -> f64 {
        let total_penalty: f64 = issues.iter().map(|i| i.severity.score_weight()).sum();
        let base_score = 100.0;
        let file_factor = if total_files > 0 { total_files as f64 } else { 1.0 };
        let score = base_score - (total_penalty / file_factor);
        score.max(0.0).min(100.0)
    }

    pub fn generate_review_report(&self, result: &ReviewResult) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("  综合评分: {:.1}/100  ", result.score));
        if result.passed {
            report.push_str("✅ 通过\n");
        } else {
            report.push_str("❌ 未通过\n");
        }
        report.push('\n');

        report.push_str("  问题统计:\n");
        report.push_str(&format!("    🔴 严重: {}\n", result.summary.critical_issues));
        report.push_str(&format!("    🟠 重要: {}\n", result.summary.major_issues));
        report.push_str(&format!("    🟡 次要: {}\n", result.summary.minor_issues));
        report.push_str(&format!("    💡 信息: {}\n", result.summary.total_issues - result.summary.critical_issues - result.summary.major_issues - result.summary.minor_issues));
        report.push('\n');

        if !result.summary.categories.is_empty() {
            report.push_str("  分类统计:\n");
            for (cat, count) in &result.summary.categories {
                report.push_str(&format!("    • {cat}: {count}\n"));
            }
            report.push('\n');
        }

        if !result.issues.is_empty() {
            report.push_str("  问题详情 (前20条):\n");
            for (i, issue) in result.issues.iter().take(20).enumerate() {
                let line_str = issue.line.map(|l| format!(":{l}")).unwrap_or_default();
                let icon = match issue.severity {
                    ReviewSeverity::Critical => "🔴",
                    ReviewSeverity::Major => "🟠",
                    ReviewSeverity::Minor => "🟡",
                    ReviewSeverity::Info => "💡",
                };
                report.push_str(&format!("\n  {}. {} [{}] {}\n", i + 1, icon, issue.category.label(), issue.title));
                report.push_str(&format!("     文件: {}{}\n", issue.file, line_str));
                report.push_str(&format!("     描述: {}\n", issue.description));
                if let Some(suggestion) = &issue.suggestion {
                    report.push_str(&format!("     建议: {suggestion}\n"));
                }
            }
        }

        report
    }
}

impl Default for CodeReviewService {
    fn default() -> Self {
        Self::new()
    }
}
