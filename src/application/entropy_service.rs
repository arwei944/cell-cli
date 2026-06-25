use crate::domain::entropy::{
    build_file_metrics, aggregate_dimensions, calculate_coupling, calculate_test_entropy,
    calculate_overall_score, is_high_risk_file, DimensionWeights, EntropyDimensions, EntropyGrade,
    EntropyReport, FileCouplingInfo, FileEntropy, TestFileInfo,
};
use crate::domain::errors::CellResult;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

pub fn run_entropy_check(path: &str) -> CellResult<EntropyReport> {
    let root = Path::new(path);
    let mut files: Vec<FileEntropy> = Vec::new();
    let mut total_lines = 0;

    let mut file_contents: HashMap<String, String> = HashMap::new();
    let mut coupling_info: Vec<FileCouplingInfo> = Vec::new();
    let mut test_info: Vec<TestFileInfo> = Vec::new();

    let t_start = std::time::Instant::now();

    let mut rust_files: Vec<(String, String)> = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let ext = entry.path().extension().and_then(|s| s.to_str());
        if ext == Some("rs") || ext == Some("go") || ext == Some("ts") || ext == Some("js") {
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let rel_path = entry
                .path()
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    entry
                        .path()
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            rust_files.push((rel_path, content));
        }
    }
    let t_read = t_start.elapsed().as_micros();

    let mut t_metrics = 0u128;
    let mut t_coupling = 0u128;
    let mut t_test = 0u128;

    for (rel_path, content) in &rust_files {
        let t1 = std::time::Instant::now();
        let (file_entropy, _complexity, _structural, _naming) =
            build_file_metrics(rel_path, content);
        t_metrics += t1.elapsed().as_micros();
        total_lines += file_entropy.lines;
        files.push(file_entropy);

        let t2 = std::time::Instant::now();
        let (incoming, outgoing, cross_layer) = analyze_dependencies_fast(rel_path, content, &rust_files);
        t_coupling += t2.elapsed().as_micros();
        coupling_info.push(FileCouplingInfo {
            path: rel_path.clone(),
            incoming,
            outgoing,
            cross_layer,
            in_cycle: false,
        });

        let t3 = std::time::Instant::now();
        let is_test = is_test_file(rel_path, content);
        let (test_count, assertion_count, _test_lines_count) = count_test_metrics(content, is_test);
        t_test += t3.elapsed().as_micros();
        test_info.push(TestFileInfo {
            path: rel_path.clone(),
            code_lines: if is_test { 0 } else { content.lines().count() },
            test_lines: if is_test { content.lines().count() } else { 0 },
            test_count,
            assertion_count,
            is_test_file: is_test,
        });

        file_contents.insert(rel_path.clone(), content.clone());
    }

    let t_cycle_start = std::time::Instant::now();
    detect_cycles(&mut coupling_info);
    let t_cycle = t_cycle_start.elapsed().as_micros();

    let t_aggregate_start = std::time::Instant::now();
    let structural_scores: Vec<f64> = files.iter().map(|f| f.structural_score).collect();
    let complexity_scores: Vec<f64> = files.iter().map(|f| f.complexity_score).collect();
    let naming_scores: Vec<f64> = files.iter().map(|f| f.naming_score).collect();

    let structural_dim = aggregate_dimensions(&structural_scores);
    let complexity_dim = aggregate_dimensions(&complexity_scores);
    let naming_dim = aggregate_dimensions(&naming_scores);
    let coupling_dim = calculate_coupling(&coupling_info);
    let test_dim = calculate_test_entropy(&test_info);
    let t_aggregate = t_aggregate_start.elapsed().as_micros();

    let weights = DimensionWeights::default();
    let dimensions = EntropyDimensions {
        structural: structural_dim,
        complexity: complexity_dim,
        coupling: coupling_dim,
        naming: naming_dim,
        test: test_dim,
    };

    let overall_score = calculate_overall_score(&dimensions, &weights);
    let grade = EntropyGrade::from_score(overall_score);

    let high_risk_files: Vec<String> = files
        .iter()
        .filter(|f| is_high_risk_file(f))
        .map(|f| f.path.clone())
        .collect();

    let total = t_start.elapsed().as_millis();
    tracing::debug!(
        "熵值计算耗时: {}ms (读取文件: {}us, 指标计算: {}us, 依赖分析: {}us, 测试统计: {}us, 循环检测: {}us, 聚合: {}us, 文件数: {})",
        total, t_read, t_metrics, t_coupling, t_test, t_cycle, t_aggregate, files.len()
    );

    Ok(EntropyReport {
        overall_score,
        grade,
        dimensions,
        dimension_weights: weights,
        file_count: files.len(),
        total_lines,
        breakdown: files,
        high_risk_files,
    })
}

fn analyze_dependencies_fast(
    file_path: &str,
    content: &str,
    all_files: &[(String, String)],
) -> (usize, usize, bool) {
    let mut imports = HashSet::new();
    let mut used_by = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") || trimmed.starts_with("import ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                imports.insert(parts[1].to_string());
            }
        }
    }

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .replace(['/', '\\'], "_");

    for (other_path, other_content) in all_files {
        if other_path != file_path {
            let other_stem = Path::new(other_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if content.contains(other_stem) {
                imports.insert(other_stem.to_string());
            }
            if other_content.contains(&file_stem) {
                used_by.insert(other_path.clone());
            }
        }
    }

    let cross_layer = check_cross_layer_dependency(file_path, &imports);

    (used_by.len(), imports.len(), cross_layer)
}

fn check_cross_layer_dependency(file_path: &str, imports: &HashSet<String>) -> bool {
    let path_lower = file_path.to_lowercase();
    let layer = detect_layer(&path_lower);

    for import in imports {
        let import_lower = import.to_lowercase();
        let import_layer = if import_lower.contains("domain") {
            Some("domain")
        } else if import_lower.contains("application") {
            Some("application")
        } else if import_lower.contains("adapter") {
            Some("adapters")
        } else if import_lower.contains("interface") {
            Some("interfaces")
        } else {
            None
        };

        if let (Some(file_layer), Some(imp_layer)) = (layer, import_layer) {
            if is_outward_dependency(file_layer, imp_layer) {
                return true;
            }
        }
    }
    false
}

fn detect_layer(path: &str) -> Option<&str> {
    if path.contains("domain") {
        Some("domain")
    } else if path.contains("application") {
        Some("application")
    } else if path.contains("adapter") {
        Some("adapters")
    } else if path.contains("interface") {
        Some("interfaces")
    } else {
        None
    }
}

fn is_outward_dependency(from: &str, to: &str) -> bool {
    let order = ["domain", "application", "adapters", "interfaces"];
    let from_idx = order.iter().position(|l| *l == from).unwrap_or(0);
    let to_idx = order.iter().position(|l| *l == to).unwrap_or(0);
    to_idx > from_idx
}

fn detect_cycles(files: &mut [FileCouplingInfo]) {
    let n = files.len();
    let mut adj = vec![vec![false; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i != j && files[i].path.contains(&files[j].path.replace(['/', '\\'], "_")) {
                adj[i][j] = true;
            }
        }
    }

    let in_cycle = detect_cyclic_nodes(&adj);
    for i in 0..n {
        files[i].in_cycle = in_cycle[i];
    }
}

fn detect_cyclic_nodes(adj: &[Vec<bool>]) -> Vec<bool> {
    let n = adj.len();
    let mut visited = vec![false; n];
    let mut rec_stack = vec![false; n];
    let mut in_cycle = vec![false; n];

    fn dfs(
        node: usize,
        adj: &[Vec<bool>],
        visited: &mut [bool],
        rec_stack: &mut [bool],
        in_cycle: &mut [bool],
    ) -> bool {
        visited[node] = true;
        rec_stack[node] = true;

        for (next, &is_edge) in adj[node].iter().enumerate() {
            if is_edge {
                if !visited[next] && dfs(next, adj, visited, rec_stack, in_cycle) {
                    in_cycle[node] = true;
                    return true;
                } else if rec_stack[next] {
                    in_cycle[node] = true;
                    in_cycle[next] = true;
                    return true;
                }
            }
        }

        rec_stack[node] = false;
        false
    }

    for i in 0..n {
        if !visited[i] {
            dfs(i, adj, &mut visited, &mut rec_stack, &mut in_cycle);
        }
    }

    in_cycle
}

fn is_test_file(path: &str, content: &str) -> bool {
    let path_lower = path.to_lowercase();
    if path_lower.contains("test")
        || path_lower.ends_with("_test.rs")
        || path_lower.ends_with(".test.ts")
        || path_lower.ends_with(".test.js")
        || path_lower.ends_with(".spec.ts")
        || path_lower.ends_with(".spec.js")
    {
        return true;
    }
    if content.contains("#[cfg(test)]") || content.contains("mod tests") {
        return true;
    }
    false
}

fn count_test_metrics(content: &str, is_test_file: bool) -> (usize, usize, usize) {
    let mut test_count = 0;
    let mut assertion_count = 0;
    let mut test_lines = 0;
    let mut in_test_module = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn test_")
            || trimmed.starts_with("it(")
            || trimmed.starts_with("test(")
        {
            test_count += 1;
        }
        if trimmed.contains("assert!")
            || trimmed.contains("assert_eq!")
            || trimmed.contains("assert_ne!")
            || trimmed.contains("expect(")
            || trimmed.contains(".toBe")
            || trimmed.contains(".toEqual")
        {
            assertion_count += 1;
        }
        if trimmed.contains("#[cfg(test)]") || trimmed.contains("mod tests") {
            in_test_module = true;
        }
        if in_test_module {
            test_lines += 1;
        }
    }

    if is_test_file && test_lines == 0 {
        test_lines = content.lines().count();
    }

    (test_count, assertion_count, test_lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_report() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(&file_path, "fn hello() {}").unwrap();

        let report = run_entropy_check(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(report.file_count, 1);
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 100.0);
        assert!(report.total_lines > 0);
    }

    #[test]
    fn test_entropy_report_has_grade() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(&file_path, "fn hello() {}").unwrap();

        let report = run_entropy_check(dir.path().to_str().unwrap()).unwrap();
        assert!(matches!(
            report.grade,
            EntropyGrade::Healthy
                | EntropyGrade::Notice
                | EntropyGrade::Warning
                | EntropyGrade::Danger
                | EntropyGrade::Critical
        ));
    }

    #[test]
    fn test_entropy_report_all_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(&file_path, "fn hello() { assert_eq!(1, 1); }").unwrap();

        let report = run_entropy_check(dir.path().to_str().unwrap()).unwrap();
        assert!(report.dimensions.structural >= 0.0);
        assert!(report.dimensions.complexity >= 0.0);
        assert!(report.dimensions.coupling >= 0.0);
        assert!(report.dimensions.naming >= 0.0);
        assert!(report.dimensions.test >= 0.0);
    }

    #[test]
    fn test_is_test_file_detection() {
        assert!(is_test_file("foo_test.rs", ""));
        assert!(is_test_file("bar.test.ts", ""));
        assert!(!is_test_file("lib.rs", ""));
        assert!(is_test_file("lib.rs", "#[cfg(test)]\nmod tests {}"));
    }

    #[test]
    fn test_count_test_metrics() {
        let code = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_one() {
        assert_eq!(1, 1);
    }
    #[test]
    fn test_two() {
        assert!(true);
    }
}
"#;
        let (tests, asserts, _) = count_test_metrics(code, true);
        assert_eq!(tests, 2);
        assert_eq!(asserts, 2);
    }
}
