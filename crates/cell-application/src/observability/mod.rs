//! 可观测性服务：统一入口
//! Observability service: unified entry point

pub mod dashboard;
pub mod metrics;
pub mod phases;

pub use dashboard::build_dashboard;
pub use metrics::ObservabilityService;
