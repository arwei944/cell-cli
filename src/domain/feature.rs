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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Adjustment,
}
