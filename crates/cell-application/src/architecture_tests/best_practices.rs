use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Violation {
    file: String,
    line: usize,
    content: String,
}

impl Violation {
    fn new(file: &str, line: usize, content: &str) -> Self {
        Self {
            file: file.to_string(),
            line,
            content: content.trim().to_string(),
        }
    }
}

struct FileAnalysis {
    path: String,
    lines: Vec<String>,
    test_lines: Vec<bool>,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name != "target" {
                    files.extend(collect_rs_files(&path));
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files
}

fn collect_all_src_files() -> Vec<PathBuf> {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut files = Vec::new();

    if crates_dir.is_dir() {
        for entry in fs::read_dir(&crates_dir).unwrap() {
            let entry = entry.unwrap();
            let src_path = entry.path().join("src");
            if src_path.is_dir() {
                files.extend(collect_rs_files(&src_path));
            }
        }
    }

    let top_src = root.join("src");
    if top_src.is_dir() {
        files.extend(collect_rs_files(&top_src));
    }

    files
}

fn strip_comments_and_strings(line: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            if c == '\\' {
                chars.next();
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        result.push(c);
    }
    result
}

fn compute_test_lines(lines: &[String]) -> Vec<bool> {
    let mut result = vec![false; lines.len()];
    let mut in_test = false;
    let mut brace_depth: i32 = 0;
    let mut test_brace_depth: i32 = 0;

    for (i, line) in lines.iter().enumerate() {
        let clean = strip_comments_and_strings(line);
        let trimmed = clean.trim();

        if (trimmed.contains("#[cfg(test)]")
            || trimmed.contains("#[test]")
            || trimmed.contains("#[cfg(all(test"))
            && !in_test {
                in_test = true;
                test_brace_depth = brace_depth;
            }

        let open = trimmed.matches('{').count() as i32;
        let close = trimmed.matches('}').count() as i32;
        brace_depth += open - close;

        if in_test {
            result[i] = true;
            if brace_depth <= test_brace_depth && close > 0 {
                in_test = false;
            }
        }
    }

    result
}

fn analyze_file(file_path: &Path) -> Option<FileAnalysis> {
    let content = fs::read_to_string(file_path).ok()?;
    let lines: Vec<String> = content.lines().map(std::string::ToString::to_string).collect();
    let test_lines = compute_test_lines(&lines);
    let file_str = file_path.to_string_lossy().replace('\\', "/");

    Some(FileAnalysis {
        path: file_str,
        lines,
        test_lines,
    })
}

fn analyze_all_files(allowlist: &[&str]) -> Vec<FileAnalysis> {
    let files = collect_all_src_files();
    let mut results = Vec::new();

    for file_path in &files {
        let file_str = file_path.to_string_lossy().replace('\\', "/");
        if allowlist.iter().any(|a| file_str.contains(a)) {
            continue;
        }
        if let Some(analysis) = analyze_file(file_path) {
            results.push(analysis);
        }
    }

    results
}

fn find_patterns_in_analysis<F>(
    analyses: &[FileAnalysis],
    pattern: &str,
    filter: F,
) -> Vec<Violation>
where
    F: Fn(&str) -> bool,
{
    let mut violations = Vec::new();

    for analysis in analyses {
        for (line_idx, line) in analysis.lines.iter().enumerate() {
            if analysis.test_lines[line_idx] {
                continue;
            }

            let clean_line = strip_comments_and_strings(line);

            if clean_line.contains(pattern) && filter(&clean_line) {
                violations.push(Violation::new(&analysis.path, line_idx + 1, line));
            }
        }
    }

    violations
}

fn print_violations(name: &str, violations: &[Violation]) {
    if violations.is_empty() {
        return;
    }
    eprintln!("\n=== {} violations ({}) ===", name, violations.len());
    for v in violations {
        eprintln!("  {}:{} - {}", v.file, v.line, v.content);
    }
    eprintln!("=== end ===\n");
}

const BASE_ALLOWLIST: &[&str] = &["architecture_tests"];

#[test]
fn no_unwrap_in_production_code() {
    const MAX_ALLOWED_UNWRAP: usize = 900;
    let mut allowlist = Vec::from(BASE_ALLOWLIST);
    allowlist.extend_from_slice(&["simplicity_checker"]);

    let analyses = analyze_all_files(&allowlist);
    let violations = find_patterns_in_analysis(
        &analyses,
        ".unwrap()",
        |line| {
            !line.contains("clippy::unwrap_used")
                && !line.contains("allow(unwrap")
        },
    );

    print_violations("unwrap()", &violations);
    assert!(
        violations.len() <= MAX_ALLOWED_UNWRAP,
        "Found {} .unwrap() calls in production code (max allowed: {}). \
         Use proper error handling instead. New unwrap() calls are not allowed.",
        violations.len(),
        MAX_ALLOWED_UNWRAP
    );
}

#[test]
fn no_expect_in_production_code() {
    const MAX_ALLOWED_EXPECT: usize = 20;
    let analyses = analyze_all_files(BASE_ALLOWLIST);
    let violations = find_patterns_in_analysis(
        &analyses,
        ".expect(",
        |line| {
            !line.contains("clippy::expect_used")
                && !line.contains("allow(expect")
        },
    );

    print_violations("expect()", &violations);
    assert!(
        violations.len() <= MAX_ALLOWED_EXPECT,
        "Found {} .expect() calls in production code (max allowed: {}). \
         Use proper error handling instead. New expect() calls are not allowed.",
        violations.len(),
        MAX_ALLOWED_EXPECT
    );
}

#[test]
fn no_unsafe_in_production_code() {
    let analyses = analyze_all_files(BASE_ALLOWLIST);
    let mut violations = Vec::new();

    for analysis in &analyses {
        for (line_idx, line) in analysis.lines.iter().enumerate() {
            if analysis.test_lines[line_idx] {
                continue;
            }
            let clean = strip_comments_and_strings(line);
            let trimmed = clean.trim();

            let has_unsafe = trimmed.starts_with("unsafe ")
                || trimmed.starts_with("pub unsafe ")
                || trimmed.starts_with("async unsafe ");

            if has_unsafe && (trimmed.contains("fn ") || trimmed.contains('{')) {
                violations.push(Violation::new(&analysis.path, line_idx + 1, line));
            }
        }
    }

    print_violations("unsafe", &violations);
    assert!(
        violations.is_empty(),
        "Found {} unsafe blocks/functions in production code. This project should not need unsafe.",
        violations.len()
    );
}

#[test]
fn no_todo_in_production_code() {
    let mut allowlist = BASE_ALLOWLIST.to_vec();
    allowlist.extend_from_slice(&["generate_service.rs"]);

    let analyses = analyze_all_files(&allowlist);
    let patterns = ["todo!(", "unimplemented!(", "unreachable!("];
    let mut all_violations = Vec::new();

    for pattern in &patterns {
        let violations = find_patterns_in_analysis(&analyses, pattern, |_| true);
        all_violations.extend(violations);
    }

    print_violations("todo/unimplemented/unreachable", &all_violations);
    assert!(
        all_violations.is_empty(),
        "Found {} todo!/unimplemented!/unreachable!() calls in production code.",
        all_violations.len()
    );
}

#[test]
fn no_dbg_in_production_code() {
    let analyses = analyze_all_files(BASE_ALLOWLIST);
    let violations = find_patterns_in_analysis(&analyses, "dbg!(", |_| true);

    print_violations("dbg!()", &violations);
    assert!(
        violations.is_empty(),
        "Found {} dbg!() macro calls in production code. Debug code should not be committed.",
        violations.len()
    );
}

#[test]
fn no_println_in_library_code() {
    let mut allowlist = BASE_ALLOWLIST.to_vec();
    allowlist.extend_from_slice(&[
        "commands/",
        "cli.rs",
        "main.rs",
        "progress_bar.rs",
        "web_dashboard.rs",
        "ast_analyzer.rs",
        "dev_env_service.rs",
        "entropy_bank_service.rs",
        "onboarding_service.rs",
        "self_verify_service.rs",
        "template_service.rs",
        "entropy.rs",
        "fingerprint.rs",
    ]);

    let analyses = analyze_all_files(&allowlist);
    let violations = find_patterns_in_analysis(
        &analyses,
        "println!(",
        |line| !line.contains("eprintln!("),
    );

    print_violations("println!() in library code", &violations);
    assert!(
        violations.is_empty(),
        "Found {} println!() calls in library code. Use tracing/log or return values instead.",
        violations.len()
    );
}

#[test]
fn no_hardcoded_windows_paths() {
    let analyses = analyze_all_files(BASE_ALLOWLIST);
    let mut violations = Vec::new();

    for analysis in &analyses {
        for (line_idx, line) in analysis.lines.iter().enumerate() {
            if analysis.test_lines[line_idx] {
                continue;
            }
            let clean = strip_comments_and_strings(line);

            let has_c_drive = clean.contains("C:\\\\") || clean.contains("C:/");

            if has_c_drive {
                violations.push(Violation::new(&analysis.path, line_idx + 1, line));
            }
        }
    }

    print_violations("hardcoded Windows paths", &violations);
    assert!(
        violations.is_empty(),
        "Found {} hardcoded Windows paths. Use PathBuf and configuration instead.",
        violations.len()
    );
}

#[test]
fn no_circular_module_dependencies() {
    let analyses = analyze_all_files(BASE_ALLOWLIST);
    let mut module_deps: HashMap<String, Vec<String>> = HashMap::new();

    for analysis in &analyses {
        let file_path = Path::new(&analysis.path);
        let file_name = file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if file_name.is_empty() || file_name == "mod" || file_name == "lib" {
            continue;
        }

        let mut deps = Vec::new();

        for (line_idx, line) in analysis.lines.iter().enumerate() {
            if analysis.test_lines[line_idx] {
                continue;
            }
            let clean = strip_comments_and_strings(line);
            let trimmed = clean.trim();

            if trimmed.starts_with("use crate::") || trimmed.starts_with("pub use crate::") {
                let rest = trimmed
                    .trim_start_matches("pub use ")
                    .trim_start_matches("use ");
                let dep_path = rest
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .split(" as ")
                    .next()
                    .unwrap_or("")
                    .trim_end_matches("::*")
                    .trim_end_matches('{')
                    .trim()
                    .trim_end_matches("::")
                    .to_string();

                let dep_module = dep_path
                    .split("::")
                    .last()
                    .unwrap_or("")
                    .to_string();

                if !dep_module.is_empty()
                    && dep_module != file_name
                    && !deps.contains(&dep_module)
                {
                    deps.push(dep_module);
                }
            }
        }

        if !deps.is_empty() {
            module_deps.insert(file_name, deps);
        }
    }

    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    for module in module_deps.keys() {
        find_cycles(
            module.clone(),
            &module_deps,
            &mut visited,
            &mut stack,
            &mut cycles,
        );
    }

    if !cycles.is_empty() {
        eprintln!("\n=== Circular dependencies ({}) ===", cycles.len());
        for cycle in &cycles {
            eprintln!("  {}", cycle.join(" → "));
        }
        eprintln!("=== end ===\n");
    }

    assert!(
        cycles.is_empty(),
        "Found {} circular module dependencies.",
        cycles.len()
    );
}

fn find_cycles(
    module: String,
    deps: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if stack.contains(&module) {
        let idx = stack.iter().position(|m| m == &module).unwrap();
        let cycle: Vec<String> = stack[idx..].to_vec();
        if cycle.len() >= 2
            && !cycles.iter().any(|c| {
                if c.len() != cycle.len() {
                    return false;
                }
                let mut c_sorted = c.clone();
                let mut cycle_sorted = cycle.clone();
                c_sorted.sort();
                cycle_sorted.sort();
                c_sorted == cycle_sorted
            })
        {
            cycles.push(cycle);
        }
        return;
    }

    if visited.contains(&module) {
        return;
    }

    visited.insert(module.clone());
    stack.push(module.clone());

    if let Some(module_deps) = deps.get(&module) {
        for dep in module_deps {
            find_cycles(dep.clone(), deps, visited, stack, cycles);
        }
    }

    stack.pop();
}
