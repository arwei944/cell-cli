//! 代码质量检测器：Issue 构造器函数
//! Code quality checker: Issue constructor helpers

use super::types::{Category, Issue, Severity, FnInfo};

pub fn long_fn_issue(path: &str, f: &FnInfo, t: usize) -> Issue {
    Issue { path: path.into(), line: Some(f.start_line), severity: Severity::Warning,
        category: Category::LongFunction,
        message: format!("'{}' has {} lines, exceeds threshold {}", f.name, f.lines, t),
        suggestion: "Split the function, each should do one thing".into() }
}

pub fn many_args_issue(path: &str, f: &FnInfo, t: usize) -> Issue {
    Issue { path: path.into(), line: Some(f.start_line), severity: Severity::Warning,
        category: Category::ManyArgs,
        message: format!("'{}' has {} args, exceeds threshold {}", f.name, f.args, t),
        suggestion: "Use a struct to wrap parameters".into() }
}

pub fn nesting_issue(path: &str, f: &FnInfo, t: usize) -> Issue {
    Issue { path: path.into(), line: Some(f.start_line), severity: Severity::Warning,
        category: Category::DeepNesting,
        message: format!("'{}' nests {} levels, exceeds threshold {}", f.name, f.nesting, t),
        suggestion: "Extract inner logic to a separate function to reduce nesting".into() }
}

pub fn complexity_issue(path: &str, f: &FnInfo, t: usize) -> Issue {
    Issue { path: path.into(), line: Some(f.start_line), severity: Severity::Warning,
        category: Category::ComplexFunction,
        message: format!("'{}' cyclomatic complexity {}, exceeds threshold {}", f.name, f.complexity, t),
        suggestion: "Simplify logic, split complex conditions".into() }
}

pub fn clone_issue(path: &str, f: &FnInfo, t: usize) -> Issue {
    Issue { path: path.into(), line: Some(f.start_line), severity: Severity::Warning,
        category: Category::CloneOveruse,
        message: format!("'{}' called clone() {} times, exceeds threshold {}", f.name, f.clone_count, t),
        suggestion: "Consider references or smart pointers to reduce clone".into() }
}
