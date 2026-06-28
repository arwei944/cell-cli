use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureUnit {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub status: FeatureStatus,
    pub extension_points: Vec<ExtensionPointRef>,
    pub dependencies: Vec<String>,
    pub mounts: Vec<MountPoint>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FeatureStatus {
    Design,
    Development,
    Testing,
    Staging,
    Production,
    Deprecated,
    Retired,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionPointRef {
    pub name: String,
    pub kind: ExtensionPointKind,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExtensionPointKind {
    Validation,
    Calculation,
    Notification,
    Export,
    Transformation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MountPoint {
    pub host: String,
    pub extension_point: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyBankAccount {
    pub owner: String,
    pub balance: f64,
    pub total_deposited: f64,
    pub total_withdrawn: f64,
    pub transactions: Vec<EntropyTransaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyTransaction {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub amount: f64,
    pub reason: String,
    pub related_feature: Option<String>,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Adjustment,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureFlag {
    pub name: String,
    pub description: String,
    pub flag_type: FeatureFlagType,
    pub enabled: bool,
    pub percentage: Option<f64>,
    pub target_users: Vec<String>,
    pub target_groups: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlagType {
    Release,
    Ops,
    Experiment,
    Permission,
}

impl FeatureFlagType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Ops => "ops",
            Self::Experiment => "experiment",
            Self::Permission => "permission",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Release => "功能发布开关 - 控制新功能的逐步发布",
            Self::Ops => "运维控制开关 - 控制运维相关的功能",
            Self::Experiment => "实验开关 - A/B测试和灰度发布",
            Self::Permission => "权限开关 - 控制用户权限相关功能",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureFlagConfig {
    pub flags: Vec<FeatureFlag>,
}

impl Default for FeatureFlagConfig {
    fn default() -> Self {
        Self {
            flags: vec![
                FeatureFlag {
                    name: "dark_mode".to_string(),
                    description: "深色模式".to_string(),
                    flag_type: FeatureFlagType::Release,
                    enabled: false,
                    percentage: None,
                    target_users: vec![],
                    target_groups: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    metadata: std::collections::HashMap::new(),
                },
            ],
        }
    }
}

impl FeatureFlagConfig {
    pub fn is_enabled(&self, name: &str) -> bool {
        self.flags.iter()
            .find(|f| f.name == name)
            .is_some_and(|f| f.enabled)
    }

    pub fn get_flag(&self, name: &str) -> Option<&FeatureFlag> {
        self.flags.iter().find(|f| f.name == name)
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(flag) = self.flags.iter_mut().find(|f| f.name == name) {
            flag.enabled = enabled;
            flag.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn list_by_type(&self, flag_type: &FeatureFlagType) -> Vec<&FeatureFlag> {
        self.flags.iter()
            .filter(|f| &f.flag_type == flag_type)
            .collect()
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }
}
