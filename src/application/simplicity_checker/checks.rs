//! 代码质量检测器：单维度检查函数
//! Code quality checker: per-dimension check functions

use super::types::{Category, Issue, Severity};

/// 检测魔法数字
pub fn check_magic_numbers(
    path: &str,
    content: &str,
    max_magic_numbers: usize,
    issues: &mut Vec<Issue>,
) {
    let allowed = ["0", "1", "2", "10", "100", "1000", "0.0", "1.0", "2.0"];
    let mut count = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("///") { continue; }
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") { continue; }
        if trimmed.starts_with("const ") || trimmed.starts_with("pub const ") { continue; }
        if trimmed.starts_with("let ") || trimmed.starts_with("let mut ") { continue; }

        for word in trimmed.split(|c: char| !c.is_ascii_alphanumeric() && c != '.') {
            if word.is_empty() { continue; }
            if word.chars().all(|c| c.is_ascii_digit() || c == '.')
               && word.chars().any(|c| c.is_ascii_digit()) {
                if !allowed.contains(&word) && count < max_magic_numbers + 2 {
                    issues.push(Issue {
                        path: path.into(), line: Some(i + 1), severity: Severity::Info,
                        category: Category::MagicNumber,
                        message: format!("magic number '{}', recommend defining a constant", word),
                        suggestion: format!("const XXX: {} = {};", if word.contains('.') { "f64" } else { "usize" }, word),
                    });
                    count += 1;
                }
            }
        }
    }
}

/// 检测 TODO/FIXME/HACK 标记
pub fn check_todos(path: &str, content: &str, issues: &mut Vec<Issue>) {
    for (i, line) in content.lines().enumerate() {
        let upper = line.to_uppercase();
        if (upper.contains("TODO") || upper.contains("FIXME") || upper.contains("HACK"))
           && line.trim_start().starts_with("//") {
            issues.push(Issue {
                path: path.into(), line: Some(i + 1), severity: Severity::Info,
                category: Category::TodoMarker,
                message: line.trim().to_string(),
                suggestion: "Schedule time to complete or clean up TODOs".into(),
            });
        }
    }
}

/// 检测 unwrap() 使用
pub fn check_unwrap(path: &str, content: &str, issues: &mut Vec<Issue>) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") { continue; }
        if trimmed.contains(".unwrap()") && !trimmed.contains("expect") {
            issues.push(Issue {
                path: path.into(), line: Some(i + 1), severity: Severity::Warning,
                category: Category::UnwrapUsage,
                message: "used unwrap(), may cause panic".into(),
                suggestion: "Use expect() or match to handle errors".into(),
            });
        }
    }
}

/// 检测 unsafe 代码块
pub fn check_unsafe(path: &str, content: &str, issues: &mut Vec<Issue>) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("unsafe ") || trimmed.starts_with("unsafe{") {
            issues.push(Issue {
                path: path.into(), line: Some(i + 1), severity: Severity::Warning,
                category: Category::UnsafeCode,
                message: "used unsafe code block".into(),
                suggestion: "Evaluate if unsafe is really needed, add safety comments".into(),
            });
        }
    }
}

/// 检测重复导入
pub fn check_duplicate_imports(path: &str, content: &str, issues: &mut Vec<Issue>) {
    use std::collections::HashSet;
    let mut imports = HashSet::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            let import = trimmed.trim_end_matches(';').to_string();
            if imports.contains(&import) {
                issues.push(Issue {
                    path: path.into(), line: Some(i + 1), severity: Severity::Warning,
                    category: Category::DuplicateImport,
                    message: format!("duplicate import: {}", import),
                    suggestion: "Remove the duplicate use statement".into(),
                });
            }
            imports.insert(import);
        }
    }
}

/// 检测低效字符串拼接
pub fn check_string_concat(path: &str, content: &str, issues: &mut Vec<Issue>) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") { continue; }

        if trimmed.contains(".push_str(") && line.matches("push_str").count() > 2 {
            issues.push(Issue {
                path: path.into(), line: Some(i + 1), severity: Severity::Info,
                category: Category::InefficientString,
                message: "multiple push_str, consider format! or String::with_capacity".into(),
                suggestion: "Use format! macro or pre-allocate capacity for better performance".into(),
            });
        }

        if line.matches("to_string()").count() > 3 {
            issues.push(Issue {
                path: path.into(), line: Some(i + 1), severity: Severity::Info,
                category: Category::InefficientString,
                message: "multiple to_string() calls, consider reducing conversions".into(),
                suggestion: "Convert only when needed, or use references".into(),
            });
        }
    }
}
