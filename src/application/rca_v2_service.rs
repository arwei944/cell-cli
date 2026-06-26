use crate::domain::errors::CellResult;
use crate::domain::rca_engine_v2::{
    RCAEngineV2, RCAResultV2, RCASignal, SignalSeverity, SignalType,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    pub id: String,
    pub signal: RCASignal,
    pub result: RCAResultV2,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub id: String,
    pub signal_component: String,
    pub signal_severity: String,
    pub signal_type: String,
    pub has_root_cause: bool,
    pub confidence: Option<f64>,
    pub created_at: DateTime<Utc>,
}

pub struct RCAV2Service {
    engine: RCAEngineV2,
    analyses: HashMap<String, AnalysisRecord>,
}

impl RCAV2Service {
    pub fn new() -> Self {
        Self {
            engine: RCAEngineV2::new(),
            analyses: HashMap::new(),
        }
    }

    pub fn analyze_signal(&mut self, signal: RCASignal) -> CellResult<AnalysisRecord> {
        let result = self.engine.analyze(&[signal.clone()]);
        let id = Uuid::new_v4().to_string();

        let record = AnalysisRecord {
            id: id.clone(),
            signal,
            result,
            created_at: Utc::now(),
        };

        self.analyses.insert(id, record.clone());
        Ok(record)
    }

    pub fn analyze_signals(&mut self, signals: Vec<RCASignal>) -> CellResult<AnalysisRecord> {
        let result = self.engine.analyze(&signals);
        let id = Uuid::new_v4().to_string();

        let first_signal = signals.into_iter().next().unwrap_or_else(|| RCASignal::new(
            Uuid::new_v4().to_string(),
            SignalType::Log,
            "unknown",
            "unknown",
            "multiple signals",
            SignalSeverity::Info,
            Utc::now(),
        ));

        let record = AnalysisRecord {
            id: id.clone(),
            signal: first_signal,
            result,
            created_at: Utc::now(),
        };

        self.analyses.insert(id, record.clone());
        Ok(record)
    }

    pub fn list_analyses(&self) -> Vec<AnalysisSummary> {
        self.analyses
            .values()
            .map(|record| AnalysisSummary {
                id: record.id.clone(),
                signal_component: record.signal.component.clone(),
                signal_severity: record.signal.severity.label().to_string(),
                signal_type: record.signal.signal_type.label().to_string(),
                has_root_cause: record.result.root_cause.is_some(),
                confidence: record.result.root_cause.as_ref().map(|rc| rc.confidence),
                created_at: record.created_at,
            })
            .collect()
    }

    pub fn get_analysis_detail(&self, id: &str) -> CellResult<AnalysisRecord> {
        self.analyses
            .get(id)
            .cloned()
            .ok_or_else(|| crate::domain::errors::CellError::NotFound(format!("Analysis {} not found", id)))
    }

    pub fn format_result(&self, record: &AnalysisRecord) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n📊 RCA Analysis Result (ID: {})\n{}",
            record.id,
            "─".repeat(60)
        ));

        output.push_str(&format!(
            "\n\n  Signal Info:\n    Component: {}\n    Type: {}\n    Severity: {}\n    Source: {}\n    Message: {}",
            record.signal.component,
            record.signal.signal_type.label(),
            record.signal.severity.label(),
            record.signal.source,
            record.signal.message,
        ));

        output.push_str(&format!(
            "\n\n  Analysis Summary:\n    Signals Analyzed: {}\n    Signals in Window: {}\n    Analysis Time: {}ms\n    Timestamp: {}",
            record.result.signals_analyzed,
            record.result.signals_within_window,
            record.result.analysis_time_ms,
            record.result.analysis_timestamp,
        ));

        if let Some(root_cause) = &record.result.root_cause {
            output.push_str(&format!(
                "\n\n  🎯 Root Cause Found:\n    Component: {}\n    Type: {}\n    Confidence: {:.2}%\n    Signal Count: {}",
                root_cause.component,
                root_cause.root_cause_type.label(),
                root_cause.confidence * 100.0,
                root_cause.signal_count,
            ));

            if !root_cause.evidence.is_empty() {
                output.push_str("\n\n    Evidence:\n");
                for (i, ev) in root_cause.evidence.iter().enumerate().take(5) {
                    output.push_str(&format!("      {}. {}\n", i + 1, ev));
                }
                if root_cause.evidence.len() > 5 {
                    output.push_str(&format!("      ... and {} more\n", root_cause.evidence.len() - 5));
                }
            }
        } else {
            output.push_str("\n\n  ❌ No root cause identified\n");
        }

        if !record.result.candidates.is_empty() {
            output.push_str(&format!(
                "\n  📋 Top {} Candidates:\n",
                record.result.candidates.len().min(3)
            ));
            for (i, candidate) in record.result.candidates.iter().enumerate().take(3) {
                output.push_str(&format!(
                    "    {}. {} ({}) - {:.2}% confidence\n",
                    i + 1,
                    candidate.component,
                    candidate.root_cause_type.label(),
                    candidate.confidence * 100.0,
                ));
            }
        }

        output.push_str(&format!("\n{}", "─".repeat(60)));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_signal(component: &str, message: &str, severity: SignalSeverity) -> RCASignal {
        RCASignal::new(
            Uuid::new_v4().to_string(),
            SignalType::Log,
            "test-source",
            component,
            message,
            severity,
            Utc::now(),
        )
    }

    #[test]
    fn test_analyze_signal_database_error() {
        let mut service = RCAV2Service::new();
        let signal = create_test_signal(
            "user-service",
            "database connection refused timeout db",
            SignalSeverity::Critical,
        );

        let result = service.analyze_signal(signal).unwrap();

        assert!(result.result.root_cause.is_some());
        assert_eq!(result.result.root_cause.unwrap().component, "user-service");
    }

    #[test]
    fn test_analyze_signal_no_root_cause() {
        let mut service = RCAV2Service::new();
        let signal = create_test_signal(
            "test-component",
            "normal operation message",
            SignalSeverity::Info,
        );

        let result = service.analyze_signal(signal).unwrap();

        assert!(!result.result.root_cause.is_some());
    }

    #[test]
    fn test_list_analyses() {
        let mut service = RCAV2Service::new();

        let signal1 = create_test_signal(
            "service-a",
            "database timeout error",
            SignalSeverity::Error,
        );
        let signal2 = create_test_signal(
            "service-b",
            "network connection refused",
            SignalSeverity::Critical,
        );

        service.analyze_signal(signal1).unwrap();
        service.analyze_signal(signal2).unwrap();

        let summaries = service.list_analyses();
        assert_eq!(summaries.len(), 2);
    }

    #[test]
    fn test_get_analysis_detail() {
        let mut service = RCAV2Service::new();
        let signal = create_test_signal(
            "api-gateway",
            "health check failed service is down",
            SignalSeverity::Critical,
        );

        let record = service.analyze_signal(signal).unwrap();
        let id = record.id.clone();

        let retrieved = service.get_analysis_detail(&id).unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.signal.component, "api-gateway");
    }

    #[test]
    fn test_get_analysis_detail_not_found() {
        let service = RCAV2Service::new();

        let result = service.get_analysis_detail("non-existent-id");
        assert!(matches!(result, Err(crate::domain::errors::CellError::NotFound(_))));
    }
}