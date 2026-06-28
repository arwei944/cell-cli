use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration_ms: u64,
    pub description: String,
    pub timestamp: String,
    pub metrics: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkHistory {
    pub benchmarks: Vec<BenchmarkResult>,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub benchmark_name: String,
    pub baseline_avg: f64,
    pub current_avg: f64,
    pub change_percent: f64,
    pub is_regression: bool,
    pub threshold_percent: f64,
}

pub struct BenchmarkService;

impl BenchmarkService {
    pub fn new() -> Self {
        Self
    }

    pub fn run_benchmarks(&self, project_path: &str) -> CellResult<Vec<BenchmarkResult>> {
        let results = vec![
            self.benchmark_build(project_path)?,
            self.benchmark_tests(project_path)?,
            self.benchmark_entropy(project_path)?,
            self.benchmark_arch_validate(project_path)?,
        ];

        self.save_benchmark_results(project_path, &results)?;
        Ok(results)
    }

    pub fn compare_with_baseline(&self, project_path: &str, benchmark_name: &str) -> CellResult<BenchmarkComparison> {
        let history = self.load_history(project_path)?;
        
        let relevant: Vec<&BenchmarkResult> = history.benchmarks
            .iter()
            .filter(|b| b.name == benchmark_name)
            .collect();

        if relevant.len() < 2 {
            return Err(cell_domain::errors::CellError::Config(
                "Not enough benchmark data for comparison".to_string()
            ));
        }

        let baseline_count = (relevant.len() / 2).max(3).min(relevant.len() - 1);
        let baseline_avg: f64 = relevant
            .iter()
            .take(baseline_count)
            .map(|b| b.duration_ms as f64)
            .sum::<f64>() / baseline_count as f64;

        let current_count = (relevant.len() - baseline_count).min(3);
        let current_avg: f64 = relevant
            .iter()
            .rev()
            .take(current_count)
            .map(|b| b.duration_ms as f64)
            .sum::<f64>() / current_count as f64;

        let change_percent = ((current_avg - baseline_avg) / baseline_avg) * 100.0;
        let threshold_percent = 10.0;
        let is_regression = change_percent > threshold_percent;

        Ok(BenchmarkComparison {
            benchmark_name: benchmark_name.to_string(),
            baseline_avg,
            current_avg,
            change_percent,
            is_regression,
            threshold_percent,
        })
    }

    pub fn list_benchmarks(&self, project_path: &str) -> CellResult<Vec<String>> {
        let history = self.load_history(project_path)?;
        let mut names: Vec<String> = history.benchmarks
            .iter()
            .map(|b| b.name.clone())
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    fn benchmark_build(&self, project_path: &str) -> CellResult<BenchmarkResult> {
        let start = std::time::Instant::now();
        
        let output = std::process::Command::new("cargo")
            .current_dir(project_path)
            .args(["build", "--release"])
            .output();

        let duration = start.elapsed();
        let success = output.is_ok_and(|o| o.status.success());

        let mut metrics = std::collections::HashMap::new();
        metrics.insert("success".to_string(), if success { 1.0 } else { 0.0 });

        Ok(BenchmarkResult {
            name: "build_release".to_string(),
            duration_ms: duration.as_millis() as u64,
            description: "Release 构建时间".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics,
        })
    }

    fn benchmark_tests(&self, project_path: &str) -> CellResult<BenchmarkResult> {
        let start = std::time::Instant::now();
        
        let output = std::process::Command::new("cargo")
            .current_dir(project_path)
            .args(["test", "--release", "--", "--quiet"])
            .output();

        let duration = start.elapsed();
        let success = output.is_ok_and(|o| o.status.success());

        let mut metrics = std::collections::HashMap::new();
        metrics.insert("success".to_string(), if success { 1.0 } else { 0.0 });

        Ok(BenchmarkResult {
            name: "test_suite".to_string(),
            duration_ms: duration.as_millis() as u64,
            description: "测试套件执行时间".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics,
        })
    }

    fn benchmark_entropy(&self, project_path: &str) -> CellResult<BenchmarkResult> {
        let start = std::time::Instant::now();
        
        let result = crate::entropy_service::run_entropy_check(project_path);
        let duration = start.elapsed();

        let mut metrics = std::collections::HashMap::new();
        if let Ok(report) = &result {
            metrics.insert("entropy_score".to_string(), report.overall_score);
            metrics.insert("files_analyzed".to_string(), report.file_count as f64);
        }

        Ok(BenchmarkResult {
            name: "entropy_calculation".to_string(),
            duration_ms: duration.as_millis() as u64,
            description: "熵值计算时间".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics,
        })
    }

    fn benchmark_arch_validate(&self, project_path: &str) -> CellResult<BenchmarkResult> {
        use crate::arch_service::{ArchitectureRules, validate_architecture};
        
        let start = std::time::Instant::now();
        
        let rules = ArchitectureRules::default();
        let result = validate_architecture(Path::new(project_path), &rules);
        let duration = start.elapsed();

        let mut metrics = std::collections::HashMap::new();
        metrics.insert("violations".to_string(), result.violations.len() as f64);
        metrics.insert("layers".to_string(), result.layer_stats.len() as f64);
        metrics.insert("passed".to_string(), if result.passed { 1.0 } else { 0.0 });

        Ok(BenchmarkResult {
            name: "arch_validate".to_string(),
            duration_ms: duration.as_millis() as u64,
            description: "架构验证时间".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics,
        })
    }

    fn history_path(project_path: &str) -> std::path::PathBuf {
        Path::new(project_path).join(".cell/benchmarks/history.json")
    }

    fn save_benchmark_results(&self, project_path: &str, results: &[BenchmarkResult]) -> CellResult<()> {
        let mut history = self.load_history(project_path).unwrap_or_else(|_| BenchmarkHistory {
            benchmarks: Vec::new(),
            started_at: chrono::Utc::now().to_rfc3339(),
        });

        for result in results {
            history.benchmarks.push(result.clone());
        }

        if history.benchmarks.len() > 1000 {
            history.benchmarks = history.benchmarks.split_off(history.benchmarks.len() - 1000);
        }

        let path = Self::history_path(project_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&history)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn load_history(&self, project_path: &str) -> CellResult<BenchmarkHistory> {
        let path = Self::history_path(project_path);
        if !path.exists() {
            return Ok(BenchmarkHistory {
                benchmarks: Vec::new(),
                started_at: chrono::Utc::now().to_rfc3339(),
            });
        }
        let content = std::fs::read_to_string(&path)?;
        let history: BenchmarkHistory = serde_json::from_str(&content)
            .map_err(|e| cell_domain::errors::CellError::Config(format!("Invalid benchmark history: {e}")))?;
        Ok(history)
    }
}

impl Default for BenchmarkService {
    fn default() -> Self {
        Self::new()
    }
}

fn _duration_to_ms(d: Duration) -> u64 {
    d.as_millis() as u64
}
