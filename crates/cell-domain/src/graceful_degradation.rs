use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DegradationLevel {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
}

impl DegradationLevel {
    pub fn label(&self) -> &str {
        match self {
            Self::L0 => "Normal",
            Self::L1 => "Performance Degradation",
            Self::L2 => "Feature Degradation",
            Self::L3 => "Core Limited",
            Self::L4 => "System Failsafe",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::L0 => "系统正常运行，所有功能可用",
            Self::L1 => "非关键功能性能下降，核心功能正常",
            Self::L2 => "部分功能关闭，核心功能保留",
            Self::L3 => "仅核心功能运行，其余降级",
            Self::L4 => "系统进入安全模式，仅保活",
        }
    }

    pub fn from_usize(level: usize) -> Option<Self> {
        match level {
            0 => Some(Self::L0),
            1 => Some(Self::L1),
            2 => Some(Self::L2),
            3 => Some(Self::L3),
            4 => Some(Self::L4),
            _ => None,
        }
    }

    pub fn level_number(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDegradation {
    pub feature_id: String,
    pub feature_name: String,
    pub min_level: DegradationLevel,
    pub current_status: FeatureStatus,
    pub impact_score: f64,
    pub fallback_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureStatus {
    Active,
    Degraded,
    Disabled,
    Failsafe,
}

impl FeatureStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Degraded => "Degraded",
            Self::Disabled => "Disabled",
            Self::Failsafe => "Failsafe",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationRule {
    pub id: String,
    pub name: String,
    pub metric: String,
    pub threshold: f64,
    pub operator: ComparisonOp,
    pub target_level: DegradationLevel,
    pub weight: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    Equal,
}

impl ComparisonOp {
    pub fn compare(&self, value: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan => value > threshold,
            Self::LessThan => value < threshold,
            Self::Equal => (value - threshold).abs() < f64::EPSILON,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::Equal => "==",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationState {
    pub current_level: DegradationLevel,
    pub target_level: DegradationLevel,
    pub active_features: Vec<String>,
    pub degraded_features: Vec<String>,
    pub disabled_features: Vec<String>,
    pub triggered_rules: Vec<String>,
    pub last_updated: String,
}

impl Default for DegradationState {
    fn default() -> Self {
        Self {
            current_level: DegradationLevel::L0,
            target_level: DegradationLevel::L0,
            active_features: Vec::new(),
            degraded_features: Vec::new(),
            disabled_features: Vec::new(),
            triggered_rules: Vec::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

pub struct GracefulDegradation {
    features: HashMap<String, FeatureDegradation>,
    rules: Vec<DegradationRule>,
    state: DegradationState,
}

impl GracefulDegradation {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            rules: Vec::new(),
            state: DegradationState::default(),
        }
    }

    pub fn register_feature(&mut self, feature: FeatureDegradation) {
        self.features.insert(feature.feature_id.clone(), feature);
    }

    pub fn register_rule(&mut self, rule: DegradationRule) {
        self.rules.push(rule);
    }

    pub fn get_state(&self) -> &DegradationState {
        &self.state
    }

    pub fn current_level(&self) -> DegradationLevel {
        self.state.current_level
    }

    pub fn evaluate(&mut self, metrics: &HashMap<String, f64>) -> DegradationLevel {
        let mut max_level = DegradationLevel::L0;
        let mut triggered = Vec::new();

        for rule in &self.rules {
            if let Some(value) = metrics.get(&rule.metric)
                && rule.operator.compare(*value, rule.threshold) {
                    triggered.push(rule.id.clone());
                    if rule.target_level > max_level {
                        max_level = rule.target_level;
                    }
                }
        }

        if max_level != self.state.target_level {
            self.transition_to(max_level);
        }

        self.state.triggered_rules = triggered;
        self.state.last_updated = chrono::Utc::now().to_rfc3339();

        max_level
    }

    fn transition_to(&mut self, target: DegradationLevel) {
        let prev = self.state.current_level;
        self.state.target_level = target;
        self.state.current_level = target;

        let mut active = Vec::new();
        let mut degraded = Vec::new();
        let mut disabled = Vec::new();

        for feature in self.features.values_mut() {
            feature.current_status = if feature.min_level <= target {
                match target {
                    DegradationLevel::L0 => FeatureStatus::Active,
                    DegradationLevel::L1 => {
                        if feature.impact_score < 30.0 {
                            FeatureStatus::Degraded
                        } else {
                            FeatureStatus::Active
                        }
                    }
                    DegradationLevel::L2 => {
                        if feature.impact_score < 50.0 {
                            FeatureStatus::Disabled
                        } else if feature.impact_score < 70.0 {
                            FeatureStatus::Degraded
                        } else {
                            FeatureStatus::Active
                        }
                    }
                    DegradationLevel::L3 => {
                        if feature.impact_score < 80.0 {
                            FeatureStatus::Disabled
                        } else {
                            FeatureStatus::Degraded
                        }
                    }
                    DegradationLevel::L4 => FeatureStatus::Disabled,
                }
            } else {
                FeatureStatus::Active
            };

            match feature.current_status {
                FeatureStatus::Active => active.push(feature.feature_id.clone()),
                FeatureStatus::Degraded => degraded.push(feature.feature_id.clone()),
                FeatureStatus::Disabled | FeatureStatus::Failsafe => {
                    disabled.push(feature.feature_id.clone());
                }
            }
        }

        self.state.active_features = active;
        self.state.degraded_features = degraded;
        self.state.disabled_features = disabled;

        let _ = prev;
    }

    pub fn force_level(&mut self, level: DegradationLevel) {
        self.transition_to(level);
    }

    pub fn reset(&mut self) {
        self.transition_to(DegradationLevel::L0);
        self.state.triggered_rules.clear();
    }

    pub fn active_features(&self) -> Vec<&FeatureDegradation> {
        self.features
            .values()
            .filter(|f| f.current_status == FeatureStatus::Active)
            .collect()
    }

    pub fn degraded_features(&self) -> Vec<&FeatureDegradation> {
        self.features
            .values()
            .filter(|f| f.current_status == FeatureStatus::Degraded)
            .collect()
    }

    pub fn disabled_features(&self) -> Vec<&FeatureDegradation> {
        self.features
            .values()
            .filter(|f| f.current_status == FeatureStatus::Disabled)
            .collect()
    }

    pub fn feature_status(&self, feature_id: &str) -> Option<FeatureStatus> {
        self.features.get(feature_id).map(|f| f.current_status.clone())
    }

    pub fn register_default_rules(&mut self) {
        self.rules.push(DegradationRule {
            id: "high-cpu".to_string(),
            name: "High CPU Usage".to_string(),
            metric: "cpu_usage".to_string(),
            threshold: 85.0,
            operator: ComparisonOp::GreaterThan,
            target_level: DegradationLevel::L1,
            weight: 2.0,
            description: "CPU usage exceeds 85%".to_string(),
        });

        self.rules.push(DegradationRule {
            id: "high-memory".to_string(),
            name: "High Memory Usage".to_string(),
            metric: "memory_usage".to_string(),
            threshold: 90.0,
            operator: ComparisonOp::GreaterThan,
            target_level: DegradationLevel::L1,
            weight: 2.0,
            description: "Memory usage exceeds 90%".to_string(),
        });

        self.rules.push(DegradationRule {
            id: "high-error-rate".to_string(),
            name: "High Error Rate".to_string(),
            metric: "error_rate".to_string(),
            threshold: 5.0,
            operator: ComparisonOp::GreaterThan,
            target_level: DegradationLevel::L2,
            weight: 5.0,
            description: "Error rate exceeds 5%".to_string(),
        });

        self.rules.push(DegradationRule {
            id: "high-latency".to_string(),
            name: "High Latency".to_string(),
            metric: "p99_latency_ms".to_string(),
            threshold: 1000.0,
            operator: ComparisonOp::GreaterThan,
            target_level: DegradationLevel::L2,
            weight: 3.0,
            description: "P99 latency exceeds 1s".to_string(),
        });

        self.rules.push(DegradationRule {
            id: "critical-down".to_string(),
            name: "Critical Dependency Down".to_string(),
            metric: "critical_dependency_health".to_string(),
            threshold: 50.0,
            operator: ComparisonOp::LessThan,
            target_level: DegradationLevel::L3,
            weight: 8.0,
            description: "Critical dependency health below 50%".to_string(),
        });

        self.rules.push(DegradationRule {
            id: "system-overload".to_string(),
            name: "System Overload".to_string(),
            metric: "system_load".to_string(),
            threshold: 95.0,
            operator: ComparisonOp::GreaterThan,
            target_level: DegradationLevel::L4,
            weight: 10.0,
            description: "System load exceeds 95%".to_string(),
        });
    }
}

impl Default for GracefulDegradation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_degradation() -> GracefulDegradation {
        let mut gd = GracefulDegradation::new();
        gd.register_default_rules();

        gd.register_feature(FeatureDegradation {
            feature_id: "core-api".to_string(),
            feature_name: "Core API".to_string(),
            min_level: DegradationLevel::L3,
            current_status: FeatureStatus::Active,
            impact_score: 95.0,
            fallback_strategy: "Return cached data".to_string(),
        });

        gd.register_feature(FeatureDegradation {
            feature_id: "search".to_string(),
            feature_name: "Search Service".to_string(),
            min_level: DegradationLevel::L1,
            current_status: FeatureStatus::Active,
            impact_score: 60.0,
            fallback_strategy: "Simplified search".to_string(),
        });

        gd.register_feature(FeatureDegradation {
            feature_id: "analytics".to_string(),
            feature_name: "Analytics".to_string(),
            min_level: DegradationLevel::L1,
            current_status: FeatureStatus::Active,
            impact_score: 20.0,
            fallback_strategy: "Disabled".to_string(),
        });

        gd
    }

    #[test]
    fn test_initial_state() {
        let gd = create_test_degradation();
        assert_eq!(gd.current_level(), DegradationLevel::L0);
        assert_eq!(gd.active_features().len(), 3);
    }

    #[test]
    fn test_evaluate_no_degradation() {
        let mut gd = create_test_degradation();
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 50.0);
        metrics.insert("memory_usage".to_string(), 60.0);
        metrics.insert("error_rate".to_string(), 1.0);

        let level = gd.evaluate(&metrics);
        assert_eq!(level, DegradationLevel::L0);
    }

    #[test]
    fn test_evaluate_l1_cpu_high() {
        let mut gd = create_test_degradation();
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 90.0);

        let level = gd.evaluate(&metrics);
        assert_eq!(level, DegradationLevel::L1);
    }

    #[test]
    fn test_evaluate_l2_high_errors() {
        let mut gd = create_test_degradation();
        let mut metrics = HashMap::new();
        metrics.insert("error_rate".to_string(), 10.0);

        let level = gd.evaluate(&metrics);
        assert_eq!(level, DegradationLevel::L2);
    }

    #[test]
    fn test_evaluate_l3_critical_down() {
        let mut gd = create_test_degradation();
        let mut metrics = HashMap::new();
        metrics.insert("critical_dependency_health".to_string(), 30.0);

        let level = gd.evaluate(&metrics);
        assert_eq!(level, DegradationLevel::L3);
    }

    #[test]
    fn test_evaluate_l4_system_overload() {
        let mut gd = create_test_degradation();
        let mut metrics = HashMap::new();
        metrics.insert("system_load".to_string(), 98.0);

        let level = gd.evaluate(&metrics);
        assert_eq!(level, DegradationLevel::L4);
    }

    #[test]
    fn test_force_level() {
        let mut gd = create_test_degradation();
        gd.force_level(DegradationLevel::L2);

        assert_eq!(gd.current_level(), DegradationLevel::L2);
    }

    #[test]
    fn test_reset() {
        let mut gd = create_test_degradation();
        gd.force_level(DegradationLevel::L3);
        gd.reset();

        assert_eq!(gd.current_level(), DegradationLevel::L0);
        assert!(gd.get_state().triggered_rules.is_empty());
    }

    #[test]
    fn test_feature_status_query() {
        let gd = create_test_degradation();
        assert_eq!(gd.feature_status("core-api"), Some(FeatureStatus::Active));
        assert_eq!(gd.feature_status("nonexistent"), None);
    }

    #[test]
    fn test_l1_degraded_features() {
        let mut gd = create_test_degradation();
        gd.force_level(DegradationLevel::L1);

        let degraded = gd.degraded_features();
        assert!(!degraded.is_empty());
    }

    #[test]
    fn test_l2_disabled_features() {
        let mut gd = create_test_degradation();
        gd.force_level(DegradationLevel::L2);

        let disabled = gd.disabled_features();
        assert!(!disabled.is_empty());
    }

    #[test]
    fn test_l4_all_disabled() {
        let mut gd = create_test_degradation();
        gd.force_level(DegradationLevel::L4);

        assert_eq!(gd.active_features().len(), 0);
    }

    #[test]
    fn test_degradation_level_ordering() {
        assert!(DegradationLevel::L0 < DegradationLevel::L1);
        assert!(DegradationLevel::L1 < DegradationLevel::L2);
        assert!(DegradationLevel::L2 < DegradationLevel::L3);
        assert!(DegradationLevel::L3 < DegradationLevel::L4);
    }

    #[test]
    fn test_comparison_op() {
        assert!(ComparisonOp::GreaterThan.compare(10.0, 5.0));
        assert!(ComparisonOp::LessThan.compare(5.0, 10.0));
        assert!(ComparisonOp::Equal.compare(5.0, 5.0));
    }

    #[test]
    fn test_level_from_usize() {
        assert_eq!(DegradationLevel::from_usize(0), Some(DegradationLevel::L0));
        assert_eq!(DegradationLevel::from_usize(4), Some(DegradationLevel::L4));
        assert_eq!(DegradationLevel::from_usize(5), None);
    }

    #[test]
    fn test_triggered_rules_tracked() {
        let mut gd = create_test_degradation();
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 90.0);

        gd.evaluate(&metrics);
        assert!(!gd.get_state().triggered_rules.is_empty());
    }

    #[test]
    fn test_core_feature_survives_l3() {
        let mut gd = create_test_degradation();
        gd.force_level(DegradationLevel::L3);

        let core_active = gd.active_features().iter().any(|f| f.feature_id == "core-api");
        let core_degraded = gd.degraded_features().iter().any(|f| f.feature_id == "core-api");
        assert!(core_active || core_degraded);
    }
}
