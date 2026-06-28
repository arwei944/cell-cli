use thiserror::Error;

pub type CellResult<T> = Result<T, CellError>;

#[derive(Debug, Error)]
pub enum CellError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("entropy exceeded threshold: {0}")]
    EntropyExceeded(String),

    #[error("{0}")]
    Other(String),
}

impl From<crate::plugin_system::PluginError> for CellError {
    fn from(e: crate::plugin_system::PluginError) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<crate::rule_engine::RuleEngineError> for CellError {
    fn from(e: crate::rule_engine::RuleEngineError) -> Self {
        use crate::rule_engine::RuleEngineError::{RuleNotFound, RuleSetNotFound, VersionNotFound, DependencyNotFound, RuleAlreadyExists, RuleSetAlreadyExists, InvalidStatusTransition, CircularDependency, EvaluationFailed};
        match e {
            RuleNotFound(msg) | RuleSetNotFound(msg) | VersionNotFound(msg) | DependencyNotFound(msg) => {
                Self::NotFound(msg)
            }
            RuleAlreadyExists(msg) | RuleSetAlreadyExists(msg) => Self::AlreadyExists(msg),
            InvalidStatusTransition(msg) | CircularDependency(msg) => Self::Validation(msg),
            EvaluationFailed(msg) => Self::Other(msg),
        }
    }
}
