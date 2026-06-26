use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Archived,
}

impl ExperimentStatus {
    pub fn label(&self) -> &str {
        match self {
            ExperimentStatus::Draft => "Draft",
            ExperimentStatus::Running => "Running",
            ExperimentStatus::Paused => "Paused",
            ExperimentStatus::Completed => "Completed",
            ExperimentStatus::Archived => "Archived",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ExperimentStatus::Completed | ExperimentStatus::Archived)
    }

    pub fn can_transition_to(&self, target: &ExperimentStatus) -> bool {
        match (self, target) {
            (ExperimentStatus::Draft, ExperimentStatus::Running) => true,
            (ExperimentStatus::Draft, ExperimentStatus::Archived) => true,
            (ExperimentStatus::Running, ExperimentStatus::Paused) => true,
            (ExperimentStatus::Running, ExperimentStatus::Completed) => true,
            (ExperimentStatus::Paused, ExperimentStatus::Running) => true,
            (ExperimentStatus::Paused, ExperimentStatus::Completed) => true,
            (ExperimentStatus::Completed, ExperimentStatus::Archived) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExperimentType {
    UI,
    Algorithm,
    Feature,
    Price,
}

impl ExperimentType {
    pub fn label(&self) -> &str {
        match self {
            ExperimentType::UI => "UI",
            ExperimentType::Algorithm => "Algorithm",
            ExperimentType::Feature => "Feature",
            ExperimentType::Price => "Price",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_control: bool,
    pub weight: u32,
    pub config: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

impl Variant {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        is_control: bool,
        weight: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            is_control,
            weight,
            config: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantAllocation {
    pub variant_id: Uuid,
    pub variant_name: String,
    pub allocated_users: u64,
    pub allocation_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExperimentMetrics {
    pub total_users: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub click_count: u64,
    pub impression_count: u64,
    pub avg_session_duration_seconds: f64,
    pub custom_metrics: HashMap<String, f64>,
}

impl ExperimentMetrics {
    pub fn new() -> Self {
        Self {
            total_users: 0,
            conversions: 0,
            revenue: 0.0,
            click_count: 0,
            impression_count: 0,
            avg_session_duration_seconds: 0.0,
            custom_metrics: HashMap::new(),
        }
    }

    pub fn conversion_rate(&self) -> f64 {
        if self.total_users == 0 {
            0.0
        } else {
            self.conversions as f64 / self.total_users as f64
        }
    }

    pub fn click_through_rate(&self) -> f64 {
        if self.impression_count == 0 {
            0.0
        } else {
            self.click_count as f64 / self.impression_count as f64
        }
    }

    pub fn revenue_per_user(&self) -> f64 {
        if self.total_users == 0 {
            0.0
        } else {
            self.revenue / self.total_users as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub experiment_type: ExperimentType,
    pub status: ExperimentStatus,
    pub variants: Vec<Variant>,
    pub metrics: HashMap<Uuid, ExperimentMetrics>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub traffic_percentage: f64,
    pub metadata: HashMap<String, String>,
}

impl Experiment {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        experiment_type: ExperimentType,
        traffic_percentage: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            experiment_type,
            status: ExperimentStatus::Draft,
            variants: Vec::new(),
            metrics: HashMap::new(),
            created_at: now,
            updated_at: now,
            started_at: None,
            ended_at: None,
            traffic_percentage: traffic_percentage.clamp(0.0, 100.0),
            metadata: HashMap::new(),
        }
    }

    pub fn add_variant(&mut self, variant: Variant) {
        self.metrics.insert(variant.id, ExperimentMetrics::new());
        self.variants.push(variant);
    }

    pub fn control_variant(&self) -> Option<&Variant> {
        self.variants.iter().find(|v| v.is_control)
    }

    pub fn total_weight(&self) -> u32 {
        self.variants.iter().map(|v| v.weight).sum()
    }

    pub fn variant_allocations(&self) -> Vec<VariantAllocation> {
        let total_weight = self.total_weight() as f64;
        if total_weight == 0.0 {
            return Vec::new();
        }
        self.variants
            .iter()
            .map(|v| VariantAllocation {
                variant_id: v.id,
                variant_name: v.name.clone(),
                allocated_users: self.metrics.get(&v.id).map(|m| m.total_users).unwrap_or(0),
                allocation_percentage: v.weight as f64 / total_weight * 100.0,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ABTestError {
    InvalidStateTransition(String),
    ExperimentNotFound(String),
    ExperimentAlreadyTerminal(String),
    VariantNotFound(String),
    NoControlVariant(String),
    InsufficientVariants(String),
    InvalidTrafficPercentage(String),
    InvalidWeight(String),
}

impl std::fmt::Display for ABTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ABTestError::InvalidStateTransition(msg) => write!(f, "Invalid state transition: {}", msg),
            ABTestError::ExperimentNotFound(id) => write!(f, "Experiment not found: {}", id),
            ABTestError::ExperimentAlreadyTerminal(id) => write!(f, "Experiment is already in terminal state: {}", id),
            ABTestError::VariantNotFound(id) => write!(f, "Variant not found: {}", id),
            ABTestError::NoControlVariant(id) => write!(f, "No control variant in experiment: {}", id),
            ABTestError::InsufficientVariants(msg) => write!(f, "Insufficient variants: {}", msg),
            ABTestError::InvalidTrafficPercentage(msg) => write!(f, "Invalid traffic percentage: {}", msg),
            ABTestError::InvalidWeight(msg) => write!(f, "Invalid weight: {}", msg),
        }
    }
}

impl std::error::Error for ABTestError {}

fn hash_user_id(user_id: &str, experiment_id: &Uuid) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    let bytes = experiment_id.as_bytes();
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    for &byte in user_id.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub struct ABTestManager {
    experiments: HashMap<Uuid, Experiment>,
}

impl ABTestManager {
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
        }
    }

    pub fn create_experiment(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        experiment_type: ExperimentType,
        traffic_percentage: f64,
        control_variant_name: impl Into<String>,
        variants: Vec<(String, u32)>,
    ) -> Result<&Experiment, ABTestError> {
        if !(0.0..=100.0).contains(&traffic_percentage) {
            return Err(ABTestError::InvalidTrafficPercentage(format!(
                "Traffic percentage must be between 0 and 100, got {}",
                traffic_percentage
            )));
        }

        if variants.is_empty() {
            return Err(ABTestError::InsufficientVariants(
                "At least one variant is required".to_string(),
            ));
        }

        for (_, weight) in &variants {
            if *weight == 0 {
                return Err(ABTestError::InvalidWeight(
                    "Variant weight must be greater than 0".to_string(),
                ));
            }
        }

        let mut experiment = Experiment::new(name, description, experiment_type, traffic_percentage);

        let control_name = control_variant_name.into();
        let control_weight = variants
            .first()
            .map(|(_, w)| *w)
            .unwrap_or(50);
        let control = Variant::new(&control_name, "Control group", true, control_weight);
        experiment.add_variant(control);

        for (name, weight) in variants.iter().skip(1) {
            let variant = Variant::new(name, format!("Test variant: {}", name), false, *weight);
            experiment.add_variant(variant);
        }

        let id = experiment.id;
        self.experiments.insert(id, experiment);
        Ok(self.experiments.get(&id).unwrap())
    }

    pub fn start_experiment(&mut self, experiment_id: Uuid) -> Result<&Experiment, ABTestError> {
        let experiment = self
            .experiments
            .get_mut(&experiment_id)
            .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;

        let from_status = experiment.status.clone();
        if !from_status.can_transition_to(&ExperimentStatus::Running) {
            return Err(ABTestError::InvalidStateTransition(format!(
                "Cannot start experiment from status {}",
                from_status.label()
            )));
        }

        if experiment.variants.len() < 2 {
            return Err(ABTestError::InsufficientVariants(
                "At least 2 variants required to start experiment".to_string(),
            ));
        }

        if experiment.control_variant().is_none() {
            return Err(ABTestError::NoControlVariant(experiment_id.to_string()));
        }

        experiment.status = ExperimentStatus::Running;
        experiment.started_at = Some(Utc::now());
        experiment.updated_at = Utc::now();

        Ok(self.experiments.get(&experiment_id).unwrap())
    }

    pub fn pause_experiment(&mut self, experiment_id: Uuid) -> Result<&Experiment, ABTestError> {
        let experiment = self
            .experiments
            .get_mut(&experiment_id)
            .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;

        let from_status = experiment.status.clone();
        if !from_status.can_transition_to(&ExperimentStatus::Paused) {
            return Err(ABTestError::InvalidStateTransition(format!(
                "Cannot pause experiment from status {}",
                from_status.label()
            )));
        }

        experiment.status = ExperimentStatus::Paused;
        experiment.updated_at = Utc::now();

        Ok(self.experiments.get(&experiment_id).unwrap())
    }

    pub fn end_experiment(&mut self, experiment_id: Uuid) -> Result<&Experiment, ABTestError> {
        let experiment = self
            .experiments
            .get_mut(&experiment_id)
            .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;

        let from_status = experiment.status.clone();
        if !from_status.can_transition_to(&ExperimentStatus::Completed) {
            return Err(ABTestError::InvalidStateTransition(format!(
                "Cannot end experiment from status {}",
                from_status.label()
            )));
        }

        experiment.status = ExperimentStatus::Completed;
        experiment.ended_at = Some(Utc::now());
        experiment.updated_at = Utc::now();

        Ok(self.experiments.get(&experiment_id).unwrap())
    }

    pub fn get_variant(&self, experiment_id: Uuid, user_id: &str) -> Result<Option<&Variant>, ABTestError> {
        let experiment = self
            .experiments
            .get(&experiment_id)
            .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;

        if experiment.status != ExperimentStatus::Running {
            return Ok(None);
        }

        let hash = hash_user_id(user_id, &experiment.id);
        let traffic_bucket = (hash % 10000) as f64 / 100.0;

        if traffic_bucket >= experiment.traffic_percentage {
            return Ok(None);
        }

        let total_weight = experiment.total_weight() as u64;
        if total_weight == 0 {
            return Ok(None);
        }

        let variant_hash = hash_user_id(user_id, &experiment.id) ^ 0x9E3779B97F4A7C15;
        let bucket = variant_hash % total_weight;

        let mut cumulative = 0u64;
        for variant in &experiment.variants {
            cumulative += variant.weight as u64;
            if bucket < cumulative {
                return Ok(Some(variant));
            }
        }

        Ok(experiment.variants.first())
    }

    pub fn record_impression(&mut self, experiment_id: Uuid, user_id: &str) -> Result<(), ABTestError> {
        let variant = self.get_variant(experiment_id, user_id)?;
        if let Some(variant) = variant {
            let variant_id = variant.id;
            let experiment = self
                .experiments
                .get_mut(&experiment_id)
                .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;
            let metrics = experiment.metrics.entry(variant_id).or_default();
            metrics.impression_count += 1;
            metrics.total_users += 1;
            experiment.updated_at = Utc::now();
        }
        Ok(())
    }

    pub fn record_conversion(&mut self, experiment_id: Uuid, user_id: &str, revenue: f64) -> Result<(), ABTestError> {
        let variant = self.get_variant(experiment_id, user_id)?;
        if let Some(variant) = variant {
            let variant_id = variant.id;
            let experiment = self
                .experiments
                .get_mut(&experiment_id)
                .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;
            let metrics = experiment.metrics.entry(variant_id).or_default();
            metrics.conversions += 1;
            metrics.revenue += revenue;
            experiment.updated_at = Utc::now();
        }
        Ok(())
    }

    pub fn record_click(&mut self, experiment_id: Uuid, user_id: &str) -> Result<(), ABTestError> {
        let variant = self.get_variant(experiment_id, user_id)?;
        if let Some(variant) = variant {
            let variant_id = variant.id;
            let experiment = self
                .experiments
                .get_mut(&experiment_id)
                .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;
            let metrics = experiment.metrics.entry(variant_id).or_default();
            metrics.click_count += 1;
            experiment.updated_at = Utc::now();
        }
        Ok(())
    }

    pub fn get_experiment(&self, experiment_id: Uuid) -> Option<&Experiment> {
        self.experiments.get(&experiment_id)
    }

    pub fn list_experiments(&self) -> Vec<&Experiment> {
        self.experiments.values().collect()
    }

    pub fn running_experiments(&self) -> Vec<&Experiment> {
        self.experiments
            .values()
            .filter(|e| e.status == ExperimentStatus::Running)
            .collect()
    }

    pub fn get_variant_metrics(&self, experiment_id: Uuid, variant_id: Uuid) -> Result<&ExperimentMetrics, ABTestError> {
        let experiment = self
            .experiments
            .get(&experiment_id)
            .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;
        experiment
            .metrics
            .get(&variant_id)
            .ok_or_else(|| ABTestError::VariantNotFound(variant_id.to_string()))
    }

    pub fn compare_with_control(&self, experiment_id: Uuid, variant_id: Uuid) -> Result<HashMap<String, f64>, ABTestError> {
        let experiment = self
            .experiments
            .get(&experiment_id)
            .ok_or_else(|| ABTestError::ExperimentNotFound(experiment_id.to_string()))?;

        let control = experiment
            .control_variant()
            .ok_or_else(|| ABTestError::NoControlVariant(experiment_id.to_string()))?;

        let control_metrics = experiment
            .metrics
            .get(&control.id)
            .ok_or_else(|| ABTestError::VariantNotFound(control.id.to_string()))?;

        let variant_metrics = experiment
            .metrics
            .get(&variant_id)
            .ok_or_else(|| ABTestError::VariantNotFound(variant_id.to_string()))?;

        let mut comparison = HashMap::new();
        comparison.insert("conversion_rate_diff".to_string(), variant_metrics.conversion_rate() - control_metrics.conversion_rate());
        comparison.insert("conversion_rate_lift_pct".to_string(), if control_metrics.conversion_rate() > 0.0 {
            (variant_metrics.conversion_rate() - control_metrics.conversion_rate()) / control_metrics.conversion_rate() * 100.0
        } else {
            0.0
        });
        comparison.insert("ctr_diff".to_string(), variant_metrics.click_through_rate() - control_metrics.click_through_rate());
        comparison.insert("revenue_per_user_diff".to_string(), variant_metrics.revenue_per_user() - control_metrics.revenue_per_user());
        comparison.insert("total_users_diff".to_string(), variant_metrics.total_users as f64 - control_metrics.total_users as f64);

        Ok(comparison)
    }
}

impl Default for ABTestManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> (ABTestManager, Uuid) {
        let mut manager = ABTestManager::new();
        let experiment = manager
            .create_experiment(
                "test-experiment",
                "A test experiment",
                ExperimentType::Feature,
                100.0,
                "control",
                vec![("control".to_string(), 50), ("variant_a".to_string(), 50)],
            )
            .unwrap();
        let id = experiment.id;
        (manager, id)
    }

    #[test]
    fn test_create_experiment() {
        let (manager, id) = create_test_manager();
        let experiment = manager.get_experiment(id).unwrap();

        assert_eq!(experiment.name, "test-experiment");
        assert_eq!(experiment.description, "A test experiment");
        assert_eq!(experiment.experiment_type, ExperimentType::Feature);
        assert_eq!(experiment.status, ExperimentStatus::Draft);
        assert_eq!(experiment.variants.len(), 2);
        assert_eq!(experiment.traffic_percentage, 100.0);
        assert!(experiment.control_variant().is_some());
        assert_eq!(experiment.control_variant().unwrap().name, "control");
    }

    #[test]
    fn test_create_experiment_invalid_traffic() {
        let mut manager = ABTestManager::new();
        let result = manager.create_experiment(
            "test",
            "desc",
            ExperimentType::UI,
            150.0,
            "control",
            vec![("control".to_string(), 50), ("v1".to_string(), 50)],
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ABTestError::InvalidTrafficPercentage(_)
        ));
    }

    #[test]
    fn test_create_experiment_no_variants() {
        let mut manager = ABTestManager::new();
        let result = manager.create_experiment(
            "test",
            "desc",
            ExperimentType::Algorithm,
            50.0,
            "control",
            vec![],
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ABTestError::InsufficientVariants(_)
        ));
    }

    #[test]
    fn test_start_experiment() {
        let (mut manager, id) = create_test_manager();
        let result = manager.start_experiment(id);
        assert!(result.is_ok());

        let experiment = result.unwrap();
        assert_eq!(experiment.status, ExperimentStatus::Running);
        assert!(experiment.started_at.is_some());
    }

    #[test]
    fn test_start_experiment_invalid_state() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();
        manager.end_experiment(id).unwrap();

        let result = manager.start_experiment(id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ABTestError::InvalidStateTransition(_)
        ));
    }

    #[test]
    fn test_pause_experiment() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();

        let result = manager.pause_experiment(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ExperimentStatus::Paused);
    }

    #[test]
    fn test_pause_experiment_from_draft_fails() {
        let (mut manager, id) = create_test_manager();
        let result = manager.pause_experiment(id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ABTestError::InvalidStateTransition(_)
        ));
    }

    #[test]
    fn test_end_experiment() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();

        let result = manager.end_experiment(id);
        assert!(result.is_ok());
        let experiment = result.unwrap();
        assert_eq!(experiment.status, ExperimentStatus::Completed);
        assert!(experiment.ended_at.is_some());
    }

    #[test]
    fn test_end_experiment_from_paused() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();
        manager.pause_experiment(id).unwrap();

        let result = manager.end_experiment(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ExperimentStatus::Completed);
    }

    #[test]
    fn test_user_bucket_consistency() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();

        let user_id = "user-12345";
        let v1 = manager.get_variant(id, user_id).unwrap().unwrap();
        let v2 = manager.get_variant(id, user_id).unwrap().unwrap();
        let v3 = manager.get_variant(id, user_id).unwrap().unwrap();

        assert_eq!(v1.id, v2.id);
        assert_eq!(v2.id, v3.id);
    }

    #[test]
    fn test_different_users_different_variants() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();

        let mut variant_a_count = 0;
        let mut variant_b_count = 0;

        for i in 0..1000 {
            let user_id = format!("user-{}", i);
            let variant = manager.get_variant(id, &user_id).unwrap().unwrap();
            if variant.name == "control" {
                variant_a_count += 1;
            } else {
                variant_b_count += 1;
            }
        }

        assert!(variant_a_count > 0, "Expected some control users");
        assert!(variant_b_count > 0, "Expected some variant users");
        assert_eq!(variant_a_count + variant_b_count, 1000);
    }

    #[test]
    fn test_traffic_percentage_filtering() {
        let mut manager = ABTestManager::new();
        let experiment = manager
            .create_experiment(
                "traffic-test",
                "Test traffic filtering",
                ExperimentType::Price,
                10.0,
                "control",
                vec![("control".to_string(), 50), ("v1".to_string(), 50)],
            )
            .unwrap();
        let id = experiment.id;
        manager.start_experiment(id).unwrap();

        let mut in_experiment = 0;
        let total = 10000;

        for i in 0..total {
            let user_id = format!("user-{}", i);
            if manager.get_variant(id, &user_id).unwrap().is_some() {
                in_experiment += 1;
            }
        }

        let ratio = in_experiment as f64 / total as f64 * 100.0;
        assert!(
            (ratio - 10.0).abs() < 5.0,
            "Expected ~10% traffic, got {}% ({} out of {})",
            ratio,
            in_experiment,
            total
        );
    }

    #[test]
    fn test_multiple_variants() {
        let mut manager = ABTestManager::new();
        let experiment = manager
            .create_experiment(
                "multi-variant",
                "Multi variant test",
                ExperimentType::UI,
                100.0,
                "control",
                vec![
                    ("control".to_string(), 25),
                    ("a".to_string(), 25),
                    ("b".to_string(), 25),
                    ("c".to_string(), 25),
                ],
            )
            .unwrap();
        let id = experiment.id;
        manager.start_experiment(id).unwrap();

        let experiment = manager.get_experiment(id).unwrap();
        assert_eq!(experiment.variants.len(), 4);
        assert_eq!(experiment.total_weight(), 100);

        let allocations = experiment.variant_allocations();
        assert_eq!(allocations.len(), 4);
        for alloc in &allocations {
            assert!((alloc.allocation_percentage - 25.0).abs() < 1.0);
        }
    }

    #[test]
    fn test_experiment_metrics() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();

        for i in 0..100 {
            let user_id = format!("user-{}", i);
            manager.record_impression(id, &user_id).unwrap();
        }

        for i in 0..50 {
            let user_id = format!("user-{}", i);
            manager.record_click(id, &user_id).unwrap();
        }

        for i in 0..20 {
            let user_id = format!("user-{}", i);
            manager.record_conversion(id, &user_id, 10.0).unwrap();
        }

        let experiment = manager.get_experiment(id).unwrap();
        let total_users: u64 = experiment.metrics.values().map(|m| m.total_users).sum();
        let total_clicks: u64 = experiment.metrics.values().map(|m| m.click_count).sum();
        let total_conversions: u64 = experiment.metrics.values().map(|m| m.conversions).sum();
        let total_revenue: f64 = experiment.metrics.values().map(|m| m.revenue).sum();

        assert_eq!(total_users, 100);
        assert_eq!(total_clicks, 50);
        assert_eq!(total_conversions, 20);
        assert!((total_revenue - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_status_transitions() {
        assert!(ExperimentStatus::Draft.can_transition_to(&ExperimentStatus::Running));
        assert!(ExperimentStatus::Draft.can_transition_to(&ExperimentStatus::Archived));
        assert!(!ExperimentStatus::Draft.can_transition_to(&ExperimentStatus::Paused));

        assert!(ExperimentStatus::Running.can_transition_to(&ExperimentStatus::Paused));
        assert!(ExperimentStatus::Running.can_transition_to(&ExperimentStatus::Completed));
        assert!(!ExperimentStatus::Running.can_transition_to(&ExperimentStatus::Draft));

        assert!(ExperimentStatus::Paused.can_transition_to(&ExperimentStatus::Running));
        assert!(ExperimentStatus::Paused.can_transition_to(&ExperimentStatus::Completed));
        assert!(!ExperimentStatus::Paused.can_transition_to(&ExperimentStatus::Archived));

        assert!(ExperimentStatus::Completed.can_transition_to(&ExperimentStatus::Archived));
        assert!(!ExperimentStatus::Completed.can_transition_to(&ExperimentStatus::Running));

        assert!(ExperimentStatus::Completed.is_terminal());
        assert!(ExperimentStatus::Archived.is_terminal());
        assert!(!ExperimentStatus::Running.is_terminal());
    }

    #[test]
    fn test_get_variant_not_running() {
        let (manager, id) = create_test_manager();
        let result = manager.get_variant(id, "user-1").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_experiments() {
        let mut manager = ABTestManager::new();

        manager.create_experiment(
            "exp1",
            "desc1",
            ExperimentType::Feature,
            50.0,
            "control",
            vec![("control".to_string(), 50), ("v1".to_string(), 50)],
        ).unwrap();

        manager.create_experiment(
            "exp2",
            "desc2",
            ExperimentType::UI,
            100.0,
            "control",
            vec![("control".to_string(), 30), ("v1".to_string(), 70)],
        ).unwrap();

        assert_eq!(manager.list_experiments().len(), 2);
        assert_eq!(manager.running_experiments().len(), 0);
    }

    #[test]
    fn test_variant_allocation_weights() {
        let mut manager = ABTestManager::new();
        let experiment = manager
            .create_experiment(
                "weight-test",
                "Test weight allocation",
                ExperimentType::Algorithm,
                100.0,
                "control",
                vec![
                    ("control".to_string(), 10),
                    ("a".to_string(), 20),
                    ("b".to_string(), 70),
                ],
            )
            .unwrap();
        let id = experiment.id;
        manager.start_experiment(id).unwrap();

        let experiment = manager.get_experiment(id).unwrap();
        let allocations = experiment.variant_allocations();

        let control_alloc = allocations.iter().find(|a| a.variant_name == "control").unwrap();
        let a_alloc = allocations.iter().find(|a| a.variant_name == "a").unwrap();
        let b_alloc = allocations.iter().find(|a| a.variant_name == "b").unwrap();

        assert!((control_alloc.allocation_percentage - 10.0).abs() < 0.1);
        assert!((a_alloc.allocation_percentage - 20.0).abs() < 0.1);
        assert!((b_alloc.allocation_percentage - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_compare_with_control() {
        let (mut manager, id) = create_test_manager();
        manager.start_experiment(id).unwrap();

        let experiment = manager.get_experiment(id).unwrap();
        let variant_id = experiment.variants.iter().find(|v| !v.is_control).unwrap().id;
        let control_id = experiment.control_variant().unwrap().id;

        for i in 0..200 {
            let user_id = format!("user-{}", i);
            manager.record_impression(id, &user_id).unwrap();
        }

        let experiment = manager.get_experiment(id).unwrap();
        let control_metrics = experiment.metrics.get(&control_id).unwrap();
        let variant_metrics = experiment.metrics.get(&variant_id).unwrap();

        assert!(control_metrics.total_users > 0);
        assert!(variant_metrics.total_users > 0);

        let comparison = manager.compare_with_control(id, variant_id).unwrap();
        assert!(comparison.contains_key("conversion_rate_diff"));
        assert!(comparison.contains_key("conversion_rate_lift_pct"));
        assert!(comparison.contains_key("ctr_diff"));
        assert!(comparison.contains_key("revenue_per_user_diff"));
        assert!(comparison.contains_key("total_users_diff"));
    }

    #[test]
    fn test_experiment_type_labels() {
        assert_eq!(ExperimentType::UI.label(), "UI");
        assert_eq!(ExperimentType::Algorithm.label(), "Algorithm");
        assert_eq!(ExperimentType::Feature.label(), "Feature");
        assert_eq!(ExperimentType::Price.label(), "Price");
    }

    #[test]
    fn test_experiment_status_labels() {
        assert_eq!(ExperimentStatus::Draft.label(), "Draft");
        assert_eq!(ExperimentStatus::Running.label(), "Running");
        assert_eq!(ExperimentStatus::Paused.label(), "Paused");
        assert_eq!(ExperimentStatus::Completed.label(), "Completed");
        assert_eq!(ExperimentStatus::Archived.label(), "Archived");
    }

    #[test]
    fn test_ab_test_error_display() {
        let err = ABTestError::ExperimentNotFound("test-id".to_string());
        assert!(err.to_string().contains("test-id"));

        let err = ABTestError::InvalidStateTransition("bad transition".to_string());
        assert!(err.to_string().contains("bad transition"));
    }

    #[test]
    fn test_running_experiments_filter() {
        let mut manager = ABTestManager::new();

        let exp1 = manager.create_experiment(
            "exp1",
            "desc1",
            ExperimentType::Feature,
            50.0,
            "control",
            vec![("control".to_string(), 50), ("v1".to_string(), 50)],
        ).unwrap();
        let id1 = exp1.id;

        let exp2 = manager.create_experiment(
            "exp2",
            "desc2",
            ExperimentType::UI,
            100.0,
            "control",
            vec![("control".to_string(), 30), ("v1".to_string(), 70)],
        ).unwrap();
        let id2 = exp2.id;

        manager.start_experiment(id1).unwrap();

        assert_eq!(manager.running_experiments().len(), 1);
        assert_eq!(manager.running_experiments()[0].id, id1);
        assert_eq!(manager.list_experiments().len(), 2);
        let _ = id2;
    }

    #[test]
    fn test_variant_with_config() {
        let variant = Variant::new("test-variant", "A test variant", false, 50)
            .with_config("button_color", "blue")
            .with_config("button_text", "Click me");

        assert_eq!(variant.config.get("button_color").unwrap(), "blue");
        assert_eq!(variant.config.get("button_text").unwrap(), "Click me");
        assert_eq!(variant.weight, 50);
        assert!(!variant.is_control);
    }
}
