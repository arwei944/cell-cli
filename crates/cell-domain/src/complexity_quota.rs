use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QuotaLevel {
    System,
    Team,
    Cell,
    Feature,
}

impl QuotaLevel {
    pub fn label(&self) -> &str {
        match self {
            Self::System => "System",
            Self::Team => "Team",
            Self::Cell => "Cell",
            Self::Feature => "Feature",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            Self::System => 0,
            Self::Team => 1,
            Self::Cell => 2,
            Self::Feature => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityQuota {
    pub id: String,
    pub name: String,
    pub level: QuotaLevel,
    pub parent_id: Option<String>,
    pub cyclomatic_complexity_limit: f64,
    pub structural_complexity_limit: f64,
    pub coupling_complexity_limit: f64,
    pub naming_quality_min: f64,
    pub test_coverage_min: f64,
    pub current_cyclomatic: f64,
    pub current_structural: f64,
    pub current_coupling: f64,
    pub current_naming: f64,
    pub current_test_coverage: f64,
    pub children: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ComplexityQuota {
    pub fn new(id: impl Into<String>, name: impl Into<String>, level: QuotaLevel) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            name: name.into(),
            level,
            parent_id: None,
            cyclomatic_complexity_limit: 100.0,
            structural_complexity_limit: 100.0,
            coupling_complexity_limit: 100.0,
            naming_quality_min: 60.0,
            test_coverage_min: 60.0,
            current_cyclomatic: 0.0,
            current_structural: 0.0,
            current_coupling: 0.0,
            current_naming: 100.0,
            current_test_coverage: 100.0,
            children: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_limits(
        mut self,
        cyclo: f64,
        structural: f64,
        coupling: f64,
        naming: f64,
        test_cov: f64,
    ) -> Self {
        self.cyclomatic_complexity_limit = cyclo;
        self.structural_complexity_limit = structural;
        self.coupling_complexity_limit = coupling;
        self.naming_quality_min = naming;
        self.test_coverage_min = test_cov;
        self
    }

    pub fn cyclomatic_usage(&self) -> f64 {
        if self.cyclomatic_complexity_limit == 0.0 {
            return 0.0;
        }
        (self.current_cyclomatic / self.cyclomatic_complexity_limit) * 100.0
    }

    pub fn structural_usage(&self) -> f64 {
        if self.structural_complexity_limit == 0.0 {
            return 0.0;
        }
        (self.current_structural / self.structural_complexity_limit) * 100.0
    }

    pub fn coupling_usage(&self) -> f64 {
        if self.coupling_complexity_limit == 0.0 {
            return 0.0;
        }
        (self.current_coupling / self.coupling_complexity_limit) * 100.0
    }

    pub fn naming_gap(&self) -> f64 {
        self.current_naming - self.naming_quality_min
    }

    pub fn test_coverage_gap(&self) -> f64 {
        self.current_test_coverage - self.test_coverage_min
    }

    pub fn overall_usage(&self) -> f64 {
        let cyclo = self.cyclomatic_usage();
        let structural = self.structural_usage();
        let coupling = self.coupling_usage();
        (cyclo + structural + coupling) / 3.0
    }

    pub fn is_compliant(&self) -> bool {
        self.current_cyclomatic <= self.cyclomatic_complexity_limit
            && self.current_structural <= self.structural_complexity_limit
            && self.current_coupling <= self.coupling_complexity_limit
            && self.current_naming >= self.naming_quality_min
            && self.current_test_coverage >= self.test_coverage_min
    }

    pub fn violations(&self) -> Vec<QuotaViolation> {
        let mut violations = Vec::new();

        if self.current_cyclomatic > self.cyclomatic_complexity_limit {
            violations.push(QuotaViolation {
                metric: "cyclomatic_complexity".to_string(),
                current: self.current_cyclomatic,
                limit: self.cyclomatic_complexity_limit,
                severity: self.severity_for_overage(
                    self.current_cyclomatic,
                    self.cyclomatic_complexity_limit,
                ),
            });
        }

        if self.current_structural > self.structural_complexity_limit {
            violations.push(QuotaViolation {
                metric: "structural_complexity".to_string(),
                current: self.current_structural,
                limit: self.structural_complexity_limit,
                severity: self.severity_for_overage(
                    self.current_structural,
                    self.structural_complexity_limit,
                ),
            });
        }

        if self.current_coupling > self.coupling_complexity_limit {
            violations.push(QuotaViolation {
                metric: "coupling_complexity".to_string(),
                current: self.current_coupling,
                limit: self.coupling_complexity_limit,
                severity: self.severity_for_overage(
                    self.current_coupling,
                    self.coupling_complexity_limit,
                ),
            });
        }

        if self.current_naming < self.naming_quality_min {
            violations.push(QuotaViolation {
                metric: "naming_quality".to_string(),
                current: self.current_naming,
                limit: self.naming_quality_min,
                severity: self.severity_for_underage(
                    self.current_naming,
                    self.naming_quality_min,
                ),
            });
        }

        if self.current_test_coverage < self.test_coverage_min {
            violations.push(QuotaViolation {
                metric: "test_coverage".to_string(),
                current: self.current_test_coverage,
                limit: self.test_coverage_min,
                severity: self.severity_for_underage(
                    self.current_test_coverage,
                    self.test_coverage_min,
                ),
            });
        }

        violations
    }

    fn severity_for_overage(&self, current: f64, limit: f64) -> ViolationSeverity {
        if limit == 0.0 {
            return ViolationSeverity::Critical;
        }
        let ratio = current / limit;
        if ratio <= 1.1 {
            ViolationSeverity::Warning
        } else if ratio <= 1.3 {
            ViolationSeverity::Error
        } else {
            ViolationSeverity::Critical
        }
    }

    fn severity_for_underage(&self, current: f64, minimum: f64) -> ViolationSeverity {
        if minimum == 0.0 {
            return ViolationSeverity::Critical;
        }
        let ratio = current / minimum;
        if ratio >= 0.9 {
            ViolationSeverity::Warning
        } else if ratio >= 0.7 {
            ViolationSeverity::Error
        } else {
            ViolationSeverity::Critical
        }
    }

    pub fn update_metrics(
        &mut self,
        cyclo: f64,
        structural: f64,
        coupling: f64,
        naming: f64,
        test_cov: f64,
    ) {
        self.current_cyclomatic = cyclo;
        self.current_structural = structural;
        self.current_coupling = coupling;
        self.current_naming = naming;
        self.current_test_coverage = test_cov;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

impl ViolationSeverity {
    pub fn label(&self) -> &str {
        match self {
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaViolation {
    pub metric: String,
    pub current: f64,
    pub limit: f64,
    pub severity: ViolationSeverity,
}

pub struct QuotaManager {
    quotas: HashMap<String, ComplexityQuota>,
    hierarchy: HashMap<QuotaLevel, Vec<String>>,
}

impl QuotaManager {
    pub fn new() -> Self {
        Self {
            quotas: HashMap::new(),
            hierarchy: HashMap::new(),
        }
    }

    pub fn add_quota(&mut self, quota: ComplexityQuota) {
        let level = quota.level.clone();
        let id = quota.id.clone();

        if let Some(parent_id) = &quota.parent_id
            && let Some(parent) = self.quotas.get_mut(parent_id)
                && !parent.children.contains(&id) {
                    parent.children.push(id.clone());
                }

        self.quotas.insert(id.clone(), quota);
        self.hierarchy.entry(level).or_default().push(id);
    }

    pub fn get_quota(&self, id: &str) -> Option<&ComplexityQuota> {
        self.quotas.get(id)
    }

    pub fn get_quota_mut(&mut self, id: &str) -> Option<&mut ComplexityQuota> {
        self.quotas.get_mut(id)
    }

    pub fn list_by_level(&self, level: &QuotaLevel) -> Vec<&ComplexityQuota> {
        self.hierarchy
            .get(level)
            .map(|ids| ids.iter().filter_map(|id| self.quotas.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn get_children(&self, parent_id: &str) -> Vec<&ComplexityQuota> {
        self.quotas
            .get(parent_id)
            .map(|p| p.children.iter().filter_map(|c| self.quotas.get(c)).collect())
            .unwrap_or_default()
    }

    pub fn all_quotas(&self) -> Vec<&ComplexityQuota> {
        self.quotas.values().collect()
    }

    pub fn validate_hierarchy(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for quota in self.quotas.values() {
            if let Some(parent_id) = &quota.parent_id {
                if !self.quotas.contains_key(parent_id) {
                    errors.push(format!(
                        "Quota '{}' references non-existent parent '{}'",
                        quota.id, parent_id
                    ));
                } else if let Some(parent) = self.quotas.get(parent_id)
                    && parent.level.order() >= quota.level.order() {
                        errors.push(format!(
                            "Quota '{}' ({}) has parent '{}' at same or lower level ({})",
                            quota.id,
                            quota.level.label(),
                            parent_id,
                            parent.level.label()
                        ));
                    }
            }
        }

        errors
    }

    pub fn non_compliant_quotas(&self) -> Vec<&ComplexityQuota> {
        self.quotas
            .values()
            .filter(|q| !q.is_compliant())
            .collect()
    }

    pub fn all_violations(&self) -> Vec<(String, QuotaViolation)> {
        let mut all = Vec::new();
        for quota in self.quotas.values() {
            for violation in quota.violations() {
                all.push((quota.id.clone(), violation));
            }
        }
        all
    }

    pub fn violations_by_severity(&self, severity: &ViolationSeverity) -> Vec<(String, QuotaViolation)> {
        self.all_violations()
            .into_iter()
            .filter(|(_, v)| &v.severity == severity)
            .collect()
    }

    pub fn propagate_usage_to_parents(&mut self, child_id: &str) {
        let mut current_id = child_id.to_string();
        let mut visited = HashSet::new();

        while let Some(parent_id) = {
            let quota = self.quotas.get(&current_id).cloned();
            quota.and_then(|q| q.parent_id)
        } {
            if visited.contains(&parent_id) {
                break;
            }
            visited.insert(parent_id.clone());

            let children = self.get_children(&parent_id);
            let child_count = children.len() as f64;

            if child_count > 0.0 {
                let total_cyclo: f64 = children.iter().map(|c| c.current_cyclomatic).sum();
                let total_structural: f64 = children.iter().map(|c| c.current_structural).sum();
                let total_coupling: f64 = children.iter().map(|c| c.current_coupling).sum();
                let avg_naming: f64 = children.iter().map(|c| c.current_naming).sum::<f64>() / child_count;
                let avg_test: f64 = children.iter().map(|c| c.current_test_coverage).sum::<f64>() / child_count;

                if let Some(parent) = self.quotas.get_mut(&parent_id) {
                    parent.current_cyclomatic = total_cyclo;
                    parent.current_structural = total_structural;
                    parent.current_coupling = total_coupling;
                    parent.current_naming = avg_naming;
                    parent.current_test_coverage = avg_test;
                    parent.updated_at = chrono::Utc::now().to_rfc3339();
                }
            }

            current_id = parent_id;
        }
    }
}

use std::collections::HashSet;

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quota(id: &str, level: QuotaLevel) -> ComplexityQuota {
        ComplexityQuota::new(id, format!("Quota {id}"), level)
            .with_limits(100.0, 100.0, 100.0, 60.0, 60.0)
    }

    #[test]
    fn test_quota_creation() {
        let quota = create_test_quota("q1", QuotaLevel::Cell);
        assert_eq!(quota.id, "q1");
        assert_eq!(quota.level, QuotaLevel::Cell);
        assert!(quota.is_compliant());
    }

    #[test]
    fn test_usage_calculation() {
        let mut quota = create_test_quota("q1", QuotaLevel::Cell);
        quota.current_cyclomatic = 50.0;
        quota.current_structural = 75.0;
        quota.current_coupling = 100.0;

        assert_eq!(quota.cyclomatic_usage(), 50.0);
        assert_eq!(quota.structural_usage(), 75.0);
        assert_eq!(quota.coupling_usage(), 100.0);
    }

    #[test]
    fn test_compliant_when_within_limits() {
        let mut quota = create_test_quota("q1", QuotaLevel::Cell);
        quota.update_metrics(50.0, 50.0, 50.0, 80.0, 80.0);
        assert!(quota.is_compliant());
        assert!(quota.violations().is_empty());
    }

    #[test]
    fn test_violation_when_over_limit() {
        let mut quota = create_test_quota("q1", QuotaLevel::Cell);
        quota.update_metrics(150.0, 50.0, 50.0, 80.0, 80.0);

        assert!(!quota.is_compliant());
        let violations = quota.violations();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].metric, "cyclomatic_complexity");
    }

    #[test]
    fn test_naming_quality_violation() {
        let mut quota = create_test_quota("q1", QuotaLevel::Cell);
        quota.update_metrics(50.0, 50.0, 50.0, 40.0, 80.0);

        let violations = quota.violations();
        assert!(violations.iter().any(|v| v.metric == "naming_quality"));
    }

    #[test]
    fn test_test_coverage_violation() {
        let mut quota = create_test_quota("q1", QuotaLevel::Cell);
        quota.update_metrics(50.0, 50.0, 50.0, 80.0, 40.0);

        let violations = quota.violations();
        assert!(violations.iter().any(|v| v.metric == "test_coverage"));
    }

    #[test]
    fn test_severity_levels() {
        let mut quota_warn = create_test_quota("q1", QuotaLevel::Cell);
        quota_warn.update_metrics(105.0, 50.0, 50.0, 80.0, 80.0);
        let violations = quota_warn.violations();
        assert_eq!(violations[0].severity, ViolationSeverity::Warning);

        let mut quota_err = create_test_quota("q2", QuotaLevel::Cell);
        quota_err.update_metrics(120.0, 50.0, 50.0, 80.0, 80.0);
        let violations = quota_err.violations();
        assert_eq!(violations[0].severity, ViolationSeverity::Error);

        let mut quota_crit = create_test_quota("q3", QuotaLevel::Cell);
        quota_crit.update_metrics(150.0, 50.0, 50.0, 80.0, 80.0);
        let violations = quota_crit.violations();
        assert_eq!(violations[0].severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_quota_manager() {
        let mut manager = QuotaManager::new();
        manager.add_quota(create_test_quota("sys", QuotaLevel::System));
        manager.add_quota(create_test_quota("team-a", QuotaLevel::Team));

        assert_eq!(manager.all_quotas().len(), 2);
        assert_eq!(manager.list_by_level(&QuotaLevel::System).len(), 1);
    }

    #[test]
    fn test_hierarchy_validation() {
        let mut manager = QuotaManager::new();

        let sys = create_test_quota("sys", QuotaLevel::System);
        manager.add_quota(sys);

        let mut team = create_test_quota("team-a", QuotaLevel::Team);
        team.parent_id = Some("sys".to_string());
        manager.add_quota(team);

        let errors = manager.validate_hierarchy();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_hierarchy_invalid_parent() {
        let mut manager = QuotaManager::new();

        let mut team = create_test_quota("team-a", QuotaLevel::Team);
        team.parent_id = Some("nonexistent".to_string());
        manager.add_quota(team);

        let errors = manager.validate_hierarchy();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_propagate_usage() {
        let mut manager = QuotaManager::new();

        let sys = create_test_quota("sys", QuotaLevel::System);
        manager.add_quota(sys);

        let mut cell1 = create_test_quota("cell1", QuotaLevel::Cell);
        cell1.parent_id = Some("sys".to_string());
        cell1.update_metrics(30.0, 20.0, 10.0, 80.0, 70.0);
        manager.add_quota(cell1);

        let mut cell2 = create_test_quota("cell2", QuotaLevel::Cell);
        cell2.parent_id = Some("sys".to_string());
        cell2.update_metrics(50.0, 40.0, 30.0, 90.0, 80.0);
        manager.add_quota(cell2);

        manager.propagate_usage_to_parents("cell1");
        manager.propagate_usage_to_parents("cell2");

        let sys_quota = manager.get_quota("sys").unwrap();
        assert_eq!(sys_quota.current_cyclomatic, 80.0);
        assert_eq!(sys_quota.current_structural, 60.0);
    }

    #[test]
    fn test_non_compliant_quotas() {
        let mut manager = QuotaManager::new();

        let mut good = create_test_quota("good", QuotaLevel::Cell);
        good.update_metrics(50.0, 50.0, 50.0, 80.0, 80.0);
        manager.add_quota(good);

        let mut bad = create_test_quota("bad", QuotaLevel::Cell);
        bad.update_metrics(150.0, 50.0, 50.0, 80.0, 80.0);
        manager.add_quota(bad);

        assert_eq!(manager.non_compliant_quotas().len(), 1);
    }

    #[test]
    fn test_quota_level_order() {
        assert!(QuotaLevel::System.order() < QuotaLevel::Team.order());
        assert!(QuotaLevel::Team.order() < QuotaLevel::Cell.order());
        assert!(QuotaLevel::Cell.order() < QuotaLevel::Feature.order());
    }

    #[test]
    fn test_overall_usage() {
        let mut quota = create_test_quota("q1", QuotaLevel::Cell);
        quota.current_cyclomatic = 50.0;
        quota.current_structural = 50.0;
        quota.current_coupling = 50.0;

        assert_eq!(quota.overall_usage(), 50.0);
    }

    #[test]
    fn test_violations_by_severity() {
        let mut manager = QuotaManager::new();

        let mut warn = create_test_quota("warn", QuotaLevel::Cell);
        warn.update_metrics(105.0, 50.0, 50.0, 80.0, 80.0);
        manager.add_quota(warn);

        let mut crit = create_test_quota("crit", QuotaLevel::Cell);
        crit.update_metrics(150.0, 50.0, 50.0, 80.0, 80.0);
        manager.add_quota(crit);

        let criticals = manager.violations_by_severity(&ViolationSeverity::Critical);
        assert_eq!(criticals.len(), 1);
    }
}
