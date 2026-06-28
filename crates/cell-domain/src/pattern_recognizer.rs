use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PatternId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternCategory {
    Architectural,
    Performance,
    Reliability,
    Security,
    Maintainability,
    Observability,
    AntiPattern,
}

impl PatternCategory {
    pub fn label(&self) -> &str {
        match self {
            Self::Architectural => "Architectural",
            Self::Performance => "Performance",
            Self::Reliability => "Reliability",
            Self::Security => "Security",
            Self::Maintainability => "Maintainability",
            Self::Observability => "Observability",
            Self::AntiPattern => "AntiPattern",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatternSeverity {
    Info,
    Warning,
    Major,
    Critical,
}

impl PatternSeverity {
    pub fn label(&self) -> &str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Major => "Major",
            Self::Critical => "Critical",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Self::Info => 1.0,
            Self::Warning => 2.0,
            Self::Major => 5.0,
            Self::Critical => 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    pub id: PatternId,
    pub name: String,
    pub category: PatternCategory,
    pub severity: PatternSeverity,
    pub description: String,
    pub solution: String,
    pub indicators: Vec<String>,
    pub rules: Vec<PatternRule>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl PatternDefinition {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: PatternCategory,
        severity: PatternSeverity,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: PatternId(id.into()),
            name: name.into(),
            category,
            severity,
            description: String::new(),
            solution: String::new(),
            indicators: Vec::new(),
            rules: Vec::new(),
            tags: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_solution(mut self, solution: impl Into<String>) -> Self {
        self.solution = solution.into();
        self
    }

    pub fn add_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.indicators.push(indicator.into());
        self
    }

    pub fn add_rule(mut self, rule: PatternRule) -> Self {
        self.rules.push(rule);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRule {
    pub id: String,
    pub name: String,
    pub rule_type: RuleType,
    pub condition: RuleCondition,
    pub threshold: f64,
    pub operator: ComparisonOperator,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleType {
    Metric,
    Structural,
    Naming,
    Dependency,
    Behavioral,
}

impl RuleType {
    pub fn label(&self) -> &str {
        match self {
            Self::Metric => "Metric",
            Self::Structural => "Structural",
            Self::Naming => "Naming",
            Self::Dependency => "Dependency",
            Self::Behavioral => "Behavioral",
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
            Self::GreaterThan => value > threshold,
            Self::LessThan => value < threshold,
            Self::Equal => (value - threshold).abs() < f64::EPSILON,
            Self::NotEqual => (value - threshold).abs() >= f64::EPSILON,
            Self::GreaterThanOrEqual => value >= threshold,
            Self::LessThanOrEqual => value <= threshold,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub metric_key: String,
    pub label: String,
}

impl RuleCondition {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            metric_key: key.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: PatternId,
    pub pattern_name: String,
    pub category: PatternCategory,
    pub severity: PatternSeverity,
    pub confidence: f64,
    pub matched_rules: Vec<String>,
    pub evidence: Vec<String>,
    pub description: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysisResult {
    pub target: String,
    pub total_patterns: usize,
    pub matched_patterns: usize,
    pub matches: Vec<PatternMatch>,
    pub overall_score: f64,
    pub category_scores: HashMap<String, f64>,
}

pub struct PatternRecognizer {
    patterns: HashMap<PatternId, PatternDefinition>,
}

impl PatternRecognizer {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    pub fn register_pattern(&mut self, pattern: PatternDefinition) {
        self.patterns.insert(pattern.id.clone(), pattern);
    }

    pub fn get_pattern(&self, id: &PatternId) -> Option<&PatternDefinition> {
        self.patterns.get(id)
    }

    pub fn list_patterns(&self) -> Vec<&PatternDefinition> {
        self.patterns.values().collect()
    }

    pub fn patterns_by_category(&self, category: &PatternCategory) -> Vec<&PatternDefinition> {
        self.patterns
            .values()
            .filter(|p| &p.category == category)
            .collect()
    }

    pub fn patterns_by_severity(&self, severity: &PatternSeverity) -> Vec<&PatternDefinition> {
        self.patterns
            .values()
            .filter(|p| &p.severity == severity)
            .collect()
    }

    pub fn analyze(
        &self,
        target: &str,
        metrics: &HashMap<String, f64>,
    ) -> PatternAnalysisResult {
        let mut matches = Vec::new();
        let mut category_scores: HashMap<String, f64> = HashMap::new();

        for pattern in self.patterns.values() {
            let result = self.match_pattern(pattern, metrics);
            if result.confidence > 0.0 {
                *category_scores
                    .entry(pattern.category.label().to_string())
                    .or_insert(0.0) = result.severity.weight().mul_add(result.confidence, *category_scores
                    .entry(pattern.category.label().to_string())
                    .or_insert(0.0));

                matches.push(result);
            }
        }

        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.severity.weight().partial_cmp(&a.severity.weight()).unwrap_or(std::cmp::Ordering::Equal))
        });

        let overall_score = if matches.is_empty() {
            100.0
        } else {
            let total_penalty: f64 = matches
                .iter()
                .map(|m| m.severity.weight() * m.confidence)
                .sum();
            (100.0 - total_penalty).max(0.0)
        };

        PatternAnalysisResult {
            target: target.to_string(),
            total_patterns: self.patterns.len(),
            matched_patterns: matches.len(),
            matches,
            overall_score,
            category_scores,
        }
    }

    fn match_pattern(
        &self,
        pattern: &PatternDefinition,
        metrics: &HashMap<String, f64>,
    ) -> PatternMatch {
        let mut matched_rules = Vec::new();
        let mut evidence = Vec::new();
        let mut total_weight = 0.0;
        let mut matched_weight = 0.0;

        for rule in &pattern.rules {
            total_weight += rule.weight;

            if let Some(value) = metrics.get(&rule.condition.metric_key)
                && rule.operator.compare(*value, rule.threshold) {
                    matched_rules.push(rule.id.clone());
                    matched_weight += rule.weight;
                    evidence.push(format!(
                        "{}: {:.2} {} {:.2}",
                        rule.condition.label, value,
                        self.operator_label(&rule.operator),
                        rule.threshold
                    ));
                }
        }

        let confidence = if total_weight > 0.0 {
            matched_weight / total_weight
        } else {
            0.0
        };

        PatternMatch {
            pattern_id: pattern.id.clone(),
            pattern_name: pattern.name.clone(),
            category: pattern.category.clone(),
            severity: pattern.severity.clone(),
            confidence,
            matched_rules,
            evidence,
            description: pattern.description.clone(),
            solution: pattern.solution.clone(),
        }
    }

    fn operator_label(&self, op: &ComparisonOperator) -> &str {
        match op {
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::Equal => "==",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThanOrEqual => "<=",
        }
    }

    pub fn top_matches<'a>(&self, result: &'a PatternAnalysisResult, n: usize) -> Vec<&'a PatternMatch> {
        result.matches.iter().take(n).collect()
    }

    pub fn critical_matches<'a>(&self, result: &'a PatternAnalysisResult) -> Vec<&'a PatternMatch> {
        result
            .matches
            .iter()
            .filter(|m| m.severity == PatternSeverity::Critical)
            .collect()
    }

    pub fn register_builtin_patterns(&mut self) {
        let god_component = PatternDefinition::new(
            "god-component",
            "God Component",
            PatternCategory::Architectural,
            PatternSeverity::Major,
        )
        .with_description("A single component handles too many responsibilities")
        .with_solution("Split the component into smaller, focused components")
        .add_indicator("High cyclomatic complexity")
        .add_indicator("Many dependencies")
        .add_rule(PatternRule {
            id: "r1".to_string(),
            name: "High cyclomatic complexity".to_string(),
            rule_type: RuleType::Metric,
            condition: RuleCondition::new("cyclomatic_complexity", "Cyclomatic complexity"),
            threshold: 50.0,
            operator: ComparisonOperator::GreaterThan,
            weight: 3.0,
        })
        .add_rule(PatternRule {
            id: "r2".to_string(),
            name: "Many outgoing dependencies".to_string(),
            rule_type: RuleType::Dependency,
            condition: RuleCondition::new("outgoing_dependencies", "Outgoing dependencies"),
            threshold: 20.0,
            operator: ComparisonOperator::GreaterThan,
            weight: 2.0,
        });

        let cyclic_dependency = PatternDefinition::new(
            "cyclic-dependency",
            "Cyclic Dependency",
            PatternCategory::AntiPattern,
            PatternSeverity::Critical,
        )
        .with_description("Components form a circular dependency chain")
        .with_solution("Refactor to break the cycle using dependency inversion")
        .add_indicator("Circular imports")
        .add_rule(PatternRule {
            id: "r1".to_string(),
            name: "Cycle detected".to_string(),
            rule_type: RuleType::Structural,
            condition: RuleCondition::new("cycle_count", "Cycle count"),
            threshold: 0.0,
            operator: ComparisonOperator::GreaterThan,
            weight: 10.0,
        });

        let low_test_coverage = PatternDefinition::new(
            "low-test-coverage",
            "Low Test Coverage",
            PatternCategory::Reliability,
            PatternSeverity::Warning,
        )
        .with_description("Test coverage is below recommended threshold")
        .with_solution("Add unit tests and integration tests to improve coverage")
        .add_indicator("Coverage < 70%")
        .add_rule(PatternRule {
            id: "r1".to_string(),
            name: "Low coverage".to_string(),
            rule_type: RuleType::Metric,
            condition: RuleCondition::new("test_coverage", "Test coverage"),
            threshold: 70.0,
            operator: ComparisonOperator::LessThan,
            weight: 5.0,
        });

        let high_coupling = PatternDefinition::new(
            "high-coupling",
            "High Coupling",
            PatternCategory::Maintainability,
            PatternSeverity::Major,
        )
        .with_description("Components are tightly coupled to each other")
        .with_solution("Introduce interfaces and reduce direct dependencies")
        .add_indicator("High coupling score")
        .add_rule(PatternRule {
            id: "r1".to_string(),
            name: "Coupling too high".to_string(),
            rule_type: RuleType::Metric,
            condition: RuleCondition::new("coupling_score", "Coupling score"),
            threshold: 75.0,
            operator: ComparisonOperator::GreaterThan,
            weight: 5.0,
        });

        let poor_naming = PatternDefinition::new(
            "poor-naming",
            "Poor Naming Quality",
            PatternCategory::Maintainability,
            PatternSeverity::Warning,
        )
        .with_description("Names are inconsistent or unclear")
        .with_solution("Follow naming conventions and use descriptive names")
        .add_indicator("Low naming quality score")
        .add_rule(PatternRule {
            id: "r1".to_string(),
            name: "Naming quality low".to_string(),
            rule_type: RuleType::Naming,
            condition: RuleCondition::new("naming_quality", "Naming quality"),
            threshold: 60.0,
            operator: ComparisonOperator::LessThan,
            weight: 4.0,
        });

        let missing_observability = PatternDefinition::new(
            "missing-observability",
            "Missing Observability",
            PatternCategory::Observability,
            PatternSeverity::Major,
        )
        .with_description("Component lacks proper monitoring and logging")
        .with_solution("Add metrics, logs, and traces for observability")
        .add_indicator("No metrics exported")
        .add_rule(PatternRule {
            id: "r1".to_string(),
            name: "Missing metrics".to_string(),
            rule_type: RuleType::Metric,
            condition: RuleCondition::new("metrics_count", "Metrics count"),
            threshold: 1.0,
            operator: ComparisonOperator::LessThan,
            weight: 3.0,
        });

        self.register_pattern(god_component);
        self.register_pattern(cyclic_dependency);
        self.register_pattern(low_test_coverage);
        self.register_pattern(high_coupling);
        self.register_pattern(poor_naming);
        self.register_pattern(missing_observability);
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recognizer() -> PatternRecognizer {
        let mut recognizer = PatternRecognizer::new();
        recognizer.register_builtin_patterns();
        recognizer
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = PatternDefinition::new(
            "test-pattern",
            "Test Pattern",
            PatternCategory::Performance,
            PatternSeverity::Warning,
        );

        assert_eq!(pattern.id.0, "test-pattern");
        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.category, PatternCategory::Performance);
        assert_eq!(pattern.severity, PatternSeverity::Warning);
    }

    #[test]
    fn test_register_pattern() {
        let mut recognizer = PatternRecognizer::new();
        let pattern = PatternDefinition::new("p1", "Pattern 1", PatternCategory::Architectural, PatternSeverity::Info);
        recognizer.register_pattern(pattern);

        assert_eq!(recognizer.list_patterns().len(), 1);
    }

    #[test]
    fn test_builtin_patterns() {
        let recognizer = create_test_recognizer();
        assert_eq!(recognizer.list_patterns().len(), 6);
    }

    #[test]
    fn test_analyze_no_matches() {
        let recognizer = create_test_recognizer();
        let metrics = HashMap::new();
        let result = recognizer.analyze("test-cell", &metrics);

        assert_eq!(result.matched_patterns, 0);
        assert_eq!(result.overall_score, 100.0);
    }

    #[test]
    fn test_analyze_god_component() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("cyclomatic_complexity".to_string(), 80.0);
        metrics.insert("outgoing_dependencies".to_string(), 25.0);

        let result = recognizer.analyze("test-cell", &metrics);

        assert!(result.matched_patterns > 0);
        let god_match = result.matches.iter().find(|m| m.pattern_name == "God Component");
        assert!(god_match.is_some());
        assert!(god_match.unwrap().confidence > 0.5);
    }

    #[test]
    fn test_analyze_cyclic_dependency() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("cycle_count".to_string(), 3.0);

        let result = recognizer.analyze("test-cell", &metrics);

        let cycle_match = result.matches.iter().find(|m| m.pattern_name == "Cyclic Dependency");
        assert!(cycle_match.is_some());
        assert_eq!(cycle_match.unwrap().severity, PatternSeverity::Critical);
    }

    #[test]
    fn test_analyze_low_coverage() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("test_coverage".to_string(), 50.0);

        let result = recognizer.analyze("test-cell", &metrics);

        let coverage_match = result.matches.iter().find(|m| m.pattern_name == "Low Test Coverage");
        assert!(coverage_match.is_some());
    }

    #[test]
    fn test_confidence_calculation() {
        let mut recognizer = PatternRecognizer::new();

        let pattern = PatternDefinition::new("test", "Test", PatternCategory::Performance, PatternSeverity::Warning)
            .add_rule(PatternRule {
                id: "r1".to_string(),
                name: "Rule 1".to_string(),
                rule_type: RuleType::Metric,
                condition: RuleCondition::new("metric_a", "Metric A"),
                threshold: 10.0,
                operator: ComparisonOperator::GreaterThan,
                weight: 1.0,
            })
            .add_rule(PatternRule {
                id: "r2".to_string(),
                name: "Rule 2".to_string(),
                rule_type: RuleType::Metric,
                condition: RuleCondition::new("metric_b", "Metric B"),
                threshold: 10.0,
                operator: ComparisonOperator::GreaterThan,
                weight: 1.0,
            });

        recognizer.register_pattern(pattern);

        let mut metrics = HashMap::new();
        metrics.insert("metric_a".to_string(), 20.0);
        metrics.insert("metric_b".to_string(), 5.0);

        let result = recognizer.analyze("test", &metrics);
        assert_eq!(result.matches[0].confidence, 0.5);
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
    fn test_patterns_by_category() {
        let recognizer = create_test_recognizer();
        let anti_patterns = recognizer.patterns_by_category(&PatternCategory::AntiPattern);
        assert_eq!(anti_patterns.len(), 1);
    }

    #[test]
    fn test_patterns_by_severity() {
        let recognizer = create_test_recognizer();
        let critical = recognizer.patterns_by_severity(&PatternSeverity::Critical);
        assert_eq!(critical.len(), 1);
    }

    #[test]
    fn test_critical_matches() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("cycle_count".to_string(), 1.0);

        let result = recognizer.analyze("test", &metrics);
        let criticals = recognizer.critical_matches(&result);
        assert_eq!(criticals.len(), 1);
    }

    #[test]
    fn test_top_matches() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("cyclomatic_complexity".to_string(), 80.0);
        metrics.insert("test_coverage".to_string(), 50.0);
        metrics.insert("cycle_count".to_string(), 2.0);

        let result = recognizer.analyze("test", &metrics);
        let top = recognizer.top_matches(&result, 2);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_severity_weights() {
        assert_eq!(PatternSeverity::Info.weight(), 1.0);
        assert_eq!(PatternSeverity::Warning.weight(), 2.0);
        assert_eq!(PatternSeverity::Major.weight(), 5.0);
        assert_eq!(PatternSeverity::Critical.weight(), 10.0);
    }

    #[test]
    fn test_matches_sorted_by_confidence() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("cyclomatic_complexity".to_string(), 100.0);
        metrics.insert("test_coverage".to_string(), 40.0);

        let result = recognizer.analyze("test", &metrics);
        for i in 1..result.matches.len() {
            assert!(result.matches[i - 1].confidence >= result.matches[i].confidence);
        }
    }

    #[test]
    fn test_category_scores() {
        let recognizer = create_test_recognizer();
        let mut metrics = HashMap::new();
        metrics.insert("test_coverage".to_string(), 50.0);

        let result = recognizer.analyze("test", &metrics);
        assert!(!result.category_scores.is_empty());
        assert!(result.category_scores.contains_key("Reliability"));
    }
}
