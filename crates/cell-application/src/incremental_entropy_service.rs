use crate::entropy_service;
use cell_domain::entropy::{build_file_metrics, calculate_overall_score, DimensionWeights, EntropyDimensions, EntropyGrade, EntropyReport, FileEntropy};
use cell_domain::errors::CellResult;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalEntropyConfig {
    pub use_cache: bool,
    pub cache_dir: String,
    pub include_dependents: bool,
}

impl Default for IncrementalEntropyConfig {
    fn default() -> Self {
        Self {
            use_cache: true,
            cache_dir: ".cell/cache".to_string(),
            include_dependents: true,
        }
    }
}

pub struct IncrementalEntropyService {
    config: IncrementalEntropyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntropyCache {
    pub file_path: String,
    pub complexity_score: f64,
    pub structural_score: f64,
    pub naming_score: f64,
    pub lines: usize,
    pub last_modified: u64,
}

#[derive(Debug, Clone)]
pub struct IncrementalResult {
    pub changed_files: Vec<String>,
    pub new_files: usize,
    pub deleted_files: usize,
    pub affected_files: usize,
    pub overall_change: f64,
    pub high_risk_changes: Vec<String>,
    pub report: EntropyReport,
    pub duration_ms: u64,
    pub from_cache: usize,
    pub is_incremental: bool,
}

impl IncrementalEntropyService {
    pub fn new() -> Self {
        Self {
            config: IncrementalEntropyConfig::default(),
        }
    }

    pub fn with_config(config: IncrementalEntropyConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, project_path: &str) -> CellResult<IncrementalResult> {
        let start = std::time::Instant::now();

        let changed_files = self.get_changed_files(project_path)?;
        let cached_files = if self.config.use_cache {
            self.load_cache(project_path)?
        } else {
            HashMap::new()
        };

        let is_incremental = !changed_files.is_empty() && !changed_files.iter().any(|f| f == "增量模式无可用变更，使用全量模式");

        let mut files_to_analyze: Vec<String> = Vec::new();
        let mut from_cache = 0;
        let mut new_files = 0;
        let mut deleted_files = 0;

        if is_incremental {
            for file in &changed_files {
                let file_path = Path::new(project_path).join(file);

                if !file_path.exists() {
                    deleted_files += 1;
                    continue;
                }

                if let Some(cached) = cached_files.get(file)
                    && let Ok(metadata) = std::fs::metadata(&file_path)
                        && let Ok(modified) = metadata.modified() {
                            let modified_secs = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .map_or(0, |d| d.as_secs());

                            if modified_secs == cached.last_modified {
                                from_cache += 1;
                                continue;
                            }
                        }

                if !cached_files.contains_key(file) {
                    new_files += 1;
                }

                files_to_analyze.push(file.clone());
            }
        }

        let mut affected_files = HashMap::new();
        if self.config.include_dependents && is_incremental {
            for file in &files_to_analyze {
                let dependents = self.find_dependent_files(project_path, file);
                for dep in dependents {
                    if !files_to_analyze.contains(&dep) && !affected_files.contains_key(&dep) {
                        affected_files.insert(dep.clone(), true);
                    }
                }
            }
        }

        let all_files: Vec<String> = if is_incremental {
            files_to_analyze.iter().chain(affected_files.keys()).cloned().collect()
        } else {
            Vec::new()
        };

        let (report, overall_change, high_risk_changes) = if is_incremental && !all_files.is_empty() {
            let mut file_entropies: Vec<FileEntropy> = Vec::with_capacity(all_files.len());
            let mut total_lines = 0;
            let mut structural_sum = 0.0;
            let mut complexity_sum = 0.0;
            let mut naming_sum = 0.0;

            let results: Vec<_> = all_files
                .par_iter()
                .filter_map(|file| {
                    let file_path = Path::new(project_path).join(file);
                    std::fs::read_to_string(&file_path).ok().map(|content| {
                        let (file_entropy, _, _, _) = build_file_metrics(file, &content);
                        (file_entropy, content)
                    })
                })
                .collect();

            for (file_entropy, _content) in results {
                structural_sum += file_entropy.structural_score;
                complexity_sum += file_entropy.complexity_score;
                naming_sum += file_entropy.naming_score;
                total_lines += file_entropy.lines;
                file_entropies.push(file_entropy);
            }

            let file_count = file_entropies.len();
            let dimensions = if file_count > 0 {
                EntropyDimensions {
                    structural: structural_sum / file_count as f64,
                    complexity: complexity_sum / file_count as f64,
                    coupling: 0.0,
                    naming: naming_sum / file_count as f64,
                    test: 0.0,
                }
            } else {
                EntropyDimensions {
                    structural: 0.0,
                    complexity: 0.0,
                    coupling: 0.0,
                    naming: 0.0,
                    test: 0.0,
                }
            };

            let weights = DimensionWeights::default();
            let overall_score = calculate_overall_score(&dimensions, &weights);
            let grade = EntropyGrade::from_score(overall_score);

            let high_risk: Vec<String> = file_entropies
                .iter()
                .filter(|fe| fe.complexity_score > 60.0 || fe.lines > 500)
                .map(|fe| fe.path.clone())
                .collect();

            let change = self.calculate_entropy_change(&all_files, &cached_files, &dimensions);

            let report = EntropyReport {
                overall_score,
                grade,
                dimensions,
                dimension_weights: weights,
                file_count,
                total_lines,
                breakdown: file_entropies,
                high_risk_files: high_risk.clone(),
            };

            (report, change, high_risk)
        } else {
            let full_report = entropy_service::run_entropy_check(project_path)?;
            (full_report, 0.0, Vec::new())
        };

        if self.config.use_cache && !all_files.is_empty() {
            let mut new_cache: HashMap<String, FileEntropyCache> = cached_files;
            for file in &all_files {
                let file_path = Path::new(project_path).join(file);
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    let (fe, _, _, _) = build_file_metrics(file, &content);
                    let last_modified = std::fs::metadata(&file_path)
                        .and_then(|m| m.modified())
                        .map_or(0, |t| t.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()));

                    new_cache.insert(file.clone(), FileEntropyCache {
                        file_path: file.clone(),
                        complexity_score: fe.complexity_score,
                        structural_score: fe.structural_score,
                        naming_score: fe.naming_score,
                        lines: fe.lines,
                        last_modified,
                    });
                }
            }
            let _ = self.save_cache(project_path, &new_cache);
        }

        let duration = start.elapsed().as_millis() as u64;

        Ok(IncrementalResult {
            changed_files,
            new_files,
            deleted_files,
            affected_files: affected_files.len(),
            overall_change,
            high_risk_changes,
            report,
            duration_ms: duration,
            from_cache,
            is_incremental,
        })
    }

    fn get_changed_files(&self, project_path: &str) -> CellResult<Vec<String>> {
        let mut files = Vec::new();

        let git_check = Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(project_path)
            .output();

        if git_check.as_ref().map_or(true, |o| !o.status.success()) {
            return Ok(vec!["增量模式无可用变更，使用全量模式".to_string()]);
        }

        let output = Command::new("git")
            .args(["diff", "--name-only", "HEAD"])
            .current_dir(project_path)
            .output();

        if let Ok(output) = output
            && output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if !line.is_empty() && (line.ends_with(".rs") || line.ends_with(".go") || line.ends_with(".ts") || line.ends_with(".js")) {
                        files.push(line.to_string());
                    }
                }
            }

        let staged = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(project_path)
            .output();

        if let Ok(output) = staged
            && output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if !line.is_empty() && !files.contains(&line.to_string())
                        && (line.ends_with(".rs") || line.ends_with(".go") || line.ends_with(".ts") || line.ends_with(".js")) {
                            files.push(line.to_string());
                        }
                }
            }

        let untracked = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(project_path)
            .output();

        if let Ok(output) = untracked
            && output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if !line.is_empty() && !files.contains(&line.to_string())
                        && (line.ends_with(".rs") || line.ends_with(".go") || line.ends_with(".ts") || line.ends_with(".js")) {
                            files.push(line.to_string());
                        }
                }
            }

        if files.is_empty() {
            files.push("增量模式无可用变更，使用全量模式".to_string());
        }

        Ok(files)
    }

    fn find_dependent_files(&self, project_path: &str, target_file: &str) -> Vec<String> {
        use walkdir::WalkDir;

        let mut dependents = Vec::new();
        let target_name = Path::new(target_file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let rel_path = entry.path()
                    .strip_prefix(project_path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                if content.contains(target_name) && rel_path != target_file {
                    dependents.push(rel_path);
                }
            }
        }

        dependents
    }

    fn calculate_entropy_change(
        &self,
        files: &[String],
        cached: &HashMap<String, FileEntropyCache>,
        current_dimensions: &EntropyDimensions,
    ) -> f64 {
        let mut total_change = 0.0;
        let mut count = 0;

        for path in files {
            if let Some(cached_entropy) = cached.get(path) {
                let current = cached_entropy.complexity_score;
                total_change += current;
                count += 1;
            }
        }

        if count > 0 {
            let cached_avg = total_change / f64::from(count);
            (current_dimensions.complexity - cached_avg).abs()
        } else {
            0.0
        }
    }

    fn load_cache(&self, project_path: &str) -> CellResult<HashMap<String, FileEntropyCache>> {
        let cache_file = Path::new(project_path).join(&self.config.cache_dir).join("entropy_cache.json");

        if !cache_file.exists() {
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(&cache_file)?;
        let cache: HashMap<String, FileEntropyCache> = serde_json::from_str(&content)
            .unwrap_or_default();

        Ok(cache)
    }

    fn save_cache(&self, project_path: &str, cache: &HashMap<String, FileEntropyCache>) -> CellResult<()> {
        let cache_dir = Path::new(project_path).join(&self.config.cache_dir);
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("entropy_cache.json");
        let content = serde_json::to_string(cache)?;
        std::fs::write(cache_file, content)?;

        Ok(())
    }

    pub fn format_result(&self, result: &IncrementalResult) -> String {
        let mut output = String::new();

        let mode = if result.is_incremental { "增量" } else { "全量" };

        output.push_str(&format!(
            "\n📊 {}熵值分析 (耗时: {}ms)\n\n",
            mode, result.duration_ms
        ));

        if result.is_incremental {
            output.push_str("## 变更概览\n\n");
            output.push_str("| 指标 | 数量 |\n");
            output.push_str("|------|------|\n");
            output.push_str(&format!("| 变更文件 | {} |\n", result.changed_files.len()));
            output.push_str(&format!("| 新增文件 | {} |\n", result.new_files));
            output.push_str(&format!("| 删除文件 | {} |\n", result.deleted_files));
            output.push_str(&format!("| 依赖影响 | {} |\n", result.affected_files));
            output.push_str(&format!("| 缓存命中 | {} |\n", result.from_cache));

            output.push('\n');
            output.push_str("## 熵值变化\n\n");
            let change_indicator = if result.overall_change > 1.0 { "📈" } else if result.overall_change < -1.0 { "📉" } else { "➖" };
            output.push_str(&format!("{} 复杂度变化: {:.2}\n\n", change_indicator, result.overall_change));

            if !result.changed_files.is_empty() {
                output.push_str("### 变更的文件\n\n");
                for file in result.changed_files.iter().take(10) {
                    output.push_str(&format!("- {file}\n"));
                }
                if result.changed_files.len() > 10 {
                    output.push_str(&format!("- ... 及其他 {} 个文件\n", result.changed_files.len() - 10));
                }
                output.push('\n');
            }
        }

        output.push_str(&format!(
            "## 整体状态\n\n- **{}熵值**: {:.2} ({:?})\n",
            mode,
            result.report.overall_score,
            result.report.grade
        ));
        output.push_str(&format!("- **文件数**: {}\n", result.report.file_count));
        output.push_str(&format!("- **总行数**: {}\n", result.report.total_lines));

        if !result.high_risk_changes.is_empty() {
            output.push_str("\n## ⚠️  高风险文件\n\n");
            for file in &result.high_risk_changes {
                output.push_str(&format!("- {file}\n"));
            }
        }

        output
    }
}

impl Default for IncrementalEntropyService {
    fn default() -> Self {
        Self::new()
    }
}
