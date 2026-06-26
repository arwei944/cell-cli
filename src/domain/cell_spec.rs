use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellSpec {
    pub name: String,
    pub description: String,
    pub version: String,
    pub cell_version: Option<String>,
    pub owners: Vec<String>,
    pub ports: Vec<PortSpec>,
    pub adapters: Vec<AdapterSpec>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub architecture: Option<ArchitectureConfig>,
    pub lint: Option<LintConfig>,
    pub entropy: Option<EntropyConfig>,
    pub domain: Option<DomainConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchitectureConfig {
    pub style: Option<String>,
    pub layers: Vec<LayerConfig>,
    pub strict_dependency_check: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayerConfig {
    pub name: String,
    pub path: String,
    pub allowed_dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LintConfig {
    pub rules: Vec<LintRuleConfig>,
    pub severity_overrides: std::collections::HashMap<String, String>,
    pub exclude_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LintRuleConfig {
    pub id: String,
    pub enabled: bool,
    pub severity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyConfig {
    pub max_entropy: f64,
    pub guard_on_commit: bool,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainConfig {
    pub entities: Vec<EntitySpec>,
    pub value_objects: Vec<ValueObjectSpec>,
    pub aggregates: Vec<AggregateSpec>,
    pub domain_events: Vec<DomainEventSpec>,
    pub domain_services: Vec<DomainServiceSpec>,
    pub repositories: Vec<RepositorySpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntitySpec {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueObjectSpec {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AggregateSpec {
    pub name: String,
    pub description: Option<String>,
    pub root_entity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainEventSpec {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainServiceSpec {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepositorySpec {
    pub name: String,
    pub entity: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldSpec {
    pub name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub required: bool,
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
            cell_version: Some("0.1.0".to_string()),
            owners: vec![],
            ports: vec![],
            adapters: vec![],
            dependencies: vec![],
            tags: vec![],
            architecture: Some(ArchitectureConfig {
                style: Some("hexagonal".to_string()),
                layers: vec![
                    LayerConfig {
                        name: "domain".to_string(),
                        path: "src/domain".to_string(),
                        allowed_dependencies: vec![],
                    },
                    LayerConfig {
                        name: "application".to_string(),
                        path: "src/application".to_string(),
                        allowed_dependencies: vec!["domain".to_string()],
                    },
                    LayerConfig {
                        name: "adapters".to_string(),
                        path: "src/adapters".to_string(),
                        allowed_dependencies: vec!["domain".to_string(), "application".to_string()],
                    },
                    LayerConfig {
                        name: "interfaces".to_string(),
                        path: "src/interfaces".to_string(),
                        allowed_dependencies: vec!["domain".to_string(), "application".to_string(), "adapters".to_string()],
                    },
                ],
                strict_dependency_check: true,
            }),
            lint: Some(LintConfig {
                rules: vec![],
                severity_overrides: std::collections::HashMap::new(),
                exclude_paths: vec![
                    "tests/".to_string(),
                    "examples/".to_string(),
                ],
            }),
            entropy: Some(EntropyConfig {
                max_entropy: 7.0,
                guard_on_commit: false,
                exclude_patterns: vec![
                    "**/tests/**".to_string(),
                    "**/migrations/**".to_string(),
                ],
            }),
            domain: Some(DomainConfig {
                entities: vec![],
                value_objects: vec![],
                aggregates: vec![],
                domain_events: vec![],
                domain_services: vec![],
                repositories: vec![],
            }),
        }
    }
}

impl CellSpec {
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.name.is_empty() {
            errors.push("Cell name is required".to_string());
        }
        
        if self.version.is_empty() {
            errors.push("Version is required".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
