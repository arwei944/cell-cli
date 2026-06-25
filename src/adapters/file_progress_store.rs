use crate::application::ports::progress_store::ProgressStorePort;
use crate::domain::errors::CellResult;
use crate::domain::progress::ProgressLog;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileProgressStore;

impl FileProgressStore {
    pub fn new() -> Self {
        Self
    }

    fn progress_dir(project_path: &str) -> PathBuf {
        Path::new(project_path).join(".cell").join("progress")
    }

    fn current_file(project_path: &str) -> PathBuf {
        Self::progress_dir(project_path).join("current.json")
    }

    fn history_dir(project_path: &str) -> PathBuf {
        Self::progress_dir(project_path).join("history")
    }

    fn ensure_dir(path: &Path) -> CellResult<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

impl Default for FileProgressStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressStorePort for FileProgressStore {
    fn load_current(&self, project_path: &str) -> CellResult<Option<ProgressLog>> {
        let path = Self::current_file(project_path);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        let log: ProgressLog = serde_json::from_str(&content)?;
        Ok(Some(log))
    }

    fn save_current(&self, project_path: &str, log: &ProgressLog) -> CellResult<()> {
        let dir = Self::progress_dir(project_path);
        Self::ensure_dir(&dir)?;
        let path = Self::current_file(project_path);
        let content = serde_json::to_string_pretty(log)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn list_history(&self, project_path: &str) -> CellResult<Vec<ProgressLog>> {
        let dir = Self::history_dir(project_path);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut logs = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(log) = serde_json::from_str::<ProgressLog>(&content) {
                    logs.push(log);
                }
            }
        }
        logs.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        Ok(logs)
    }

    fn archive(&self, project_path: &str, log: &ProgressLog) -> CellResult<()> {
        let dir = Self::history_dir(project_path);
        Self::ensure_dir(&dir)?;
        let filename = format!(
            "{}_{}.json",
            log.started_at.format("%Y%m%d_%H%M%S"),
            log.task_name.replace(|c: char| !c.is_alphanumeric() && c != '-', "_")
        );
        let path = dir.join(&filename);
        let content = serde_json::to_string_pretty(log)?;
        fs::write(&path, content)?;

        let current_path = Self::current_file(project_path);
        if current_path.exists() {
            fs::remove_file(&current_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::progress::ProgressStatus;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_current() {
        let dir = tempdir().unwrap();
        let store = FileProgressStore::new();
        let mut log = ProgressLog::new("test-task", "A test task");
        log.status = ProgressStatus::InProgress;

        store.save_current(dir.path().to_str().unwrap(), &log).unwrap();
        let loaded = store.load_current(dir.path().to_str().unwrap()).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().task_name, "test-task");
    }

    #[test]
    fn test_archive_moves_to_history() {
        let dir = tempdir().unwrap();
        let store = FileProgressStore::new();
        let mut log = ProgressLog::new("archive-test", "Test archiving");
        log.status = ProgressStatus::Completed;

        store.save_current(dir.path().to_str().unwrap(), &log).unwrap();
        store.archive(dir.path().to_str().unwrap(), &log).unwrap();

        let current = store.load_current(dir.path().to_str().unwrap()).unwrap();
        assert!(current.is_none());

        let history = store.list_history(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].task_name, "archive-test");
    }

    #[test]
    fn test_load_none_when_no_current() {
        let dir = tempdir().unwrap();
        let store = FileProgressStore::new();
        let result = store.load_current(dir.path().to_str().unwrap()).unwrap();
        assert!(result.is_none());
    }
}
