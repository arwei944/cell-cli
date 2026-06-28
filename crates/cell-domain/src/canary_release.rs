use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReleaseStatus {
    Created,
    Progressing,
    Paused,
    Promoted,
    RolledBack,
    Failed,
}

impl ReleaseStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Created => "Created",
            Self::Progressing => "Progressing",
            Self::Paused => "Paused",
            Self::Promoted => "Promoted",
            Self::RolledBack => "RolledBack",
            Self::Failed => "Failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Promoted | Self::RolledBack | Self::Failed
        )
    }

    pub fn can_transition_to(&self, target: &Self) -> bool {
        matches!(
            (self, target),
            (Self::Created | Self::Paused, Self::Progressing)
                | (Self::Progressing, Self::Paused | Self::Promoted | Self::RolledBack | Self::Failed)
                | (Self::Paused, Self::RolledBack)
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReleaseStage {
    Zero = 0,
    Five = 1,
    Twenty = 2,
    Fifty = 3,
    Hundred = 4,
}

impl ReleaseStage {
    pub fn percentage(&self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::Five => 5,
            Self::Twenty => 20,
            Self::Fifty => 50,
            Self::Hundred => 100,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Zero => Some(Self::Five),
            Self::Five => Some(Self::Twenty),
            Self::Twenty => Some(Self::Fifty),
            Self::Fifty => Some(Self::Hundred),
            Self::Hundred => None,
        }
    }

    pub fn prev(&self) -> Option<Self> {
        match self {
            Self::Zero => None,
            Self::Five => Some(Self::Zero),
            Self::Twenty => Some(Self::Five),
            Self::Fifty => Some(Self::Twenty),
            Self::Hundred => Some(Self::Fifty),
        }
    }

    pub fn stages() -> Vec<Self> {
        vec![
            Self::Zero,
            Self::Five,
            Self::Twenty,
            Self::Fifty,
            Self::Hundred,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficShift {
    pub version: String,
    pub weight: u32,
    pub traffic_percentage: f64,
}

impl TrafficShift {
    pub fn new(version: impl Into<String>, traffic_percentage: f64) -> Self {
        Self {
            version: version.into(),
            weight: (traffic_percentage * 100.0) as u32,
            traffic_percentage,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CanaryStrategy {
    TimeBasedProgressive {
        interval_seconds: u64,
    },
    MetricBasedAuto {
        error_rate_threshold: f64,
        latency_threshold_ms: f64,
        success_rate_threshold: f64,
        min_traffic_samples: u64,
    },
    UserGroupBased {
        groups: Vec<String>,
    },
}

impl Default for CanaryStrategy {
    fn default() -> Self {
        Self::TimeBasedProgressive {
            interval_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseMetrics {
    pub error_rate: f64,
    pub latency_p99_ms: f64,
    pub traffic_count: u64,
    pub success_rate: f64,
}

impl Default for ReleaseMetrics {
    fn default() -> Self {
        Self {
            error_rate: 0.0,
            latency_p99_ms: 0.0,
            traffic_count: 0,
            success_rate: 100.0,
        }
    }
}

impl ReleaseMetrics {
    pub fn new(error_rate: f64, latency_p99_ms: f64, traffic_count: u64, success_rate: f64) -> Self {
        Self {
            error_rate,
            latency_p99_ms,
            traffic_count,
            success_rate,
        }
    }

    pub fn is_healthy(&self, strategy: &CanaryStrategy) -> bool {
        match strategy {
            CanaryStrategy::MetricBasedAuto {
                error_rate_threshold,
                latency_threshold_ms,
                success_rate_threshold,
                min_traffic_samples,
            } => {
                if self.traffic_count < *min_traffic_samples {
                    return false;
                }
                self.error_rate <= *error_rate_threshold
                    && self.latency_p99_ms <= *latency_threshold_ms
                    && self.success_rate >= *success_rate_threshold
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub from_status: ReleaseStatus,
    pub to_status: ReleaseStatus,
    pub from_stage: Option<ReleaseStage>,
    pub to_stage: Option<ReleaseStage>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRelease {
    pub id: Uuid,
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub status: ReleaseStatus,
    pub current_stage: ReleaseStage,
    pub strategy: CanaryStrategy,
    pub metrics: ReleaseMetrics,
    pub traffic_shifts: Vec<TrafficShift>,
    pub history: Vec<ReleaseHistoryEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

impl CanaryRelease {
    pub fn new(
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
        strategy: CanaryStrategy,
    ) -> Self {
        let now = Utc::now();
        let name = name.into();
        let old_version = old_version.into();
        let new_version = new_version.into();
        let traffic_shifts = vec![
            TrafficShift::new(&old_version, 100.0),
            TrafficShift::new(&new_version, 0.0),
        ];

        Self {
            id: Uuid::new_v4(),
            name,
            old_version,
            new_version,
            status: ReleaseStatus::Created,
            current_stage: ReleaseStage::Zero,
            strategy,
            metrics: ReleaseMetrics::default(),
            traffic_shifts,
            history: Vec::new(),
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            failure_reason: None,
        }
    }

    fn add_history_entry(
        &mut self,
        from_status: ReleaseStatus,
        to_status: ReleaseStatus,
        from_stage: Option<ReleaseStage>,
        to_stage: Option<ReleaseStage>,
        reason: impl Into<String>,
    ) {
        self.history.push(ReleaseHistoryEntry {
            timestamp: Utc::now(),
            from_status,
            to_status,
            from_stage,
            to_stage,
            reason: reason.into(),
        });
    }

    fn update_traffic_shifts(&mut self, new_percentage: f64) {
        let old_pct = 100.0 - new_percentage;
        self.traffic_shifts = vec![
            TrafficShift::new(self.old_version.clone(), old_pct),
            TrafficShift::new(self.new_version.clone(), new_percentage),
        ];
    }

    pub fn new_traffic_percentage(&self) -> f64 {
        self.traffic_shifts
            .iter()
            .find(|t| t.version == self.new_version)
            .map_or(0.0, |t| t.traffic_percentage)
    }

    pub fn old_traffic_percentage(&self) -> f64 {
        self.traffic_shifts
            .iter()
            .find(|t| t.version == self.old_version)
            .map_or(100.0, |t| t.traffic_percentage)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanaryError {
    InvalidStateTransition(String),
    ReleaseNotFound(String),
    ReleaseAlreadyTerminal(String),
    NoMoreStages(String),
    InvalidStage(String),
}

impl std::fmt::Display for CanaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStateTransition(msg) => write!(f, "Invalid state transition: {msg}"),
            Self::ReleaseNotFound(id) => write!(f, "Release not found: {id}"),
            Self::ReleaseAlreadyTerminal(id) => write!(f, "Release is already in terminal state: {id}"),
            Self::NoMoreStages(msg) => write!(f, "No more stages: {msg}"),
            Self::InvalidStage(msg) => write!(f, "Invalid stage: {msg}"),
        }
    }
}

impl std::error::Error for CanaryError {}

pub struct CanaryManager {
    releases: HashMap<Uuid, CanaryRelease>,
}

impl CanaryManager {
    pub fn new() -> Self {
        Self {
            releases: HashMap::new(),
        }
    }

    pub fn create_release(
        &mut self,
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
        strategy: CanaryStrategy,
    ) -> &CanaryRelease {
        let release = CanaryRelease::new(name, old_version, new_version, strategy);
        let id = release.id;
        self.releases.insert(id, release);
        self.releases.get(&id).unwrap()
    }

    pub fn start_release(&mut self, release_id: Uuid) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        let from_status = release.status.clone();
        if !from_status.can_transition_to(&ReleaseStatus::Progressing) {
            return Err(CanaryError::InvalidStateTransition(format!(
                "Cannot start release from status {}",
                from_status.label()
            )));
        }

        let from_stage = release.current_stage;
        let target_stage = ReleaseStage::Five;
        release.status = ReleaseStatus::Progressing;
        release.current_stage = target_stage;
        release.started_at = Some(Utc::now());
        release.updated_at = Utc::now();
        release.update_traffic_shifts(f64::from(target_stage.percentage()));
        release.add_history_entry(
            from_status,
            ReleaseStatus::Progressing,
            Some(from_stage),
            Some(target_stage),
            "Release started",
        );

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn advance_stage(&mut self, release_id: Uuid) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        if release.status.is_terminal() {
            return Err(CanaryError::ReleaseAlreadyTerminal(release_id.to_string()));
        }

        if release.status != ReleaseStatus::Progressing {
            return Err(CanaryError::InvalidStateTransition(format!(
                "Cannot advance stage when status is {}",
                release.status.label()
            )));
        }

        let from_stage = release.current_stage;
        let target_stage = from_stage
            .next()
            .ok_or_else(|| CanaryError::NoMoreStages("Already at 100%".to_string()))?;

        release.current_stage = target_stage;
        release.updated_at = Utc::now();
        release.update_traffic_shifts(f64::from(target_stage.percentage()));
        release.add_history_entry(
            release.status.clone(),
            release.status.clone(),
            Some(from_stage),
            Some(target_stage),
            "Stage advanced",
        );

        if target_stage == ReleaseStage::Hundred {
            let from_status = release.status.clone();
            release.status = ReleaseStatus::Promoted;
            release.completed_at = Some(Utc::now());
            release.updated_at = Utc::now();
            release.add_history_entry(
                from_status,
                ReleaseStatus::Promoted,
                None,
                None,
                "Release fully promoted",
            );
        }

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn pause_release(&mut self, release_id: Uuid) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        let from_status = release.status.clone();
        if !from_status.can_transition_to(&ReleaseStatus::Paused) {
            return Err(CanaryError::InvalidStateTransition(format!(
                "Cannot pause release from status {}",
                from_status.label()
            )));
        }

        release.status = ReleaseStatus::Paused;
        release.updated_at = Utc::now();
        release.add_history_entry(
            from_status,
            ReleaseStatus::Paused,
            None,
            None,
            "Release paused",
        );

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn resume_release(&mut self, release_id: Uuid) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        let from_status = release.status.clone();
        if !from_status.can_transition_to(&ReleaseStatus::Progressing) {
            return Err(CanaryError::InvalidStateTransition(format!(
                "Cannot resume release from status {}",
                from_status.label()
            )));
        }

        release.status = ReleaseStatus::Progressing;
        release.updated_at = Utc::now();
        release.add_history_entry(
            from_status,
            ReleaseStatus::Progressing,
            None,
            None,
            "Release resumed",
        );

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn promote_release(&mut self, release_id: Uuid) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        let from_status = release.status.clone();
        if !from_status.can_transition_to(&ReleaseStatus::Promoted) {
            return Err(CanaryError::InvalidStateTransition(format!(
                "Cannot promote release from status {}",
                from_status.label()
            )));
        }

        let from_stage = release.current_stage;
        release.status = ReleaseStatus::Promoted;
        release.current_stage = ReleaseStage::Hundred;
        release.completed_at = Some(Utc::now());
        release.updated_at = Utc::now();
        release.update_traffic_shifts(100.0);
        release.add_history_entry(
            from_status,
            ReleaseStatus::Promoted,
            Some(from_stage),
            Some(ReleaseStage::Hundred),
            "Release manually promoted",
        );

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn rollback_release(&mut self, release_id: Uuid, reason: impl Into<String>) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        let from_status = release.status.clone();
        if !from_status.can_transition_to(&ReleaseStatus::RolledBack) {
            return Err(CanaryError::InvalidStateTransition(format!(
                "Cannot rollback release from status {}",
                from_status.label()
            )));
        }

        let from_stage = release.current_stage;
        release.status = ReleaseStatus::RolledBack;
        release.current_stage = ReleaseStage::Zero;
        release.completed_at = Some(Utc::now());
        release.updated_at = Utc::now();
        release.update_traffic_shifts(0.0);
        release.add_history_entry(
            from_status,
            ReleaseStatus::RolledBack,
            Some(from_stage),
            Some(ReleaseStage::Zero),
            reason,
        );

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn update_metrics(
        &mut self,
        release_id: Uuid,
        metrics: ReleaseMetrics,
    ) -> Result<&CanaryRelease, CanaryError> {
        let release = self
            .releases
            .get_mut(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        release.metrics = metrics;
        release.updated_at = Utc::now();

        Ok(self.releases.get(&release_id).unwrap())
    }

    pub fn auto_evaluate(&mut self, release_id: Uuid) -> Result<AutoEvaluationResult, CanaryError> {
        let release = self
            .releases
            .get(&release_id)
            .ok_or_else(|| CanaryError::ReleaseNotFound(release_id.to_string()))?;

        if release.status != ReleaseStatus::Progressing {
            return Ok(AutoEvaluationResult::NoAction);
        }

        let strategy = release.strategy.clone();
        let metrics = release.metrics.clone();

        match strategy {
            CanaryStrategy::MetricBasedAuto {
                min_traffic_samples,
                ..
            } => {
                if metrics.traffic_count < min_traffic_samples {
                    return Ok(AutoEvaluationResult::NoAction);
                }
                if metrics.is_healthy(&strategy) {
                    self.advance_stage(release_id)?;
                    Ok(AutoEvaluationResult::Advanced)
                } else {
                    self.rollback_release(release_id, "Metrics exceeded thresholds")?;
                    Ok(AutoEvaluationResult::RolledBack)
                }
            }
            _ => Ok(AutoEvaluationResult::NoAction),
        }
    }

    pub fn get_release(&self, release_id: Uuid) -> Option<&CanaryRelease> {
        self.releases.get(&release_id)
    }

    pub fn list_releases(&self) -> Vec<&CanaryRelease> {
        self.releases.values().collect()
    }

    pub fn active_releases(&self) -> Vec<&CanaryRelease> {
        self.releases
            .values()
            .filter(|r| !r.status.is_terminal())
            .collect()
    }
}

impl Default for CanaryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoEvaluationResult {
    Advanced,
    RolledBack,
    NoAction,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_strategy() -> CanaryStrategy {
        CanaryStrategy::MetricBasedAuto {
            error_rate_threshold: 2.0,
            latency_threshold_ms: 500.0,
            success_rate_threshold: 98.0,
            min_traffic_samples: 100,
        }
    }

    fn create_healthy_metrics() -> ReleaseMetrics {
        ReleaseMetrics::new(0.5, 100.0, 500, 99.5)
    }

    fn create_unhealthy_metrics() -> ReleaseMetrics {
        ReleaseMetrics::new(5.0, 800.0, 500, 95.0)
    }

    #[test]
    fn test_create_release() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release(
            "test-release",
            "v1.0.0",
            "v2.0.0",
            create_test_strategy(),
        );

        assert_eq!(release.name, "test-release");
        assert_eq!(release.old_version, "v1.0.0");
        assert_eq!(release.new_version, "v2.0.0");
        assert_eq!(release.status, ReleaseStatus::Created);
        assert_eq!(release.current_stage, ReleaseStage::Zero);
        assert_eq!(release.new_traffic_percentage(), 0.0);
        assert_eq!(release.old_traffic_percentage(), 100.0);
    }

    #[test]
    fn test_start_release() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        let result = manager.start_release(id);
        assert!(result.is_ok());

        let release = result.unwrap();
        assert_eq!(release.status, ReleaseStatus::Progressing);
        assert_eq!(release.current_stage, ReleaseStage::Five);
        assert_eq!(release.new_traffic_percentage(), 5.0);
        assert!(release.started_at.is_some());
        assert!(!release.history.is_empty());
    }

    #[test]
    fn test_start_release_invalid_state() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.promote_release(id).unwrap();

        let result = manager.start_release(id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CanaryError::InvalidStateTransition(_)
        ));
    }

    #[test]
    fn test_advance_stage_step_by_step() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();

        let release = manager.advance_stage(id).unwrap();
        assert_eq!(release.current_stage, ReleaseStage::Twenty);
        assert_eq!(release.new_traffic_percentage(), 20.0);

        let release = manager.advance_stage(id).unwrap();
        assert_eq!(release.current_stage, ReleaseStage::Fifty);
        assert_eq!(release.new_traffic_percentage(), 50.0);

        let release = manager.advance_stage(id).unwrap();
        assert_eq!(release.current_stage, ReleaseStage::Hundred);
        assert_eq!(release.new_traffic_percentage(), 100.0);
        assert_eq!(release.status, ReleaseStatus::Promoted);
    }

    #[test]
    fn test_advance_stage_at_max() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.advance_stage(id).unwrap();
        manager.advance_stage(id).unwrap();
        manager.advance_stage(id).unwrap();

        let result = manager.advance_stage(id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CanaryError::ReleaseAlreadyTerminal(_)));
    }

    #[test]
    fn test_pause_and_resume() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();

        let release = manager.pause_release(id).unwrap();
        assert_eq!(release.status, ReleaseStatus::Paused);

        let release = manager.resume_release(id).unwrap();
        assert_eq!(release.status, ReleaseStatus::Progressing);
    }

    #[test]
    fn test_pause_from_created_fails() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        let result = manager.pause_release(id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CanaryError::InvalidStateTransition(_)
        ));
    }

    #[test]
    fn test_promote_release() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        let release = manager.promote_release(id).unwrap();

        assert_eq!(release.status, ReleaseStatus::Promoted);
        assert_eq!(release.current_stage, ReleaseStage::Hundred);
        assert_eq!(release.new_traffic_percentage(), 100.0);
        assert!(release.completed_at.is_some());
    }

    #[test]
    fn test_rollback_release() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.advance_stage(id).unwrap();

        let release = manager.rollback_release(id, "high error rate").unwrap();

        assert_eq!(release.status, ReleaseStatus::RolledBack);
        assert_eq!(release.current_stage, ReleaseStage::Zero);
        assert_eq!(release.new_traffic_percentage(), 0.0);
        assert!(release.completed_at.is_some());
    }

    #[test]
    fn test_rollback_from_paused() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.pause_release(id).unwrap();

        let release = manager.rollback_release(id, "manual rollback").unwrap();
        assert_eq!(release.status, ReleaseStatus::RolledBack);
    }

    #[test]
    fn test_auto_advance_healthy_metrics() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.update_metrics(id, create_healthy_metrics()).unwrap();

        let result = manager.auto_evaluate(id).unwrap();
        assert_eq!(result, AutoEvaluationResult::Advanced);

        let release = manager.get_release(id).unwrap();
        assert_eq!(release.current_stage, ReleaseStage::Twenty);
    }

    #[test]
    fn test_auto_rollback_unhealthy_metrics() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.update_metrics(id, create_unhealthy_metrics()).unwrap();

        let result = manager.auto_evaluate(id).unwrap();
        assert_eq!(result, AutoEvaluationResult::RolledBack);

        let release = manager.get_release(id).unwrap();
        assert_eq!(release.status, ReleaseStatus::RolledBack);
    }

    #[test]
    fn test_auto_evaluate_insufficient_samples() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();

        let mut metrics = create_unhealthy_metrics();
        metrics.traffic_count = 10;
        manager.update_metrics(id, metrics).unwrap();

        let result = manager.auto_evaluate(id).unwrap();
        assert_eq!(result, AutoEvaluationResult::NoAction);
    }

    #[test]
    fn test_time_based_strategy_no_auto() {
        let mut manager = CanaryManager::new();
        let strategy = CanaryStrategy::TimeBasedProgressive {
            interval_seconds: 60,
        };
        let release = manager.create_release("test", "v1", "v2", strategy);
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.update_metrics(id, create_healthy_metrics()).unwrap();

        let result = manager.auto_evaluate(id).unwrap();
        assert_eq!(result, AutoEvaluationResult::NoAction);
    }

    #[test]
    fn test_multi_version_coexistence() {
        let mut manager = CanaryManager::new();

        let r1 = manager.create_release("release-a", "v1.0.0", "v1.1.0", create_test_strategy());
        let id1 = r1.id;
        let r2 = manager.create_release("release-b", "v2.0.0", "v2.1.0", create_test_strategy());
        let id2 = r2.id;

        manager.start_release(id1).unwrap();
        manager.start_release(id2).unwrap();

        manager.advance_stage(id1).unwrap();

        let release1 = manager.get_release(id1).unwrap();
        let release2 = manager.get_release(id2).unwrap();

        assert_eq!(release1.current_stage, ReleaseStage::Twenty);
        assert_eq!(release2.current_stage, ReleaseStage::Five);
        assert_eq!(release1.new_version, "v1.1.0");
        assert_eq!(release2.new_version, "v2.1.0");

        assert_eq!(manager.active_releases().len(), 2);
        assert_eq!(manager.list_releases().len(), 2);
    }

    #[test]
    fn test_release_history() {
        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", create_test_strategy());
        let id = release.id;

        manager.start_release(id).unwrap();
        manager.advance_stage(id).unwrap();
        manager.pause_release(id).unwrap();
        manager.resume_release(id).unwrap();
        manager.rollback_release(id, "test rollback").unwrap();

        let release = manager.get_release(id).unwrap();
        assert!(release.history.len() >= 5);

        let statuses: Vec<&ReleaseStatus> = release.history.iter().map(|h| &h.to_status).collect();
        assert!(statuses.contains(&&ReleaseStatus::Progressing));
        assert!(statuses.contains(&&ReleaseStatus::Paused));
        assert!(statuses.contains(&&ReleaseStatus::RolledBack));
    }

    #[test]
    fn test_release_stage_percentages() {
        assert_eq!(ReleaseStage::Zero.percentage(), 0);
        assert_eq!(ReleaseStage::Five.percentage(), 5);
        assert_eq!(ReleaseStage::Twenty.percentage(), 20);
        assert_eq!(ReleaseStage::Fifty.percentage(), 50);
        assert_eq!(ReleaseStage::Hundred.percentage(), 100);
    }

    #[test]
    fn test_release_stage_navigation() {
        assert_eq!(ReleaseStage::Zero.next(), Some(ReleaseStage::Five));
        assert_eq!(ReleaseStage::Five.next(), Some(ReleaseStage::Twenty));
        assert_eq!(ReleaseStage::Twenty.next(), Some(ReleaseStage::Fifty));
        assert_eq!(ReleaseStage::Fifty.next(), Some(ReleaseStage::Hundred));
        assert_eq!(ReleaseStage::Hundred.next(), None);

        assert_eq!(ReleaseStage::Zero.prev(), None);
        assert_eq!(ReleaseStage::Five.prev(), Some(ReleaseStage::Zero));
        assert_eq!(ReleaseStage::Twenty.prev(), Some(ReleaseStage::Five));
        assert_eq!(ReleaseStage::Fifty.prev(), Some(ReleaseStage::Twenty));
        assert_eq!(ReleaseStage::Hundred.prev(), Some(ReleaseStage::Fifty));
    }

    #[test]
    fn test_release_status_transitions() {
        assert!(ReleaseStatus::Created.can_transition_to(&ReleaseStatus::Progressing));
        assert!(!ReleaseStatus::Created.can_transition_to(&ReleaseStatus::Paused));
        assert!(!ReleaseStatus::Created.can_transition_to(&ReleaseStatus::Promoted));

        assert!(ReleaseStatus::Progressing.can_transition_to(&ReleaseStatus::Paused));
        assert!(ReleaseStatus::Progressing.can_transition_to(&ReleaseStatus::Promoted));
        assert!(ReleaseStatus::Progressing.can_transition_to(&ReleaseStatus::RolledBack));

        assert!(ReleaseStatus::Paused.can_transition_to(&ReleaseStatus::Progressing));
        assert!(ReleaseStatus::Paused.can_transition_to(&ReleaseStatus::RolledBack));
        assert!(!ReleaseStatus::Paused.can_transition_to(&ReleaseStatus::Promoted));

        assert!(ReleaseStatus::Promoted.is_terminal());
        assert!(ReleaseStatus::RolledBack.is_terminal());
        assert!(ReleaseStatus::Failed.is_terminal());
        assert!(!ReleaseStatus::Progressing.is_terminal());
    }

    #[test]
    fn test_user_group_based_strategy() {
        let strategy = CanaryStrategy::UserGroupBased {
            groups: vec!["beta-testers".to_string(), "internal".to_string()],
        };

        let mut manager = CanaryManager::new();
        let release = manager.create_release("test", "v1", "v2", strategy.clone());
        let id = release.id;

        manager.start_release(id).unwrap();
        let release = manager.get_release(id).unwrap();

        assert_eq!(release.strategy, strategy);
        assert_eq!(release.current_stage, ReleaseStage::Five);
    }

    #[test]
    fn test_release_not_found() {
        let mut manager = CanaryManager::new();
        let fake_id = Uuid::new_v4();

        let result = manager.start_release(fake_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CanaryError::ReleaseNotFound(_)));
    }

    #[test]
    fn test_traffic_shift_creation() {
        let shift = TrafficShift::new("v2.0.0", 25.0);
        assert_eq!(shift.version, "v2.0.0");
        assert_eq!(shift.traffic_percentage, 25.0);
        assert_eq!(shift.weight, 2500);
    }

    #[test]
    fn test_release_stage_ordering() {
        assert!(ReleaseStage::Zero < ReleaseStage::Five);
        assert!(ReleaseStage::Five < ReleaseStage::Twenty);
        assert!(ReleaseStage::Twenty < ReleaseStage::Fifty);
        assert!(ReleaseStage::Fifty < ReleaseStage::Hundred);
    }

    #[test]
    fn test_canary_error_display() {
        let err = CanaryError::ReleaseNotFound("test-id".to_string());
        assert!(err.to_string().contains("test-id"));

        let err = CanaryError::InvalidStateTransition("bad transition".to_string());
        assert!(err.to_string().contains("bad transition"));
    }
}
