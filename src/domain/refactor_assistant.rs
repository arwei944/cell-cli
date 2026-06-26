use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RefactorType {
    Rename,
    ExtractFunction,
    ExtractClass,
    ExtractInterface,
    SplitModule,
    MergeModule,
    MoveFile,
    ChangeSignature,
}

impl Default for RefactorType {
    fn default() -> Self {
        RefactorType::Rename
    }
}

impl fmt::Display for RefactorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorType::Rename => write!(f, "Rename"),
            RefactorType::ExtractFunction => write!(f, "ExtractFunction"),
            RefactorType::ExtractClass => write!(f, "ExtractClass"),
            RefactorType::ExtractInterface => write!(f, "ExtractInterface"),
            RefactorType::SplitModule => write!(f, "SplitModule"),
            RefactorType::MergeModule => write!(f, "MergeModule"),
            RefactorType::MoveFile => write!(f, "MoveFile"),
            RefactorType::ChangeSignature => write!(f, "ChangeSignature"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RefactorSeverity {
    Info,
    Minor,
    Major,
    Critical,
}

impl Default for RefactorSeverity {
    fn default() -> Self {
        RefactorSeverity::Info
    }
}

impl fmt::Display for RefactorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorSeverity::Info => write!(f, "Info"),
            RefactorSeverity::Minor => write!(f, "Minor"),
            RefactorSeverity::Major => write!(f, "Major"),
            RefactorSeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl RefactorSeverity {
    pub fn weight(&self) -> f64 {
        match self {
            RefactorSeverity::Info => 1.0,
            RefactorSeverity::Minor => 2.0,
            RefactorSeverity::Major => 5.0,
            RefactorSeverity::Critical => 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RefactorStatus {
    Proposed,
    Planned,
    InProgress,
    Completed,
    RolledBack,
}

impl Default for RefactorStatus {
    fn default() -> Self {
        RefactorStatus::Proposed
    }
}

impl fmt::Display for RefactorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorStatus::Proposed => write!(f, "Proposed"),
            RefactorStatus::Planned => write!(f, "Planned"),
            RefactorStatus::InProgress => write!(f, "InProgress"),
            RefactorStatus::Completed => write!(f, "Completed"),
            RefactorStatus::RolledBack => write!(f, "RolledBack"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CodeSmellType {
    LongFunction,
    GodClass,
    DuplicateCode,
    LongParameterList,
    DeepInheritance,
    ShotgunSurgery,
    FeatureEnvy,
    DataClass,
}

impl fmt::Display for CodeSmellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeSmellType::LongFunction => write!(f, "LongFunction"),
            CodeSmellType::GodClass => write!(f, "GodClass"),
            CodeSmellType::DuplicateCode => write!(f, "DuplicateCode"),
            CodeSmellType::LongParameterList => write!(f, "LongParameterList"),
            CodeSmellType::DeepInheritance => write!(f, "DeepInheritance"),
            CodeSmellType::ShotgunSurgery => write!(f, "ShotgunSurgery"),
            CodeSmellType::FeatureEnvy => write!(f, "FeatureEnvy"),
            CodeSmellType::DataClass => write!(f, "DataClass"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmell {
    pub smell_type: CodeSmellType,
    pub description: String,
    pub severity: RefactorSeverity,
    pub location: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorProposal {
    pub id: String,
    pub refactor_type: RefactorType,
    pub title: String,
    pub description: String,
    pub severity: RefactorSeverity,
    pub status: RefactorStatus,
    pub affected_files: Vec<String>,
    pub estimated_effort_hours: f64,
    pub benefit_score: f64,
    pub priority_score: f64,
    pub created_at: String,
    pub updated_at: String,
    pub code_smells: Vec<CodeSmell>,
}

impl RefactorProposal {
    pub fn new(
        id: impl Into<String>,
        refactor_type: RefactorType,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            refactor_type,
            title: title.into(),
            description: description.into(),
            severity: RefactorSeverity::Info,
            status: RefactorStatus::Proposed,
            affected_files: Vec::new(),
            estimated_effort_hours: 0.0,
            benefit_score: 0.0,
            priority_score: 0.0,
            created_at: now.clone(),
            updated_at: now,
            code_smells: Vec::new(),
        }
    }

    pub fn with_severity(mut self, severity: RefactorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_affected_files(mut self, files: Vec<String>) -> Self {
        self.affected_files = files;
        self
    }

    pub fn with_estimated_effort(mut self, hours: f64) -> Self {
        self.estimated_effort_hours = hours;
        self
    }

    pub fn with_benefit_score(mut self, score: f64) -> Self {
        self.benefit_score = score;
        self
    }

    pub fn with_code_smells(mut self, smells: Vec<CodeSmell>) -> Self {
        self.code_smells = smells;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorStep {
    pub id: String,
    pub description: String,
    pub order: usize,
    pub file_path: String,
    pub expected_changes: String,
    pub rollback_instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorImpact {
    pub blast_radius: usize,
    pub affected_modules: Vec<String>,
    pub affected_tests: Vec<String>,
    pub risk_level: RefactorSeverity,
    pub estimated_recovery_minutes: f64,
    pub breaking_changes: bool,
    pub dependencies_affected: bool,
    pub impact_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorExecutionPlan {
    pub proposal_id: String,
    pub steps: Vec<RefactorStep>,
    pub total_steps: usize,
    pub estimated_duration_minutes: f64,
    pub prerequisites: Vec<String>,
    pub rollback_plan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorHistoryEntry {
    pub proposal_id: String,
    pub proposal_title: String,
    pub refactor_type: RefactorType,
    pub status: RefactorStatus,
    pub executed_at: String,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefactorError {
    ProposalNotFound(String),
    InvalidStatus(String),
    ExecutionFailed(String),
    RollbackFailed(String),
    PlanNotReady(String),
}

impl fmt::Display for RefactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefactorError::ProposalNotFound(id) => write!(f, "Proposal not found: {}", id),
            RefactorError::InvalidStatus(msg) => write!(f, "Invalid status: {}", msg),
            RefactorError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            RefactorError::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            RefactorError::PlanNotReady(msg) => write!(f, "Plan not ready: {}", msg),
        }
    }
}

impl std::error::Error for RefactorError {}

pub type RefactorResult<T> = Result<T, RefactorError>;

pub struct RefactorAssistant {
    proposals: HashMap<String, RefactorProposal>,
    execution_plans: HashMap<String, RefactorExecutionPlan>,
    history: Vec<RefactorHistoryEntry>,
    smell_detectors: Vec<SmellDetector>,
}

struct SmellDetector {
    smell_type: CodeSmellType,
    metric_name: String,
    threshold: f64,
    operator: ComparisonOperator,
    severity: RefactorSeverity,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl ComparisonOperator {
    fn compare(&self, value: f64, threshold: f64) -> bool {
        match self {
            ComparisonOperator::GreaterThan => value > threshold,
            ComparisonOperator::LessThan => value < threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold,
            ComparisonOperator::LessThanOrEqual => value <= threshold,
        }
    }
}

impl RefactorAssistant {
    pub fn new() -> Self {
        let mut assistant = Self {
            proposals: HashMap::new(),
            execution_plans: HashMap::new(),
            history: Vec::new(),
            smell_detectors: Vec::new(),
        };
        assistant.load_builtin_detectors();
        assistant
    }

    fn load_builtin_detectors(&mut self) {
        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::LongFunction,
            metric_name: "function_lines".to_string(),
            threshold: 50.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Minor,
            description: "函数过长，难以理解和维护".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::GodClass,
            metric_name: "class_method_count".to_string(),
            threshold: 30.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Major,
            description: "类拥有过多方法，承担了过多职责".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::DuplicateCode,
            metric_name: "duplication_ratio".to_string(),
            threshold: 10.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Major,
            description: "存在大量重复代码，维护成本高".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::LongParameterList,
            metric_name: "parameter_count".to_string(),
            threshold: 8.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Minor,
            description: "函数参数列表过长，调用困难".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::DeepInheritance,
            metric_name: "inheritance_depth".to_string(),
            threshold: 5.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Minor,
            description: "继承层次过深，增加了复杂度".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::ShotgunSurgery,
            metric_name: "change_impact_files".to_string(),
            threshold: 10.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Major,
            description: "一个小改动需要修改很多文件".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::FeatureEnvy,
            metric_name: "external_calls_ratio".to_string(),
            threshold: 60.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Minor,
            description: "方法过度依赖其他类的数据和方法".to_string(),
        });

        self.smell_detectors.push(SmellDetector {
            smell_type: CodeSmellType::DataClass,
            metric_name: "data_only_ratio".to_string(),
            threshold: 80.0,
            operator: ComparisonOperator::GreaterThan,
            severity: RefactorSeverity::Info,
            description: "类只包含数据字段，缺乏行为方法".to_string(),
        });
    }

    pub fn add_proposal(&mut self, proposal: RefactorProposal) {
        let id = proposal.id.clone();
        self.proposals.insert(id, proposal);
    }

    pub fn get_proposal(&self, id: &str) -> RefactorResult<&RefactorProposal> {
        self.proposals
            .get(id)
            .ok_or_else(|| RefactorError::ProposalNotFound(id.to_string()))
    }

    pub fn all_proposals(&self) -> Vec<&RefactorProposal> {
        let mut proposals: Vec<&RefactorProposal> = self.proposals.values().collect();
        proposals.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        proposals
    }

    pub fn detect_code_smells(&self, target: &str, metrics: &HashMap<String, f64>) -> Vec<CodeSmell> {
        let mut smells = Vec::new();

        for detector in &self.smell_detectors {
            if let Some(value) = metrics.get(&detector.metric_name) {
                if detector.operator.compare(*value, detector.threshold) {
                    let evidence = format!(
                        "{}: {:.2} (threshold: {:.2})",
                        detector.metric_name, value, detector.threshold
                    );
                    smells.push(CodeSmell {
                        smell_type: detector.smell_type.clone(),
                        description: detector.description.clone(),
                        severity: detector.severity.clone(),
                        location: target.to_string(),
                        evidence,
                    });
                }
            }
        }

        smells.sort_by(|a, b| b.severity.cmp(&a.severity));
        smells
    }

    pub fn generate_proposals(&self, target: &str, metrics: &HashMap<String, f64>) -> Vec<RefactorProposal> {
        let smells = self.detect_code_smells(target, metrics);
        let mut proposals = Vec::new();
        let mut counter = 0u64;

        for smell in &smells {
            counter += 1;
            let refactor_type = self.smell_to_refactor_type(&smell.smell_type);
            let title = format!("{} - {}", smell.smell_type, target);
            let benefit = smell.severity.weight() * 10.0;
            let effort = self.estimate_effort(&smell.smell_type);

            let mut proposal = RefactorProposal::new(
                format!("{}-refactor-{}", target.replace('/', "-"), counter),
                refactor_type,
                title,
                smell.description.clone(),
            )
            .with_severity(smell.severity.clone())
            .with_benefit_score(benefit)
            .with_estimated_effort(effort)
            .with_code_smells(vec![smell.clone()]);

            proposal.affected_files = vec![target.to_string()];
            proposal.priority_score = self.calculate_priority(&proposal);
            proposal.updated_at = chrono::Utc::now().to_rfc3339();

            proposals.push(proposal);
        }

        proposals.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        proposals
    }

    fn smell_to_refactor_type(&self, smell: &CodeSmellType) -> RefactorType {
        match smell {
            CodeSmellType::LongFunction => RefactorType::ExtractFunction,
            CodeSmellType::GodClass => RefactorType::ExtractClass,
            CodeSmellType::DuplicateCode => RefactorType::ExtractFunction,
            CodeSmellType::LongParameterList => RefactorType::ChangeSignature,
            CodeSmellType::DeepInheritance => RefactorType::ExtractClass,
            CodeSmellType::ShotgunSurgery => RefactorType::SplitModule,
            CodeSmellType::FeatureEnvy => RefactorType::MoveFile,
            CodeSmellType::DataClass => RefactorType::ExtractInterface,
        }
    }

    fn estimate_effort(&self, smell: &CodeSmellType) -> f64 {
        match smell {
            CodeSmellType::LongFunction => 2.0,
            CodeSmellType::GodClass => 8.0,
            CodeSmellType::DuplicateCode => 4.0,
            CodeSmellType::LongParameterList => 1.0,
            CodeSmellType::DeepInheritance => 6.0,
            CodeSmellType::ShotgunSurgery => 12.0,
            CodeSmellType::FeatureEnvy => 3.0,
            CodeSmellType::DataClass => 2.0,
        }
    }

    pub fn calculate_priority(&self, proposal: &RefactorProposal) -> f64 {
        let severity_weight = proposal.severity.weight();
        let benefit = proposal.benefit_score;
        let effort = proposal.estimated_effort_hours.max(0.1);
        let impact_factor = (proposal.affected_files.len() as f64).sqrt();

        (severity_weight * benefit * impact_factor) / effort
    }

    pub fn assess_impact(&self, proposal_id: &str) -> RefactorResult<RefactorImpact> {
        let proposal = self.get_proposal(proposal_id)?;

        let blast_radius = proposal.affected_files.len() * 3;
        let mut affected_modules = Vec::new();
        let mut affected_tests = Vec::new();

        for file in &proposal.affected_files {
            affected_modules.push(file.clone());
            affected_tests.push(format!("{}_test", file));
        }

        let impact_score = proposal.severity.weight() * (blast_radius as f64) * 2.0;

        let risk_level = if impact_score > 80.0 {
            RefactorSeverity::Critical
        } else if impact_score > 50.0 {
            RefactorSeverity::Major
        } else if impact_score > 20.0 {
            RefactorSeverity::Minor
        } else {
            RefactorSeverity::Info
        };

        let estimated_recovery = (blast_radius as f64) * 15.0;
        let breaking_changes = matches!(
            proposal.refactor_type,
            RefactorType::ChangeSignature | RefactorType::SplitModule
        );
        let dependencies_affected = blast_radius > 5;

        Ok(RefactorImpact {
            blast_radius,
            affected_modules,
            affected_tests,
            risk_level,
            estimated_recovery_minutes: estimated_recovery,
            breaking_changes,
            dependencies_affected,
            impact_score: impact_score.min(100.0),
        })
    }

    pub fn generate_execution_plan(&mut self, proposal_id: &str) -> RefactorResult<RefactorExecutionPlan> {
        let proposal = self.get_proposal(proposal_id)?;

        let mut steps = Vec::new();
        let mut step_counter = 0u64;

        step_counter += 1;
        steps.push(RefactorStep {
            id: format!("step-{}-backup", proposal_id),
            description: "备份原始文件".to_string(),
            order: step_counter as usize,
            file_path: proposal.affected_files.get(0).cloned().unwrap_or_default(),
            expected_changes: "创建文件备份".to_string(),
            rollback_instructions: "从备份恢复文件".to_string(),
        });

        step_counter += 1;
        steps.push(RefactorStep {
            id: format!("step-{}-apply", proposal_id),
            description: format!("应用重构: {}", proposal.title),
            order: step_counter as usize,
            file_path: proposal.affected_files.get(0).cloned().unwrap_or_default(),
            expected_changes: proposal.description.clone(),
            rollback_instructions: "恢复备份文件并撤销所有更改".to_string(),
        });

        step_counter += 1;
        steps.push(RefactorStep {
            id: format!("step-{}-verify", proposal_id),
            description: "验证重构结果".to_string(),
            order: step_counter as usize,
            file_path: proposal.affected_files.get(0).cloned().unwrap_or_default(),
            expected_changes: "运行测试验证功能正常".to_string(),
            rollback_instructions: "如果测试失败，执行回滚".to_string(),
        });

        let total_steps = steps.len();
        let estimated_duration = proposal.estimated_effort_hours * 60.0;

        let plan = RefactorExecutionPlan {
            proposal_id: proposal_id.to_string(),
            steps,
            total_steps,
            estimated_duration_minutes: estimated_duration,
            prerequisites: vec!["代码审查".to_string(), "测试环境准备".to_string()],
            rollback_plan: "按步骤逆序执行回滚，先恢复文件，再验证".to_string(),
        };

        self.execution_plans
            .insert(proposal_id.to_string(), plan.clone());

        Ok(plan)
    }

    pub fn apply_refactor(&mut self, proposal_id: &str) -> RefactorResult<()> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| RefactorError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status != RefactorStatus::Planned && proposal.status != RefactorStatus::Proposed {
            return Err(RefactorError::InvalidStatus(format!(
                "Cannot apply refactor in status: {}",
                proposal.status
            )));
        }

        if !self.execution_plans.contains_key(proposal_id) {
            return Err(RefactorError::PlanNotReady(
                "Execution plan not generated".to_string(),
            ));
        }

        let start = std::time::Instant::now();

        proposal.status = RefactorStatus::InProgress;
        proposal.updated_at = chrono::Utc::now().to_rfc3339();

        let title = proposal.title.clone();
        let refactor_type = proposal.refactor_type.clone();

        proposal.status = RefactorStatus::Completed;
        proposal.updated_at = chrono::Utc::now().to_rfc3339();

        let duration = start.elapsed();
        self.history.push(RefactorHistoryEntry {
            proposal_id: proposal_id.to_string(),
            proposal_title: title,
            refactor_type,
            status: RefactorStatus::Completed,
            executed_at: chrono::Utc::now().to_rfc3339(),
            duration_ms: duration.as_millis() as u64,
            error_message: None,
        });

        Ok(())
    }

    pub fn rollback_refactor(&mut self, proposal_id: &str) -> RefactorResult<()> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| RefactorError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status != RefactorStatus::Completed && proposal.status != RefactorStatus::InProgress {
            return Err(RefactorError::InvalidStatus(format!(
                "Cannot rollback refactor in status: {}",
                proposal.status
            )));
        }

        let title = proposal.title.clone();
        let refactor_type = proposal.refactor_type.clone();

        proposal.status = RefactorStatus::RolledBack;
        proposal.updated_at = chrono::Utc::now().to_rfc3339();

        self.history.push(RefactorHistoryEntry {
            proposal_id: proposal_id.to_string(),
            proposal_title: title,
            refactor_type,
            status: RefactorStatus::RolledBack,
            executed_at: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0,
            error_message: None,
        });

        Ok(())
    }

    pub fn history(&self) -> &[RefactorHistoryEntry] {
        &self.history
    }

    pub fn filter_by_severity(&self, severity: &RefactorSeverity) -> Vec<&RefactorProposal> {
        self.proposals
            .values()
            .filter(|p| &p.severity == severity)
            .collect()
    }

    pub fn filter_by_status(&self, status: &RefactorStatus) -> Vec<&RefactorProposal> {
        self.proposals
            .values()
            .filter(|p| &p.status == status)
            .collect()
    }

    pub fn sorted_by_priority(&self) -> Vec<&RefactorProposal> {
        self.all_proposals()
    }

    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }

    pub fn completed_count(&self) -> usize {
        self.proposals
            .values()
            .filter(|p| p.status == RefactorStatus::Completed)
            .count()
    }
}

impl Default for RefactorAssistant {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_assistant() -> RefactorAssistant {
        RefactorAssistant::new()
    }

    fn create_test_proposal(id: &str) -> RefactorProposal {
        RefactorProposal::new(
            id,
            RefactorType::ExtractFunction,
            "Test Refactor",
            "Test description",
        )
        .with_severity(RefactorSeverity::Major)
        .with_affected_files(vec!["src/main.rs".to_string(), "src/utils.rs".to_string()])
        .with_estimated_effort(4.0)
        .with_benefit_score(25.0)
    }

    fn create_metrics_with_smells() -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("function_lines".to_string(), 60.0);
        metrics.insert("class_method_count".to_string(), 35.0);
        metrics.insert("duplication_ratio".to_string(), 15.0);
        metrics.insert("parameter_count".to_string(), 10.0);
        metrics.insert("inheritance_depth".to_string(), 7.0);
        metrics.insert("change_impact_files".to_string(), 12.0);
        metrics.insert("external_calls_ratio".to_string(), 70.0);
        metrics.insert("data_only_ratio".to_string(), 85.0);
        metrics
    }

    #[test]
    fn test_refactor_type_display() {
        assert_eq!(RefactorType::Rename.to_string(), "Rename");
        assert_eq!(RefactorType::ExtractFunction.to_string(), "ExtractFunction");
        assert_eq!(RefactorType::ExtractClass.to_string(), "ExtractClass");
        assert_eq!(RefactorType::ExtractInterface.to_string(), "ExtractInterface");
        assert_eq!(RefactorType::SplitModule.to_string(), "SplitModule");
        assert_eq!(RefactorType::MergeModule.to_string(), "MergeModule");
        assert_eq!(RefactorType::MoveFile.to_string(), "MoveFile");
        assert_eq!(RefactorType::ChangeSignature.to_string(), "ChangeSignature");
    }

    #[test]
    fn test_severity_weights() {
        assert_eq!(RefactorSeverity::Info.weight(), 1.0);
        assert_eq!(RefactorSeverity::Minor.weight(), 2.0);
        assert_eq!(RefactorSeverity::Major.weight(), 5.0);
        assert_eq!(RefactorSeverity::Critical.weight(), 10.0);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(RefactorSeverity::Critical > RefactorSeverity::Major);
        assert!(RefactorSeverity::Major > RefactorSeverity::Minor);
        assert!(RefactorSeverity::Minor > RefactorSeverity::Info);
    }

    #[test]
    fn test_proposal_creation() {
        let proposal = create_test_proposal("test-001");
        assert_eq!(proposal.id, "test-001");
        assert_eq!(proposal.refactor_type, RefactorType::ExtractFunction);
        assert_eq!(proposal.severity, RefactorSeverity::Major);
        assert_eq!(proposal.status, RefactorStatus::Proposed);
        assert_eq!(proposal.affected_files.len(), 2);
        assert_eq!(proposal.estimated_effort_hours, 4.0);
        assert_eq!(proposal.benefit_score, 25.0);
    }

    #[test]
    fn test_add_and_get_proposal() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("test-001");
        assistant.add_proposal(proposal);

        assert_eq!(assistant.proposal_count(), 1);
        let found = assistant.get_proposal("test-001").unwrap();
        assert_eq!(found.id, "test-001");
    }

    #[test]
    fn test_get_proposal_not_found() {
        let assistant = create_test_assistant();
        let result = assistant.get_proposal("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RefactorError::ProposalNotFound(_)
        ));
    }

    #[test]
    fn test_detect_code_smells() {
        let assistant = create_test_assistant();
        let metrics = create_metrics_with_smells();
        let smells = assistant.detect_code_smells("src/main.rs", &metrics);

        assert!(!smells.is_empty());
        assert!(smells.len() >= 8);

        for i in 1..smells.len() {
            assert!(smells[i - 1].severity >= smells[i].severity);
        }
    }

    #[test]
    fn test_generate_proposals() {
        let assistant = create_test_assistant();
        let metrics = create_metrics_with_smells();
        let proposals = assistant.generate_proposals("user-module", &metrics);

        assert!(!proposals.is_empty());
        assert!(proposals.len() >= 8);

        for i in 1..proposals.len() {
            assert!(proposals[i - 1].priority_score >= proposals[i].priority_score);
        }
    }

    #[test]
    fn test_priority_calculation() {
        let assistant = create_test_assistant();
        let mut p1 = create_test_proposal("p1");
        p1.severity = RefactorSeverity::Critical;
        p1.benefit_score = 50.0;
        p1.estimated_effort_hours = 2.0;

        let mut p2 = create_test_proposal("p2");
        p2.severity = RefactorSeverity::Minor;
        p2.benefit_score = 10.0;
        p2.estimated_effort_hours = 8.0;

        let score1 = assistant.calculate_priority(&p1);
        let score2 = assistant.calculate_priority(&p2);

        assert!(score1 > score2);
    }

    #[test]
    fn test_assess_impact() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("impact-test");
        assistant.add_proposal(proposal);

        let impact = assistant.assess_impact("impact-test").unwrap();
        assert!(impact.blast_radius > 0);
        assert!(!impact.affected_modules.is_empty());
        assert!(!impact.affected_tests.is_empty());
        assert!(impact.estimated_recovery_minutes > 0.0);
        assert!(impact.impact_score >= 0.0);
    }

    #[test]
    fn test_generate_execution_plan() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("plan-test");
        assistant.add_proposal(proposal);

        let plan = assistant.generate_execution_plan("plan-test").unwrap();
        assert_eq!(plan.proposal_id, "plan-test");
        assert!(!plan.steps.is_empty());
        assert!(plan.total_steps > 0);
        assert!(plan.estimated_duration_minutes > 0.0);
    }

    #[test]
    fn test_apply_refactor() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("apply-test");
        assistant.add_proposal(proposal);
        assistant.generate_execution_plan("apply-test").unwrap();

        let result = assistant.apply_refactor("apply-test");
        assert!(result.is_ok());

        let proposal = assistant.get_proposal("apply-test").unwrap();
        assert_eq!(proposal.status, RefactorStatus::Completed);
        assert_eq!(assistant.completed_count(), 1);
        assert!(!assistant.history().is_empty());
    }

    #[test]
    fn test_apply_refactor_without_plan() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("no-plan-test");
        assistant.add_proposal(proposal);

        let result = assistant.apply_refactor("no-plan-test");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RefactorError::PlanNotReady(_)
        ));
    }

    #[test]
    fn test_rollback_refactor() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("rollback-test");
        assistant.add_proposal(proposal);
        assistant.generate_execution_plan("rollback-test").unwrap();
        assistant.apply_refactor("rollback-test").unwrap();

        let result = assistant.rollback_refactor("rollback-test");
        assert!(result.is_ok());

        let proposal = assistant.get_proposal("rollback-test").unwrap();
        assert_eq!(proposal.status, RefactorStatus::RolledBack);
    }

    #[test]
    fn test_rollback_invalid_status() {
        let mut assistant = create_test_assistant();
        let proposal = create_test_proposal("rollback-invalid");
        assistant.add_proposal(proposal);

        let result = assistant.rollback_refactor("rollback-invalid");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RefactorError::InvalidStatus(_)
        ));
    }

    #[test]
    fn test_filter_by_severity() {
        let mut assistant = create_test_assistant();

        let mut p1 = create_test_proposal("p1");
        p1.severity = RefactorSeverity::Critical;
        assistant.add_proposal(p1);

        let mut p2 = create_test_proposal("p2");
        p2.severity = RefactorSeverity::Major;
        assistant.add_proposal(p2);

        let critical = assistant.filter_by_severity(&RefactorSeverity::Critical);
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].id, "p1");
    }

    #[test]
    fn test_filter_by_status() {
        let mut assistant = create_test_assistant();
        assistant.add_proposal(create_test_proposal("p1"));

        let mut p2 = create_test_proposal("p2");
        p2.status = RefactorStatus::Completed;
        assistant.add_proposal(p2);

        let completed = assistant.filter_by_status(&RefactorStatus::Completed);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, "p2");
    }

    #[test]
    fn test_sorted_by_priority() {
        let mut assistant = create_test_assistant();

        let mut p1 = create_test_proposal("p1");
        p1.severity = RefactorSeverity::Critical;
        p1.benefit_score = 100.0;
        p1.estimated_effort_hours = 1.0;
        p1.priority_score = 100.0;
        assistant.add_proposal(p1);

        let mut p2 = create_test_proposal("p2");
        p2.severity = RefactorSeverity::Info;
        p2.benefit_score = 10.0;
        p2.estimated_effort_hours = 10.0;
        p2.priority_score = 1.0;
        assistant.add_proposal(p2);

        let sorted = assistant.sorted_by_priority();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].id, "p1");
        assert_eq!(sorted[1].id, "p2");
    }

    #[test]
    fn test_history_records() {
        let mut assistant = create_test_assistant();

        for i in 0..3 {
            let id = format!("hist-{}", i);
            let proposal = create_test_proposal(&id);
            assistant.add_proposal(proposal);
            assistant.generate_execution_plan(&id).unwrap();
            assistant.apply_refactor(&id).unwrap();
        }

        assert_eq!(assistant.history().len(), 3);
        for entry in assistant.history() {
            assert_eq!(entry.status, RefactorStatus::Completed);
            assert!(!entry.executed_at.is_empty());
        }
    }

    #[test]
    fn test_code_smell_types() {
        assert_eq!(CodeSmellType::LongFunction.to_string(), "LongFunction");
        assert_eq!(CodeSmellType::GodClass.to_string(), "GodClass");
        assert_eq!(CodeSmellType::DuplicateCode.to_string(), "DuplicateCode");
        assert_eq!(CodeSmellType::LongParameterList.to_string(), "LongParameterList");
        assert_eq!(CodeSmellType::DeepInheritance.to_string(), "DeepInheritance");
        assert_eq!(CodeSmellType::ShotgunSurgery.to_string(), "ShotgunSurgery");
        assert_eq!(CodeSmellType::FeatureEnvy.to_string(), "FeatureEnvy");
        assert_eq!(CodeSmellType::DataClass.to_string(), "DataClass");
    }

    #[test]
    fn test_refactor_status_display() {
        assert_eq!(RefactorStatus::Proposed.to_string(), "Proposed");
        assert_eq!(RefactorStatus::Planned.to_string(), "Planned");
        assert_eq!(RefactorStatus::InProgress.to_string(), "InProgress");
        assert_eq!(RefactorStatus::Completed.to_string(), "Completed");
        assert_eq!(RefactorStatus::RolledBack.to_string(), "RolledBack");
    }

    #[test]
    fn test_default_values() {
        assert_eq!(RefactorType::default(), RefactorType::Rename);
        assert_eq!(RefactorSeverity::default(), RefactorSeverity::Info);
        assert_eq!(RefactorStatus::default(), RefactorStatus::Proposed);
    }

    #[test]
    fn test_multiple_proposals_sorting() {
        let assistant = create_test_assistant();
        let metrics = create_metrics_with_smells();
        let proposals = assistant.generate_proposals("multi-sort", &metrics);

        assert!(proposals.len() >= 3);

        for i in 1..proposals.len() {
            assert!(
                proposals[i - 1].priority_score >= proposals[i].priority_score,
                "Proposal {} should have higher or equal priority than {}",
                i - 1,
                i
            );
        }
    }

    #[test]
    fn test_error_display() {
        let err = RefactorError::ProposalNotFound("test-123".to_string());
        assert!(err.to_string().contains("test-123"));

        let err = RefactorError::InvalidStatus("bad status".to_string());
        assert!(err.to_string().contains("bad status"));
    }

    #[test]
    fn test_default_assistant() {
        let assistant = RefactorAssistant::default();
        assert_eq!(assistant.proposal_count(), 0);
        assert!(assistant.history().is_empty());
    }
}
