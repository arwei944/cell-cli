use crate::entropy_service;
use cell_domain::entropy::{EntropyDimensions, EntropyGrade, EntropyReport};
use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyHistoryEntry {
    pub timestamp: i64,
    pub commit_hash: String,
    pub overall_score: f64,
    pub grade: EntropyGrade,
    pub dimensions: EntropyDimensions,
    pub file_count: usize,
    pub total_lines: usize,
}

#[derive(Debug, Clone)]
pub struct EntropyTrend {
    pub history: Vec<EntropyHistoryEntry>,
    pub current: EntropyReport,
    pub trend: TrendDirection,
    pub change_rate: f64,
    pub avg_score: f64,
    pub max_score: f64,
    pub min_score: f64,
    pub high_risk_files_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

impl TrendDirection {
    pub fn label(&self) -> &str {
        match self {
            Self::Increasing => "📈 上升",
            Self::Decreasing => "📉 下降",
            Self::Stable => "➖ 稳定",
        }
    }
}

pub struct EntropyTrendService;

impl EntropyTrendService {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, project_path: &str) -> CellResult<EntropyTrend> {
        let history = self.load_history(project_path)?;
        let mut history_with_git = self.enrich_with_git_history(project_path, history)?;
        
        if history_with_git.len() < 2 {
            let full_history = self.build_full_history(project_path)?;
            history_with_git.extend(full_history);
        }

        self.save_history(project_path, &history_with_git)?;

        let current_report = entropy_service::run_entropy_check(project_path)?;
        let high_risk_count = current_report.high_risk_files.len();
        let overall_score = current_report.overall_score;
        
        let mut full_history = history_with_git.clone();
        
        let current_entry = EntropyHistoryEntry {
            timestamp: chrono::Utc::now().timestamp(),
            commit_hash: self.get_current_commit(project_path).unwrap_or_default(),
            overall_score,
            grade: current_report.grade.clone(),
            dimensions: current_report.dimensions.clone(),
            file_count: current_report.file_count,
            total_lines: current_report.total_lines,
        };
        full_history.push(current_entry);
        
        full_history.sort_by_key(|e| e.timestamp);
        let recent_history: Vec<&EntropyHistoryEntry> = full_history.iter().rev().take(10).rev().collect();
        
        let (trend, change_rate) = self.calculate_trend(&recent_history);
        let avg_score = if recent_history.is_empty() {
            0.0
        } else {
            recent_history.iter().map(|e| e.overall_score).sum::<f64>() / recent_history.len() as f64
        };
        let max_score = recent_history.iter().map(|e| e.overall_score).fold(0.0, f64::max);
        let min_score = recent_history.iter().map(|e| e.overall_score).fold(f64::MAX, f64::min);

        let mut warnings = Vec::new();
        if trend == TrendDirection::Increasing && change_rate > 5.0 {
            warnings.push(format!("⚠️  熵值正在快速上升！变化率: {change_rate:.1}%"));
        }
        if overall_score > 60.0 {
            warnings.push(format!("⚠️  当前熵值 ({overall_score:.1}) 已超过警告阈值"));
        }

        Ok(EntropyTrend {
            history: full_history,
            current: current_report,
            trend,
            change_rate,
            avg_score,
            max_score,
            min_score: if min_score == f64::MAX { 0.0 } else { min_score },
            high_risk_files_count: high_risk_count,
            warnings,
        })
    }

    fn calculate_trend(&self, history: &[&EntropyHistoryEntry]) -> (TrendDirection, f64) {
        if history.len() < 2 {
            return (TrendDirection::Stable, 0.0);
        }

        let first = history.first().unwrap().overall_score;
        let last = history.last().unwrap().overall_score;
        
        if first == 0.0 {
            return (TrendDirection::Stable, 0.0);
        }

        let change_rate = ((last - first) / first).abs() * 100.0;
        
        let direction = if last > first * 1.02 {
            TrendDirection::Increasing
        } else if last < first * 0.98 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        (direction, change_rate)
    }

    fn load_history(&self, project_path: &str) -> CellResult<Vec<EntropyHistoryEntry>> {
        let history_file = Path::new(project_path).join(".cell/cache/entropy_history.json");
        
        if !history_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&history_file)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    fn save_history(&self, project_path: &str, history: &[EntropyHistoryEntry]) -> CellResult<()> {
        let cache_dir = Path::new(project_path).join(".cell/cache");
        std::fs::create_dir_all(&cache_dir)?;

        let history_file = cache_dir.join("entropy_history.json");
        let mut limited_history: VecDeque<EntropyHistoryEntry> = history.iter().cloned().collect();
        while limited_history.len() > 50 {
            limited_history.pop_front();
        }

        let content = serde_json::to_string(&limited_history)?;
        std::fs::write(history_file, content)?;

        Ok(())
    }

    fn enrich_with_git_history(&self, project_path: &str, mut history: Vec<EntropyHistoryEntry>) -> CellResult<Vec<EntropyHistoryEntry>> {
        let output = Command::new("git")
            .args(["log", "--oneline", "-20", "--format=%H %ai"])
            .current_dir(project_path)
            .output();

        if let Ok(output) = output
            && output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let commit = parts[0].to_string();
                        let date_str = parts[1].split(' ').next().unwrap_or("");
                        if let Ok(datetime) = chrono::DateTime::parse_from_str(date_str, "%Y-%m-%d") {
                            let timestamp = datetime.timestamp();
                            
                            if !history.iter().any(|e| e.commit_hash == commit)
                                && let Ok(report) = self.run_entropy_at_commit(project_path, &commit) {
                                    history.push(EntropyHistoryEntry {
                                        timestamp,
                                        commit_hash: commit,
                                        overall_score: report.overall_score,
                                        grade: report.grade,
                                        dimensions: report.dimensions,
                                        file_count: report.file_count,
                                        total_lines: report.total_lines,
                                    });
                                }
                        }
                    }
                }
            }

        history.sort_by_key(|e| e.timestamp);
        Ok(history)
    }

    fn run_entropy_at_commit(&self, project_path: &str, commit: &str) -> CellResult<EntropyReport> {
        let stash_output = Command::new("git")
            .args(["stash"])
            .current_dir(project_path)
            .output();

        let _ = Command::new("git")
            .args(["checkout", commit])
            .current_dir(project_path)
            .output();

        let result = entropy_service::run_entropy_check(project_path);

        let _ = Command::new("git")
            .args(["checkout", "-"])
            .current_dir(project_path)
            .output();

        if stash_output.as_ref().is_ok_and(|o| o.status.success()) {
            let _ = Command::new("git")
                .args(["stash", "pop"])
                .current_dir(project_path)
                .output();
        }

        result
    }

    fn build_full_history(&self, project_path: &str) -> CellResult<Vec<EntropyHistoryEntry>> {
        let mut history = Vec::new();
        let output = Command::new("git")
            .args(["log", "--oneline", "-10", "--format=%H %ai"])
            .current_dir(project_path)
            .output();

        if let Ok(output) = output
            && output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        let commit = parts[0].to_string();
                        let date_str = parts[1].split(' ').next().unwrap_or("");
                        if let Ok(datetime) = chrono::DateTime::parse_from_str(date_str, "%Y-%m-%d") {
                            let timestamp = datetime.timestamp();
                            
                            if let Ok(report) = self.run_entropy_at_commit(project_path, &commit) {
                                history.push(EntropyHistoryEntry {
                                    timestamp,
                                    commit_hash: commit,
                                    overall_score: report.overall_score,
                                    grade: report.grade,
                                    dimensions: report.dimensions,
                                    file_count: report.file_count,
                                    total_lines: report.total_lines,
                                });
                            }
                        }
                    }
                }
            }

        history.sort_by_key(|e| e.timestamp);
        Ok(history)
    }

    fn get_current_commit(&self, project_path: &str) -> CellResult<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(project_path)
            .output()
            .map_err(|e| cell_domain::errors::CellError::Config(format!("Git command failed: {e}")))?;

        if !output.status.success() {
            return Err(cell_domain::errors::CellError::Config(
                "Failed to get current commit".to_string()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn format_trend(&self, trend: &EntropyTrend) -> String {
        let mut output = String::new();

        output.push_str("\n📈 熵值趋势分析\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!("当前熵值: {:.2} {} ({})\n", 
            trend.current.overall_score, 
            trend.current.grade.emoji(), 
            trend.current.grade.label()
        ));
        output.push_str(&format!("趋势方向: {}\n", trend.trend.label()));
        output.push_str(&format!("变化率: {:.2}%\n", trend.change_rate));
        output.push_str(&format!("平均熵值: {:.2}\n", trend.avg_score));
        output.push_str(&format!("最高熵值: {:.2}\n", trend.max_score));
        output.push_str(&format!("最低熵值: {:.2}\n", trend.min_score));
        output.push_str(&format!("分析文件: {} 个\n", trend.current.file_count));
        output.push_str(&format!("代码行数: {} 行\n", trend.current.total_lines));
        output.push_str(&format!("高风险文件: {} 个\n", trend.high_risk_files_count));

        if !trend.warnings.is_empty() {
            output.push_str("\n⚠️  警告:\n");
            for w in &trend.warnings {
                output.push_str(&format!("  {w}\n"));
            }
        }

        output.push_str("\n📊 历史趋势图:\n");
        output.push_str("────────────────────────────────────────────────────────────\n\n");
        
        let recent: Vec<&EntropyHistoryEntry> = trend.history.iter().rev().take(10).rev().collect();
        if recent.len() >= 2 {
            let min_score = recent.iter().map(|e| e.overall_score).fold(f64::MAX, f64::min);
            let max_score = recent.iter().map(|e| e.overall_score).fold(0.0, f64::max);
            let range = (max_score - min_score).max(1.0);

            for entry in recent {
                let normalized = ((entry.overall_score - min_score) / range) * 40.0;
                let bar_length = normalized.round() as usize;
                let date = chrono::DateTime::from_timestamp(entry.timestamp, 0).map_or_else(|| "----".to_string(), |d| d.format("%m-%d").to_string());
                
                output.push_str(&format!(
                    "  {} [{}] {:6.1} {}\n",
                    date,
                    entry.commit_hash.chars().take(7).collect::<String>(),
                    entry.overall_score,
                    "█".repeat(bar_length)
                ));
            }
        } else {
            output.push_str("  暂无足够历史数据，请运行多次熵值检查以建立趋势。\n");
        }

        output.push_str("\n💡 提示: 熵值越低表示系统越健康，持续上升趋势需要关注。\n");

        output
    }
}

impl Default for EntropyTrendService {
    fn default() -> Self {
        Self::new()
    }
}
