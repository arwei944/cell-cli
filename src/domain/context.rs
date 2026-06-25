use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevContext {
    pub context_id: Uuid,
    pub project_name: String,
    pub cell_name: String,
    pub current_task: String,
    pub task_description: String,
    pub related_files: Vec<FileRef>,
    pub open_questions: Vec<String>,
    pub decisions: Vec<DecisionRef>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileRef {
    pub path: String,
    pub role: FileRole,
    pub description: String,
    pub modification_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum FileRole {
    DomainModel,
    Port,
    Adapter,
    UseCase,
    Test,
    Config,
    Documentation,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionRef {
    pub adr_id: String,
    pub title: String,
    pub status: DecisionStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
}

impl Default for DevContext {
    fn default() -> Self {
        Self {
            context_id: Uuid::new_v4(),
            project_name: String::new(),
            cell_name: String::new(),
            current_task: String::new(),
            task_description: String::new(),
            related_files: Vec::new(),
            open_questions: Vec::new(),
            decisions: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}
