use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: MigrationStatus,
    pub created_at: String,
    pub applied_at: Option<String>,
    pub rollback_at: Option<String>,
    pub description: String,
    pub changes: Vec<SchemaChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Pending,
    Applied,
    RolledBack,
    Failed,
}

impl MigrationStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Pending => "待执行",
            Self::Applied => "已应用",
            Self::RolledBack => "已回滚",
            Self::Failed => "失败",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub type_: ChangeType,
    pub target: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    CreateTable,
    DropTable,
    AddColumn,
    DropColumn,
    ModifyColumn,
    AddIndex,
    DropIndex,
    Custom,
}

impl ChangeType {
    pub fn label(&self) -> &str {
        match self {
            Self::CreateTable => "创建表",
            Self::DropTable => "删除表",
            Self::AddColumn => "添加列",
            Self::DropColumn => "删除列",
            Self::ModifyColumn => "修改列",
            Self::AddIndex => "添加索引",
            Self::DropIndex => "删除索引",
            Self::Custom => "自定义",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationHistory {
    pub migrations: Vec<Migration>,
    pub current_version: String,
    pub schema_hash: String,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResult {
    pub expected_hash: String,
    pub actual_hash: String,
    pub has_drift: bool,
    pub differences: Vec<String>,
}

pub struct DbMigrationService;

impl DbMigrationService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_migration(&self, project_path: &str, name: &str, description: &str, changes: Vec<SchemaChange>) -> CellResult<Migration> {
        let mut history = self.load_history(project_path)?;
        
        let version = self.next_version(&history.current_version);
        let id = format!("{version}_{name}");
        let now = chrono::Utc::now().to_rfc3339();

        let migration = Migration {
            id,
            name: name.to_string(),
            version,
            status: MigrationStatus::Pending,
            created_at: now,
            applied_at: None,
            rollback_at: None,
            description: description.to_string(),
            changes,
        };

        history.migrations.push(migration.clone());
        self.save_history(project_path, &history)?;

        Ok(migration)
    }

    pub fn list_migrations(&self, project_path: &str) -> CellResult<Vec<Migration>> {
        let history = self.load_history(project_path)?;
        Ok(history.migrations)
    }

    pub fn get_migration(&self, project_path: &str, id: &str) -> CellResult<Migration> {
        let history = self.load_history(project_path)?;
        history.migrations.into_iter().find(|m| m.id == id)
            .ok_or_else(|| cell_domain::errors::CellError::Config(format!("Migration '{id}' not found")))
    }

    pub fn migrate(&self, project_path: &str, target_version: Option<&str>) -> CellResult<Vec<Migration>> {
        let mut history = self.load_history(project_path)?;
        let mut applied = Vec::new();

        let pending: Vec<&mut Migration> = history.migrations.iter_mut()
            .filter(|m| m.status == MigrationStatus::Pending)
            .filter(|m| target_version.is_none_or(|t| m.version.as_str() <= t))
            .collect();

        for migration in pending {
            migration.status = MigrationStatus::Applied;
            migration.applied_at = Some(chrono::Utc::now().to_rfc3339());
            applied.push(migration.clone());
        }

        if !applied.is_empty() {
            let last = applied.last().unwrap();
            history.current_version = last.version.clone();
            history.schema_hash = self.calculate_hash(&history.migrations);
            history.last_updated = chrono::Utc::now().to_rfc3339();
            self.save_history(project_path, &history)?;
        }

        Ok(applied)
    }

    pub fn rollback(&self, project_path: &str, target_version: &str) -> CellResult<Vec<Migration>> {
        let mut history = self.load_history(project_path)?;
        let mut rolled_back = Vec::new();

        let to_rollback: Vec<&mut Migration> = history.migrations.iter_mut()
            .filter(|m| m.status == MigrationStatus::Applied)
            .filter(|m| m.version.as_str() > target_version)
            .collect();

        for migration in to_rollback {
            migration.status = MigrationStatus::RolledBack;
            migration.rollback_at = Some(chrono::Utc::now().to_rfc3339());
            rolled_back.push(migration.clone());
        }

        if !rolled_back.is_empty() {
            history.current_version = target_version.to_string();
            history.schema_hash = self.calculate_hash(&history.migrations);
            history.last_updated = chrono::Utc::now().to_rfc3339();
            self.save_history(project_path, &history)?;
        }

        Ok(rolled_back)
    }

    pub fn status(&self, project_path: &str) -> CellResult<MigrationHistory> {
        self.load_history(project_path)
    }

    pub fn detect_drift(&self, project_path: &str, actual_schema: &str) -> CellResult<DriftResult> {
        let history = self.load_history(project_path)?;
        let actual_hash = self.hash_string(actual_schema);
        let expected_hash = history.schema_hash;

        let has_drift = actual_hash != expected_hash;
        let differences = if has_drift {
            vec!["Schema hash mismatch".to_string()]
        } else {
            Vec::new()
        };

        Ok(DriftResult {
            expected_hash,
            actual_hash,
            has_drift,
            differences,
        })
    }

    pub fn validate_migration(&self, project_path: &str, id: &str) -> CellResult<bool> {
        let migration = self.get_migration(project_path, id)?;
        
        for change in &migration.changes {
            match change.type_ {
                ChangeType::DropTable | ChangeType::DropColumn
                    if !change.details.contains("cascade") => {
                        return Ok(false);
                    }
                _ => {}
            }
        }

        Ok(true)
    }

    fn next_version(&self, current: &str) -> String {
        let parts: Vec<&str> = current.split('.').collect();
        if parts.len() == 3 {
            let major = parts[0].parse::<u32>().unwrap_or(0);
            let minor = parts[1].parse::<u32>().unwrap_or(0);
            let patch = parts[2].parse::<u32>().unwrap_or(0);
            format!("{}.{}.{}", major, minor, patch + 1)
        } else {
            "0.0.1".to_string()
        }
    }

    fn calculate_hash(&self, migrations: &[Migration]) -> String {
        let applied: Vec<&Migration> = migrations.iter()
            .filter(|m| m.status == MigrationStatus::Applied)
            .collect();
        let content = serde_json::to_string(&applied).unwrap_or_default();
        self.hash_string(&content)
    }

    fn hash_string(&self, s: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn history_path(project_path: &str) -> std::path::PathBuf {
        Path::new(project_path).join(".cell/migrations/history.json")
    }

    fn load_history(&self, project_path: &str) -> CellResult<MigrationHistory> {
        let path = Self::history_path(project_path);
        if !path.exists() {
            return Ok(MigrationHistory {
                migrations: Vec::new(),
                current_version: "0.0.0".to_string(),
                schema_hash: String::new(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            });
        }
        let content = std::fs::read_to_string(&path)?;
        let history: MigrationHistory = serde_json::from_str(&content)
            .map_err(|e| cell_domain::errors::CellError::Config(format!("Invalid migration history: {e}")))?;
        Ok(history)
    }

    fn save_history(&self, project_path: &str, history: &MigrationHistory) -> CellResult<()> {
        let path = Self::history_path(project_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(history)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for DbMigrationService {
    fn default() -> Self {
        Self::new()
    }
}