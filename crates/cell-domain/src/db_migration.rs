use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum MigrationStatus {
    #[default]
    Pending,
    Applying,
    Applied,
    Failed,
    RolledBack,
}


impl fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Applying => write!(f, "Applying"),
            Self::Applied => write!(f, "Applied"),
            Self::Failed => write!(f, "Failed"),
            Self::RolledBack => write!(f, "RolledBack"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum MigrationType {
    #[default]
    Schema,
    Data,
    Index,
    Config,
}


impl fmt::Display for MigrationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schema => write!(f, "Schema"),
            Self::Data => write!(f, "Data"),
            Self::Index => write!(f, "Index"),
            Self::Config => write!(f, "Config"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: String,
    pub description: String,
    pub migration_type: MigrationType,
    pub up_sql: String,
    pub down_sql: String,
    pub checksum: String,
    pub status: MigrationStatus,
    pub duration_ms: u64,
    pub applied_at: Option<String>,
    pub error_message: Option<String>,
}

impl Migration {
    pub fn new(
        version: impl Into<String>,
        description: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: impl Into<String>,
    ) -> Self {
        let version = version.into();
        let up_sql = up_sql.into();
        let down_sql = down_sql.into();
        let checksum = Self::compute_checksum(&version, &up_sql, &down_sql);
        Self {
            version,
            description: description.into(),
            migration_type: MigrationType::default(),
            up_sql,
            down_sql,
            checksum,
            status: MigrationStatus::Pending,
            duration_ms: 0,
            applied_at: None,
            error_message: None,
        }
    }

    pub fn with_type(mut self, migration_type: MigrationType) -> Self {
        self.migration_type = migration_type;
        self
    }

    fn compute_checksum(version: &str, up_sql: &str, down_sql: &str) -> String {
        let mut hash = 0u64;
        for byte in version.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
        for byte in up_sql.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
        for byte in down_sql.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
        format!("{hash:016x}")
    }

    pub fn is_pending(&self) -> bool {
        self.status == MigrationStatus::Pending
    }

    pub fn is_applied(&self) -> bool {
        self.status == MigrationStatus::Applied
    }

    pub fn is_failed(&self) -> bool {
        self.status == MigrationStatus::Failed
    }

    pub fn is_rolled_back(&self) -> bool {
        self.status == MigrationStatus::RolledBack
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    NotFound(String),
    AlreadyApplied(String),
    AlreadyRegistered(String),
    InvalidVersion(String),
    ChainBroken(String),
    ApplyFailed(String),
    RollbackFailed(String),
    ChecksumMismatch(String),
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(v) => write!(f, "Migration not found: {v}"),
            Self::AlreadyApplied(v) => write!(f, "Migration already applied: {v}"),
            Self::AlreadyRegistered(v) => {
                write!(f, "Migration already registered: {v}")
            }
            Self::InvalidVersion(v) => write!(f, "Invalid version: {v}"),
            Self::ChainBroken(v) => write!(f, "Migration chain broken at: {v}"),
            Self::ApplyFailed(v) => write!(f, "Apply failed: {v}"),
            Self::RollbackFailed(v) => write!(f, "Rollback failed: {v}"),
            Self::ChecksumMismatch(v) => write!(f, "Checksum mismatch: {v}"),
        }
    }
}

impl std::error::Error for MigrationError {}

pub type MigrationResult<T> = Result<T, MigrationError>;

#[derive(Debug, Clone)]
pub struct MigrationManager {
    migrations: HashMap<String, Migration>,
    order: Vec<String>,
    rollback_points: Vec<String>,
}

impl MigrationManager {
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
            order: Vec::new(),
            rollback_points: Vec::new(),
        }
    }

    pub fn register_migration(&mut self, migration: Migration) -> MigrationResult<()> {
        let version = migration.version.clone();
        if self.migrations.contains_key(&version) {
            return Err(MigrationError::AlreadyRegistered(version));
        }
        self.order.push(version.clone());
        self.order.sort();
        self.migrations.insert(version, migration);
        Ok(())
    }

    pub fn get_migration(&self, version: &str) -> MigrationResult<&Migration> {
        self.migrations
            .get(version)
            .ok_or_else(|| MigrationError::NotFound(version.to_string()))
    }

    pub fn pending_migrations(&self) -> Vec<&Migration> {
        let mut result: Vec<&Migration> = self
            .order
            .iter()
            .filter_map(|v| self.migrations.get(v))
            .filter(|m| m.is_pending())
            .collect();
        result.sort_by(|a, b| a.version.cmp(&b.version));
        result
    }

    pub fn applied_migrations(&self) -> Vec<&Migration> {
        let mut result: Vec<&Migration> = self
            .order
            .iter()
            .filter_map(|v| self.migrations.get(v))
            .filter(|m| m.is_applied())
            .collect();
        result.sort_by(|a, b| a.version.cmp(&b.version));
        result
    }

    pub fn apply_migration(&mut self, version: &str) -> MigrationResult<()> {
        self.validate_chain()?;

        let migration = self
            .migrations
            .get_mut(version)
            .ok_or_else(|| MigrationError::NotFound(version.to_string()))?;

        if migration.is_applied() {
            return Err(MigrationError::AlreadyApplied(version.to_string()));
        }

        migration.status = MigrationStatus::Applying;
        let start = std::time::Instant::now();

        let up_sql = migration.up_sql.clone();
        if up_sql.is_empty() {
            migration.status = MigrationStatus::Failed;
            migration.error_message = Some("Empty up SQL".to_string());
            return Err(MigrationError::ApplyFailed(
                "Empty up SQL".to_string(),
            ));
        }

        let duration = start.elapsed();
        let migration = self.migrations.get_mut(version).unwrap();
        migration.status = MigrationStatus::Applied;
        migration.duration_ms = duration.as_millis() as u64;
        migration.applied_at = Some(chrono::Utc::now().to_rfc3339());
        migration.error_message = None;

        Ok(())
    }

    pub fn rollback(&mut self, version: &str) -> MigrationResult<()> {
        let migration = self
            .migrations
            .get_mut(version)
            .ok_or_else(|| MigrationError::NotFound(version.to_string()))?;

        if !migration.is_applied() {
            return Err(MigrationError::RollbackFailed(format!(
                "Migration {version} is not applied"
            )));
        }

        let down_sql = migration.down_sql.clone();
        if down_sql.is_empty() {
            migration.status = MigrationStatus::Failed;
            migration.error_message = Some("Empty down SQL".to_string());
            return Err(MigrationError::RollbackFailed(
                "Empty down SQL".to_string(),
            ));
        }

        migration.status = MigrationStatus::RolledBack;
        migration.applied_at = None;
        migration.error_message = None;

        Ok(())
    }

    pub fn rollback_to_pending(&mut self, version: &str) -> MigrationResult<()> {
        self.rollback(version)?;
        let migration = self
            .migrations
            .get_mut(version)
            .ok_or_else(|| MigrationError::NotFound(version.to_string()))?;
        migration.status = MigrationStatus::Pending;
        Ok(())
    }

    pub fn migrate_all(&mut self) -> MigrationResult<Vec<String>> {
        self.validate_chain()?;

        let pending: Vec<String> = self
            .pending_migrations()
            .iter()
            .map(|m| m.version.clone())
            .collect();

        let mut applied = Vec::new();
        let mut failed_version: Option<String> = None;
        let mut error_msg: Option<String> = None;

        for version in &pending {
            match self.apply_migration(version) {
                Ok(()) => applied.push(version.clone()),
                Err(e) => {
                    failed_version = Some(version.clone());
                    error_msg = Some(e.to_string());
                    break;
                }
            }
        }

        if let Some(failed) = failed_version {
            for v in applied.iter().rev() {
                let _ = self.rollback_to_pending(v);
            }
            if let Some(m) = self.migrations.get_mut(&failed) {
                m.status = MigrationStatus::Pending;
                m.error_message = None;
            }
            return Err(MigrationError::ApplyFailed(format!(
                "Migration {} failed: {}. All applied migrations have been rolled back.",
                failed,
                error_msg.unwrap_or_default()
            )));
        }

        Ok(applied)
    }

    pub fn validate_chain(&self) -> MigrationResult<()> {
        let applied: Vec<&Migration> = self.applied_migrations();
        let all_sorted: Vec<&Migration> = {
            let mut all: Vec<&Migration> = self.migrations.values().collect();
            all.sort_by(|a, b| a.version.cmp(&b.version));
            all
        };

        if applied.is_empty() {
            return Ok(());
        }

        for (i, m) in all_sorted.iter().enumerate() {
            if m.is_applied() {
                continue;
            }
            let has_later_applied = all_sorted.iter().skip(i + 1).any(|later| later.is_applied());
            if has_later_applied {
                return Err(MigrationError::ChainBroken(m.version.clone()));
            }
        }

        Ok(())
    }

    pub fn save_rollback_point(&mut self, name: impl Into<String>) {
        self.rollback_points.push(name.into());
    }

    pub fn rollback_to_point(&mut self, _name: &str) -> MigrationResult<Vec<String>> {
        let applied: Vec<String> = self
            .applied_migrations()
            .iter()
            .map(|m| m.version.clone())
            .collect();

        let mut rolled_back = Vec::new();
        for v in applied.iter().rev() {
            self.rollback(v)?;
            rolled_back.push(v.clone());
        }

        Ok(rolled_back)
    }

    pub fn migration_count(&self) -> usize {
        self.migrations.len()
    }

    pub fn applied_count(&self) -> usize {
        self.applied_migrations().len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_migrations().len()
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_migration(version: &str, desc: &str) -> Migration {
        Migration::new(
            version,
            desc,
            "CREATE TABLE test (id INT);",
            "DROP TABLE test;",
        )
    }

    #[test]
    fn test_migration_creation() {
        let migration = create_test_migration("001_initial", "Initial schema");

        assert_eq!(migration.version, "001_initial");
        assert_eq!(migration.description, "Initial schema");
        assert_eq!(migration.status, MigrationStatus::Pending);
        assert_eq!(migration.migration_type, MigrationType::Schema);
        assert!(!migration.checksum.is_empty());
        assert_eq!(migration.duration_ms, 0);
        assert!(migration.applied_at.is_none());
        assert!(migration.error_message.is_none());
    }

    #[test]
    fn test_migration_with_type() {
        let migration =
            create_test_migration("002_data", "Seed data").with_type(MigrationType::Data);

        assert_eq!(migration.migration_type, MigrationType::Data);
    }

    #[test]
    fn test_migration_status_checks() {
        let mut migration = create_test_migration("001_test", "Test");

        assert!(migration.is_pending());
        assert!(!migration.is_applied());
        assert!(!migration.is_failed());
        assert!(!migration.is_rolled_back());

        migration.status = MigrationStatus::Applied;
        assert!(migration.is_applied());

        migration.status = MigrationStatus::Failed;
        assert!(migration.is_failed());

        migration.status = MigrationStatus::RolledBack;
        assert!(migration.is_rolled_back());
    }

    #[test]
    fn test_migration_checksum_consistency() {
        let m1 = create_test_migration("001_test", "Test");
        let m2 = create_test_migration("001_test", "Test");

        assert_eq!(m1.checksum, m2.checksum);

        let m3 = Migration::new("001_test", "Test", "CREATE TABLE different;", "DROP TABLE test;");
        assert_ne!(m1.checksum, m3.checksum);
    }

    #[test]
    fn test_register_migration() {
        let mut manager = MigrationManager::new();
        let migration = create_test_migration("001_initial", "Initial schema");

        assert!(manager.register_migration(migration).is_ok());
        assert_eq!(manager.migration_count(), 1);
    }

    #[test]
    fn test_register_duplicate_migration() {
        let mut manager = MigrationManager::new();
        let m1 = create_test_migration("001_initial", "Initial");
        let m2 = create_test_migration("001_initial", "Initial dup");

        assert!(manager.register_migration(m1).is_ok());
        let result = manager.register_migration(m2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MigrationError::AlreadyRegistered(_)
        ));
    }

    #[test]
    fn test_get_migration() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();

        let m = manager.get_migration("001_initial").unwrap();
        assert_eq!(m.version, "001_initial");
        assert_eq!(m.description, "Initial");
    }

    #[test]
    fn test_get_migration_not_found() {
        let manager = MigrationManager::new();
        let result = manager.get_migration("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MigrationError::NotFound(_)));
    }

    #[test]
    fn test_pending_migrations() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users table"))
            .unwrap();

        let pending = manager.pending_migrations();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].version, "001_initial");
        assert_eq!(pending[1].version, "002_users");
    }

    #[test]
    fn test_apply_migration() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();

        let result = manager.apply_migration("001_initial");
        assert!(result.is_ok());

        let m = manager.get_migration("001_initial").unwrap();
        assert!(m.is_applied());
        assert!(m.applied_at.is_some());
    }

    #[test]
    fn test_apply_already_applied() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager.apply_migration("001_initial").unwrap();

        let result = manager.apply_migration("001_initial");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MigrationError::AlreadyApplied(_)
        ));
    }

    #[test]
    fn test_rollback_migration() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager.apply_migration("001_initial").unwrap();

        let result = manager.rollback("001_initial");
        assert!(result.is_ok());

        let m = manager.get_migration("001_initial").unwrap();
        assert!(m.is_rolled_back());
        assert!(m.applied_at.is_none());
    }

    #[test]
    fn test_rollback_not_applied() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();

        let result = manager.rollback("001_initial");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MigrationError::RollbackFailed(_)
        ));
    }

    #[test]
    fn test_applied_migrations() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users"))
            .unwrap();
        manager
            .register_migration(create_test_migration("003_posts", "Posts"))
            .unwrap();

        manager.apply_migration("001_initial").unwrap();
        manager.apply_migration("002_users").unwrap();

        let applied = manager.applied_migrations();
        assert_eq!(applied.len(), 2);
        assert_eq!(applied[0].version, "001_initial");
        assert_eq!(applied[1].version, "002_users");
    }

    #[test]
    fn test_validate_chain_valid() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users"))
            .unwrap();
        manager
            .register_migration(create_test_migration("003_posts", "Posts"))
            .unwrap();

        manager.apply_migration("001_initial").unwrap();
        manager.apply_migration("002_users").unwrap();

        assert!(manager.validate_chain().is_ok());
    }

    #[test]
    fn test_validate_chain_broken() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users"))
            .unwrap();
        manager
            .register_migration(create_test_migration("003_posts", "Posts"))
            .unwrap();

        manager.apply_migration("001_initial").unwrap();
        manager.migrations.get_mut("003_posts").unwrap().status = MigrationStatus::Applied;

        let result = manager.validate_chain();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MigrationError::ChainBroken(_)
        ));
    }

    #[test]
    fn test_migrate_all_success() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users"))
            .unwrap();
        manager
            .register_migration(create_test_migration("003_posts", "Posts"))
            .unwrap();

        let result = manager.migrate_all();
        assert!(result.is_ok());
        let applied = result.unwrap();
        assert_eq!(applied.len(), 3);
        assert_eq!(applied[0], "001_initial");
        assert_eq!(applied[1], "002_users");
        assert_eq!(applied[2], "003_posts");

        assert_eq!(manager.applied_count(), 3);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_migrate_all_failure_rollback() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users"))
            .unwrap();

        let mut bad_migration = create_test_migration("003_bad", "Bad migration");
        bad_migration.up_sql = String::new();
        manager.register_migration(bad_migration).unwrap();

        let result = manager.migrate_all();
        assert!(result.is_err());

        assert_eq!(manager.pending_count(), 3);
        assert_eq!(manager.applied_count(), 0);
    }

    #[test]
    fn test_migration_status_display() {
        assert_eq!(MigrationStatus::Pending.to_string(), "Pending");
        assert_eq!(MigrationStatus::Applying.to_string(), "Applying");
        assert_eq!(MigrationStatus::Applied.to_string(), "Applied");
        assert_eq!(MigrationStatus::Failed.to_string(), "Failed");
        assert_eq!(MigrationStatus::RolledBack.to_string(), "RolledBack");
    }

    #[test]
    fn test_migration_type_display() {
        assert_eq!(MigrationType::Schema.to_string(), "Schema");
        assert_eq!(MigrationType::Data.to_string(), "Data");
        assert_eq!(MigrationType::Index.to_string(), "Index");
        assert_eq!(MigrationType::Config.to_string(), "Config");
    }

    #[test]
    fn test_save_rollback_point() {
        let mut manager = MigrationManager::new();
        manager.save_rollback_point("before_big_change");
        assert_eq!(manager.rollback_points.len(), 1);
    }

    #[test]
    fn test_rollback_to_point() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_users", "Users"))
            .unwrap();
        manager.apply_migration("001_initial").unwrap();
        manager.apply_migration("002_users").unwrap();

        manager.save_rollback_point("v1");

        let result = manager.rollback_to_point("v1");
        assert!(result.is_ok());
        assert_eq!(manager.applied_count(), 0);
    }

    #[test]
    fn test_counts() {
        let mut manager = MigrationManager::new();
        assert_eq!(manager.migration_count(), 0);
        assert_eq!(manager.applied_count(), 0);
        assert_eq!(manager.pending_count(), 0);

        manager
            .register_migration(create_test_migration("001_initial", "Initial"))
            .unwrap();
        assert_eq!(manager.migration_count(), 1);
        assert_eq!(manager.pending_count(), 1);

        manager.apply_migration("001_initial").unwrap();
        assert_eq!(manager.applied_count(), 1);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_version_ordering() {
        let mut manager = MigrationManager::new();
        manager
            .register_migration(create_test_migration("003_third", "Third"))
            .unwrap();
        manager
            .register_migration(create_test_migration("001_first", "First"))
            .unwrap();
        manager
            .register_migration(create_test_migration("002_second", "Second"))
            .unwrap();

        let pending = manager.pending_migrations();
        assert_eq!(pending[0].version, "001_first");
        assert_eq!(pending[1].version, "002_second");
        assert_eq!(pending[2].version, "003_third");
    }

    #[test]
    fn test_migration_error_display() {
        let err = MigrationError::NotFound("001_test".to_string());
        assert!(err.to_string().contains("001_test"));

        let err = MigrationError::ChainBroken("002_gap".to_string());
        assert!(err.to_string().contains("002_gap"));
    }

    #[test]
    fn test_default_manager() {
        let manager = MigrationManager::default();
        assert_eq!(manager.migration_count(), 0);
    }
}
