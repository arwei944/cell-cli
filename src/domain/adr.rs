use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MicroAdr {
    pub id: String,
    pub title: String,
    pub status: AdrStatus,
    pub context: String,
    pub decision: String,
    pub consequences: AdrConsequences,
    pub alternatives: Vec<Alternative>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub superseded_by: Option<String>,
    pub supersedes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdrConsequences {
    pub positive: Vec<String>,
    pub negative: Vec<String>,
    pub neutral: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alternative {
    pub name: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

impl MicroAdr {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            status: AdrStatus::Proposed,
            context: String::new(),
            decision: String::new(),
            consequences: AdrConsequences {
                positive: Vec::new(),
                negative: Vec::new(),
                neutral: Vec::new(),
            },
            alternatives: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: Vec::new(),
            superseded_by: None,
            supersedes: Vec::new(),
        }
    }
}

pub fn create_adr(title: &str, next_id: usize) -> MicroAdr {
    let id = format!("{:04}", next_id);
    MicroAdr::new(&id, title)
}
