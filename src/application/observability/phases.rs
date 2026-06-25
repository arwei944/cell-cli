//! 可观测性服务：项目阶段计算
//! Observability service: project phase calculation

use crate::domain::observability::{PhaseStatus, ProjectPhase};
use std::path::Path;

const PHASE_WEIGHTS: [f64; 5] = [0.15, 0.20, 0.35, 0.20, 0.10];

/// 构建项目阶段进度数据
pub fn build_project_phases(
    project_path: &str,
    decision_count: usize,
    file_count: usize,
    total_lines: usize,
    has_progress: bool,
    has_evolution: bool,
) -> Vec<ProjectPhase> {
    let cell_dir = Path::new(project_path).join(".cell");
    let has_decisions = decision_count > 0 || cell_dir.join("decisions").exists();
    let has_progress_data = has_progress || cell_dir.join("progress").exists();
    let has_evolution_data = has_evolution || cell_dir.join("evolution").exists();

    let has_src = Path::new(project_path).join("src").exists();
    let has_tests = file_count > 5;
    let has_ci = Path::new(project_path).join(".github").exists();
    let has_readme = Path::new(project_path).join("README.md").exists();

    vec![
        build_requirement_phase(has_readme, decision_count, has_decisions, has_progress_data, has_evolution_data),
        build_architecture_phase(has_decisions, file_count, project_path),
        build_development_phase(has_src, file_count, total_lines, has_tests, project_path),
        build_test_phase(has_tests, total_lines, file_count, project_path),
        build_release_phase(has_ci, has_readme, total_lines, project_path),
    ]
}

fn build_requirement_phase(
    has_readme: bool,
    decision_count: usize,
    has_decisions: bool,
    has_progress: bool,
    has_evolution: bool,
) -> ProjectPhase {
    let total = 5;
    let mut done = 0;
    if has_readme { done += 1; }
    if decision_count >= 2 { done += 1; }
    if has_decisions { done += 1; }
    if has_progress { done += 1; }
    if has_evolution { done += 1; }
    phase("Requirement Analysis", "需求收集、分析、技术选型决策", total, done)
}

fn build_architecture_phase(has_decisions: bool, file_count: usize, project_path: &str) -> ProjectPhase {
    let total = 5;
    let mut done = 0;
    if has_decisions { done += 1; }
    if file_count > 10 { done += 1; }
    if Path::new(project_path).join("src/domain").exists() { done += 1; }
    if Path::new(project_path).join("src/application").exists() { done += 1; }
    if Path::new(project_path).join("src/adapters").exists() { done += 1; }
    phase("Architecture Design", "架构分层、接口定义、核心抽象设计", total, done)
}

fn build_development_phase(
    has_src: bool,
    file_count: usize,
    total_lines: usize,
    has_tests: bool,
    project_path: &str,
) -> ProjectPhase {
    let total = 8;
    let mut done = 0;
    if has_src { done += 1; }
    if file_count > 15 { done += 1; }
    if total_lines > 1000 { done += 1; }
    if total_lines > 3000 { done += 1; }
    if total_lines > 5000 { done += 1; }
    if file_count > 30 { done += 1; }
    if has_tests { done += 1; }
    if Path::new(project_path).join("src/interfaces").exists() { done += 1; }
    phase("Development", "核心功能开发、单元测试、集成测试", total, done)
}

fn build_test_phase(
    has_tests: bool,
    total_lines: usize,
    file_count: usize,
    project_path: &str,
) -> ProjectPhase {
    let total = 4;
    let mut done = 0;
    if has_tests { done += 1; }
    if total_lines > 500 { done += 1; }
    if file_count > 20 { done += 1; }
    if Path::new(project_path).join("src/domain").join("tests").exists() { done += 1; }
    phase("Testing & Verification", "单元测试、集成测试、架构验证、熵值门禁", total, done)
}

fn build_release_phase(
    has_ci: bool,
    has_readme: bool,
    total_lines: usize,
    project_path: &str,
) -> ProjectPhase {
    let total = 4;
    let mut done = 0;
    if has_ci { done += 1; }
    if Path::new(project_path).join("Cargo.toml").exists() { done += 1; }
    if has_readme { done += 1; }
    if total_lines > 8000 { done += 1; }
    phase("Release & Deploy", "CI/CD配置、文档完善、版本发布", total, done)
}

fn phase(name: &str, description: &str, total: usize, done: usize) -> ProjectPhase {
    let status = if done == 0 {
        PhaseStatus::NotStarted
    } else if done >= total {
        PhaseStatus::Completed
    } else {
        PhaseStatus::InProgress
    };
    ProjectPhase {
        name: name.to_string(),
        description: description.to_string(),
        progress: done as f64 / total as f64 * 100.0,
        total_tasks: total,
        completed_tasks: done,
        status,
    }
}

/// 计算整体进度和当前阶段索引
pub fn calculate_overall_progress(phases: &[ProjectPhase]) -> (f64, usize) {
    let mut overall = 0.0;
    let mut current_idx = 0;
    let mut max_progress = 0.0;

    for (i, phase) in phases.iter().enumerate() {
        overall += phase.progress * PHASE_WEIGHTS[i];
        if phase.progress < 100.0 && max_progress < 100.0 {
            current_idx = i;
            max_progress = phase.progress;
        }
    }

    if overall >= 100.0 {
        current_idx = phases.len() - 1;
    }

    (overall.clamp(0.0, 100.0), current_idx)
}
