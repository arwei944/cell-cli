use std::fs;
use std::path::{Path, PathBuf};

fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> &'static Path {
    project_root()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files
}

fn all_src_files() -> Vec<PathBuf> {
    let workspace = workspace_root();
    let mut files = Vec::new();
    let crates_dir = workspace.join("crates");
    if crates_dir.exists() {
        for entry in fs::read_dir(&crates_dir).unwrap() {
            let entry = entry.unwrap();
            let src_dir = entry.path().join("src");
            if src_dir.exists() {
                files.extend(collect_rs_files(&src_dir));
            }
        }
    }
    let src_dir = workspace.join("src");
    if src_dir.exists() {
        files.extend(collect_rs_files(&src_dir));
    }
    files
}

fn read_file_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

fn is_test_module_file(lines: &[String]) -> bool {
    lines.iter().any(|l| l.contains("#[cfg(test)]"))
}

fn find_matching_brace(lines: &[String], start_line: usize, start_col: usize) -> Option<usize> {
    let mut depth = 0;
    let mut first = true;
    for (i, line) in lines.iter().enumerate().skip(start_line) {
        for (j, c) in line.chars().enumerate() {
            if i == start_line && j < start_col && first {
                continue;
            }
            first = false;
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn find_test_module_ranges(lines: &[String]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if line.contains("#[cfg(test)]") {
            let mut brace_line = i;
            let mut found = false;
            while brace_line < lines.len() {
                if let Some(col) = lines[brace_line].find('{')
                    && let Some(end) = find_matching_brace(lines, brace_line, col) {
                        ranges.push((i, end));
                        i = end + 1;
                        found = true;
                        break;
                    }
                brace_line += 1;
            }
            if !found {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    ranges
}

fn is_in_test_module(line_idx: usize, test_ranges: &[(usize, usize)]) -> bool {
    test_ranges
        .iter()
        .any(|&(start, end)| line_idx >= start && line_idx <= end)
}

#[test]
fn file_size_non_test_under_limit() {
    let max_lines = 800;
    let exceptions = ["lib.rs", "cli.rs", "layer_deps.rs"];
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        if exceptions.contains(&file_name) {
            continue;
        }
        let lines = read_file_lines(&file);
        if !is_test_module_file(&lines) && lines.len() > max_lines {
            offenders.push((file, lines.len()));
        }
    }

    assert!(
        offenders.is_empty(),
        "Files exceeding {} lines (non-test):\n{}",
        max_lines,
        offenders
            .iter()
            .map(|(f, n)| format!("  {}: {} lines", f.display(), n))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn file_size_test_module_under_limit() {
    let max_lines = 1500;
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let lines = read_file_lines(&file);
        if is_test_module_file(&lines) && lines.len() > max_lines {
            offenders.push((file, lines.len()));
        }
    }

    assert!(
        offenders.is_empty(),
        "Test files exceeding {} lines:\n{}",
        max_lines,
        offenders
            .iter()
            .map(|(f, n)| format!("  {}: {} lines", f.display(), n))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn function_length_non_test_under_limit() {
    let max_lines = 650;
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let lines = read_file_lines(&file);
        let test_ranges = find_test_module_ranges(&lines);
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            let trimmed = line.trim();
            if (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("pub(super) fn "))
                && !is_in_test_module(i, &test_ranges)
                && let Some(col) = line.find('{')
                    && let Some(end) = find_matching_brace(&lines, i, col) {
                        let len = end - i + 1;
                        if len > max_lines {
                            let fn_name = trimmed
                                .split("fn ")
                                .nth(1)
                                .and_then(|s| s.split('(').next())
                                .unwrap_or("unknown");
                            offenders.push((file.clone(), fn_name.to_string(), i + 1, len));
                        }
                        i = end + 1;
                        continue;
                    }
            i += 1;
        }
    }

    assert!(
        offenders.is_empty(),
        "Non-test functions exceeding {} lines:\n{}",
        max_lines,
        offenders
            .iter()
            .map(|(f, name, line, n)| format!(
                "  {}:{} - {} ({} lines)",
                f.display(),
                line,
                name,
                n
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn function_length_test_under_limit() {
    let max_lines = 150;
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let lines = read_file_lines(&file);
        let test_ranges = find_test_module_ranges(&lines);
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            let trimmed = line.trim();
            if (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn "))
                && is_in_test_module(i, &test_ranges)
                && let Some(col) = line.find('{')
                    && let Some(end) = find_matching_brace(&lines, i, col) {
                        let len = end - i + 1;
                        if len > max_lines {
                            let fn_name = trimmed
                                .split("fn ")
                                .nth(1)
                                .and_then(|s| s.split('(').next())
                                .unwrap_or("unknown");
                            offenders.push((file.clone(), fn_name.to_string(), i + 1, len));
                        }
                        i = end + 1;
                        continue;
                    }
            i += 1;
        }
    }

    assert!(
        offenders.is_empty(),
        "Test functions exceeding {} lines:\n{}",
        max_lines,
        offenders
            .iter()
            .map(|(f, name, line, n)| format!(
                "  {}:{} - {} ({} lines)",
                f.display(),
                line,
                name,
                n
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn function_params_under_limit() {
    let max_params = 7;
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let lines = read_file_lines(&file);
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("pub(super) fn "))
                && let Some(start) = line.find('(') {
                    let mut params_str = String::new();
                    let mut depth = 0;
                    let mut found_start = false;
                    for c in line[start..].chars() {
                        if c == '(' {
                            depth += 1;
                            found_start = true;
                        } else if c == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        if found_start && depth > 0 {
                            params_str.push(c);
                        }
                    }
                    if params_str.len() >= 2 {
                        params_str = params_str[1..].to_string();
                    } else {
                        continue;
                    }

                    if params_str.trim().is_empty() {
                        continue;
                    }

                    let mut params = Vec::new();
                    let mut current = String::new();
                    let mut paren_depth = 0;
                    let mut angle_depth = 0;
                    for c in params_str.chars() {
                        match c {
                            '(' => paren_depth += 1,
                            ')' => paren_depth -= 1,
                            '<' => angle_depth += 1,
                            '>' => angle_depth -= 1,
                            ',' if paren_depth == 0 && angle_depth == 0 => {
                                params.push(current.trim().to_string());
                                current.clear();
                                continue;
                            }
                            _ => {}
                        }
                        current.push(c);
                    }
                    if !current.trim().is_empty() {
                        params.push(current.trim().to_string());
                    }

                    let non_self_params: Vec<_> = params
                        .iter()
                        .filter(|p| {
                            !p.starts_with("self")
                                && !p.starts_with("&self")
                                && !p.starts_with("&mut self")
                                && !p.starts_with("mut self")
                        })
                        .collect();

                    if non_self_params.len() > max_params {
                        let fn_name = trimmed
                            .split("fn ")
                            .nth(1)
                            .and_then(|s| s.split('(').next())
                            .unwrap_or("unknown");
                        offenders.push((
                            file.clone(),
                            fn_name.to_string(),
                            i + 1,
                            non_self_params.len(),
                        ));
                    }
                }
        }
    }

    assert!(
        offenders.is_empty(),
        "Functions with more than {} parameters (excluding self):\n{}",
        max_params,
        offenders
            .iter()
            .map(|(f, name, line, n)| format!(
                "  {}:{} - {} ({} params)",
                f.display(),
                line,
                name,
                n
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn struct_fields_under_limit() {
    let max_fields = 25;
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let lines = read_file_lines(&file);
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            let trimmed = line.trim();
            if (trimmed.starts_with("struct ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub(crate) struct ")
                || trimmed.starts_with("pub(super) struct "))
                && trimmed.contains('{')
                && let Some(col) = line.find('{')
                    && let Some(end) = find_matching_brace(&lines, i, col) {
                        let mut field_count = 0;
                        for line in &lines[i..=end] {
                            let t = line.trim();
                            if !t.is_empty()
                                && !t.starts_with("//")
                                && !t.starts_with("/*")
                                && t.contains(':')
                                && !t.starts_with("struct ")
                                && !t.starts_with("pub struct ")
                                && !t.starts_with("pub(crate) struct ")
                                && !t.starts_with("pub(super) struct ")
                            {
                                field_count += 1;
                            }
                        }
                        if field_count > max_fields {
                            let struct_name = trimmed
                                .split("struct ")
                                .nth(1)
                                .and_then(|s| s.split(&[' ', '{'][..]).next())
                                .unwrap_or("unknown");
                            offenders.push((
                                file.clone(),
                                struct_name.to_string(),
                                i + 1,
                                field_count,
                            ));
                        }
                        i = end + 1;
                        continue;
                    }
            i += 1;
        }
    }

    assert!(
        offenders.is_empty(),
        "Structs with more than {} fields:\n{}",
        max_fields,
        offenders
            .iter()
            .map(|(f, name, line, n)| format!(
                "  {}:{} - {} ({} fields)",
                f.display(),
                line,
                name,
                n
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn module_file_count_under_limit() {
    fn count_rs_files_in_dir(dir: &Path) -> usize {
        let mut count = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                    count += 1;
                }
            }
        }
        count
    }

    fn check_dirs(dir: &Path, offenders: &mut Vec<(PathBuf, usize)>, max: usize) {
        if dir.is_dir() {
            let count = count_rs_files_in_dir(dir);
            if count > max {
                offenders.push((dir.to_path_buf(), count));
            }
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    check_dirs(&path, offenders, max);
                }
            }
        }
    }

    let max_files = 80;
    let mut offenders = Vec::new();

    let workspace = workspace_root();
    let crates_dir = workspace.join("crates");
    if crates_dir.exists() {
        for entry in fs::read_dir(&crates_dir).unwrap() {
            let entry = entry.unwrap();
            let src_dir = entry.path().join("src");
            if src_dir.exists() {
                check_dirs(&src_dir, &mut offenders, max_files);
            }
        }
    }
    let src_dir = workspace.join("src");
    if src_dir.exists() {
        check_dirs(&src_dir, &mut offenders, max_files);
    }

    assert!(
        offenders.is_empty(),
        "Directories with more than {} .rs files:\n{}",
        max_files,
        offenders
            .iter()
            .map(|(d, n)| format!("  {}: {} files", d.display(), n))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn enum_variants_under_limit() {
    let max_variants = 60;
    let mut offenders = Vec::new();

    for file in all_src_files() {
        let lines = read_file_lines(&file);
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            let trimmed = line.trim();
            if (trimmed.starts_with("enum ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub(crate) enum ")
                || trimmed.starts_with("pub(super) enum "))
                && trimmed.contains('{')
                && let Some(col) = line.find('{')
                    && let Some(end) = find_matching_brace(&lines, i, col) {
                        let mut variant_count = 0;
                        for line in &lines[i..=end] {
                            let t = line.trim();
                            if !t.is_empty()
                                && !t.starts_with("//")
                                && !t.starts_with("/*")
                                && (t.starts_with(char::is_alphabetic) || t.starts_with('_'))
                                && !t.starts_with("enum ")
                                && !t.starts_with("pub enum ")
                                && !t.starts_with("pub(crate) enum ")
                                && !t.starts_with("pub(super) enum ")
                                && t.ends_with(',')
                            {
                                variant_count += 1;
                            }
                        }
                        if variant_count > max_variants {
                            let enum_name = trimmed
                                .split("enum ")
                                .nth(1)
                                .and_then(|s| s.split(&[' ', '{'][..]).next())
                                .unwrap_or("unknown");
                            offenders.push((
                                file.clone(),
                                enum_name.to_string(),
                                i + 1,
                                variant_count,
                            ));
                        }
                        i = end + 1;
                        continue;
                    }
            i += 1;
        }
    }

    assert!(
        offenders.is_empty(),
        "Enums with more than {} variants:\n{}",
        max_variants,
        offenders
            .iter()
            .map(|(f, name, line, n)| format!(
                "  {}:{} - {} ({} variants)",
                f.display(),
                line,
                name,
                n
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn module_nesting_depth_under_limit() {
    fn check_depth(
        dir: &Path,
        current_depth: usize,
        base: &Path,
        offenders: &mut Vec<(PathBuf, usize)>,
        max: usize,
    ) {
        if dir.is_dir() {
            let has_rs_files = fs::read_dir(dir).unwrap().any(|e| {
                let p = e.unwrap().path();
                p.is_file() && p.extension().is_some_and(|ext| ext == "rs")
            });
            if has_rs_files && current_depth > max {
                offenders.push((dir.strip_prefix(base).unwrap_or(dir).to_path_buf(), current_depth));
            }
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    check_depth(&path, current_depth + 1, base, offenders, max);
                }
            }
        }
    }

    let max_depth = 4;
    let mut offenders = Vec::new();

    let workspace = workspace_root();
    let crates_dir = workspace.join("crates");
    if crates_dir.exists() {
        for entry in fs::read_dir(&crates_dir).unwrap() {
            let entry = entry.unwrap();
            let src_dir = entry.path().join("src");
            if src_dir.exists() {
                check_depth(&src_dir, 1, &src_dir, &mut offenders, max_depth);
            }
        }
    }
    let src_dir = workspace.join("src");
    if src_dir.exists() {
        check_depth(&src_dir, 1, &src_dir, &mut offenders, max_depth);
    }

    assert!(
        offenders.is_empty(),
        "Module nesting depth exceeding {} (relative to src/):\n{}",
        max_depth,
        offenders
            .iter()
            .map(|(d, n)| format!("  {}: depth {}", d.display(), n))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
