//! 代码质量检测器：主入口和检查协调器
//! Code quality checker: main entry and check coordinator

pub mod analysis;
pub mod checks;
pub mod format;
pub mod issues;
pub mod types;

pub use types::{
    Category, FileReport, FnInfo, Grade, Issue, Severity, SimplicityReport, StructInfo, Summary,
};

use crate::domain::errors::CellResult;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// 代码质量检查器
/// Code quality checker
pub struct SimplicityChecker {
    /// 单文件最大行数 (默认 300)
    pub max_file_lines: usize,
    /// 单函数最大行数 (默认 30)
    pub max_fn_lines: usize,
    /// 结构体最大字段数 (默认 8)
    pub max_struct_fields: usize,
    /// 最小注释率 (默认 0.10)
    pub min_comment_ratio: f64,
    /// 最大嵌套深度 (默认 3)
    pub max_nesting: usize,
    /// 函数最大参数数 (默认 3)
    pub max_args: usize,
    /// 最大圈复杂度 (默认 6)
    pub max_complexity: usize,
    /// 最大魔法数 (默认 1)
    pub max_magic_numbers: usize,
    /// 最大 TODO 数 (默认 2)
    pub max_todos: usize,
    /// 最大 unwrap() 使用 (默认 0)
    pub max_unwraps: usize,
    /// 单函数最大 clone() 调用 (默认 2)
    pub max_clones_per_fn: usize,
}

impl Default for SimplicityChecker {
    fn default() -> Self {
        Self {
            max_file_lines: 500,
            max_fn_lines: 30,
            max_struct_fields: 8,
            min_comment_ratio: 0.10,
            max_nesting: 3,
            max_args: 3,
            max_complexity: 6,
            max_magic_numbers: 1,
            max_todos: 2,
            max_unwraps: 0,
            max_clones_per_fn: 2,
        }
    }
}

impl SimplicityChecker {
    pub fn new() -> Self { Self::default() }

    /// 严格模式 - 更严格的限制
    pub fn strict() -> Self {
        Self {
            max_file_lines: 300,
            max_fn_lines: 20,
            max_struct_fields: 6,
            min_comment_ratio: 0.15,
            max_nesting: 2,
            max_args: 3,
            max_complexity: 4,
            max_magic_numbers: 0,
            max_todos: 0,
            max_unwraps: 0,
            max_clones_per_fn: 1,
        }
    }

    pub fn with_max_file_lines(mut self, lines: usize) -> Self {
        self.max_file_lines = lines;
        self
    }

    pub fn with_max_fn_lines(mut self, lines: usize) -> Self {
        self.max_fn_lines = lines;
        self
    }

    /// 执行代码质量检查，遍历指定根目录下 src 目录的所有 .rs 文件
    pub fn check(&self, root: &str) -> CellResult<SimplicityReport> {
        let src = Path::new(root).join("src");
        if !src.exists() {
            return Err(crate::domain::errors::CellError::NotFound("src directory not found".into()));
        }

        let mut files = Vec::new();
        let mut all_issues = Vec::new();
        let mut total_lines = 0;
        let mut total_fns = 0;

        for entry in WalkDir::new(&src).into_iter().filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") { continue; }

            let rel = path.strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            let content = std::fs::read_to_string(path).unwrap_or_default();
            let file_report = self.analyze_file(&rel, &content);

            total_lines += file_report.lines;
            total_fns += file_report.fn_count;
            all_issues.extend(file_report.issues.clone());
            files.push(file_report);
        }

        files.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

        let score = if files.is_empty() { 100.0 }
            else { files.iter().map(|f| f.score).sum::<f64>() / files.len() as f64 };

        let grade = match score as i32 {
            90..=100 => Grade::S,
            80..=89 => Grade::A,
            70..=79 => Grade::B,
            60..=69 => Grade::C,
            50..=59 => Grade::D,
            _ => Grade::F,
        };

        let summary = build_summary(&files, &all_issues, self);
        let dimension_scores = build_dimension_scores(&all_issues, self);

        Ok(SimplicityReport {
            total_files: files.len(),
            total_lines,
            total_functions: total_fns,
            score,
            grade,
            files,
            issues: all_issues,
            summary,
            dimension_scores,
        })
    }

    fn calc_dim_score(&self, issues: &[Issue], cats: &[Category]) -> f64 {
        let count = issues.iter().filter(|i| cats.contains(&i.category)).count();
        let penalty = match cats.len() {
            1 => count as f64 * 5.0,
            2 => count as f64 * 3.0,
            3 | 4 => count as f64 * 1.5,
            _ => count as f64 * 1.0,
        };
        (100.0 - penalty).max(0.0)
    }

    fn analyze_file(&self, path: &str, content: &str) -> FileReport {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();

        let fns = analysis::extract_fns(content);
        let structs = analysis::extract_structs(content);
        let (comment_lines, code_lines) = analysis::count_comments(&lines);
        let comment_ratio = if code_lines > 0 { comment_lines as f64 / code_lines as f64 } else { 0.0 };

        let mut issues = Vec::new();
        push_file_level_issues(self, path, total, comment_ratio, &mut issues);
        run_content_checks(self, path, content, &mut issues);
        push_struct_issues(self, path, &structs, &mut issues);
        push_function_issues(self, path, &fns, &mut issues);

        let avg_fn_lines = if fns.is_empty() { 0.0 } else { fns.iter().map(|f| f.lines).sum::<usize>() as f64 / fns.len() as f64 };
        let max_fn_lines = fns.iter().map(|f| f.lines).max().unwrap_or(0);

        let score = compute_file_score(self, total, &fns, &structs, &issues, comment_ratio);

        FileReport {
            path: path.into(),
            lines: total,
            fn_count: fns.len(),
            struct_count: structs.len(),
            avg_fn_lines,
            max_fn_lines,
            comment_ratio,
            score: score.max(0.0),
            issues,
        }
    }

    pub fn format(&self, report: &SimplicityReport) -> String {
        format::format_report(report)
    }
}

fn build_summary(files: &[FileReport], all_issues: &[Issue], checker: &SimplicityChecker) -> Summary {
    Summary {
        long_files: files.iter().filter(|f| f.lines > checker.max_file_lines).count(),
        long_functions: all_issues.iter().filter(|i| i.category == Category::LongFunction).count(),
        large_structs: all_issues.iter().filter(|i| i.category == Category::LargeStruct).count(),
        low_comment_files: files.iter().filter(|f| f.comment_ratio < checker.min_comment_ratio && f.lines > 100).count(),
        complex_functions: all_issues.iter().filter(|i| i.category == Category::ComplexFunction).count(),
        deep_nesting: all_issues.iter().filter(|i| i.category == Category::DeepNesting).count(),
        many_args: all_issues.iter().filter(|i| i.category == Category::ManyArgs).count(),
        magic_numbers: all_issues.iter().filter(|i| i.category == Category::MagicNumber).count(),
        todo_markers: all_issues.iter().filter(|i| i.category == Category::TodoMarker).count(),
        duplicate_imports: all_issues.iter().filter(|i| i.category == Category::DuplicateImport).count(),
        unwrap_usage: all_issues.iter().filter(|i| i.category == Category::UnwrapUsage).count(),
        unsafe_usage: all_issues.iter().filter(|i| i.category == Category::UnsafeCode).count(),
        clone_overuse: all_issues.iter().filter(|i| i.category == Category::CloneOveruse).count(),
        string_concat_inefficient: all_issues.iter().filter(|i| i.category == Category::InefficientString).count(),
    }
}

fn build_dimension_scores(issues: &[Issue], checker: &SimplicityChecker) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    map.insert("Readability".into(), checker.calc_dim_score(issues, &[Category::LongFile, Category::LongFunction, Category::LargeStruct]));
    map.insert("Maintainability".into(), checker.calc_dim_score(issues, &[Category::ComplexFunction, Category::DeepNesting, Category::ManyArgs, Category::MagicNumber]));
    map.insert("Code Quality".into(), checker.calc_dim_score(issues, &[Category::UnwrapUsage, Category::UnsafeCode, Category::CloneOveruse]));
    map.insert("Documentation".into(), checker.calc_dim_score(issues, &[Category::LowComments, Category::TodoMarker]));
    map.insert("Performance".into(), checker.calc_dim_score(issues, &[Category::InefficientString, Category::CloneOveruse, Category::DuplicateImport]));
    map
}

fn push_file_level_issues(checker: &SimplicityChecker, path: &str, total: usize, comment_ratio: f64, issues: &mut Vec<Issue>) {
    if total > checker.max_file_lines {
        issues.push(Issue {
            path: path.into(), line: None, severity: Severity::Warning,
            category: Category::LongFile,
            message: format!("File has {} lines, exceeds threshold {}", total, checker.max_file_lines),
            suggestion: "Split into multiple modules with single responsibilities".into(),
        });
    }
    if comment_ratio < checker.min_comment_ratio && total > 100 {
        issues.push(Issue {
            path: path.into(), line: None, severity: Severity::Info,
            category: Category::LowComments,
            message: format!("Comment ratio {:.1}%, below recommended {:.0}%", comment_ratio * 100.0, checker.min_comment_ratio * 100.0),
            suggestion: "Add comments explaining design intent for complex logic".into(),
        });
    }
}

fn run_content_checks(checker: &SimplicityChecker, path: &str, content: &str, issues: &mut Vec<Issue>) {
    checks::check_magic_numbers(path, content, checker.max_magic_numbers, issues);
    checks::check_todos(path, content, issues);
    checks::check_unwrap(path, content, issues);
    checks::check_unsafe(path, content, issues);
    checks::check_duplicate_imports(path, content, issues);
    checks::check_string_concat(path, content, issues);
}

fn push_struct_issues(checker: &SimplicityChecker, path: &str, structs: &[StructInfo], issues: &mut Vec<Issue>) {
    for s in structs {
        if s.field_count > checker.max_struct_fields {
            issues.push(Issue {
                path: path.into(), line: Some(s.start_line), severity: Severity::Warning,
                category: Category::LargeStruct,
                message: format!("Struct '{}' has {} fields, exceeds threshold {}", s.name, s.field_count, checker.max_struct_fields),
                suggestion: "Consider splitting the struct or using composition".into(),
            });
        }
    }
}

fn push_function_issues(checker: &SimplicityChecker, path: &str, fns: &[FnInfo], issues: &mut Vec<Issue>) {
    use issues::{clone_issue, complexity_issue, long_fn_issue, many_args_issue, nesting_issue};
    for f in fns {
        if f.lines > checker.max_fn_lines { issues.push(long_fn_issue(path, f, checker.max_fn_lines)); }
        if f.args > checker.max_args { issues.push(many_args_issue(path, f, checker.max_args)); }
        if f.nesting > checker.max_nesting { issues.push(nesting_issue(path, f, checker.max_nesting)); }
        if f.complexity > checker.max_complexity { issues.push(complexity_issue(path, f, checker.max_complexity)); }
        if f.clone_count > checker.max_clones_per_fn { issues.push(clone_issue(path, f, checker.max_clones_per_fn)); }
    }
}

fn compute_file_score(
    checker: &SimplicityChecker,
    total: usize,
    fns: &[FnInfo],
    structs: &[StructInfo],
    issues: &[Issue],
    comment_ratio: f64,
) -> f64 {
    let mut score = 100.0;
    if total > checker.max_file_lines {
        score -= ((total - checker.max_file_lines) as f64 / 40.0).min(15.0);
    }
    score -= fns.iter().filter(|f| f.lines > checker.max_fn_lines).count() as f64 * 4.0;
    score -= fns.iter().filter(|f| f.complexity > checker.max_complexity).count() as f64 * 3.0;
    score -= fns.iter().filter(|f| f.nesting > checker.max_nesting).count() as f64 * 3.0;
    score -= fns.iter().filter(|f| f.args > checker.max_args).count() as f64 * 2.0;
    score -= fns.iter().filter(|f| f.clone_count > checker.max_clones_per_fn).count() as f64 * 2.0;
    score -= structs.iter().filter(|s| s.field_count > checker.max_struct_fields).count() as f64 * 2.0;

    let magic_count = issues.iter().filter(|i| i.category == Category::MagicNumber).count();
    score -= magic_count as f64 * 1.0;
    let unwrap_count = issues.iter().filter(|i| i.category == Category::UnwrapUsage).count();
    score -= unwrap_count as f64 * 1.5;
    if comment_ratio < checker.min_comment_ratio && total > 100 {
        score -= 5.0;
    }
    let todo_count = issues.iter().filter(|i| i.category == Category::TodoMarker).count();
    if todo_count > checker.max_todos {
        score -= (todo_count - checker.max_todos) as f64 * 0.5;
    }
    score
}
