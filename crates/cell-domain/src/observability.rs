use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardData {
    pub project_name: String,
    pub overall_progress: f64,
    pub current_phase: String,
    pub current_phase_index: usize,
    pub phases: Vec<ProjectPhase>,
    pub current_task: Option<String>,
    pub task_status: String,
    pub entropy_score: f64,
    pub entropy_grade: String,
    pub entropy_trend: Vec<EntropySnapshot>,
    pub high_risk_files: Vec<String>,
    pub recent_events: Vec<RecentEvent>,
    pub decisions_count: usize,
    pub issues_count: usize,
    pub improvements_count: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub active_blockers: usize,
    pub file_count: usize,
    pub total_lines: usize,
    pub dimensions: DimensionData,
    pub recent_decisions: Vec<DecisionSummary>,
    pub metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectPhase {
    pub name: String,
    pub description: String,
    pub progress: f64,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub status: PhaseStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum PhaseStatus {
    NotStarted,
    InProgress,
    Completed,
}

impl PhaseStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::NotStarted => "未开始",
            Self::InProgress => "进行中",
            Self::Completed => "已完成",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DimensionData {
    pub structural: f64,
    pub complexity: f64,
    pub coupling: f64,
    pub naming: f64,
    pub test: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropySnapshot {
    pub timestamp: DateTime<Utc>,
    pub score: f64,
    pub grade: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub message: String,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionSummary {
    pub id: String,
    pub title: String,
    pub category: String,
    pub status: String,
    pub made_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetrics {
    pub avg_command_time_ms: f64,
    pub command_count: u64,
    pub success_rate: f64,
    pub accuracy_score: f64,
    pub efficiency_score: f64,
    pub last_updated: DateTime<Utc>,
    pub command_stats: HashMap<String, CommandStat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandStat {
    pub count: u64,
    pub avg_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub success_count: u64,
    pub failure_count: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_command_time_ms: 0.0,
            command_count: 0,
            success_rate: 100.0,
            accuracy_score: 85.0,
            efficiency_score: 75.0,
            last_updated: Utc::now(),
            command_stats: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandExecution {
    pub command: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<f64>,
    pub success: Option<bool>,
    pub error: Option<String>,
    pub args: Vec<String>,
}

impl CommandExecution {
    pub fn new(command: &str, args: Vec<String>) -> Self {
        Self {
            command: command.to_string(),
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            success: None,
            error: None,
            args,
        }
    }

    pub fn complete(&mut self, success: bool, error: Option<&str>) {
        self.completed_at = Some(Utc::now());
        let started = self.started_at;
        let ended = self.completed_at.expect("completed_at just set above");
        self.duration_ms = Some(
            (ended - started)
                .num_microseconds()
                .unwrap_or(0) as f64
                / 1000.0,
        );
        self.success = Some(success);
        self.error = error.map(std::string::ToString::to_string);
    }
}
