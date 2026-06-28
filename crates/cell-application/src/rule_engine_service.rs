use cell_domain::errors::CellResult;
use cell_domain::rule_engine::{
    ActionResult, Rule, RuleEngine, RuleType, RuleStatus, RuleVersion,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInfo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub status: RuleStatus,
    pub priority: i32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Rule> for RuleInfo {
    fn from(rule: &Rule) -> Self {
        Self {
            id: rule.id,
            name: rule.name.clone(),
            description: rule.description.clone(),
            rule_type: rule.rule_type.clone(),
            status: rule.status.clone(),
            priority: rule.priority,
            tags: rule.tags.clone(),
            created_at: rule.created_at,
            updated_at: rule.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: u32,
    pub change_note: String,
    pub created_at: DateTime<Utc>,
}

impl From<&RuleVersion> for VersionInfo {
    fn from(version: &RuleVersion) -> Self {
        Self {
            version: version.version,
            change_note: version.change_note.clone(),
            created_at: version.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResponse {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub matched: bool,
    pub action_results: Vec<ActionResult>,
    pub final_context: HashMap<String, serde_json::Value>,
}

pub struct RuleEngineService {
    engine: RuleEngine,
}

impl RuleEngineService {
    pub fn new() -> Self {
        Self { engine: RuleEngine::new() }
    }

    pub fn list_rules(&self) -> Vec<RuleInfo> {
        self.engine.list_rules().into_iter().map(RuleInfo::from).collect()
    }

    pub fn evaluate_rule(
        &self,
        rule_id: Uuid,
        data: HashMap<String, serde_json::Value>,
    ) -> CellResult<EvaluationResponse> {
        let mut context = data;
        let result = self.engine.evaluate_rule(rule_id, &mut context)?;
        Ok(EvaluationResponse {
            rule_id: result.rule_id,
            rule_name: result.rule_name,
            matched: result.matched,
            action_results: result.action_results,
            final_context: context,
        })
    }

    pub fn activate_rule(&mut self, rule_id: Uuid) -> CellResult<()> {
        self.engine.activate_rule(rule_id)?;
        Ok(())
    }

    pub fn get_rule_version_history(&self, rule_id: Uuid) -> CellResult<Vec<VersionInfo>> {
        let versions = self.engine.get_versions(rule_id)?;
        Ok(versions.iter().map(VersionInfo::from).collect())
    }

    pub fn create_rule(&mut self, request: CreateRuleRequest) -> CellResult<RuleInfo> {
        let mut rule = Rule::new(request.name, request.description, request.rule_type);
        rule.priority = request.priority.unwrap_or(0);
        rule.tags = request.tags.unwrap_or_default();
        self.engine.register_rule(rule.clone())?;
        Ok(RuleInfo::from(&rule))
    }

    pub fn get_rule(&self, rule_id: Uuid) -> CellResult<RuleInfo> {
        let rule = self.engine.get_rule(rule_id)?;
        Ok(RuleInfo::from(rule))
    }
}

impl Default for RuleEngineService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_rule() {
        let mut service = RuleEngineService::new();
        let request = CreateRuleRequest {
            name: "test".to_string(),
            description: "desc".to_string(),
            rule_type: RuleType::Validation,
            priority: Some(10),
            tags: Some(vec!["tag".to_string()]),
        };
        let result = service.create_rule(request).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.priority, 10);
    }

    #[test]
    fn test_list_rules() {
        let mut service = RuleEngineService::new();
        service.create_rule(CreateRuleRequest {
            name: "r1".to_string(),
            description: "d1".to_string(),
            rule_type: RuleType::Validation,
            priority: None,
            tags: None,
        }).unwrap();
        service.create_rule(CreateRuleRequest {
            name: "r2".to_string(),
            description: "d2".to_string(),
            rule_type: RuleType::Calculation,
            priority: None,
            tags: None,
        }).unwrap();
        let rules = service.list_rules();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_activate_rule() {
        let mut service = RuleEngineService::new();
        let rule = service.create_rule(CreateRuleRequest {
            name: "r".to_string(),
            description: "d".to_string(),
            rule_type: RuleType::Validation,
            priority: None,
            tags: None,
        }).unwrap();
        service.activate_rule(rule.id).unwrap();
        let updated = service.get_rule(rule.id).unwrap();
        assert_eq!(updated.status, RuleStatus::Active);
    }

    #[test]
    fn test_evaluate_rule() {
        let mut service = RuleEngineService::new();
        let rule = service.create_rule(CreateRuleRequest {
            name: "r".to_string(),
            description: "d".to_string(),
            rule_type: RuleType::Validation,
            priority: None,
            tags: None,
        }).unwrap();
        service.activate_rule(rule.id).unwrap();
        let mut ctx: HashMap<String, serde_json::Value> = HashMap::new();
        ctx.insert("score".to_string(), json!(80));
        let result = service.evaluate_rule(rule.id, ctx).unwrap();
        assert!(result.matched);
    }

    #[test]
    fn test_version_history() {
        let mut service = RuleEngineService::new();
        let rule = service.create_rule(CreateRuleRequest {
            name: "r".to_string(),
            description: "d".to_string(),
            rule_type: RuleType::Decision,
            priority: None,
            tags: None,
        }).unwrap();
        let history = service.get_rule_version_history(rule.id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].version, 1);
    }
}