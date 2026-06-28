use cell_application::decision_service::calculate_decision_metrics;
use cell_application::ports::decision_store::DecisionStorePort;
use cell_domain::decision::{DecisionMetrics, DecisionRecord};
use cell_domain::decision::simple_id;
use cell_domain::errors::CellResult;
use std::path::Path;

pub struct FileDecisionStore;

impl FileDecisionStore {
    pub fn new() -> Self {
        Self
    }

    fn decisions_dir(path: &str) -> std::path::PathBuf {
        Path::new(path).join(".cell").join("decisions")
    }
}

impl Default for FileDecisionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionStorePort for FileDecisionStore {
    fn save(&self, path: &str, decision: &DecisionRecord) -> CellResult<()> {
        let dir = Self::decisions_dir(path);
        std::fs::create_dir_all(&dir)?;

        let file_path = dir.join(format!("{}.json", decision.id));
        let content = serde_json::to_string_pretty(decision)?;
        std::fs::write(file_path, content)?;

        Ok(())
    }

    fn load_all(&self, path: &str) -> CellResult<Vec<DecisionRecord>> {
        let dir = Self::decisions_dir(path);
        let mut decisions = Vec::new();

        if !dir.exists() {
            return Ok(decisions);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&file_path)?;
                if let Ok(decision) = serde_json::from_str::<DecisionRecord>(&content) {
                    decisions.push(decision);
                }
            }
        }

        Ok(decisions)
    }

    fn load_by_id(&self, path: &str, id: &str) -> CellResult<Option<DecisionRecord>> {
        let all = self.load_all(path)?;
        let id_lower = id.to_lowercase();
        Ok(all.into_iter().find(|d| {
            let full = d.id.to_string();
            let full_nohyphen = full.replace('-', "");
            let simple = simple_id(&d.id).to_lowercase();
            full == id || full_nohyphen == id_lower || simple == id_lower
        }))
    }

    fn delete(&self, path: &str, id: &str) -> CellResult<()> {
        let dir = Self::decisions_dir(path);
        if let Ok(Some(decision)) = self.load_by_id(path, id) {
            let file_path = dir.join(format!("{}.json", decision.id));
            if file_path.exists() {
                std::fs::remove_file(file_path)?;
            }
        }
        Ok(())
    }

    fn get_metrics(&self, path: &str) -> CellResult<DecisionMetrics> {
        let decisions = self.load_all(path)?;
        Ok(calculate_decision_metrics(&decisions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_domain::decision::DecisionCategory;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let store = FileDecisionStore::new();

        let decision = DecisionRecord::new(
            "测试决策",
            "背景",
            "决策",
            "理由",
            DecisionCategory::Architecture,
        );

        store.save(dir.path().to_str().unwrap(), &decision).unwrap();

        let loaded = store.load_all(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "测试决策");
    }

    #[test]
    fn test_load_by_id() {
        let dir = tempdir().unwrap();
        let store = FileDecisionStore::new();

        let decision = DecisionRecord::new(
            "测试决策",
            "背景",
            "决策",
            "理由",
            DecisionCategory::Architecture,
        );

        store.save(dir.path().to_str().unwrap(), &decision).unwrap();

        let loaded = store
            .load_by_id(dir.path().to_str().unwrap(), &decision.id.to_string())
            .unwrap();
        assert!(loaded.is_some());
    }

    #[test]
    fn test_metrics() {
        let dir = tempdir().unwrap();
        let store = FileDecisionStore::new();

        let d1 = DecisionRecord::new("决策1", "", "", "", DecisionCategory::Architecture);
        let d2 = DecisionRecord::new("决策2", "", "", "", DecisionCategory::Technology);

        store.save(dir.path().to_str().unwrap(), &d1).unwrap();
        store.save(dir.path().to_str().unwrap(), &d2).unwrap();

        let metrics = store.get_metrics(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(metrics.total_decisions, 2);
    }
}
