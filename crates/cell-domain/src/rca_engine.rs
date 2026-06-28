use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SignalId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalType {
    MetricAnomaly,
    LogError,
    TraceError,
    HealthCheckFail,
    UserReported,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: SignalId,
    pub signal_type: SignalType,
    pub source: String,
    pub component: String,
    pub message: String,
    pub severity: SignalSeverity,
    pub timestamp: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
pub enum SignalSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RCACandidate {
    pub component: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub root_cause_type: RootCauseType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RootCauseType {
    NetworkIssue,
    DatabaseIssue,
    ServiceDown,
    ConfigurationError,
    ResourceExhaustion,
    DependencyFailure,
    CodeBug,
    Unknown,
}

impl RootCauseType {
    pub fn label(&self) -> &str {
        match self {
            Self::NetworkIssue => "Network Issue",
            Self::DatabaseIssue => "Database Issue",
            Self::ServiceDown => "Service Down",
            Self::ConfigurationError => "Configuration Error",
            Self::ResourceExhaustion => "Resource Exhaustion",
            Self::DependencyFailure => "Dependency Failure",
            Self::CodeBug => "Code Bug",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RCAResult {
    pub root_cause: Option<RCACandidate>,
    pub candidates: Vec<RCACandidate>,
    pub signals_analyzed: usize,
    pub components_affected: Vec<String>,
    pub analysis_time_ms: u64,
    pub analysis_timestamp: String,
}

pub struct RCAEngine {
    rules: Vec<RCARule>,
}

impl RCAEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
        };
        engine.load_builtin_rules();
        engine
    }

    fn load_builtin_rules(&mut self) {
        self.add_rule(RCARule {
            name: "database_connection_errors".to_string(),
            pattern: vec!["connection refused".to_string(), "timeout".to_string(), "db".to_string()],
            root_cause: RootCauseType::DatabaseIssue,
            confidence: 0.8,
            weight: 3.0,
        });

        self.add_rule(RCARule {
            name: "service_timeout_cascade".to_string(),
            pattern: vec!["timeout".to_string(), "504".to_string(), "upstream".to_string()],
            root_cause: RootCauseType::DependencyFailure,
            confidence: 0.7,
            weight: 2.0,
        });

        self.add_rule(RCARule {
            name: "high_cpu_memory".to_string(),
            pattern: vec!["cpu".to_string(), "memory".to_string(), "high".to_string()],
            root_cause: RootCauseType::ResourceExhaustion,
            confidence: 0.6,
            weight: 2.0,
        });

        self.add_rule(RCARule {
            name: "configuration_error".to_string(),
            pattern: vec!["config".to_string(), "invalid".to_string(), "missing".to_string()],
            root_cause: RootCauseType::ConfigurationError,
            confidence: 0.75,
            weight: 2.5,
        });

        self.add_rule(RCARule {
            name: "network_error".to_string(),
            pattern: vec!["network".to_string(), "dns".to_string(), "connect".to_string()],
            root_cause: RootCauseType::NetworkIssue,
            confidence: 0.65,
            weight: 2.0,
        });

        self.add_rule(RCARule {
            name: "exception_stacktrace".to_string(),
            pattern: vec!["nullpointer".to_string(), "exception".to_string(), "stacktrace".to_string()],
            root_cause: RootCauseType::CodeBug,
            confidence: 0.7,
            weight: 2.5,
        });

        self.add_rule(RCARule {
            name: "health_check_failure".to_string(),
            pattern: vec!["health".to_string(), "unhealthy".to_string(), "down".to_string()],
            root_cause: RootCauseType::ServiceDown,
            confidence: 0.85,
            weight: 3.0,
        });
    }

    pub fn add_rule(&mut self, rule: RCARule) {
        self.rules.push(rule);
    }

    pub fn analyze(&self, signals: &[Signal]) -> RCAResult {
        let start = std::time::Instant::now();

        let mut component_scores: HashMap<String, f64> = HashMap::new();
        let mut component_evidence: HashMap<String, Vec<String>> = HashMap::new();
        let mut component_root_cause: HashMap<String, RootCauseType> = HashMap::new();

        let mut affected_components: HashSet<String> = HashSet::new();

        for signal in signals {
            affected_components.insert(signal.component.clone());

            for rule in &self.rules {
                let match_score = rule.match_signal(signal);
                if match_score > 0.0 {
                    let entry = component_scores.entry(signal.component.clone()).or_insert(0.0);
                    *entry = match_score.mul_add(rule.weight, *entry);

                    let evidence = component_evidence.entry(signal.component.clone()).or_default();
                    evidence.push(format!(
                        "[{}] {} - {} (rule: {})",
                        signal.severity.label(),
                        signal.signal_type.label(),
                        signal.message,
                        rule.name
                    ));

                    let current_rc = component_root_cause.entry(signal.component.clone())
                        .or_insert_with(|| rule.root_cause.clone());

                    if rule.confidence > self.confidence_for_type(current_rc) {
                        *current_rc = rule.root_cause.clone();
                    }
                }
            }
        }

        let mut candidates: Vec<RCACandidate> = component_scores
            .into_iter()
            .map(|(component, score)| {
                let max_score = self.max_possible_score(signals.len());
                let confidence = (score / max_score).min(1.0);

                RCACandidate {
                    component: component.clone(),
                    confidence,
                    evidence: component_evidence.get(&component).cloned().unwrap_or_default(),
                    root_cause_type: component_root_cause.get(&component)
                        .cloned()
                        .unwrap_or(RootCauseType::Unknown),
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        let root_cause = candidates.first().cloned();

        RCAResult {
            root_cause,
            candidates,
            signals_analyzed: signals.len(),
            components_affected: affected_components.into_iter().collect(),
            analysis_time_ms: start.elapsed().as_millis() as u64,
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn max_possible_score(&self, signal_count: usize) -> f64 {
        let max_rule_weight = self.rules.iter().map(|r| r.weight).fold(0.0, f64::max);
        signal_count as f64 * max_rule_weight
    }

    fn confidence_for_type(&self, _rc_type: &RootCauseType) -> f64 {
        0.5
    }

    pub fn trace_causality_chain(
        &self,
        graph: &crate::dependency_graph::DependencyGraph,
        failing_component: &str,
    ) -> Vec<CausalLink> {
        use crate::dependency_graph::NodeId;

        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(NodeId(failing_component.to_string()));
        visited.insert(NodeId(failing_component.to_string()));

        while let Some(current) = queue.pop_front() {
            let incoming = graph.get_incoming_edges(&current);

            for edge in incoming {
                if !visited.contains(&edge.from) {
                    visited.insert(edge.from.clone());
                    queue.push_back(edge.from.clone());

                    chain.push(CausalLink {
                        from: edge.from.0.clone(),
                        to: current.0.clone(),
                        relationship: edge.edge_type.clone(),
                        strength: edge.weight,
                    });
                }
            }
        }

        chain
    }
}

impl Default for RCAEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalType {
    pub fn label(&self) -> &str {
        match self {
            Self::MetricAnomaly => "Metric Anomaly",
            Self::LogError => "Log Error",
            Self::TraceError => "Trace Error",
            Self::HealthCheckFail => "Health Check Fail",
            Self::UserReported => "User Reported",
            Self::Alert => "Alert",
        }
    }
}

impl SignalSeverity {
    pub fn label(&self) -> &str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Self::Info => 1.0,
            Self::Warning => 2.0,
            Self::Error => 3.0,
            Self::Critical => 4.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RCARule {
    pub name: String,
    pub pattern: Vec<String>,
    pub root_cause: RootCauseType,
    pub confidence: f64,
    pub weight: f64,
}

impl RCARule {
    pub fn match_signal(&self, signal: &Signal) -> f64 {
        let text = format!(
            "{} {} {} {}",
            signal.message.to_lowercase(),
            signal.component.to_lowercase(),
            signal.source.to_lowercase(),
            signal.signal_type.label().to_lowercase()
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

        let ratio = f64::from(matches) / self.pattern.len() as f64;
        ratio * self.confidence * signal.severity.weight()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub from: String,
    pub to: String,
    pub relationship: crate::dependency_graph::EdgeType,
    pub strength: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_signals() -> Vec<Signal> {
        vec![
            Signal {
                id: SignalId("s1".to_string()),
                signal_type: SignalType::HealthCheckFail,
                source: "k8s".to_string(),
                component: "user-service".to_string(),
                message: "health check failed, service is down".to_string(),
                severity: SignalSeverity::Critical,
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                metadata: HashMap::new(),
            },
            Signal {
                id: SignalId("s2".to_string()),
                signal_type: SignalType::LogError,
                source: "app".to_string(),
                component: "user-service".to_string(),
                message: "database connection refused timeout".to_string(),
                severity: SignalSeverity::Error,
                timestamp: "2024-01-01T00:00:01Z".to_string(),
                metadata: HashMap::new(),
            },
            Signal {
                id: SignalId("s3".to_string()),
                signal_type: SignalType::MetricAnomaly,
                source: "prometheus".to_string(),
                component: "order-service".to_string(),
                message: "high cpu and memory usage detected".to_string(),
                severity: SignalSeverity::Warning,
                timestamp: "2024-01-01T00:00:02Z".to_string(),
                metadata: HashMap::new(),
            },
        ]
    }

    #[test]
    fn test_rca_engine_creation() {
        let engine = RCAEngine::new();
        assert!(!engine.rules.is_empty());
    }

    #[test]
    fn test_analyze_signals() {
        let engine = RCAEngine::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        assert_eq!(result.signals_analyzed, 3);
        assert!(!result.candidates.is_empty());
    }

    #[test]
    fn test_analyze_no_signals() {
        let engine = RCAEngine::new();
        let result = engine.analyze(&[]);

        assert_eq!(result.signals_analyzed, 0);
        assert!(result.root_cause.is_none());
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn test_signal_severity_weight() {
        assert_eq!(SignalSeverity::Info.weight(), 1.0);
        assert_eq!(SignalSeverity::Warning.weight(), 2.0);
        assert_eq!(SignalSeverity::Error.weight(), 3.0);
        assert_eq!(SignalSeverity::Critical.weight(), 4.0);
    }

    #[test]
    fn test_rule_matching() {
        let rule = RCARule {
            name: "test-rule".to_string(),
            pattern: vec!["error".to_string(), "database".to_string()],
            root_cause: RootCauseType::DatabaseIssue,
            confidence: 0.8,
            weight: 2.0,
        };

        let signal = Signal {
            id: SignalId("test".to_string()),
            signal_type: SignalType::LogError,
            source: "app".to_string(),
            component: "test".to_string(),
            message: "database connection error".to_string(),
            severity: SignalSeverity::Error,
            timestamp: String::new(),
            metadata: HashMap::new(),
        };

        let score = rule.match_signal(&signal);
        assert!(score > 0.0);
    }

    #[test]
    fn test_rule_no_match() {
        let rule = RCARule {
            name: "test-rule".to_string(),
            pattern: vec!["database".to_string()],
            root_cause: RootCauseType::DatabaseIssue,
            confidence: 0.8,
            weight: 2.0,
        };

        let signal = Signal {
            id: SignalId("test".to_string()),
            signal_type: SignalType::LogError,
            source: "app".to_string(),
            component: "test".to_string(),
            message: "network timeout".to_string(),
            severity: SignalSeverity::Error,
            timestamp: String::new(),
            metadata: HashMap::new(),
        };

        let score = rule.match_signal(&signal);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_root_cause_types() {
        assert_eq!(RootCauseType::DatabaseIssue.label(), "Database Issue");
        assert_eq!(RootCauseType::ServiceDown.label(), "Service Down");
        assert_eq!(RootCauseType::Unknown.label(), "Unknown");
    }

    #[test]
    fn test_signal_type_labels() {
        assert_eq!(SignalType::MetricAnomaly.label(), "Metric Anomaly");
        assert_eq!(SignalType::LogError.label(), "Log Error");
        assert_eq!(SignalType::HealthCheckFail.label(), "Health Check Fail");
    }

    #[test]
    fn test_rca_result_has_root_cause() {
        let engine = RCAEngine::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        assert!(result.root_cause.is_some());
        let rc = result.root_cause.unwrap();
        assert!(rc.confidence > 0.0);
        assert!(rc.confidence <= 1.0);
    }

    #[test]
    fn test_candidates_sorted_by_confidence() {
        let engine = RCAEngine::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        for i in 1..result.candidates.len() {
            assert!(result.candidates[i - 1].confidence >= result.candidates[i].confidence);
        }
    }

    #[test]
    fn test_affected_components() {
        let engine = RCAEngine::new();
        let signals = create_test_signals();
        let result = engine.analyze(&signals);

        assert!(result.components_affected.contains(&"user-service".to_string()));
        assert!(result.components_affected.contains(&"order-service".to_string()));
    }

    #[test]
    fn test_add_custom_rule() {
        let mut engine = RCAEngine::new();
        let initial_count = engine.rules.len();

        engine.add_rule(RCARule {
            name: "custom-rule".to_string(),
            pattern: vec!["custom".to_string()],
            root_cause: RootCauseType::Unknown,
            confidence: 0.5,
            weight: 1.0,
        });

        assert_eq!(engine.rules.len(), initial_count + 1);
    }
}
