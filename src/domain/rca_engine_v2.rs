use crate::domain::dependency_graph::{DependencyGraph, EdgeType, NodeId, NodeType};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SignalId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SignalType {
    Metric,
    Log,
    Trace,
    Event,
    Alert,
}

impl SignalType {
    pub fn label(&self) -> &str {
        match self {
            SignalType::Metric => "Metric",
            SignalType::Log => "Log",
            SignalType::Trace => "Trace",
            SignalType::Event => "Event",
            SignalType::Alert => "Alert",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            SignalType::Metric => 1.2,
            SignalType::Log => 1.0,
            SignalType::Trace => 1.5,
            SignalType::Event => 0.8,
            SignalType::Alert => 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum SignalSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl SignalSeverity {
    pub fn label(&self) -> &str {
        match self {
            SignalSeverity::Info => "INFO",
            SignalSeverity::Warning => "WARN",
            SignalSeverity::Error => "ERROR",
            SignalSeverity::Critical => "CRITICAL",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            SignalSeverity::Info => 1.0,
            SignalSeverity::Warning => 2.0,
            SignalSeverity::Error => 3.5,
            SignalSeverity::Critical => 5.0,
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            SignalSeverity::Info => 1,
            SignalSeverity::Warning => 2,
            SignalSeverity::Error => 3,
            SignalSeverity::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RCASignal {
    pub id: SignalId,
    pub signal_type: SignalType,
    pub source: String,
    pub component: String,
    pub message: String,
    pub severity: SignalSeverity,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl RCASignal {
    pub fn new(
        id: impl Into<String>,
        signal_type: SignalType,
        source: impl Into<String>,
        component: impl Into<String>,
        message: impl Into<String>,
        severity: SignalSeverity,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: SignalId(id.into()),
            signal_type,
            source: source.into(),
            component: component.into(),
            message: message.into(),
            severity,
            timestamp,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RootCauseType {
    NetworkIssue,
    DatabaseIssue,
    ServiceDown,
    ConfigurationError,
    ResourceExhaustion,
    DependencyFailure,
    CodeBug,
    PerformanceDegradation,
    SecurityBreach,
    DataInconsistency,
    DeploymentFailure,
    Unknown,
}

impl RootCauseType {
    pub fn label(&self) -> &str {
        match self {
            RootCauseType::NetworkIssue => "Network Issue",
            RootCauseType::DatabaseIssue => "Database Issue",
            RootCauseType::ServiceDown => "Service Down",
            RootCauseType::ConfigurationError => "Configuration Error",
            RootCauseType::ResourceExhaustion => "Resource Exhaustion",
            RootCauseType::DependencyFailure => "Dependency Failure",
            RootCauseType::CodeBug => "Code Bug",
            RootCauseType::PerformanceDegradation => "Performance Degradation",
            RootCauseType::SecurityBreach => "Security Breach",
            RootCauseType::DataInconsistency => "Data Inconsistency",
            RootCauseType::DeploymentFailure => "Deployment Failure",
            RootCauseType::Unknown => "Unknown",
        }
    }

    pub fn base_confidence(&self) -> f64 {
        match self {
            RootCauseType::ServiceDown => 0.9,
            RootCauseType::ResourceExhaustion => 0.8,
            RootCauseType::DatabaseIssue => 0.85,
            RootCauseType::NetworkIssue => 0.7,
            RootCauseType::ConfigurationError => 0.75,
            RootCauseType::DependencyFailure => 0.8,
            RootCauseType::CodeBug => 0.7,
            RootCauseType::PerformanceDegradation => 0.65,
            RootCauseType::SecurityBreach => 0.6,
            RootCauseType::DataInconsistency => 0.7,
            RootCauseType::DeploymentFailure => 0.85,
            RootCauseType::Unknown => 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalPath {
    pub path: Vec<CausalLink>,
    pub strength: f64,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub from: String,
    pub to: String,
    pub relationship: EdgeType,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactScope {
    pub affected_components: Vec<String>,
    pub blast_radius: usize,
    pub affected_services: Vec<String>,
    pub affected_databases: Vec<String>,
    pub affected_queues: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn label(&self) -> &str {
        match self {
            RiskLevel::Low => "Low",
            RiskLevel::Medium => "Medium",
            RiskLevel::High => "High",
            RiskLevel::Critical => "Critical",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score < 25.0 {
            RiskLevel::Low
        } else if score < 50.0 {
            RiskLevel::Medium
        } else if score < 75.0 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseCandidate {
    pub component: String,
    pub root_cause_type: RootCauseType,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub signal_count: usize,
    pub severity_score: f64,
    pub causal_paths: Vec<CausalPath>,
    pub impact_scope: Option<ImpactScope>,
    pub multi_dimension_score: MultiDimensionScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDimensionScore {
    pub signal_strength: f64,
    pub correlation_score: f64,
    pub dependency_strength: f64,
    pub impact_score: f64,
    pub temporal_score: f64,
    pub overall: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RCAResultV2 {
    pub root_cause: Option<RootCauseCandidate>,
    pub candidates: Vec<RootCauseCandidate>,
    pub signals_analyzed: usize,
    pub signals_within_window: usize,
    pub analysis_time_ms: u64,
    pub analysis_timestamp: DateTime<Utc>,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub correlated_signal_groups: Vec<Vec<SignalId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RCARule {
    pub name: String,
    pub description: String,
    pub pattern: Vec<String>,
    pub root_cause: RootCauseType,
    pub base_confidence: f64,
    pub weight: f64,
    pub applicable_signal_types: Vec<SignalType>,
    pub category: String,
}

impl RCARule {
    pub fn match_signal(&self, signal: &RCASignal) -> f64 {
        if !self.applicable_signal_types.is_empty()
            && !self.applicable_signal_types.contains(&signal.signal_type)
        {
            return 0.0;
        }

        let text = format!(
            "{} {} {} {} {}",
            signal.message.to_lowercase(),
            signal.component.to_lowercase(),
            signal.source.to_lowercase(),
            signal.signal_type.label().to_lowercase(),
            signal
                .metadata
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        );

        let mut matches = 0;
        for pattern in &self.pattern {
            if text.contains(&pattern.to_lowercase()) {
                matches += 1;
            }
        }

        if matches == 0 {
            return 0.0;
        }

        let ratio = matches as f64 / self.pattern.len() as f64;
        ratio * self.base_confidence * signal.severity.weight() * signal.signal_type.weight()
    }
}

pub struct RCAEngineV2 {
    rules: Vec<RCARule>,
    time_window_minutes: i64,
}

impl RCAEngineV2 {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            time_window_minutes: 30,
        };
        engine.load_builtin_rules();
        engine
    }

    pub fn with_time_window(minutes: i64) -> Self {
        let mut engine = Self::new();
        engine.time_window_minutes = minutes;
        engine
    }

    pub fn rules(&self) -> &[RCARule] {
        &self.rules
    }

    pub fn add_rule(&mut self, rule: RCARule) {
        self.rules.push(rule);
    }

    fn load_builtin_rules(&mut self) {
        self.rules.push(RCARule {
            name: "database_connection_errors".to_string(),
            description: "Database connection refused, timeout, or connection pool exhaustion".to_string(),
            pattern: vec![
                "connection refused".to_string(),
                "timeout".to_string(),
                "db".to_string(),
            ],
            root_cause: RootCauseType::DatabaseIssue,
            base_confidence: 0.85,
            weight: 3.0,
            applicable_signal_types: vec![SignalType::Log, SignalType::Metric, SignalType::Alert],
            category: "database".to_string(),
        });

        self.rules.push(RCARule {
            name: "service_timeout_cascade".to_string(),
            description: "Service timeout cascading through dependencies".to_string(),
            pattern: vec!["timeout".to_string(), "504".to_string(), "upstream".to_string()],
            root_cause: RootCauseType::DependencyFailure,
            base_confidence: 0.75,
            weight: 2.5,
            applicable_signal_types: vec![SignalType::Trace, SignalType::Log, SignalType::Metric],
            category: "dependency".to_string(),
        });

        self.rules.push(RCARule {
            name: "high_cpu_memory".to_string(),
            description: "High CPU or memory usage indicating resource exhaustion".to_string(),
            pattern: vec!["cpu".to_string(), "memory".to_string(), "high".to_string()],
            root_cause: RootCauseType::ResourceExhaustion,
            base_confidence: 0.7,
            weight: 2.0,
            applicable_signal_types: vec![SignalType::Metric, SignalType::Alert],
            category: "resource".to_string(),
        });

        self.rules.push(RCARule {
            name: "configuration_error".to_string(),
            description: "Invalid or missing configuration".to_string(),
            pattern: vec!["config".to_string(), "invalid".to_string(), "missing".to_string()],
            root_cause: RootCauseType::ConfigurationError,
            base_confidence: 0.8,
            weight: 2.5,
            applicable_signal_types: vec![SignalType::Log, SignalType::Event, SignalType::Alert],
            category: "configuration".to_string(),
        });

        self.rules.push(RCARule {
            name: "network_dns_error".to_string(),
            description: "Network connectivity or DNS resolution issues".to_string(),
            pattern: vec!["network".to_string(), "dns".to_string(), "connect".to_string()],
            root_cause: RootCauseType::NetworkIssue,
            base_confidence: 0.7,
            weight: 2.0,
            applicable_signal_types: vec![SignalType::Log, SignalType::Metric, SignalType::Trace],
            category: "network".to_string(),
        });

        self.rules.push(RCARule {
            name: "exception_stacktrace".to_string(),
            description: "Code exceptions and stack traces indicating bugs".to_string(),
            pattern: vec!["nullpointer".to_string(), "exception".to_string(), "stacktrace".to_string()],
            root_cause: RootCauseType::CodeBug,
            base_confidence: 0.75,
            weight: 2.5,
            applicable_signal_types: vec![SignalType::Log, SignalType::Trace],
            category: "code".to_string(),
        });

        self.rules.push(RCARule {
            name: "health_check_failure".to_string(),
            description: "Health check failures indicating service is down".to_string(),
            pattern: vec!["health".to_string(), "unhealthy".to_string(), "down".to_string()],
            root_cause: RootCauseType::ServiceDown,
            base_confidence: 0.9,
            weight: 3.5,
            applicable_signal_types: vec![SignalType::Metric, SignalType::Alert, SignalType::Event],
            category: "availability".to_string(),
        });

        self.rules.push(RCARule {
            name: "latency_degradation".to_string(),
            description: "Increased latency and performance degradation".to_string(),
            pattern: vec!["latency".to_string(), "slow".to_string(), "response time".to_string()],
            root_cause: RootCauseType::PerformanceDegradation,
            base_confidence: 0.65,
            weight: 2.0,
            applicable_signal_types: vec![SignalType::Metric, SignalType::Trace],
            category: "performance".to_string(),
        });

        self.rules.push(RCARule {
            name: "disk_space_exhaustion".to_string(),
            description: "Disk space or storage exhaustion".to_string(),
            pattern: vec!["disk".to_string(), "space".to_string(), "full".to_string()],
            root_cause: RootCauseType::ResourceExhaustion,
            base_confidence: 0.8,
            weight: 2.5,
            applicable_signal_types: vec![SignalType::Metric, SignalType::Alert],
            category: "resource".to_string(),
        });

        self.rules.push(RCARule {
            name: "data_inconsistency".to_string(),
            description: "Data inconsistency or corruption detected".to_string(),
            pattern: vec!["inconsistent".to_string(), "corrupt".to_string(), "mismatch".to_string()],
            root_cause: RootCauseType::DataInconsistency,
            base_confidence: 0.75,
            weight: 2.5,
            applicable_signal_types: vec![SignalType::Log, SignalType::Event, SignalType::Alert],
            category: "data".to_string(),
        });

        self.rules.push(RCARule {
            name: "deployment_failure".to_string(),
            description: "Deployment or rollout failure".to_string(),
            pattern: vec!["deploy".to_string(), "rollout".to_string(), "failed".to_string()],
            root_cause: RootCauseType::DeploymentFailure,
            base_confidence: 0.85,
            weight: 3.0,
            applicable_signal_types: vec![SignalType::Event, SignalType::Log, SignalType::Alert],
            category: "deployment".to_string(),
        });

        self.rules.push(RCARule {
            name: "security_anomaly".to_string(),
            description: "Security anomalies or potential breaches".to_string(),
            pattern: vec!["unauthorized".to_string(), "breach".to_string(), "security".to_string()],
            root_cause: RootCauseType::SecurityBreach,
            base_confidence: 0.65,
            weight: 3.0,
            applicable_signal_types: vec![SignalType::Log, SignalType::Alert, SignalType::Event],
            category: "security".to_string(),
        });
    }

    pub fn analyze(&self, signals: &[RCASignal]) -> RCAResultV2 {
        let start = std::time::Instant::now();

        let (window_start, window_end) = self.calculate_time_window(signals);
        let filtered_signals = self.filter_by_time_window(signals, window_start, window_end);

        let correlated_groups = self.analyze_signal_correlation(&filtered_signals);

        let mut component_scores: HashMap<String, f64> = HashMap::new();
        let mut component_evidence: HashMap<String, Vec<String>> = HashMap::new();
        let mut component_root_cause: HashMap<String, RootCauseType> = HashMap::new();
        let mut component_signals: HashMap<String, Vec<SignalId>> = HashMap::new();
        let mut component_severity: HashMap<String, f64> = HashMap::new();

        for signal in &filtered_signals {
            component_signals
                .entry(signal.component.clone())
                .or_default()
                .push(signal.id.clone());

            let sev_score = component_severity.entry(signal.component.clone()).or_insert(0.0);
            *sev_score += signal.severity.weight();

            for rule in &self.rules {
                let match_score = rule.match_signal(signal);
                if match_score > 0.0 {
                    let entry = component_scores
                        .entry(signal.component.clone())
                        .or_insert(0.0);
                    *entry += match_score * rule.weight;

                    let evidence = component_evidence
                        .entry(signal.component.clone())
                        .or_default();
                    evidence.push(format!(
                        "[{}] {} - {} (rule: {})",
                        signal.severity.label(),
                        signal.signal_type.label(),
                        signal.message,
                        rule.name
                    ));

                    let current_rc = component_root_cause
                        .entry(signal.component.clone())
                        .or_insert_with(|| rule.root_cause.clone());

                    if rule.base_confidence > current_rc.base_confidence() {
                        *current_rc = rule.root_cause.clone();
                    }
                }
            }
        }

        let max_possible = self.max_possible_score(filtered_signals.len());

        let mut candidates: Vec<RootCauseCandidate> = component_scores
            .into_iter()
            .map(|(component, raw_score)| {
                let confidence = (raw_score / max_possible).min(1.0);

                let signal_count = component_signals
                    .get(&component)
                    .map(|s| s.len())
                    .unwrap_or(0);

                let severity_score = *component_severity.get(&component).unwrap_or(&0.0);

                let multi_dim_score = self.calculate_multi_dimension_score(
                    &component,
                    raw_score,
                    signal_count,
                    severity_score,
                    &correlated_groups,
                    component_signals.get(&component).unwrap_or(&vec![]),
                    filtered_signals.len(),
                );

                let adjusted_confidence = (confidence * 0.6 + multi_dim_score.overall * 0.4).min(1.0);

                RootCauseCandidate {
                    component: component.clone(),
                    root_cause_type: component_root_cause
                        .get(&component)
                        .cloned()
                        .unwrap_or(RootCauseType::Unknown),
                    confidence: adjusted_confidence,
                    evidence: component_evidence.get(&component).cloned().unwrap_or_default(),
                    signal_count,
                    severity_score,
                    causal_paths: Vec::new(),
                    impact_scope: None,
                    multi_dimension_score: multi_dim_score,
                }
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let root_cause = candidates.first().cloned();

        RCAResultV2 {
            root_cause,
            candidates,
            signals_analyzed: signals.len(),
            signals_within_window: filtered_signals.len(),
            analysis_time_ms: start.elapsed().as_millis() as u64,
            analysis_timestamp: Utc::now(),
            time_window_start: window_start,
            time_window_end: window_end,
            correlated_signal_groups: correlated_groups,
        }
    }

    pub fn analyze_with_graph(
        &self,
        signals: &[RCASignal],
        graph: &DependencyGraph,
    ) -> RCAResultV2 {
        let mut result = self.analyze(signals);

        let mut updated_candidates = Vec::new();

        for candidate in &result.candidates {
            let causal_paths = self.trace_causal_paths(graph, &candidate.component);

            let impact_scope = self.calculate_impact_scope(graph, &candidate.component);

            let dep_strength = causal_paths
                .iter()
                .map(|p| p.strength)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            let impact_score = match &impact_scope {
                Some(scope) => match scope.risk_level {
                    RiskLevel::Low => 0.25,
                    RiskLevel::Medium => 0.5,
                    RiskLevel::High => 0.75,
                    RiskLevel::Critical => 1.0,
                },
                None => 0.0,
            };

            let mut updated = candidate.clone();
            updated.causal_paths = causal_paths;
            updated.impact_scope = impact_scope;

            updated.multi_dimension_score.dependency_strength = dep_strength;
            updated.multi_dimension_score.impact_score = impact_score;
            updated.multi_dimension_score.overall = self.combine_scores(&updated.multi_dimension_score);

            let base_confidence = candidate.confidence;
            let graph_bonus = (dep_strength * 0.15 + impact_score * 0.1).min(0.2);
            updated.confidence = (base_confidence + graph_bonus).min(1.0);

            updated_candidates.push(updated);
        }

        updated_candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result.root_cause = updated_candidates.first().cloned();
        result.candidates = updated_candidates;

        result
    }

    fn calculate_time_window(&self, signals: &[RCASignal]) -> (DateTime<Utc>, DateTime<Utc>) {
        if signals.is_empty() {
            let now = Utc::now();
            (now - Duration::minutes(self.time_window_minutes), now)
        } else {
            let max_time = signals.iter().map(|s| s.timestamp).max().unwrap();
            let min_time = max_time - Duration::minutes(self.time_window_minutes);
            (min_time, max_time)
        }
    }

    fn filter_by_time_window(
        &self,
        signals: &[RCASignal],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<RCASignal> {
        signals
            .iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect()
    }

    fn analyze_signal_correlation(&self, signals: &[RCASignal]) -> Vec<Vec<SignalId>> {
        if signals.len() < 2 {
            return Vec::new();
        }

        let mut groups: Vec<Vec<SignalId>> = Vec::new();
        let mut visited = HashSet::new();

        for i in 0..signals.len() {
            if visited.contains(&signals[i].id) {
                continue;
            }

            let mut group = vec![signals[i].id.clone()];
            visited.insert(signals[i].id.clone());

            for j in (i + 1)..signals.len() {
                if visited.contains(&signals[j].id) {
                    continue;
                }

                if self.are_signals_correlated(&signals[i], &signals[j]) {
                    group.push(signals[j].id.clone());
                    visited.insert(signals[j].id.clone());
                }
            }

            if group.len() > 1 {
                groups.push(group);
            }
        }

        groups
    }

    fn are_signals_correlated(&self, a: &RCASignal, b: &RCASignal) -> bool {
        if a.component == b.component {
            return true;
        }

        let time_diff = (a.timestamp - b.timestamp).num_seconds().abs();
        if time_diff < 300 {
            return true;
        }

        if a.signal_type == b.signal_type && a.severity == b.severity {
            return true;
        }

        false
    }

    fn max_possible_score(&self, signal_count: usize) -> f64 {
        let max_rule_weight = self.rules.iter().map(|r| r.weight).fold(0.0, f64::max);
        let max_severity = SignalSeverity::Critical.weight();
        let max_type_weight = SignalType::Alert.weight();
        signal_count as f64 * max_rule_weight * max_severity * max_type_weight
    }

    fn calculate_multi_dimension_score(
        &self,
        _component: &str,
        raw_score: f64,
        signal_count: usize,
        severity_score: f64,
        correlated_groups: &[Vec<SignalId>],
        component_signals: &[SignalId],
        total_signals: usize,
    ) -> MultiDimensionScore {
        let max_raw = self.max_possible_score(total_signals.max(1));
        let signal_strength = (raw_score / max_raw).min(1.0);

        let count_score = if total_signals > 0 {
            (signal_count as f64 / total_signals as f64).min(1.0)
        } else {
            0.0
        };

        let mut correlation_score: f64 = 0.0;
        for group in correlated_groups {
            let overlap = group
                .iter()
                .filter(|id| component_signals.contains(id))
                .count();
            if overlap > 0 {
                correlation_score = correlation_score.max(overlap as f64 / group.len() as f64);
            }
        }

        let max_sev = SignalSeverity::Critical.weight() * signal_count.max(1) as f64;
        let temporal_score = (severity_score / max_sev).min(1.0);

        let dependency_strength = 0.0;
        let impact_score = 0.0;

        let overall = (signal_strength * 0.35
            + count_score * 0.15
            + correlation_score * 0.2
            + temporal_score * 0.15
            + dependency_strength * 0.1
            + impact_score * 0.05)
            .min(1.0);

        MultiDimensionScore {
            signal_strength,
            correlation_score,
            dependency_strength,
            impact_score,
            temporal_score,
            overall,
        }
    }

    fn combine_scores(&self, score: &MultiDimensionScore) -> f64 {
        (score.signal_strength * 0.30
            + score.correlation_score * 0.20
            + score.dependency_strength * 0.20
            + score.impact_score * 0.15
            + score.temporal_score * 0.15)
            .min(1.0)
    }

    fn trace_causal_paths(&self, graph: &DependencyGraph, component: &str) -> Vec<CausalPath> {
        let start_id = NodeId(component.to_string());
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((start_id.clone(), Vec::<CausalLink>::new(), 1.0));
        visited.insert(start_id.clone());

        while let Some((current, current_path, current_strength)) = queue.pop_front() {
            let incoming = graph.get_incoming_edges(&current);

            for edge in incoming {
                if !visited.contains(&edge.from) {
                    visited.insert(edge.from.clone());

                    let link = CausalLink {
                        from: edge.from.0.clone(),
                        to: current.0.clone(),
                        relationship: edge.edge_type.clone(),
                        strength: edge.weight,
                    };

                    let mut new_path = current_path.clone();
                    new_path.push(link);

                    let new_strength = current_strength * edge.weight;

                    if new_path.len() <= 5 {
                        queue.push_back((edge.from.clone(), new_path.clone(), new_strength));

                        let path_length = new_path.len();
                        let path_strength = new_strength / path_length as f64;
                        paths.push(CausalPath {
                            path: new_path,
                            strength: path_strength,
                            length: path_length,
                        });
                    }
                }
            }
        }

        paths.sort_by(|a, b| {
            b.strength
                .partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        paths.into_iter().take(5).collect()
    }

    fn calculate_impact_scope(
        &self,
        graph: &DependencyGraph,
        component: &str,
    ) -> Option<ImpactScope> {
        let start_id = NodeId(component.to_string());

        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start_id.clone());
        affected.insert(start_id.clone());

        let mut services = Vec::new();
        let mut databases = Vec::new();
        let mut queues = Vec::new();

        if let Some(start_node) = graph.get_node(&start_id) {
            match start_node.node_type {
                NodeType::Service | NodeType::Cell => {
                    services.push(start_node.name.clone());
                }
                NodeType::Database => {
                    databases.push(start_node.name.clone());
                }
                NodeType::Queue => {
                    queues.push(start_node.name.clone());
                }
                _ => {}
            }
        }

        while let Some(current) = queue.pop_front() {
            let outgoing = graph.get_outgoing_edges(&current);

            for edge in outgoing {
                if !affected.contains(&edge.to) {
                    affected.insert(edge.to.clone());
                    queue.push_back(edge.to.clone());

                    if let Some(node) = graph.get_node(&edge.to) {
                        match node.node_type {
                            NodeType::Service | NodeType::Cell => {
                                services.push(node.name.clone());
                            }
                            NodeType::Database => {
                                databases.push(node.name.clone());
                            }
                            NodeType::Queue => {
                                queues.push(node.name.clone());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if affected.len() <= 1 {
            return None;
        }

        services.sort();
        services.dedup();
        databases.sort();
        databases.dedup();
        queues.sort();
        queues.dedup();

        let blast_radius = affected.len() - 1;

        let risk_score = (blast_radius as f64 * 15.0
            + services.len() as f64 * 10.0
            + databases.len() as f64 * 20.0
            + queues.len() as f64 * 8.0)
            .min(100.0);

        Some(ImpactScope {
            affected_components: affected.iter().map(|n| n.0.clone()).collect(),
            blast_radius,
            affected_services: services,
            affected_databases: databases,
            affected_queues: queues,
            risk_level: RiskLevel::from_score(risk_score),
        })
    }
}

impl Default for RCAEngineV2 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dependency_graph::{DependencyEdge, DependencyNode};

    fn base_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn create_test_signals() -> Vec<RCASignal> {
        let base = base_time();
        vec![
            RCASignal::new(
                "s1",
                SignalType::Alert,
                "k8s",
                "user-service",
                "health check failed, service is down",
                SignalSeverity::Critical,
                base + Duration::seconds(0),
            ),
            RCASignal::new(
                "s2",
                SignalType::Log,
                "app",
                "user-service",
                "database connection refused timeout error",
                SignalSeverity::Error,
                base + Duration::seconds(5),
            ),
            RCASignal::new(
                "s3",
                SignalType::Metric,
                "prometheus",
                "order-service",
                "high cpu and memory usage detected",
                SignalSeverity::Warning,
                base + Duration::seconds(10),
            ),
            RCASignal::new(
                "s4",
                SignalType::Trace,
                "jaeger",
                "order-service",
                "timeout 504 from upstream payment service",
                SignalSeverity::Error,
                base + Duration::seconds(15),
            ),
            RCASignal::new(
                "s5",
                SignalType::Log,
                "app",
                "payment-service",
                "nullpointer exception stacktrace in handler",
                SignalSeverity::Error,
                base + Duration::seconds(20),
            ),
        ]
    }

    fn create_test_graph() -> DependencyGraph {
        let mut graph = DependencyGraph::new();

        graph.add_node(DependencyNode::new(
            "api-gateway",
            "API Gateway",
            NodeType::Service,
        ));
        graph.add_node(DependencyNode::new(
            "user-service",
            "User Service",
            NodeType::Service,
        ));
        graph.add_node(DependencyNode::new(
            "order-service",
            "Order Service",
            NodeType::Service,
        ));
        graph.add_node(DependencyNode::new(
            "payment-service",
            "Payment Service",
            NodeType::Service,
        ));
        graph.add_node(DependencyNode::new("user-db", "User DB", NodeType::Database));
        graph.add_node(DependencyNode::new("order-db", "Order DB", NodeType::Database));
        graph.add_node(DependencyNode::new(
            "event-queue",
            "Event Queue",
            NodeType::Queue,
        ));

        graph.add_edge(DependencyEdge::new(
            NodeId("api-gateway".to_string()),
            NodeId("user-service".to_string()),
            EdgeType::Calls,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("api-gateway".to_string()),
            NodeId("order-service".to_string()),
            EdgeType::Calls,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("user-service".to_string()),
            NodeId("user-db".to_string()),
            EdgeType::ReadsFrom,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("order-service".to_string()),
            NodeId("order-db".to_string()),
            EdgeType::ReadsFrom,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("order-service".to_string()),
            NodeId("payment-service".to_string()),
            EdgeType::Calls,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("order-service".to_string()),
            NodeId("event-queue".to_string()),
            EdgeType::Publishes,
        ));
        graph.add_edge(DependencyEdge::new(
            NodeId("payment-service".to_string()),
            NodeId("event-queue".to_string()),
            EdgeType::Publishes,
        ));

        graph
    }

    #[test]
    fn test_rca_engine_v2_creation() {
        let engine = RCAEngineV2::new();
        assert!(engine.rules().len() >= 10);
    }

    #[test]
    fn test_signal_type_labels_and_weights() {
        assert_eq!(SignalType::Metric.label(), "Metric");
        assert_eq!(SignalType::Log.label(), "Log");
        assert_eq!(SignalType::Trace.label(), "Trace");
        assert_eq!(SignalType::Event.label(), "Event");
        assert_eq!(SignalType::Alert.label(), "Alert");

        assert!(SignalType::Alert.weight() > SignalType::Metric.weight());
        assert!(SignalType::Trace.weight() > SignalType::Log.weight());
    }

    #[test]
    fn test_signal_severity_ranking() {
        assert!(SignalSeverity::Critical.rank() > SignalSeverity::Error.rank());
        assert!(SignalSeverity::Error.rank() > SignalSeverity::Warning.rank());
        assert!(SignalSeverity::Warning.rank() > SignalSeverity::Info.rank());
        assert_eq!(SignalSeverity::Critical.weight(), 5.0);
        assert_eq!(SignalSeverity::Info.label(), "INFO");
    }

    #[test]
    fn test_rule_signal_matching() {
        let engine = RCAEngineV2::new();
        let rules = engine.rules();

        let signal = RCASignal::new(
            "test",
            SignalType::Log,
            "app",
            "db-service",
            "database connection refused timeout",
            SignalSeverity::Error,
            base_time(),
        );

        let matched: Vec<_> = rules
            .iter()
            .filter(|r| r.match_signal(&signal) > 0.0)
            .collect();

        assert!(!matched.is_empty());
        assert!(matched
            .iter()
            .any(|r| r.root_cause == RootCauseType::DatabaseIssue));
    }

    #[test]
    fn test_rule_no_match() {
        let engine = RCAEngineV2::new();
        let rules = engine.rules();

        let signal = RCASignal::new(
            "test",
            SignalType::Log,
            "app",
            "test-svc",
            "normal operation everything fine",
            SignalSeverity::Info,
            base_time(),
        );

        let matched: Vec<_> = rules
            .iter()
            .filter(|r| r.match_signal(&signal) > 0.0)
            .collect();

        assert!(matched.is_empty());
    }

    #[test]
    fn test_analyze_signals_basic() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        assert_eq!(result.signals_analyzed, 5);
        assert_eq!(result.signals_within_window, 5);
        assert!(!result.candidates.is_empty());
        assert!(result.root_cause.is_some());
    }

    #[test]
    fn test_analyze_no_signals() {
        let engine = RCAEngineV2::new();
        let result = engine.analyze(&[]);

        assert_eq!(result.signals_analyzed, 0);
        assert_eq!(result.signals_within_window, 0);
        assert!(result.root_cause.is_none());
        assert!(result.candidates.is_empty());
        assert!(result.correlated_signal_groups.is_empty());
    }

    #[test]
    fn test_candidates_sorted_by_confidence() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        for i in 1..result.candidates.len() {
            assert!(result.candidates[i - 1].confidence >= result.candidates[i].confidence);
        }
    }

    #[test]
    fn test_multi_dimension_score() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        let rc = result.root_cause.unwrap();
        assert!(rc.multi_dimension_score.signal_strength >= 0.0);
        assert!(rc.multi_dimension_score.signal_strength <= 1.0);
        assert!(rc.multi_dimension_score.overall >= 0.0);
        assert!(rc.multi_dimension_score.overall <= 1.0);
        assert!(rc.confidence >= 0.0);
        assert!(rc.confidence <= 1.0);
    }

    #[test]
    fn test_time_window_filtering() {
        let engine = RCAEngineV2::with_time_window(1);
        let base = base_time();

        let signals = vec![
            RCASignal::new(
                "s1",
                SignalType::Alert,
                "k8s",
                "user-service",
                "health check down",
                SignalSeverity::Critical,
                base,
            ),
            RCASignal::new(
                "s2",
                SignalType::Log,
                "app",
                "user-service",
                "database error timeout",
                SignalSeverity::Error,
                base + Duration::minutes(5),
            ),
        ];

        let result = engine.analyze(&signals);
        assert_eq!(result.signals_analyzed, 2);
        assert_eq!(result.signals_within_window, 1);
    }

    #[test]
    fn test_signal_correlation_analysis() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        assert!(!result.correlated_signal_groups.is_empty());
        assert!(result
            .correlated_signal_groups
            .iter()
            .all(|g| g.len() >= 2));
    }

    #[test]
    fn test_causal_path_tracing_with_graph() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let graph = create_test_graph();
        let result = engine.analyze_with_graph(&signals, &graph);

        let rc = result.root_cause.unwrap();
        assert!(!rc.causal_paths.is_empty());

        for path in &rc.causal_paths {
            assert!(!path.path.is_empty());
            assert!(path.strength > 0.0);
            assert!(path.length > 0);
        }
    }

    #[test]
    fn test_impact_scope_with_graph() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let graph = create_test_graph();
        let result = engine.analyze_with_graph(&signals, &graph);

        let rc = result.root_cause.unwrap();
        let scope = rc.impact_scope.unwrap();

        assert!(scope.blast_radius > 0);
        assert!(!scope.affected_components.is_empty());
        assert_ne!(scope.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_graph_analysis_enhances_confidence() {
        let engine = RCAEngineV2::new();
        let signals = create_test_signals();
        let graph = create_test_graph();

        let basic_result = engine.analyze(&signals);
        let graph_result = engine.analyze_with_graph(&signals, &graph);

        let basic_rc = basic_result.root_cause.unwrap();
        let graph_rc = graph_result.root_cause.unwrap();

        assert!(graph_rc.confidence >= basic_rc.confidence);
        assert!(graph_rc.multi_dimension_score.dependency_strength > 0.0);
        assert!(graph_rc.impact_scope.is_some());
    }

    #[test]
    fn test_severity_sorting_in_candidates() {
        let engine = RCAEngineV2::new();
        let base = base_time();

        let signals = vec![
            RCASignal::new(
                "s1",
                SignalType::Log,
                "app",
                "svc-a",
                "health check down and unhealthy",
                SignalSeverity::Info,
                base,
            ),
            RCASignal::new(
                "s2",
                SignalType::Alert,
                "k8s",
                "svc-b",
                "service is down health check failed critical",
                SignalSeverity::Critical,
                base + Duration::seconds(1),
            ),
        ];

        let result = engine.analyze(&signals);

        assert!(!result.candidates.is_empty());
        let top = &result.candidates[0];
        assert_eq!(top.component, "svc-b");
        assert!(top.severity_score > 0.0);
    }

    #[test]
    fn test_builtin_rules_count() {
        let engine = RCAEngineV2::new();
        let rules = engine.rules();

        assert!(rules.len() >= 12);

        let categories: HashSet<_> = rules.iter().map(|r| r.category.as_str()).collect();
        assert!(categories.contains("database"));
        assert!(categories.contains("network"));
        assert!(categories.contains("resource"));
        assert!(categories.contains("performance"));
        assert!(categories.contains("security"));
        assert!(categories.contains("deployment"));
    }

    #[test]
    fn test_root_cause_type_labels() {
        assert_eq!(RootCauseType::DatabaseIssue.label(), "Database Issue");
        assert_eq!(RootCauseType::ServiceDown.label(), "Service Down");
        assert_eq!(RootCauseType::DeploymentFailure.label(), "Deployment Failure");
        assert_eq!(RootCauseType::SecurityBreach.label(), "Security Breach");
        assert_eq!(RootCauseType::PerformanceDegradation.label(), "Performance Degradation");
        assert_eq!(RootCauseType::Unknown.label(), "Unknown");
    }

    #[test]
    fn test_add_custom_rule() {
        let mut engine = RCAEngineV2::new();
        let initial_count = engine.rules().len();

        engine.add_rule(RCARule {
            name: "custom-rule".to_string(),
            description: "Custom test rule".to_string(),
            pattern: vec!["custom_pattern".to_string()],
            root_cause: RootCauseType::Unknown,
            base_confidence: 0.5,
            weight: 1.0,
            applicable_signal_types: vec![],
            category: "custom".to_string(),
        });

        assert_eq!(engine.rules().len(), initial_count + 1);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(10.0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(30.0), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(60.0), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(90.0), RiskLevel::Critical);
        assert_eq!(RiskLevel::Critical.label(), "Critical");
    }

    #[test]
    fn test_signal_with_metadata() {
        let signal = RCASignal::new(
            "s1",
            SignalType::Metric,
            "prometheus",
            "test-svc",
            "high cpu usage",
            SignalSeverity::Warning,
            base_time(),
        )
        .with_metadata("region", "us-east")
        .with_metadata("instance", "i-12345");

        assert_eq!(signal.metadata.get("region").unwrap(), "us-east");
        assert_eq!(signal.metadata.get("instance").unwrap(), "i-12345");
    }
}
