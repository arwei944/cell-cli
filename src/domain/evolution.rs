use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvolutionLog {
    pub log_id: Uuid,
    pub cycle_number: u32,
    pub phase: EvolutionPhase,
    pub issues: Vec<Issue>,
    pub improvements: Vec<Improvement>,
    pub metrics_before: Option<EvolutionMetrics>,
    pub metrics_after: Option<EvolutionMetrics>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub lessons_learned: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EvolutionPhase {
    CollectingIssues,
    Analyzing,
    GeneratingImprovements,
    Applying,
    Measuring,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub frequency: u32,
    pub reported_by: Option<String>,
    pub reported_at: DateTime<Utc>,
    pub context: Option<String>,
    pub related_components: Vec<String>,
    pub status: IssueStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    ProcessEfficiency,
    ToolIntelligence,
    QualityGate,
    HandoffCompleteness,
    CodeGeneration,
    Documentation,
    ArchitectureDrift,
    EntropyGrowth,
    Testing,
    Performance,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Trivial,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum IssueStatus {
    Reported,
    Analyzing,
    PlanningFix,
    FixApplied,
    Verified,
    WonTFix,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Improvement {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: ImprovementCategory,
    pub related_issue_ids: Vec<Uuid>,
    pub expected_impact: ImpactLevel,
    pub implementation_effort: EffortEstimate,
    pub status: ImprovementStatus,
    pub proposed_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<String>,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ImprovementCategory {
    ProcessOptimization,
    NewFeature,
    QualityGateTuning,
    TemplateImprovement,
    DocumentationUpdate,
    Automation,
    Refactoring,
    ConstraintAdjustment,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ImpactLevel {
    Transformational,
    High,
    Medium,
    Low,
    Minimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EffortEstimate {
    Minutes,
    Hours,
    Days,
    Weeks,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ImprovementStatus {
    Proposed,
    Planned,
    InProgress,
    Applied,
    Verified,
    Rejected,
    RolledBack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvolutionMetrics {
    pub measured_at: DateTime<Utc>,
    pub overall_efficiency_score: f64,
    pub dimension_scores: HashMap<String, f64>,
    pub total_issues_resolved: u32,
    pub total_improvements_applied: u32,
    pub avg_issue_resolution_time_hours: Option<f64>,
    pub handoff_completeness_score: f64,
    pub architecture_violations: u32,
    pub entropy_score: f64,
    pub test_coverage_percent: Option<f64>,
    pub build_time_seconds: Option<f64>,
}

impl EvolutionLog {
    pub fn new(cycle_number: u32) -> Self {
        Self {
            log_id: Uuid::new_v4(),
            cycle_number,
            phase: EvolutionPhase::CollectingIssues,
            issues: Vec::new(),
            improvements: Vec::new(),
            metrics_before: None,
            metrics_after: None,
            started_at: Utc::now(),
            completed_at: None,
            lessons_learned: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_issue(&mut self, title: &str, description: &str, category: IssueCategory, severity: IssueSeverity) -> Uuid {
        let issue = Issue {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            severity,
            frequency: 1,
            reported_by: None,
            reported_at: Utc::now(),
            context: None,
            related_components: Vec::new(),
            status: IssueStatus::Reported,
        };
        let id = issue.id;
        self.issues.push(issue);
        id
    }

    pub fn add_improvement(&mut self, title: &str, description: &str, category: ImprovementCategory, impact: ImpactLevel, effort: EffortEstimate) -> Uuid {
        let improvement = Improvement {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            category,
            related_issue_ids: Vec::new(),
            expected_impact: impact,
            implementation_effort: effort,
            status: ImprovementStatus::Proposed,
            proposed_at: Utc::now(),
            applied_at: None,
            applied_by: None,
            acceptance_criteria: Vec::new(),
        };
        let id = improvement.id;
        self.improvements.push(improvement);
        id
    }

    pub fn critical_issues(&self) -> Vec<&Issue> {
        self.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Critical || i.severity == IssueSeverity::High)
            .collect()
    }

    pub fn pending_improvements(&self) -> Vec<&Improvement> {
        self.improvements.iter()
            .filter(|i| i.status == ImprovementStatus::Proposed || i.status == ImprovementStatus::Planned)
            .collect()
    }

    pub fn issue_count_by_category(&self) -> HashMap<IssueCategory, usize> {
        let mut counts = HashMap::new();
        for issue in &self.issues {
            *counts.entry(issue.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn top_issue_categories(&self, top_n: usize) -> Vec<(IssueCategory, usize)> {
        let mut counts: Vec<_> = self.issue_count_by_category().into_iter().collect();
        counts.sort_by_key(|b| std::cmp::Reverse(b.1));
        counts.truncate(top_n);
        counts
    }
}

impl EvolutionMetrics {
    pub fn new() -> Self {
        Self {
            measured_at: Utc::now(),
            overall_efficiency_score: 0.0,
            dimension_scores: HashMap::new(),
            total_issues_resolved: 0,
            total_improvements_applied: 0,
            avg_issue_resolution_time_hours: None,
            handoff_completeness_score: 0.0,
            architecture_violations: 0,
            entropy_score: 0.0,
            test_coverage_percent: None,
            build_time_seconds: None,
        }
    }

    pub fn calculate_overall_score(&mut self) -> f64 {
        let mut score = 100.0;
        score -= (self.architecture_violations as f64) * 5.0;
        score -= self.entropy_score * 2.0;
        score += self.handoff_completeness_score * 10.0;
        score += (self.total_improvements_applied as f64) * 0.5;
        self.overall_efficiency_score = score.max(0.0);
        self.overall_efficiency_score
    }
}

impl Default for EvolutionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub fn suggest_improvements_based_on_issues(issues: &[Issue]) -> Vec<(String, String, ImprovementCategory, ImpactLevel, EffortEstimate)> {
    let mut suggestions = Vec::new();

    let mut process_issues = 0;
    let mut handoff_issues = 0;
    let mut quality_issues = 0;
    let mut tool_issues = 0;
    let mut perf_issues = 0;
    let mut test_issues = 0;
    let mut docs_issues = 0;

    for issue in issues {
        match issue.category {
            IssueCategory::ProcessEfficiency => process_issues += 1,
            IssueCategory::HandoffCompleteness => handoff_issues += 1,
            IssueCategory::QualityGate => quality_issues += 1,
            IssueCategory::ToolIntelligence => tool_issues += 1,
            IssueCategory::Performance => perf_issues += 1,
            IssueCategory::Testing => test_issues += 1,
            IssueCategory::Documentation => docs_issues += 1,
            _ => {}
        }
    }

    if process_issues >= 2 {
        suggestions.push((
            "优化开发流程".to_string(),
            "根据多次出现的流程效率问题，简化开发步骤，减少不必要的人工干预。".to_string(),
            ImprovementCategory::ProcessOptimization,
            ImpactLevel::High,
            EffortEstimate::Hours,
        ));
    }

    if handoff_issues >= 1 {
        suggestions.push((
            "增强交接包完整性".to_string(),
            "在交接包中增加更多维度的信息，减少接手时的信息黑洞。".to_string(),
            ImprovementCategory::NewFeature,
            ImpactLevel::High,
            EffortEstimate::Hours,
        ));
    }

    if quality_issues >= 2 {
        suggestions.push((
            "调整质量门禁阈值".to_string(),
            "根据实际情况调整架构检查和熵值门禁的阈值，使其更符合项目当前阶段。".to_string(),
            ImprovementCategory::ConstraintAdjustment,
            ImpactLevel::Medium,
            EffortEstimate::Minutes,
        ));
    }

    if tool_issues >= 1 {
        suggestions.push((
            "提升工具自动化程度".to_string(),
            "将更多手动操作自动化，减少人工错误，提升效率。".to_string(),
            ImprovementCategory::Automation,
            ImpactLevel::Transformational,
            EffortEstimate::Days,
        ));
    }

    if perf_issues >= 1 {
        suggestions.push((
            "性能优化与缓存机制".to_string(),
            "识别性能瓶颈，引入缓存、增量计算、批量处理等优化手段。对高频计算进行性能基准测试。".to_string(),
            ImprovementCategory::Refactoring,
            ImpactLevel::High,
            EffortEstimate::Hours,
        ));
    }

    if test_issues >= 1 {
        suggestions.push((
            "完善测试体系与快速验证".to_string(),
            "建立分层测试策略（单元测试/集成测试/架构测试），引入快速验证机制，减少每次构建等待时间。".to_string(),
            ImprovementCategory::ProcessOptimization,
            ImpactLevel::High,
            EffortEstimate::Hours,
        ));
    }

    if docs_issues >= 1 {
        suggestions.push((
            "完善项目文档体系".to_string(),
            "补充README、架构文档、API文档等，降低新智能体的上手成本。".to_string(),
            ImprovementCategory::DocumentationUpdate,
            ImpactLevel::Medium,
            EffortEstimate::Hours,
        ));
    }

    if suggestions.is_empty() {
        suggestions.push((
            "建立问题反馈机制".to_string(),
            "鼓励智能体在每轮开发后记录遇到的问题，为持续改进提供数据支持。".to_string(),
            ImprovementCategory::ProcessOptimization,
            ImpactLevel::Medium,
            EffortEstimate::Minutes,
        ));
    }

    suggestions
}
