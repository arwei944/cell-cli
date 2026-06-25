use crate::application::ports::evolution_store::EvolutionStorePort;
use crate::domain::errors::CellResult;
use crate::domain::evolution::EvolutionLog;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileEvolutionStore;

impl FileEvolutionStore {
    pub fn new() -> Self {
        Self
    }

    fn evolution_dir(project_path: &str) -> PathBuf {
        Path::new(project_path).join(".cell").join("evolution")
    }

    fn current_file(project_path: &str) -> PathBuf {
        Self::evolution_dir(project_path).join("current.json")
    }

    fn history_dir(project_path: &str) -> PathBuf {
        Self::evolution_dir(project_path).join("history")
    }

    fn ensure_dir(path: &Path) -> CellResult<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

impl Default for FileEvolutionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionStorePort for FileEvolutionStore {
    fn load_current_cycle(&self, project_path: &str) -> CellResult<Option<EvolutionLog>> {
        let path = Self::current_file(project_path);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        let log: EvolutionLog = serde_json::from_str(&content)?;
        Ok(Some(log))
    }

    fn save_current_cycle(&self, project_path: &str, log: &EvolutionLog) -> CellResult<()> {
        let dir = Self::evolution_dir(project_path);
        Self::ensure_dir(&dir)?;
        let path = Self::current_file(project_path);
        let content = serde_json::to_string_pretty(log)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn list_history(&self, project_path: &str) -> CellResult<Vec<EvolutionLog>> {
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
                if let Ok(log) = serde_json::from_str::<EvolutionLog>(&content) {
                    logs.push(log);
                }
            }
        }
        logs.sort_by_key(|l| std::cmp::Reverse(l.cycle_number));
        Ok(logs)
    }

    fn archive_cycle(&self, project_path: &str, log: &EvolutionLog) -> CellResult<()> {
        let dir = Self::history_dir(project_path);
        Self::ensure_dir(&dir)?;
        let filename = format!("cycle_{:04}.json", log.cycle_number);
        let path = dir.join(&filename);
        let content = serde_json::to_string_pretty(log)?;
        fs::write(&path, content)?;

        let current_path = Self::current_file(project_path);
        if current_path.exists() {
            fs::remove_file(&current_path)?;
        }
        Ok(())
    }

    fn get_next_cycle_number(&self, project_path: &str) -> CellResult<u32> {
        let history = self.list_history(project_path)?;
        let max_num = history.iter().map(|l| l.cycle_number).max().unwrap_or(0);
        Ok(max_num + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_current() {
        let dir = tempdir().unwrap();
        let store = FileEvolutionStore::new();
        let log = EvolutionLog::new(1);

        store.save_current_cycle(dir.path().to_str().unwrap(), &log).unwrap();
        let loaded = store.load_current_cycle(dir.path().to_str().unwrap()).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().cycle_number, 1);
    }

    #[test]
    fn test_archive_and_history() {
        let dir = tempdir().unwrap();
        let store = FileEvolutionStore::new();

        let mut log = EvolutionLog::new(1);
        log.phase = crate::domain::evolution::EvolutionPhase::Completed;
        store.archive_cycle(dir.path().to_str().unwrap(), &log).unwrap();

        let history = store.list_history(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].cycle_number, 1);
    }

    #[test]
    fn test_next_cycle_number() {
        let dir = tempdir().unwrap();
        let store = FileEvolutionStore::new();

        assert_eq!(store.get_next_cycle_number(dir.path().to_str().unwrap()).unwrap(), 1);

        let mut log = EvolutionLog::new(1);
        log.phase = crate::domain::evolution::EvolutionPhase::Completed;
        store.archive_cycle(dir.path().to_str().unwrap(), &log).unwrap();

        assert_eq!(store.get_next_cycle_number(dir.path().to_str().unwrap()).unwrap(), 2);
    }
}
