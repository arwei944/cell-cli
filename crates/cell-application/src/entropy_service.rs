use cell_domain::entropy::{
    build_file_metrics, aggregate_dimensions, calculate_coupling, calculate_test_entropy,
    calculate_overall_score, is_high_risk_file, DimensionWeights, EntropyDimensions, EntropyGrade,
    EntropyReport, FileCouplingInfo, FileEntropy, TestFileInfo,
};
use cell_domain::errors::CellResult;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

pub fn run_entropy_check(path: &str) -> CellResult<EntropyReport> {
    let root = Path::new(path);

    let t_start = std::time::Instant::now();

    let rust_files: Vec<(String, String)> = WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let ext = entry.path().extension().and_then(|s| s.to_str());
            if ext != Some("rs") && ext != Some("go") && ext != Some("ts") && ext != Some("js") {
                return None;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let rel_path = entry
                .path()
                .strip_prefix(root).map_or_else(|_| {
                    entry
                        .path()
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                }, |p| p.to_string_lossy().to_string());
            Some((rel_path, content))
        })
        .collect();

    let t_read = t_start.elapsed().as_micros();

    let file_stem_index: HashMap<String, String> = rust_files
        .iter()
        .map(|(path, _)| {
            let stem = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            (stem, path.clone())
        })
        .collect();

    let mut files: Vec<FileEntropy> = Vec::with_capacity(rust_files.len());
    let mut coupling_info: Vec<FileCouplingInfo> = Vec::with_capacity(rust_files.len());
    let mut test_info: Vec<TestFileInfo> = Vec::with_capacity(rust_files.len());
    let mut total_lines = 0;

    let t_metrics_start = std::time::Instant::now();

    let file_metrics_results: Vec<_> = rust_files
        .par_iter()
        .map(|(rel_path, content)| {
            let (file_entropy, _complexity, _structural, _naming) =
                build_file_metrics(rel_path, content);
            let is_test = is_test_file(rel_path, content);
            let (test_count, assertion_count, _test_lines_count) = count_test_metrics(content, is_test);

            let imports = extract_imports(content);
            let file_stem = Path::new(rel_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .replace(['/', '\\'], "_");

            (
                file_entropy,
                is_test,
                test_count,
                assertion_count,
                imports,
                file_stem,
                rel_path.clone(),
                content.clone(),
            )
        })
        .collect();

    let t_metrics = t_metrics_start.elapsed().as_micros();

    let mut path_to_imports: HashMap<String, HashSet<String>> = HashMap::new();
    let mut path_to_stem: HashMap<String, String> = HashMap::new();

    for (
        file_entropy,
        is_test,
        test_count,
        assertion_count,
        imports,
        file_stem,
        rel_path,
        content,
    ) in file_metrics_results
    {
        total_lines += file_entropy.lines;
        files.push(file_entropy);
        path_to_imports.insert(rel_path.clone(), imports.clone());
        path_to_stem.insert(rel_path.clone(), file_stem.clone());

        let code_lines = if is_test { 0 } else { content.lines().count() };
        let test_lines = if is_test { content.lines().count() } else { 0 };
        test_info.push(TestFileInfo {
            path: rel_path.clone(),
            code_lines,
            test_lines,
            test_count,
            assertion_count,
            is_test_file: is_test,
        });

        let mut outgoing = imports.len();
        for import in &imports {
            if file_stem_index.contains_key(import) {
                outgoing += 1;
            }
        }

        let cross_layer = check_cross_layer_dependency(&rel_path, &imports);

        coupling_info.push(FileCouplingInfo {
            path: rel_path,
            incoming: 0,
            outgoing,
            cross_layer,
            in_cycle: false,
        });
    }

    let t_coupling_start = std::time::Instant::now();

    for info in &mut coupling_info {
        let imports = path_to_imports.get(&info.path).unwrap();
        let file_stem = path_to_stem.get(&info.path).unwrap();

        for (other_path, other_imports) in &path_to_imports {
            if other_path != &info.path {
                if other_imports.contains(file_stem) {
                    info.incoming += 1;
                }
                if imports.iter().any(|imp| {
                    path_to_stem.get(other_path) == Some(imp)
                }) {
                    info.outgoing += 1;
                }
            }
        }
    }

    let t_coupling = t_coupling_start.elapsed().as_micros();

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
        "熵值计算耗时: {}ms (读取文件: {}us, 指标计算: {}us, 依赖分析: {}us, 循环检测: {}us, 聚合: {}us, 文件数: {})",
        total, t_read, t_metrics, t_coupling, t_cycle, t_aggregate, files.len()
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

fn extract_imports(content: &str) -> HashSet<String> {
    let mut imports = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") || trimmed.starts_with("import ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let import = parts[1].to_string();
                imports.insert(import);
            }
        }
    }
    imports
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

        if let (Some(file_layer), Some(imp_layer)) = (layer, import_layer)
            && is_outward_dependency(file_layer, imp_layer) {
                return true;
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
    if n == 0 {
        return;
    }

    let mut graph = petgraph::Graph::new();
    let mut node_indices = Vec::with_capacity(n);
    let mut path_to_node = HashMap::with_capacity(n);

    for (i, info) in files.iter().enumerate() {
        let node = graph.add_node(i);
        node_indices.push(node);
        path_to_node.insert(info.path.clone(), node);
    }

    for (i, info) in files.iter().enumerate() {
        let _imports: Vec<String> = info.path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect();

        for (j, other_info) in files.iter().enumerate() {
            if i != j {
                let other_stem = Path::new(&other_info.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                if info.path.contains(other_stem) {
                    graph.add_edge(node_indices[i], node_indices[j], ());
                }
            }
        }
    }

    let cycles = petgraph::algo::tarjan_scc(&graph);
    let mut in_cycle_set = HashSet::new();

    for component in cycles {
        if component.len() > 1 {
            for &node in &component {
                let idx = graph[node];
                in_cycle_set.insert(idx);
            }
        }
    }

    for i in 0..n {
        files[i].in_cycle = in_cycle_set.contains(&i);
    }
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
        let code = r"
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
";
        let (tests, asserts, _) = count_test_metrics(code, true);
        assert_eq!(tests, 2);
        assert_eq!(asserts, 2);
    }
}
