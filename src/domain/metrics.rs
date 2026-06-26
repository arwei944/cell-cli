use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetrics {
    pub cell_name: String,
    pub cell_version: String,
    pub request_metrics: RequestMetrics,
    pub event_metrics: EventMetrics,
    pub domain_metrics: DomainMetrics,
    pub system_metrics: SystemMetrics,
    pub labels: HashMap<String, String>,
}

impl Default for CellMetrics {
    fn default() -> Self {
        Self {
            cell_name: "unknown".to_string(),
            cell_version: "0.1.0".to_string(),
            request_metrics: RequestMetrics::default(),
            event_metrics: EventMetrics::default(),
            domain_metrics: DomainMetrics::default(),
            system_metrics: SystemMetrics::default(),
            labels: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total: u64,
    pub active: u64,
    pub errors: u64,
    pub duration_seconds: f64,
    pub duration_p50: f64,
    pub duration_p95: f64,
    pub duration_p99: f64,
    pub by_endpoint: HashMap<String, EndpointMetrics>,
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            total: 0,
            active: 0,
            errors: 0,
            duration_seconds: 0.0,
            duration_p50: 0.0,
            duration_p95: 0.0,
            duration_p99: 0.0,
            by_endpoint: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub total: u64,
    pub errors: u64,
    pub avg_duration: f64,
}

impl Default for EndpointMetrics {
    fn default() -> Self {
        Self {
            total: 0,
            errors: 0,
            avg_duration: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    pub published: u64,
    pub consumed: u64,
    pub processing_duration_seconds: f64,
    pub dlq_count: u64,
    pub by_topic: HashMap<String, TopicMetrics>,
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self {
            published: 0,
            consumed: 0,
            processing_duration_seconds: 0.0,
            dlq_count: 0,
            by_topic: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMetrics {
    pub published: u64,
    pub consumed: u64,
    pub errors: u64,
}

impl Default for TopicMetrics {
    fn default() -> Self {
        Self {
            published: 0,
            consumed: 0,
            errors: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainMetrics {
    pub aggregates_total: u64,
    pub domain_errors: u64,
    pub business_rules_evaluated: u64,
    pub by_aggregate: HashMap<String, AggregateMetrics>,
}

impl Default for DomainMetrics {
    fn default() -> Self {
        Self {
            aggregates_total: 0,
            domain_errors: 0,
            business_rules_evaluated: 0,
            by_aggregate: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMetrics {
    pub created: u64,
    pub updated: u64,
    pub deleted: u64,
    pub invariants_checked: u64,
    pub invariants_violated: u64,
}

impl Default for AggregateMetrics {
    fn default() -> Self {
        Self {
            created: 0,
            updated: 0,
            deleted: 0,
            invariants_checked: 0,
            invariants_violated: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub memory_bytes: u64,
    pub cpu_usage: f64,
    pub goroutines: u64,
    pub gc_count: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            memory_bytes: 0,
            cpu_usage: 0.0,
            goroutines: 0,
            gc_count: 0,
        }
    }
}

impl CellMetrics {
    pub fn to_prometheus(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("# HELP cell_requests_total Total number of requests"));
        lines.push(format!("# TYPE cell_requests_total counter"));
        lines.push(format!(
            "cell_requests_total{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.request_metrics.total
        ));

        lines.push(format!("# HELP cell_requests_active Number of active requests"));
        lines.push(format!("# TYPE cell_requests_active gauge"));
        lines.push(format!(
            "cell_requests_active{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.request_metrics.active
        ));

        lines.push(format!("# HELP cell_requests_errors Total number of request errors"));
        lines.push(format!("# TYPE cell_requests_errors counter"));
        lines.push(format!(
            "cell_requests_errors{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.request_metrics.errors
        ));

        lines.push(format!("# HELP cell_request_duration_seconds Request duration in seconds"));
        lines.push(format!("# TYPE cell_request_duration_seconds histogram"));
        lines.push(format!(
            "cell_request_duration_seconds_sum{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.request_metrics.duration_seconds
        ));

        lines.push(format!("# HELP cell_events_published Total number of published events"));
        lines.push(format!("# TYPE cell_events_published counter"));
        lines.push(format!(
            "cell_events_published{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.event_metrics.published
        ));

        lines.push(format!("# HELP cell_events_consumed Total number of consumed events"));
        lines.push(format!("# TYPE cell_events_consumed counter"));
        lines.push(format!(
            "cell_events_consumed{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.event_metrics.consumed
        ));

        lines.push(format!("# HELP cell_events_dlq Total number of dead letter queue events"));
        lines.push(format!("# TYPE cell_events_dlq counter"));
        lines.push(format!(
            "cell_events_dlq{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.event_metrics.dlq_count
        ));

        lines.push(format!("# HELP cell_aggregates_total Total number of aggregates"));
        lines.push(format!("# TYPE cell_aggregates_total gauge"));
        lines.push(format!(
            "cell_aggregates_total{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.domain_metrics.aggregates_total
        ));

        lines.push(format!("# HELP cell_domain_errors Total number of domain errors"));
        lines.push(format!("# TYPE cell_domain_errors counter"));
        lines.push(format!(
            "cell_domain_errors{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.domain_metrics.domain_errors
        ));

        lines.push(format!("# HELP cell_business_rules_evaluated Total number of business rules evaluated"));
        lines.push(format!("# TYPE cell_business_rules_evaluated counter"));
        lines.push(format!(
            "cell_business_rules_evaluated{{cell=\"{}\",version=\"{}\"}} {}",
            self.cell_name, self.cell_version, self.domain_metrics.business_rules_evaluated
        ));

        lines.join("\n")
    }
}
