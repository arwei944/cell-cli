pub mod arch_service;
pub mod auto_progress_service;
pub mod config_service;
pub mod coverage_service;
pub mod decision_service;
pub mod dependency_analyzer;
pub mod entropy_service;
pub mod evolution_service;
pub mod fast_verify_service;
pub mod generate_service;
pub mod handoff_service;
pub mod impact_analysis_service;
pub mod incremental_entropy_service;
pub mod init_service;
pub mod observability;
pub mod ports;
pub mod progress_bar;
pub mod progress_service;
pub mod simplicity_checker;

#[cfg(test)]
mod architecture_tests;
