use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RuleId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IssueId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Major,
    Critical,
}

impl IssueSeverity {
    pub fn label(&self) -> &str {
        match self {
            IssueSeverity::Info => "Info",
            IssueSeverity::Warning => "Warning",
            IssueSeverity::Major => "Major",
            IssueSeverity::Critical => "Critical",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            IssueSeverity::Info => 1.0,
            IssueSeverity::Warning => 2.0,
            IssueSeverity::Major => 5.0,
            IssueSeverity::Critical => 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    Performance,
    Maintainability,
    Reliability,
    Security,
    Architectural,
}

impl IssueCategory {
    pub fn label(&self) -> &str {
        match self {
            IssueCategory::Performance => "Performance",
            IssueCategory::Maintainability => "Maintainability",
            IssueCategory::Reliability => "Reliability",
            IssueCategory::Security => "Security",
            IssueCategory::Architectural => "Architectural",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdviceRule {
    pub id: RuleId,
    pub name: String,
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub condition_metric: String,
    pub condition_threshold: f64,
    pub condition_operator: ComparisonOperator,
    pub description: String,
    pub fix_suggestion: String,
}

impl AdviceRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: IssueCategory,
        severity: IssueSeverity,
    ) -> Self {
        Self {
            id: RuleId(id.into()),
            name: name.into(),
            category,
            severity,
            condition_metric: String::new(),
            condition_threshold: 0.0,
            condition_operator: ComparisonOperator::GreaterThan,
            description: String::new(),
            fix_suggestion: String::new(),
        }
    }

    pub fn with_condition(
        mut self,
        metric: impl Into<String>,
        threshold: f64,
        operator: ComparisonOperator,
    ) -> Self {
        self.condition_metric = metric.into();
        self.condition_threshold = threshold;
        self.condition_operator = operator;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_fix_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.fix_suggestion = suggestion.into();
        self
    }

    pub fn matches(&self, metrics: &HashMap<String, f64>) -> bool {
        if let Some(value) = metrics.get(&self.condition_metric) {
            self.condition_operator
                .compare(*value, self.condition_threshold)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl ComparisonOperator {
    pub fn compare(&self, value: f64, threshold: f64) -> bool {
        match self {
            ComparisonOperator::GreaterThan => value > threshold,
            ComparisonOperator::LessThan => value < threshold,
            ComparisonOperator::Equal => (value - threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (value - threshold).abs() >= f64::EPSILON,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold,
            ComparisonOperator::LessThanOrEqual => value <= threshold,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureIssue {
    pub id: IssueId,
    pub rule_id: RuleId,
    pub rule_name: String,
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub description: String,
    pub fix_suggestion: String,
    pub evidence: String,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub issues: Vec<ArchitectureIssue>,
    pub total_score: f64,
    pub category_stats: HashMap<String, CategoryStat>,
    pub priority_fixes: Vec<PriorityFix>,
    pub analyzed_at: String,
    pub rules_evaluated: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: IssueCategory,
    pub issue_count: usize,
    pub total_penalty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFix {
    pub issue_id: IssueId,
    pub rule_name: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub fix_suggestion: String,
    pub priority_score: f64,
}

pub struct ArchitectureAdvisor {
    rules: Vec<AdviceRule>,
}

impl ArchitectureAdvisor {
    pub fn new() -> Self {
        let mut advisor = Self { rules: Vec::new() };
        advisor.load_builtin_rules();
        advisor
    }

    pub fn register_rule(&mut self, rule: AdviceRule) {
        self.rules.push(rule);
    }

    pub fn rules(&self) -> &[AdviceRule] {
        &self.rules
    }

    fn load_builtin_rules(&mut self) {
        let god_class = AdviceRule::new(
            "god-class",
            "God Class",
            IssueCategory::Architectural,
            IssueSeverity::Major,
        )
        .with_condition("class_method_count", 30.0, ComparisonOperator::GreaterThan)
        .with_description("类拥有过多方法，承担了过多职责，违反单一职责原则")
        .with_fix_suggestion("将大类拆分为多个小类，每个类只负责一个功能领域");

        let long_function = AdviceRule::new(
            "long-function",
            "Long Function",
            IssueCategory::Maintainability,
            IssueSeverity::Warning,
        )
        .with_condition("function_lines", 50.0, ComparisonOperator::GreaterThan)
        .with_description("函数过长，难以理解和维护")
        .with_fix_suggestion("将长函数拆分为多个小函数，每个函数只做一件事");

        let high_coupling = AdviceRule::new(
            "high-coupling",
            "High Coupling",
            IssueCategory::Maintainability,
            IssueSeverity::Major,
        )
        .with_condition("coupling_score", 75.0, ComparisonOperator::GreaterThan)
        .with_description("模块间耦合度过高，修改一个模块可能影响大量其他模块")
        .with_fix_suggestion("引入接口层，减少直接依赖，使用依赖倒置原则");

        let low_cohesion = AdviceRule::new(
            "low-cohesion",
            "Low Cohesion",
            IssueCategory::Maintainability,
            IssueSeverity::Warning,
        )
        .with_condition("cohesion_score", 40.0, ComparisonOperator::LessThan)
        .with_description("模块内聚度低，模块内元素关系不紧密")
        .with_fix_suggestion("重新组织模块内的功能，确保每个模块有清晰单一的职责");

        let cyclic_dependency = AdviceRule::new(
            "cyclic-dependency",
            "Cyclic Dependency",
            IssueCategory::Architectural,
            IssueSeverity::Critical,
        )
        .with_condition("cycle_count", 0.0, ComparisonOperator::GreaterThan)
        .with_description("存在循环依赖，导致模块难以独立测试和部署")
        .with_fix_suggestion("重构依赖关系，使用依赖倒置或提取公共接口打破循环");

        let poor_naming = AdviceRule::new(
            "poor-naming",
            "Poor Naming Quality",
            IssueCategory::Maintainability,
            IssueSeverity::Info,
        )
        .with_condition("naming_quality", 60.0, ComparisonOperator::LessThan)
        .with_description("命名质量差，代码可读性低")
        .with_fix_suggestion("遵循命名规范，使用清晰、描述性的名称");

        let low_test_coverage = AdviceRule::new(
            "low-test-coverage",
            "Low Test Coverage",
            IssueCategory::Reliability,
            IssueSeverity::Warning,
        )
        .with_condition("test_coverage", 70.0, ComparisonOperator::LessThan)
        .with_description("测试覆盖率低于推荐阈值，代码质量难以保证")
        .with_fix_suggestion("增加单元测试和集成测试，提高测试覆盖率");

        let deep_inheritance = AdviceRule::new(
            "deep-inheritance",
            "Deep Inheritance",
            IssueCategory::Maintainability,
            IssueSeverity::Warning,
        )
        .with_condition("inheritance_depth", 5.0, ComparisonOperator::GreaterThan)
        .with_description("继承层次过深，增加了理解和维护的复杂度")
        .with_fix_suggestion("考虑使用组合替代继承，减少继承层次");

        let large_parameter_list = AdviceRule::new(
            "large-parameter-list",
            "Large Parameter List",
            IssueCategory::Maintainability,
            IssueSeverity::Warning,
        )
        .with_condition("parameter_count", 8.0, ComparisonOperator::GreaterThan)
        .with_description("函数参数列表过长，调用困难且容易出错")
        .with_fix_suggestion("将相关参数封装为对象，使用参数对象模式");

        let duplicate_code = AdviceRule::new(
            "duplicate-code",
            "Duplicate Code",
            IssueCategory::Maintainability,
            IssueSeverity::Major,
        )
        .with_condition("duplication_ratio", 10.0, ComparisonOperator::GreaterThan)
        .with_description("存在大量重复代码，维护成本高")
        .with_fix_suggestion("提取公共逻辑到共享函数或类中，消除重复");

        let security_vulnerability = AdviceRule::new(
            "security-vulnerability",
            "Potential Security Issue",
            IssueCategory::Security,
            IssueSeverity::Critical,
        )
        .with_condition("security_issues", 0.0, ComparisonOperator::GreaterThan)
        .with_description("检测到潜在的安全漏洞")
        .with_fix_suggestion("修复所有安全漏洞，遵循安全编码最佳实践");

        let performance_bottleneck = AdviceRule::new(
            "performance-bottleneck",
            "Performance Bottleneck",
            IssueCategory::Performance,
            IssueSeverity::Major,
        )
        .with_condition(
            "cyclomatic_complexity",
            50.0,
            ComparisonOperator::GreaterThan,
        )
        .with_description("代码复杂度过高，可能存在性能瓶颈")
        .with_fix_suggestion("优化算法复杂度，重构复杂逻辑");

        self.register_rule(god_class);
        self.register_rule(long_function);
        self.register_rule(high_coupling);
        self.register_rule(low_cohesion);
        self.register_rule(cyclic_dependency);
        self.register_rule(poor_naming);
        self.register_rule(low_test_coverage);
        self.register_rule(deep_inheritance);
        self.register_rule(large_parameter_list);
        self.register_rule(duplicate_code);
        self.register_rule(security_vulnerability);
        self.register_rule(performance_bottleneck);
    }

    pub fn analyze(&self, target: &str, metrics: &HashMap<String, f64>) -> AnalysisResult {
        let mut issues = Vec::new();
        let mut issue_counter = 0u64;

        for rule in &self.rules {
            if rule.matches(metrics) {
                issue_counter += 1;
                let value = metrics.get(&rule.condition_metric).copied().unwrap_or(0.0);
                let evidence = format!(
                    "{}: {:.2} (threshold: {:.2})",
                    rule.condition_metric, value, rule.condition_threshold
                );

                issues.push(ArchitectureIssue {
                    id: IssueId(format!("{}-issue-{}", target, issue_counter)),
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    category: rule.category.clone(),
                    severity: rule.severity.clone(),
                    description: rule.description.clone(),
                    fix_suggestion: rule.fix_suggestion.clone(),
                    evidence,
                    detected_at: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        issues.sort_by(|a, b| {
            b.severity
                .weight()
                .partial_cmp(&a.severity.weight())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let category_stats = self.build_category_stats(&issues);
        let priority_fixes = self.build_priority_fixes(&issues);
        let total_score = self.calculate_total_score(&issues);

        AnalysisResult {
            issues,
            total_score,
            category_stats,
            priority_fixes,
            analyzed_at: chrono::Utc::now().to_rfc3339(),
            rules_evaluated: self.rules.len(),
        }
    }

    fn build_category_stats(&self, issues: &[ArchitectureIssue]) -> HashMap<String, CategoryStat> {
        let mut stats: HashMap<String, CategoryStat> = HashMap::new();

        for issue in issues {
            let entry = stats
                .entry(issue.category.label().to_string())
                .or_insert_with(|| CategoryStat {
                    category: issue.category.clone(),
                    issue_count: 0,
                    total_penalty: 0.0,
                });
            entry.issue_count += 1;
            entry.total_penalty += issue.severity.weight();
        }

        stats
    }

    fn build_priority_fixes(&self, issues: &[ArchitectureIssue]) -> Vec<PriorityFix> {
        let mut fixes: Vec<PriorityFix> = issues
            .iter()
            .map(|issue| {
                let priority_score =
                    issue.severity.weight() * self.category_priority(&issue.category);
                PriorityFix {
                    issue_id: issue.id.clone(),
                    rule_name: issue.rule_name.clone(),
                    severity: issue.severity.clone(),
                    category: issue.category.clone(),
                    description: issue.description.clone(),
                    fix_suggestion: issue.fix_suggestion.clone(),
                    priority_score,
                }
            })
            .collect();

        fixes.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        fixes
    }

    fn category_priority(&self, category: &IssueCategory) -> f64 {
        match category {
            IssueCategory::Security => 2.0,
            IssueCategory::Architectural => 1.8,
            IssueCategory::Reliability => 1.5,
            IssueCategory::Performance => 1.3,
            IssueCategory::Maintainability => 1.0,
        }
    }

    fn calculate_total_score(&self, issues: &[ArchitectureIssue]) -> f64 {
        let total_penalty: f64 = issues.iter().map(|i| i.severity.weight()).sum();
        (100.0 - total_penalty).max(0.0)
    }

    pub fn filter_by_severity<'a>(
        &self,
        result: &'a AnalysisResult,
        severity: &IssueSeverity,
    ) -> Vec<&'a ArchitectureIssue> {
        result
            .issues
            .iter()
            .filter(|i| &i.severity == severity)
            .collect()
    }

    pub fn filter_by_category<'a>(
        &self,
        result: &'a AnalysisResult,
        category: &IssueCategory,
    ) -> Vec<&'a ArchitectureIssue> {
        result
            .issues
            .iter()
            .filter(|i| &i.category == category)
            .collect()
    }

    pub fn critical_issues<'a>(&self, result: &'a AnalysisResult) -> Vec<&'a ArchitectureIssue> {
        self.filter_by_severity(result, &IssueSeverity::Critical)
    }

    pub fn top_priority_fixes<'a>(
        &self,
        result: &'a AnalysisResult,
        n: usize,
    ) -> Vec<&'a PriorityFix> {
        result.priority_fixes.iter().take(n).collect()
    }
}

impl Default for ArchitectureAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_advisor() -> ArchitectureAdvisor {
        ArchitectureAdvisor::new()
    }

    fn create_metrics_with_issues() -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("class_method_count".to_string(), 40.0);
        metrics.insert("function_lines".to_string(), 60.0);
        metrics.insert("cycle_count".to_string(), 2.0);
        metrics.insert("test_coverage".to_string(), 50.0);
        metrics.insert("coupling_score".to_string(), 80.0);
        metrics.insert("duplication_ratio".to_string(), 15.0);
        metrics
    }

    #[test]
    fn test_advisor_creation() {
        let advisor = create_test_advisor();
        assert!(!advisor.rules().is_empty());
    }

    #[test]
    fn test_builtin_rules_count() {
        let advisor = create_test_advisor();
        assert!(advisor.rules().len() >= 10);
    }

    #[test]
    fn test_single_rule_match() {
        let advisor = create_test_advisor();
        let mut metrics = HashMap::new();
        metrics.insert("class_method_count".to_string(), 35.0);

        let result = advisor.analyze("test", &metrics);
        let god_class_issue = result.issues.iter().find(|i| i.rule_name == "God Class");

        assert!(god_class_issue.is_some());
        assert_eq!(god_class_issue.unwrap().severity, IssueSeverity::Major);
    }

    #[test]
    fn test_multiple_rules_analysis() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();

        let result = advisor.analyze("test", &metrics);
        assert!(result.issues.len() >= 5);
        assert_eq!(result.rules_evaluated, advisor.rules().len());
    }

    #[test]
    fn test_filter_by_severity() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();
        let result = advisor.analyze("test", &metrics);

        let critical_issues = advisor.filter_by_severity(&result, &IssueSeverity::Critical);
        assert!(!critical_issues.is_empty());

        let major_issues = advisor.filter_by_severity(&result, &IssueSeverity::Major);
        assert!(!major_issues.is_empty());
    }

    #[test]
    fn test_filter_by_category() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();
        let result = advisor.analyze("test", &metrics);

        let arch_issues = advisor.filter_by_category(&result, &IssueCategory::Architectural);
        assert!(!arch_issues.is_empty());

        let maintain_issues = advisor.filter_by_category(&result, &IssueCategory::Maintainability);
        assert!(!maintain_issues.is_empty());
    }

    #[test]
    fn test_score_calculation() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();
        let result = advisor.analyze("test", &metrics);

        assert!(result.total_score >= 0.0);
        assert!(result.total_score <= 100.0);
    }

    #[test]
    fn test_priority_fixes() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();
        let result = advisor.analyze("test", &metrics);

        assert!(!result.priority_fixes.is_empty());

        for i in 1..result.priority_fixes.len() {
            assert!(
                result.priority_fixes[i - 1].priority_score
                    >= result.priority_fixes[i].priority_score
            );
        }
    }

    #[test]
    fn test_top_priority_fixes() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();
        let result = advisor.analyze("test", &metrics);

        let top3 = advisor.top_priority_fixes(&result, 3);
        assert_eq!(top3.len(), 3);
    }

    #[test]
    fn test_critical_issues() {
        let advisor = create_test_advisor();
        let mut metrics = HashMap::new();
        metrics.insert("cycle_count".to_string(), 1.0);
        metrics.insert("security_issues".to_string(), 2.0);

        let result = advisor.analyze("test", &metrics);
        let criticals = advisor.critical_issues(&result);

        assert!(criticals.len() >= 2);
    }

    #[test]
    fn test_category_stats() {
        let advisor = create_test_advisor();
        let metrics = create_metrics_with_issues();
        let result = advisor.analyze("test", &metrics);

        assert!(!result.category_stats.is_empty());

        for stat in result.category_stats.values() {
            assert!(stat.issue_count > 0);
            assert!(stat.total_penalty > 0.0);
        }
    }

    #[test]
    fn test_custom_rule() {
        let mut advisor = create_test_advisor();
        let initial_count = advisor.rules().len();

        let custom_rule = AdviceRule::new(
            "custom-rule",
            "Custom Rule",
            IssueCategory::Performance,
            IssueSeverity::Warning,
        )
        .with_condition("custom_metric", 100.0, ComparisonOperator::GreaterThan)
        .with_description("自定义规则描述")
        .with_fix_suggestion("自定义修复建议");

        advisor.register_rule(custom_rule);
        assert_eq!(advisor.rules().len(), initial_count + 1);

        let mut metrics = HashMap::new();
        metrics.insert("custom_metric".to_string(), 150.0);

        let result = advisor.analyze("test", &metrics);
        let custom_issue = result.issues.iter().find(|i| i.rule_name == "Custom Rule");

        assert!(custom_issue.is_some());
    }

    #[test]
    fn test_no_issues_scenario() {
        let advisor = create_test_advisor();
        let metrics = HashMap::new();

        let result = advisor.analyze("test", &metrics);
        assert!(result.issues.is_empty());
        assert_eq!(result.total_score, 100.0);
        assert!(result.category_stats.is_empty());
        assert!(result.priority_fixes.is_empty());
    }

    #[test]
    fn test_severity_sorting() {
        let advisor = create_test_advisor();
        let mut metrics = HashMap::new();
        metrics.insert("class_method_count".to_string(), 40.0);
        metrics.insert("cycle_count".to_string(), 1.0);
        metrics.insert("function_lines".to_string(), 60.0);

        let result = advisor.analyze("test", &metrics);

        for i in 1..result.issues.len() {
            assert!(result.issues[i - 1].severity.weight() >= result.issues[i].severity.weight());
        }
    }

    #[test]
    fn test_severity_weights() {
        assert_eq!(IssueSeverity::Info.weight(), 1.0);
        assert_eq!(IssueSeverity::Warning.weight(), 2.0);
        assert_eq!(IssueSeverity::Major.weight(), 5.0);
        assert_eq!(IssueSeverity::Critical.weight(), 10.0);
    }

    #[test]
    fn test_comparison_operators() {
        assert!(ComparisonOperator::GreaterThan.compare(10.0, 5.0));
        assert!(!ComparisonOperator::GreaterThan.compare(5.0, 10.0));
        assert!(ComparisonOperator::LessThan.compare(5.0, 10.0));
        assert!(ComparisonOperator::Equal.compare(5.0, 5.0));
        assert!(ComparisonOperator::GreaterThanOrEqual.compare(5.0, 5.0));
        assert!(ComparisonOperator::LessThanOrEqual.compare(5.0, 5.0));
        assert!(ComparisonOperator::NotEqual.compare(5.0, 10.0));
    }

    #[test]
    fn test_category_labels() {
        assert_eq!(IssueCategory::Performance.label(), "Performance");
        assert_eq!(IssueCategory::Maintainability.label(), "Maintainability");
        assert_eq!(IssueCategory::Reliability.label(), "Reliability");
        assert_eq!(IssueCategory::Security.label(), "Security");
        assert_eq!(IssueCategory::Architectural.label(), "Architectural");
    }

    #[test]
    fn test_severity_labels() {
        assert_eq!(IssueSeverity::Info.label(), "Info");
        assert_eq!(IssueSeverity::Warning.label(), "Warning");
        assert_eq!(IssueSeverity::Major.label(), "Major");
        assert_eq!(IssueSeverity::Critical.label(), "Critical");
    }

    #[test]
    fn test_issue_has_evidence() {
        let advisor = create_test_advisor();
        let mut metrics = HashMap::new();
        metrics.insert("class_method_count".to_string(), 35.0);

        let result = advisor.analyze("test", &metrics);
        let issue = &result.issues[0];

        assert!(!issue.evidence.is_empty());
        assert!(issue.evidence.contains("class_method_count"));
    }

    #[test]
    fn test_rule_no_match_without_metric() {
        let advisor = create_test_advisor();
        let metrics = HashMap::new();

        let result = advisor.analyze("test", &metrics);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::Major);
        assert!(IssueSeverity::Major > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }
}
