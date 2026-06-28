use crate::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 挂载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MountStatus {
    Mounted,
    Unmounted,
    Error,
}

/// 挂载点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub id: String,
    pub path: String,
    pub status: MountStatus,
    pub created_at: String,
    pub metadata: HashMap<String, String>,
}

impl MountPoint {
    pub fn new(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            status: MountStatus::Unmounted,
            created_at: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_mounted(&self) -> bool {
        self.status == MountStatus::Mounted
    }
}

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Prepared,
    Committed,
    RolledBack,
}

/// 挂载事务（2PC）
#[derive(Debug, Clone)]
pub struct MountTransaction {
    pub id: String,
    pub points: Vec<MountPoint>,
    pub status: TransactionStatus,
    pub created_at: String,
}

impl MountTransaction {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            points: Vec::new(),
            status: TransactionStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// 挂载管理器 trait
pub trait MountManager {
    fn mount(&mut self, point: MountPoint) -> CellResult<()>;
    fn unmount(&mut self, point_id: &str) -> CellResult<()>;
    fn list(&self) -> Vec<&MountPoint>;
    fn rollback(&mut self, transaction: &MountTransaction) -> CellResult<()>;
    fn commit(&mut self, transaction: &MountTransaction) -> CellResult<()>;
}
