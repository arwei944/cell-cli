use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellSpec {
    pub name: String,
    pub description: String,
    pub version: String,
    pub ports: Vec<PortSpec>,
    pub adapters: Vec<AdapterSpec>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortSpec {
    pub name: String,
    pub kind: PortKind,
    pub description: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub is_async: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortKind {
    UseCase,
    Query,
    Repository,
    Gateway,
    Publisher,
    Subscriber,
}

impl PortKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PortKind::UseCase => "use_case",
            PortKind::Query => "query",
            PortKind::Repository => "repository",
            PortKind::Gateway => "gateway",
            PortKind::Publisher => "publisher",
            PortKind::Subscriber => "subscriber",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdapterSpec {
    pub name: String,
    pub kind: AdapterKind,
    pub port: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterKind {
    InMemory,
    Postgres,
    Redis,
    Http,
    Grpc,
    Kafka,
    Nats,
    File,
    Mock,
}

impl Default for CellSpec {
    fn default() -> Self {
        CellSpec {
            name: "my-cell".to_string(),
            description: "A Cell architecture module".to_string(),
            version: "0.1.0".to_string(),
            ports: vec![],
            adapters: vec![],
            dependencies: vec![],
            tags: vec![],
        }
    }
}
