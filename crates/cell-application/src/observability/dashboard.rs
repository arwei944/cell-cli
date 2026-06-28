//! 可观测性服务：仪表盘数据组装
//! Observability service: dashboard data assembly

use super::metrics::ObservabilityService;
use super::phases::{build_project_phases, calculate_overall_progress};
use cell_domain::decision::simple_id;
use cell_domain::entropy::EntropyReport;
use cell_domain::errors::CellResult;
use cell_domain::observability::{
    DashboardData, DecisionSummary, DimensionData, EntropySnapshot,
    RecentEvent,
};
use cell_domain::progress::ProgressLog;
use std::path::Path;

/// 构建仪表盘数据 - 组装所有来源的指标
pub fn build_dashboard(
    service: &ObservabilityService,
    project_path: &str,
    entropy_report: &EntropyReport,
    current_progress: Option<&ProgressLog>,
    decisions: &[cell_domain::decision::DecisionRecord],
    evolution: Option<&cell_domain::evolution::EvolutionLog>,
) -> CellResult<DashboardData> {
    let project_name = extract_project_name(project_path);
    let (current_task, task_status, active_blockers, recent_events) =
        build_progress_view(current_progress);
    let recent_decisions = build_recent_decisions(decisions);
    let (issues_count, improvements_count) = build_evolution_counts(evolution);
    let phases = build_project_phases(
        project_path,
        decisions.len(),
        entropy_report.file_count,
        entropy_report.total_lines,
        current_progress.is_some(),
        evolution.is_some(),
    );
    let (overall_progress, current_phase_idx) = calculate_overall_progress(&phases);
    let current_phase = phases
        .get(current_phase_idx).map_or_else(|| "Unknown".to_string(), |p| p.name.clone());
    let (total_tasks, completed_tasks) = aggregate_task_counts(&phases);
    let trend = build_entropy_trend(entropy_report);
    let metrics = service.get_performance_metrics();

    Ok(DashboardData {
        project_name,
        overall_progress,
        current_phase,
        current_phase_index: current_phase_idx,
        phases,
        current_task,
        task_status,
        entropy_score: entropy_report.overall_score,
        entropy_grade: entropy_report.grade.label().to_string(),
        entropy_trend: trend,
        high_risk_files: entropy_report.high_risk_files.clone(),
        recent_events,
        decisions_count: decisions.len(),
        issues_count,
        improvements_count,
        total_tasks,
        completed_tasks,
        active_blockers,
        file_count: entropy_report.file_count,
        total_lines: entropy_report.total_lines,
        dimensions: DimensionData {
            structural: entropy_report.dimensions.structural,
            complexity: entropy_report.dimensions.complexity,
            coupling: entropy_report.dimensions.coupling,
            naming: entropy_report.dimensions.naming,
            test: entropy_report.dimensions.test,
        },
        recent_decisions,
        metrics,
    })
}

fn extract_project_name(project_path: &str) -> String {
    Path::new(project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("cell-project")
        .to_string()
}

fn build_progress_view(progress: Option<&ProgressLog>) -> (Option<String>, String, usize, Vec<RecentEvent>) {
    if let Some(p) = progress {
        let events: Vec<RecentEvent> = p.timeline.iter().rev().take(20).map(|e| RecentEvent {
            timestamp: e.timestamp,
            event_type: format!("{:?}", e.event_type),
            message: e.message.clone(),
            author: e.author.clone(),
        }).collect();
        (Some(p.task_name.clone()), format!("{:?}", p.status), p.active_blockers_count(), events)
    } else {
        (None, "Idle".to_string(), 0, Vec::new())
    }
}

fn build_recent_decisions(decisions: &[cell_domain::decision::DecisionRecord]) -> Vec<DecisionSummary> {
    decisions.iter().take(5).map(|d| DecisionSummary {
        id: simple_id(&d.id),
        title: d.title.clone(),
        category: d.category.label().to_string(),
        status: d.status.label().to_string(),
        made_at: d.made_at,
    }).collect()
}

fn build_evolution_counts(evolution: Option<&cell_domain::evolution::EvolutionLog>) -> (usize, usize) {
    evolution.as_ref().map_or((0, 0), |e| (e.issues.len(), e.improvements.len()))
}

fn aggregate_task_counts(phases: &[cell_domain::observability::ProjectPhase]) -> (usize, usize) {
    let total: usize = phases.iter().map(|p| p.total_tasks).sum();
    let completed: usize = phases.iter().map(|p| p.completed_tasks).sum();
    (total, completed)
}

fn build_entropy_trend(report: &EntropyReport) -> Vec<EntropySnapshot> {
    vec![EntropySnapshot {
        timestamp: chrono::Utc::now(),
        score: report.overall_score,
        grade: report.grade.label().to_string(),
    }]
}
