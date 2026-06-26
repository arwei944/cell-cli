use crate::domain::canary_release::{CanaryManager, CanaryRelease, CanaryStrategy, CanaryError};
use crate::domain::errors::{CellError, CellResult};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryStatus {
    pub id: String,
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub status: String,
    pub current_stage: u8,
    pub traffic_percentage: f64,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>,
    pub started_at: Option<DateTime<chrono::Utc>>,
    pub completed_at: Option<DateTime<chrono::Utc>>,
}

impl From<&CanaryRelease> for CanaryStatus {
    fn from(release: &CanaryRelease) -> Self {
        Self {
            id: release.id.to_string(),
            name: release.name.clone(),
            old_version: release.old_version.clone(),
            new_version: release.new_version.clone(),
            status: release.status.label().to_string(),
            current_stage: release.current_stage.percentage(),
            traffic_percentage: release.new_traffic_percentage(),
            created_at: release.created_at,
            updated_at: release.updated_at,
            started_at: release.started_at,
            completed_at: release.completed_at,
        }
    }
}

pub struct CanaryService {
    manager: CanaryManager,
    name_to_id: HashMap<String, uuid::Uuid>,
}

impl CanaryService {
    pub fn new() -> Self {
        Self {
            manager: CanaryManager::new(),
            name_to_id: HashMap::new(),
        }
    }

    pub fn create_canary(
        &mut self,
        name: impl Into<String>,
        old_version: impl Into<String>,
        new_version: impl Into<String>,
    ) -> CellResult<CanaryStatus> {
        let name = name.into();
        if self.name_to_id.contains_key(&name) {
            return Err(crate::domain::errors::CellError::Validation(
                format!("Canary release '{}' already exists", name),
            ));
        }

        let strategy = CanaryStrategy::default();
        let release = self.manager.create_release(&name, old_version, new_version, strategy);
        self.name_to_id.insert(name, release.id);

        Ok(release.into())
    }

    pub fn list_canaries(&self) -> Vec<CanaryStatus> {
        self.manager
            .list_releases()
            .iter()
            .map(|r| CanaryStatus::from(*r))
            .collect()
    }

    pub fn promote_canary(&mut self, name: impl AsRef<str>) -> CellResult<CanaryStatus> {
        let id = self
            .name_to_id
            .get(name.as_ref())
            .ok_or_else(|| {
                CellError::NotFound(format!(
                    "Canary release '{}' not found",
                    name.as_ref()
                ))
            })?;

        let release = self.manager.promote_release(*id)
            .map_err(|e| cell_error_from_canary_error(e))?;
        Ok(release.into())
    }

    pub fn rollback_canary(&mut self, name: impl AsRef<str>, reason: impl Into<String>) -> CellResult<CanaryStatus> {
        let id = self
            .name_to_id
            .get(name.as_ref())
            .ok_or_else(|| {
                CellError::NotFound(format!(
                    "Canary release '{}' not found",
                    name.as_ref()
                ))
            })?;

        let release = self.manager.rollback_release(*id, reason)
            .map_err(|e| cell_error_from_canary_error(e))?;
        Ok(release.into())
    }

    pub fn get_canary_status(&self, name: impl AsRef<str>) -> CellResult<CanaryStatus> {
        let id = self
            .name_to_id
            .get(name.as_ref())
            .ok_or_else(|| {
                CellError::NotFound(format!(
                    "Canary release '{}' not found",
                    name.as_ref()
                ))
            })?;

        let release = self
            .manager
            .get_release(*id)
            .ok_or_else(|| {
                CellError::NotFound(format!(
                    "Canary release '{}' not found",
                    name.as_ref()
                ))
            })?;

        Ok(release.into())
    }

    pub fn start_canary(&mut self, name: impl AsRef<str>) -> CellResult<CanaryStatus> {
        let id = self
            .name_to_id
            .get(name.as_ref())
            .ok_or_else(|| {
                CellError::NotFound(format!(
                    "Canary release '{}' not found",
                    name.as_ref()
                ))
            })?;

        let release = self.manager.start_release(*id)
            .map_err(|e| cell_error_from_canary_error(e))?;
        Ok(release.into())
    }

    pub fn format_status(&self, status: &CanaryStatus) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n📦 {} ({}):\n", status.name, status.id));
        output.push_str(&format!("{}", "─".repeat(40)));
        output.push_str(&format!("\n   Status:        {}\n", status.status));
        output.push_str(&format!(
            "   Stage:         {}% traffic\n",
            status.current_stage
        ));
        output.push_str(&format!(
            "   Traffic:       {}% to new version\n",
            status.traffic_percentage
        ));
        output.push_str(&format!("   Old Version:   {}\n", status.old_version));
        output.push_str(&format!("   New Version:   {}\n", status.new_version));
        output.push_str(&format!(
            "   Created:       {}\n",
            status.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!(
            "   Updated:       {}\n",
            status.updated_at.format("%Y-%m-%d %H:%M:%S")
        ));

        if let Some(started) = status.started_at {
            output.push_str(&format!(
                "   Started:       {}\n",
                started.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(completed) = status.completed_at {
            output.push_str(&format!(
                "   Completed:     {}\n",
                completed.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        output
    }

    pub fn format_list(&self, canaries: &[CanaryStatus]) -> String {
        if canaries.is_empty() {
            return "No canary releases found.".to_string();
        }

        let mut output = String::new();

        output.push_str("\n📋 Canary Releases:\n");
        output.push_str(&format!("{}", "─".repeat(60)));
        output.push_str(
            "\n   NAME               STATUS          STAGE   TRAFFIC   VERSIONS\n",
        );
        output.push_str(&format!("{}", "─".repeat(60)));

        for canary in canaries {
            let status_color = match canary.status.as_str() {
                "Progressing" => "🟡",
                "Promoted" => "🟢",
                "RolledBack" => "🔴",
                "Paused" => "🟠",
                "Created" => "⚪",
                "Failed" => "🔵",
                _ => "⚪",
            };

            output.push_str(&format!(
                "\n   {:<18} {} {:<12} {:<6} {:<8} {} -> {}\n",
                canary.name,
                status_color,
                canary.status,
                format!("{}%", canary.current_stage),
                format!("{}%", canary.traffic_percentage),
                canary.old_version,
                canary.new_version
            ));
        }

        output.push_str(&format!("\n{}", "─".repeat(60)));
        output.push_str(&format!("\n   Total: {} canary releases\n", canaries.len()));

        output
    }
}

impl Default for CanaryService {
    fn default() -> Self {
        Self::new()
    }
}

fn cell_error_from_canary_error(e: CanaryError) -> CellError {
    match e {
        CanaryError::ReleaseNotFound(msg) => CellError::NotFound(msg),
        CanaryError::InvalidStateTransition(msg) => CellError::Validation(msg),
        CanaryError::ReleaseAlreadyTerminal(msg) => CellError::Validation(msg),
        CanaryError::NoMoreStages(msg) => CellError::Validation(msg),
        CanaryError::InvalidStage(msg) => CellError::Validation(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_canary() {
        let mut service = CanaryService::new();
        let result = service.create_canary("test-release", "v1.0.0", "v2.0.0");

        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.name, "test-release");
        assert_eq!(status.old_version, "v1.0.0");
        assert_eq!(status.new_version, "v2.0.0");
        assert_eq!(status.status, "Created");
        assert_eq!(status.current_stage, 0);
        assert_eq!(status.traffic_percentage, 0.0);
    }

    #[test]
    fn test_create_canary_duplicate_name() {
        let mut service = CanaryService::new();
        service.create_canary("test-release", "v1.0.0", "v2.0.0").unwrap();

        let result = service.create_canary("test-release", "v1.0.0", "v2.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_canaries() {
        let mut service = CanaryService::new();
        service.create_canary("release-1", "v1.0.0", "v1.1.0").unwrap();
        service.create_canary("release-2", "v2.0.0", "v2.1.0").unwrap();

        let canaries = service.list_canaries();
        assert_eq!(canaries.len(), 2);
        assert!(canaries.iter().any(|c| c.name == "release-1"));
        assert!(canaries.iter().any(|c| c.name == "release-2"));
    }

    #[test]
    fn test_promote_canary() {
        let mut service = CanaryService::new();
        service.create_canary("test-release", "v1.0.0", "v2.0.0").unwrap();
        service.start_canary("test-release").unwrap();

        let result = service.promote_canary("test-release");
        assert!(result.is_ok());

        let status = result.unwrap();
        assert_eq!(status.status, "Promoted");
        assert_eq!(status.current_stage, 100);
        assert_eq!(status.traffic_percentage, 100.0);
    }

    #[test]
    fn test_promote_canary_not_found() {
        let mut service = CanaryService::new();
        let result = service.promote_canary("non-existent");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_rollback_canary() {
        let mut service = CanaryService::new();
        service.create_canary("test-release", "v1.0.0", "v2.0.0").unwrap();
        service.start_canary("test-release").unwrap();

        let result = service.rollback_canary("test-release", "test rollback");
        assert!(result.is_ok());

        let status = result.unwrap();
        assert_eq!(status.status, "RolledBack");
        assert_eq!(status.current_stage, 0);
        assert_eq!(status.traffic_percentage, 0.0);
    }

    #[test]
    fn test_get_canary_status() {
        let mut service = CanaryService::new();
        service.create_canary("test-release", "v1.0.0", "v2.0.0").unwrap();

        let result = service.get_canary_status("test-release");
        assert!(result.is_ok());

        let status = result.unwrap();
        assert_eq!(status.name, "test-release");
        assert_eq!(status.status, "Created");
    }

    #[test]
    fn test_get_canary_status_not_found() {
        let service = CanaryService::new();
        let result = service.get_canary_status("non-existent");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_format_status() {
        let mut service = CanaryService::new();
        let status = service
            .create_canary("test-release", "v1.0.0", "v2.0.0")
            .unwrap();

        let formatted = service.format_status(&status);
        assert!(formatted.contains("test-release"));
        assert!(formatted.contains("v1.0.0"));
        assert!(formatted.contains("v2.0.0"));
        assert!(formatted.contains("Created"));
    }

    #[test]
    fn test_format_list() {
        let mut service = CanaryService::new();
        service.create_canary("release-1", "v1.0.0", "v1.1.0").unwrap();

        let canaries = service.list_canaries();
        let formatted = service.format_list(&canaries);

        assert!(formatted.contains("release-1"));
        assert!(formatted.contains("v1.0.0"));
        assert!(formatted.contains("v1.1.0"));
    }
}