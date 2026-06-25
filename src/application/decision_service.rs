use crate::application::ports::decision_store::DecisionStorePort;
use crate::domain::decision::{
    Alternative, Consequence, DecisionCategory, DecisionMetrics, DecisionRecord, DecisionStatus,
    ImpactLevel,
};
use crate::domain::errors::CellResult;
use chrono::Utc;
use std::collections::HashMap;

pub struct DecisionService<T: DecisionStorePort> {
    store: T,
}

impl<T: DecisionStorePort> DecisionService<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }

    pub fn record_decision(
        &self,
        path: &str,
        title: &str,
        context: &str,
        decision: &str,
        rationale: &str,
        category: DecisionCategory,
        made_by: Option<&str>,
    ) -> CellResult<DecisionRecord> {
        let mut record = DecisionRecord::new(title, context, decision, rationale, category);
        record.made_by = made_by.map(|s| s.to_string());
        self.store.save(path, &record)?;
        Ok(record)
    }

    pub fn add_alternative(
        &self,
        path: &str,
        decision_id: &str,
        name: &str,
        description: &str,
        pros: Vec<String>,
        cons: Vec<String>,
    ) -> CellResult<DecisionRecord> {
        let mut decision = self
            .store
            .load_by_id(path, decision_id)?
            .ok_or_else(|| crate::domain::errors::CellError::Config("Decision not found".to_string()))?;

        decision.alternatives.push(Alternative {
            name: name.to_string(),
            description: description.to_string(),
            pros,
            cons,
        });
        decision.updated_at = Utc::now();
        self.store.save(path, &decision)?;
        Ok(decision)
    }

    pub fn add_consequence(
        &self,
        path: &str,
        decision_id: &str,
        description: &str,
        impact: ImpactLevel,
        certainty: f64,
    ) -> CellResult<DecisionRecord> {
        let mut decision = self
            .store
            .load_by_id(path, decision_id)?
            .ok_or_else(|| crate::domain::errors::CellError::Config("Decision not found".to_string()))?;

        decision.consequences.push(Consequence {
            description: description.to_string(),
            impact,
            certainty,
        });
        decision.updated_at = Utc::now();
        self.store.save(path, &decision)?;
        Ok(decision)
    }

    pub fn add_tag(&self, path: &str, decision_id: &str, tag: &str) -> CellResult<DecisionRecord> {
        let mut decision = self
            .store
            .load_by_id(path, decision_id)?
            .ok_or_else(|| crate::domain::errors::CellError::Config("Decision not found".to_string()))?;

        if !decision.tags.iter().any(|t| t == tag) {
            decision.tags.push(tag.to_string());
            decision.updated_at = Utc::now();
            self.store.save(path, &decision)?;
        }
        Ok(decision)
    }

    pub fn add_related_file(
        &self,
        path: &str,
        decision_id: &str,
        file: &str,
    ) -> CellResult<DecisionRecord> {
        let mut decision = self
            .store
            .load_by_id(path, decision_id)?
            .ok_or_else(|| crate::domain::errors::CellError::Config("Decision not found".to_string()))?;

        if !decision.related_files.iter().any(|f| f == file) {
            decision.related_files.push(file.to_string());
            decision.updated_at = Utc::now();
            self.store.save(path, &decision)?;
        }
        Ok(decision)
    }

    pub fn update_status(
        &self,
        path: &str,
        decision_id: &str,
        status: DecisionStatus,
    ) -> CellResult<DecisionRecord> {
        let mut decision = self
            .store
            .load_by_id(path, decision_id)?
            .ok_or_else(|| crate::domain::errors::CellError::Config("Decision not found".to_string()))?;

        decision.status = status.clone();
        decision.updated_at = Utc::now();
        if status == DecisionStatus::Deprecated || status == DecisionStatus::Superseded {
            decision.revoked_at = Some(Utc::now());
        }
        self.store.save(path, &decision)?;
        Ok(decision)
    }

    pub fn list_decisions(
        &self,
        path: &str,
        category: Option<DecisionCategory>,
        status: Option<DecisionStatus>,
    ) -> CellResult<Vec<DecisionRecord>> {
        let mut decisions = self.store.load_all(path)?;

        if let Some(cat) = category {
            decisions.retain(|d| d.category == cat);
        }
        if let Some(stat) = status {
            decisions.retain(|d| d.status == stat);
        }

        decisions.sort_by(|a, b| b.made_at.cmp(&a.made_at));
        Ok(decisions)
    }

    pub fn get_decision(&self, path: &str, id: &str) -> CellResult<Option<DecisionRecord>> {
        self.store.load_by_id(path, id)
    }

    pub fn get_metrics(&self, path: &str) -> CellResult<DecisionMetrics> {
        self.store.get_metrics(path)
    }

    pub fn export_markdown(&self, path: &str, output_path: &str) -> CellResult<String> {
        let decisions = self.store.load_all(path)?;
        let mut content = String::new();

        content.push_str("# 架构决策记录 (ADR)\n\n");
        content.push_str(&format!("共 {} 条决策记录\n\n", decisions.len()));

        for d in &decisions {
            content.push_str(&d.to_markdown());
            content.push_str("\n---\n\n");
        }

        std::fs::write(output_path, &content)?;
        Ok(output_path.to_string())
    }
}

pub fn calculate_decision_metrics(decisions: &[DecisionRecord]) -> DecisionMetrics {
    let now = Utc::now();
    let seven_days_ago = now - chrono::Duration::days(7);
    let thirty_days_ago = now - chrono::Duration::days(30);

    let mut by_category: HashMap<DecisionCategory, usize> = HashMap::new();
    let mut accepted_count = 0;
    let mut rejected_count = 0;
    let mut superseded_count = 0;
    let mut last_7_days = 0;
    let mut last_30_days = 0;

    for d in decisions {
        *by_category.entry(d.category.clone()).or_insert(0) += 1;

        match d.status {
            DecisionStatus::Accepted => accepted_count += 1,
            DecisionStatus::Rejected => rejected_count += 1,
            DecisionStatus::Superseded => superseded_count += 1,
            _ => {}
        }

        if d.made_at >= seven_days_ago {
            last_7_days += 1;
        }
        if d.made_at >= thirty_days_ago {
            last_30_days += 1;
        }
    }

    DecisionMetrics {
        total_decisions: decisions.len(),
        accepted_count,
        rejected_count,
        superseded_count,
        by_category,
        last_7_days,
        last_30_days,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockDecisionStore {
        decisions: Mutex<Vec<DecisionRecord>>,
    }

    impl MockDecisionStore {
        fn new() -> Self {
            Self {
                decisions: Mutex::new(Vec::new()),
            }
        }
    }

    impl DecisionStorePort for MockDecisionStore {
        fn save(&self, _path: &str, decision: &DecisionRecord) -> CellResult<()> {
            let mut decs = self.decisions.lock().unwrap();
            if let Some(pos) = decs.iter().position(|d| d.id == decision.id) {
                decs[pos] = decision.clone();
            } else {
                decs.push(decision.clone());
            }
            Ok(())
        }

        fn load_all(&self, _path: &str) -> CellResult<Vec<DecisionRecord>> {
            Ok(self.decisions.lock().unwrap().clone())
        }

        fn load_by_id(&self, _path: &str, id: &str) -> CellResult<Option<DecisionRecord>> {
            Ok(self
                .decisions
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.id.to_string() == id || crate::domain::decision::simple_id(&d.id) == id.to_uppercase())
                .cloned())
        }

        fn delete(&self, _path: &str, _id: &str) -> CellResult<()> {
            Ok(())
        }

        fn get_metrics(&self, path: &str) -> CellResult<DecisionMetrics> {
            let decisions = self.load_all(path)?;
            Ok(calculate_decision_metrics(&decisions))
        }
    }

    #[test]
    fn test_record_decision() {
        let store = MockDecisionStore::new();
        let service = DecisionService::new(store);

        let result = service.record_decision(
            ".",
            "测试决策",
            "背景",
            "决策内容",
            "理由",
            DecisionCategory::Architecture,
            Some("agent-1"),
        );

        assert!(result.is_ok());
        let d = result.unwrap();
        assert_eq!(d.title, "测试决策");
        assert_eq!(d.status, DecisionStatus::Accepted);
        assert_eq!(d.made_by, Some("agent-1".to_string()));
    }

    #[test]
    fn test_list_decisions() {
        let store = MockDecisionStore::new();
        let service = DecisionService::new(store);

        service
            .record_decision(".", "决策1", "", "", "", DecisionCategory::Architecture, None)
            .unwrap();
        service
            .record_decision(".", "决策2", "", "", "", DecisionCategory::Technology, None)
            .unwrap();

        let all = service.list_decisions(".", None, None).unwrap();
        assert_eq!(all.len(), 2);

        let arch = service
            .list_decisions(".", Some(DecisionCategory::Architecture), None)
            .unwrap();
        assert_eq!(arch.len(), 1);
    }

    #[test]
    fn test_calculate_metrics() {
        let store = MockDecisionStore::new();
        let service = DecisionService::new(store);

        service
            .record_decision(".", "决策1", "", "", "", DecisionCategory::Architecture, None)
            .unwrap();
        service
            .record_decision(".", "决策2", "", "", "", DecisionCategory::Technology, None)
            .unwrap();

        let metrics = service.get_metrics(".").unwrap();
        assert_eq!(metrics.total_decisions, 2);
        assert_eq!(metrics.accepted_count, 2);
        assert!(metrics.last_7_days >= 2);
    }
}
