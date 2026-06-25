use crate::application::ports::evolution_store::EvolutionStorePort;
use crate::domain::errors::{CellError, CellResult};
use crate::domain::evolution::*;

pub struct EvolutionService<T: EvolutionStorePort> {
    store: T,
}

impl<T: EvolutionStorePort> EvolutionService<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }

    pub fn start_cycle(&self, project_path: &str) -> CellResult<EvolutionLog> {
        if let Some(current) = self.store.load_current_cycle(project_path)?
            && current.phase != EvolutionPhase::Completed
        {
            return Err(CellError::Config(format!(
                "There is already an active evolution cycle #{}. Complete it first.",
                current.cycle_number
            )));
        }
        let next_num = self.store.get_next_cycle_number(project_path)?;
        let log = EvolutionLog::new(next_num);
        self.store.save_current_cycle(project_path, &log)?;
        Ok(log)
    }

    pub fn get_current_cycle(&self, project_path: &str) -> CellResult<Option<EvolutionLog>> {
        self.store.load_current_cycle(project_path)
    }

    pub fn report_issue(&self, project_path: &str, title: &str, description: &str, category: IssueCategory, severity: IssueSeverity) -> CellResult<EvolutionLog> {
        let mut log = self.require_current_cycle(project_path)?;
        log.add_issue(title, description, category, severity);
        self.store.save_current_cycle(project_path, &log)?;
        Ok(log)
    }

    pub fn add_improvement(&self, project_path: &str, title: &str, description: &str, category: ImprovementCategory, impact: ImpactLevel, effort: EffortEstimate) -> CellResult<EvolutionLog> {
        let mut log = self.require_current_cycle(project_path)?;
        log.add_improvement(title, description, category, impact, effort);
        self.store.save_current_cycle(project_path, &log)?;
        Ok(log)
    }

    pub fn generate_suggestions(&self, project_path: &str) -> CellResult<Vec<Improvement>> {
        let log = self.require_current_cycle(project_path)?;
        let suggestions = suggest_improvements_based_on_issues(&log.issues);
        let improvements: Vec<Improvement> = suggestions
            .into_iter()
            .map(|(title, desc, cat, impact, effort)| {
                let mut imp = Improvement {
                    id: uuid::Uuid::new_v4(),
                    title,
                    description: desc,
                    category: cat,
                    related_issue_ids: Vec::new(),
                    expected_impact: impact,
                    implementation_effort: effort,
                    status: ImprovementStatus::Proposed,
                    proposed_at: chrono::Utc::now(),
                    applied_at: None,
                    applied_by: None,
                    acceptance_criteria: Vec::new(),
                };
                for issue in &log.issues {
                    if category_matches_issue(&imp.category, &issue.category) {
                        imp.related_issue_ids.push(issue.id);
                    }
                }
                imp
            })
            .collect();
        Ok(improvements)
    }

    pub fn apply_improvement(&self, project_path: &str, improvement_id: &str, applied_by: Option<&str>) -> CellResult<EvolutionLog> {
        let mut log = self.require_current_cycle(project_path)?;
        let id = uuid::Uuid::parse_str(improvement_id)
            .map_err(|e| CellError::Config(format!("Invalid improvement UUID: {}", e)))?;

        let idx = log.improvements.iter().position(|i| i.id == id)
            .ok_or_else(|| CellError::Config(format!("Improvement with id {} not found", improvement_id)))?;

        log.improvements[idx].status = ImprovementStatus::Applied;
        log.improvements[idx].applied_at = Some(chrono::Utc::now());
        log.improvements[idx].applied_by = applied_by.map(|s| s.to_string());

        for issue_id in &log.improvements[idx].related_issue_ids.clone() {
            if let Some(issue_idx) = log.issues.iter().position(|i| i.id == *issue_id) {
                log.issues[issue_idx].status = IssueStatus::FixApplied;
            }
        }

        self.store.save_current_cycle(project_path, &log)?;
        Ok(log)
    }

    pub fn complete_cycle(&self, project_path: &str) -> CellResult<EvolutionLog> {
        let mut log = self.require_current_cycle(project_path)?;
        log.phase = EvolutionPhase::Completed;
        log.completed_at = Some(chrono::Utc::now());
        self.store.archive_cycle(project_path, &log)?;
        Ok(log)
    }

    pub fn list_history(&self, project_path: &str) -> CellResult<Vec<EvolutionLog>> {
        self.store.list_history(project_path)
    }

    pub fn get_evolution_summary(&self, project_path: &str) -> CellResult<EvolutionSummary> {
        let history = self.store.list_history(project_path)?;
        let current = self.store.load_current_cycle(project_path)?;

        let total_issues: usize = history.iter().map(|l| l.issues.len()).sum();
        let total_improvements: usize = history.iter().map(|l| l.improvements.len()).sum();
        let applied_improvements: usize = history
            .iter()
            .map(|l| l.improvements.iter().filter(|i| i.status == ImprovementStatus::Applied || i.status == ImprovementStatus::Verified).count())
            .sum();

        let mut category_counts = std::collections::HashMap::new();
        for log in &history {
            for (cat, count) in log.issue_count_by_category() {
                *category_counts.entry(cat).or_insert(0) += count;
            }
        }

        Ok(EvolutionSummary {
            cycles_completed: history.len() as u32,
            current_cycle_active: current.is_some(),
            total_issues_reported: total_issues as u32,
            total_improvements_applied: applied_improvements as u32,
            total_improvements_proposed: total_improvements as u32,
            top_categories: category_counts,
        })
    }

    pub fn auto_diagnose(&self, project_path: &str) -> CellResult<EvolutionLog> {
        let mut log = self.require_current_cycle(project_path)?;

        let issues = scan_project_for_issues(project_path);

        for (title, desc, category, severity) in issues {
            let exists = log.issues.iter().any(|i| i.title == title);
            if !exists {
                log.add_issue(&title, &desc, category, severity);
            }
        }

        self.store.save_current_cycle(project_path, &log)?;
        Ok(log)
    }

    fn require_current_cycle(&self, project_path: &str) -> CellResult<EvolutionLog> {
        self.store.load_current_cycle(project_path)?.ok_or_else(|| {
            CellError::Config("No active evolution cycle. Start one with 'evolve cycle start' first.".to_string())
        })
    }
}

fn category_matches_issue(imp_cat: &ImprovementCategory, issue_cat: &IssueCategory) -> bool {
    matches!(
        (imp_cat, issue_cat),
        (ImprovementCategory::ProcessOptimization, IssueCategory::ProcessEfficiency)
            | (ImprovementCategory::NewFeature, IssueCategory::ToolIntelligence)
            | (ImprovementCategory::ConstraintAdjustment, IssueCategory::QualityGate)
            | (ImprovementCategory::NewFeature, IssueCategory::HandoffCompleteness)
            | (ImprovementCategory::TemplateImprovement, IssueCategory::CodeGeneration)
            | (ImprovementCategory::DocumentationUpdate, IssueCategory::Documentation)
            | (ImprovementCategory::Automation, IssueCategory::ToolIntelligence)
            | (ImprovementCategory::Refactoring, IssueCategory::Performance)
            | (ImprovementCategory::ProcessOptimization, IssueCategory::Testing)
    )
}

fn scan_project_for_issues(project_path: &str) -> Vec<(String, String, IssueCategory, IssueSeverity)> {
    use std::path::Path;
    let mut issues = Vec::new();

    let src_dir = Path::new(project_path).join("src");
    let cell_dir = Path::new(project_path).join(".cell");
    let has_tests = src_dir.join("domain").join("tests").exists()
        || src_dir.join("application").join("architecture_tests.rs").exists();

    let file_count = count_rust_files(&src_dir);
    let total_lines = count_lines(&src_dir);

    if !cell_dir.join("progress").exists() {
        issues.push((
            "缺少进度追踪工具".to_string(),
            "没有启用进度追踪，智能体间协作时容易丢失上下文。建议使用 'cell tools enable' 启用。".to_string(),
            IssueCategory::ProcessEfficiency,
            IssueSeverity::High,
        ));
    }

    if !cell_dir.join("decisions").exists() {
        issues.push((
            "缺少决策记录".to_string(),
            "没有架构决策记录（ADR），后续智能体无法理解技术选型背景。建议启用决策记录。".to_string(),
            IssueCategory::Documentation,
            IssueSeverity::High,
        ));
    }

    if !cell_dir.join("handoffs").exists() {
        issues.push((
            "缺少零漂移交接工具".to_string(),
            "没有交接包机制，智能体切换时信息丢失严重。建议启用 handoff 工具。".to_string(),
            IssueCategory::HandoffCompleteness,
            IssueSeverity::Critical,
        ));
    }

    if !cell_dir.join("evolution").exists() {
        issues.push((
            "缺少自进化系统".to_string(),
            "没有启用自进化机制，问题无法被系统地记录和改进。建议启用 evolve 工具。".to_string(),
            IssueCategory::ToolIntelligence,
            IssueSeverity::Medium,
        ));
    }

    if file_count > 10 && !has_tests {
        issues.push((
            "测试覆盖率不足".to_string(),
            format!("项目有 {} 个文件但缺少测试，容易引入回归bug。建议添加单元测试和架构测试。", file_count),
            IssueCategory::Testing,
            IssueSeverity::High,
        ));
    }

    if total_lines > 2000 {
        issues.push((
            "熵值计算可能存在性能问题".to_string(),
            format!("项目规模较大（{} 行），需要确保熵值计算性能优化。建议使用缓存和增量计算。", total_lines),
            IssueCategory::Performance,
            IssueSeverity::Medium,
        ));
    }

    if file_count > 20 {
        issues.push((
            "构建速度可能成为瓶颈".to_string(),
            format!("项目文件数达到 {}，建议配置增量编译和快速验证机制提升开发效率。", file_count),
            IssueCategory::ProcessEfficiency,
            IssueSeverity::Medium,
        ));
    }

    let has_ci = Path::new(project_path).join(".github").join("workflows").exists();
    if file_count > 15 && !has_ci {
        issues.push((
            "缺少CI/CD流水线".to_string(),
            "项目达到一定规模但缺少持续集成，无法自动验证代码质量。建议添加 GitHub Actions 配置。".to_string(),
            IssueCategory::QualityGate,
            IssueSeverity::Medium,
        ));
    }

    let readme_exists = Path::new(project_path).join("README.md").exists();
    if !readme_exists {
        issues.push((
            "缺少项目文档".to_string(),
            "没有 README 文档，新智能体需要从零开始理解项目。建议添加项目说明文档。".to_string(),
            IssueCategory::Documentation,
            IssueSeverity::High,
        ));
    }

    issues
}

fn count_rust_files(dir: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_rust_files(&path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                count += 1;
            }
        }
    }
    count
}

fn count_lines(dir: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_lines(&path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    count += content.lines().count();
                }
            }
        }
    }
    count
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EvolutionSummary {
    pub cycles_completed: u32,
    pub current_cycle_active: bool,
    pub total_issues_reported: u32,
    pub total_improvements_applied: u32,
    pub total_improvements_proposed: u32,
    pub top_categories: std::collections::HashMap<IssueCategory, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockStore {
        current: RefCell<Option<EvolutionLog>>,
        history: RefCell<Vec<EvolutionLog>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                current: RefCell::new(None),
                history: RefCell::new(Vec::new()),
            }
        }
    }

    impl EvolutionStorePort for MockStore {
        fn load_current_cycle(&self, _project_path: &str) -> CellResult<Option<EvolutionLog>> {
            Ok(self.current.borrow().clone())
        }

        fn save_current_cycle(&self, _project_path: &str, log: &EvolutionLog) -> CellResult<()> {
            *self.current.borrow_mut() = Some(log.clone());
            Ok(())
        }

        fn list_history(&self, _project_path: &str) -> CellResult<Vec<EvolutionLog>> {
            Ok(self.history.borrow().clone())
        }

        fn archive_cycle(&self, _project_path: &str, log: &EvolutionLog) -> CellResult<()> {
            self.history.borrow_mut().push(log.clone());
            *self.current.borrow_mut() = None;
            Ok(())
        }

        fn get_next_cycle_number(&self, _project_path: &str) -> CellResult<u32> {
            Ok((self.history.borrow().len() + 1) as u32)
        }
    }

    #[test]
    fn test_start_cycle() {
        let store = MockStore::new();
        let service = EvolutionService::new(store);
        let log = service.start_cycle(".").unwrap();
        assert_eq!(log.cycle_number, 1);
        assert_eq!(log.phase, EvolutionPhase::CollectingIssues);
    }

    #[test]
    fn test_report_issue() {
        let store = MockStore::new();
        let service = EvolutionService::new(store);
        service.start_cycle(".").unwrap();

        let log = service.report_issue(
            ".",
            "交接包信息不全",
            "接手时缺少架构状态信息",
            IssueCategory::HandoffCompleteness,
            IssueSeverity::High,
        ).unwrap();

        assert_eq!(log.issues.len(), 1);
        assert_eq!(log.issues[0].title, "交接包信息不全");
        assert_eq!(log.issues[0].severity, IssueSeverity::High);
    }

    #[test]
    fn test_generate_suggestions() {
        let store = MockStore::new();
        let service = EvolutionService::new(store);
        service.start_cycle(".").unwrap();

        service.report_issue(".", "流程太慢", "步骤太多", IssueCategory::ProcessEfficiency, IssueSeverity::Medium).unwrap();
        service.report_issue(".", "还是慢", "效率低", IssueCategory::ProcessEfficiency, IssueSeverity::Low).unwrap();

        let suggestions = service.generate_suggestions(".").unwrap();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_apply_improvement() {
        let store = MockStore::new();
        let service = EvolutionService::new(store);
        service.start_cycle(".").unwrap();

        let log = service.add_improvement(
            ".",
            "优化流程",
            "减少步骤",
            ImprovementCategory::ProcessOptimization,
            ImpactLevel::High,
            EffortEstimate::Hours,
        ).unwrap();

        let imp_id = log.improvements[0].id.to_string();
        let log = service.apply_improvement(".", &imp_id, Some("agent-1")).unwrap();

        assert_eq!(log.improvements[0].status, ImprovementStatus::Applied);
        assert_eq!(log.improvements[0].applied_by, Some("agent-1".to_string()));
    }

    #[test]
    fn test_no_active_cycle_fails() {
        let store = MockStore::new();
        let service = EvolutionService::new(store);
        let result = service.report_issue(".", "test", "desc", IssueCategory::Other, IssueSeverity::Low);
        assert!(result.is_err());
    }

    #[test]
    fn test_complete_cycle() {
        let store = MockStore::new();
        let service = EvolutionService::new(store);
        service.start_cycle(".").unwrap();
        let log = service.complete_cycle(".").unwrap();

        assert_eq!(log.phase, EvolutionPhase::Completed);
        assert!(log.completed_at.is_some());

        let history = service.list_history(".").unwrap();
        assert_eq!(history.len(), 1);
    }
}
