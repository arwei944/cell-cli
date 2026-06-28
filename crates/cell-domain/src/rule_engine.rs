use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RuleStatus {
    #[default]
    Draft,
    Active,
    Deprecated,
    Archived,
}


impl fmt::Display for RuleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Active => write!(f, "Active"),
            Self::Deprecated => write!(f, "Deprecated"),
            Self::Archived => write!(f, "Archived"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RuleType {
    #[default]
    Validation,
    Calculation,
    Transformation,
    Decision,
    Notification,
}


impl fmt::Display for RuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation => write!(f, "Validation"),
            Self::Calculation => write!(f, "Calculation"),
            Self::Transformation => write!(f, "Transformation"),
            Self::Decision => write!(f, "Decision"),
            Self::Notification => write!(f, "Notification"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
}

impl fmt::Display for ConditionOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::GreaterThan => write!(f, ">"),
            Self::LessThan => write!(f, "<"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::Contains => write!(f, "contains"),
            Self::NotContains => write!(f, "not_contains"),
            Self::StartsWith => write!(f, "starts_with"),
            Self::EndsWith => write!(f, "ends_with"),
            Self::In => write!(f, "in"),
            Self::NotIn => write!(f, "not_in"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleCondition {
    pub key: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

impl RuleCondition {
    pub fn new(key: impl Into<String>, operator: ConditionOperator, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            operator,
            value,
        }
    }

    pub fn evaluate(&self, context: &HashMap<String, serde_json::Value>) -> bool {
        let Some(ctx_value) = context.get(&self.key) else {
            return false;
        };
        self.compare(ctx_value, &self.value)
    }

    fn compare(&self, left: &serde_json::Value, right: &serde_json::Value) -> bool {
        match &self.operator {
            ConditionOperator::Equal => left == right,
            ConditionOperator::NotEqual => left != right,
            ConditionOperator::GreaterThan => Self::num_compare(left, right, |a, b| a > b),
            ConditionOperator::LessThan => Self::num_compare(left, right, |a, b| a < b),
            ConditionOperator::GreaterThanOrEqual => {
                Self::num_compare(left, right, |a, b| a >= b)
            }
            ConditionOperator::LessThanOrEqual => {
                Self::num_compare(left, right, |a, b| a <= b)
            }
            ConditionOperator::Contains => Self::string_contains(left, right, true),
            ConditionOperator::NotContains => Self::string_contains(left, right, false),
            ConditionOperator::StartsWith => Self::string_starts_with(left, right),
            ConditionOperator::EndsWith => Self::string_ends_with(left, right),
            ConditionOperator::In => Self::in_list(left, right),
            ConditionOperator::NotIn => !Self::in_list(left, right),
        }
    }

    fn num_compare<F>(left: &serde_json::Value, right: &serde_json::Value, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let l = left.as_f64().unwrap_or(0.0);
        let r = right.as_f64().unwrap_or(0.0);
        cmp(l, r)
    }

    fn string_contains(left: &serde_json::Value, right: &serde_json::Value, contains: bool) -> bool {
        let l = left.as_str().unwrap_or("");
        let r = right.as_str().unwrap_or("");
        if contains {
            l.contains(r)
        } else {
            !l.contains(r)
        }
    }

    fn string_starts_with(left: &serde_json::Value, right: &serde_json::Value) -> bool {
        let l = left.as_str().unwrap_or("");
        let r = right.as_str().unwrap_or("");
        l.starts_with(r)
    }

    fn string_ends_with(left: &serde_json::Value, right: &serde_json::Value) -> bool {
        let l = left.as_str().unwrap_or("");
        let r = right.as_str().unwrap_or("");
        l.ends_with(r)
    }

    fn in_list(left: &serde_json::Value, right: &serde_json::Value) -> bool {
        right
            .as_array()
            .is_some_and(|arr| arr.iter().any(|v| v == left))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    SetValue,
    AddValue,
    MultiplyValue,
    SendNotification,
    LogMessage,
    ReturnResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub target: Option<String>,
    pub value: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

impl RuleAction {
    pub fn new(
        action_type: ActionType,
        target: Option<String>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            action_type,
            target,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn execute(&self, context: &mut HashMap<String, serde_json::Value>) -> Vec<ActionResult> {
        let mut results = Vec::new();
        match &self.action_type {
            ActionType::SetValue => {
                if let Some(target) = &self.target {
                    context.insert(target.clone(), self.value.clone());
                    results.push(ActionResult::ValueChanged {
                        target: target.clone(),
                        old_value: None,
                        new_value: self.value.clone(),
                    });
                }
            }
            ActionType::AddValue => {
                if let Some(target) = &self.target {
                    let old = context.get(target).cloned();
                    let old_num = old.as_ref().and_then(serde_json::Value::as_f64).unwrap_or(0.0);
                    let add_num = self.value.as_f64().unwrap_or(0.0);
                    let new_val = serde_json::Value::from(old_num + add_num);
                    context.insert(target.clone(), new_val.clone());
                    results.push(ActionResult::ValueChanged {
                        target: target.clone(),
                        old_value: old,
                        new_value: new_val,
                    });
                }
            }
            ActionType::MultiplyValue => {
                if let Some(target) = &self.target {
                    let old = context.get(target).cloned();
                    let old_num = old.as_ref().and_then(serde_json::Value::as_f64).unwrap_or(0.0);
                    let mul_num = self.value.as_f64().unwrap_or(1.0);
                    let new_val = serde_json::Value::from(old_num * mul_num);
                    context.insert(target.clone(), new_val.clone());
                    results.push(ActionResult::ValueChanged {
                        target: target.clone(),
                        old_value: old,
                        new_value: new_val,
                    });
                }
            }
            ActionType::SendNotification => {
                results.push(ActionResult::NotificationSent {
                    message: self.value.as_str().unwrap_or("").to_string(),
                    metadata: self.metadata.clone(),
                });
            }
            ActionType::LogMessage => {
                results.push(ActionResult::Logged {
                    message: self.value.as_str().unwrap_or("").to_string(),
                });
            }
            ActionType::ReturnResult => {
                results.push(ActionResult::ResultReturned {
                    value: self.value.clone(),
                });
            }
        }
        results
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionResult {
    ValueChanged {
        target: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    },
    NotificationSent {
        message: String,
        metadata: HashMap<String, String>,
    },
    Logged {
        message: String,
    },
    ResultReturned {
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub status: RuleStatus,
    pub priority: i32,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub dependencies: Vec<Uuid>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Rule {
    pub fn new(name: impl Into<String>, description: impl Into<String>, rule_type: RuleType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            rule_type,
            status: RuleStatus::Draft,
            priority: 0,
            conditions: Vec::new(),
            actions: Vec::new(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            created_by: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_status(mut self, status: RuleStatus) -> Self {
        self.status = status;
        self
    }

    pub fn add_condition(&mut self, condition: RuleCondition) {
        self.conditions.push(condition);
        self.updated_at = Utc::now();
    }

    pub fn add_action(&mut self, action: RuleAction) {
        self.actions.push(action);
        self.updated_at = Utc::now();
    }

    pub fn add_dependency(&mut self, rule_id: Uuid) {
        if !self.dependencies.contains(&rule_id) {
            self.dependencies.push(rule_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn is_draft(&self) -> bool {
        self.status == RuleStatus::Draft
    }

    pub fn is_active(&self) -> bool {
        self.status == RuleStatus::Active
    }

    pub fn is_deprecated(&self) -> bool {
        self.status == RuleStatus::Deprecated
    }

    pub fn is_archived(&self) -> bool {
        self.status == RuleStatus::Archived
    }

    pub fn evaluate_conditions(&self, context: &HashMap<String, serde_json::Value>) -> bool {
        if self.conditions.is_empty() {
            return true;
        }
        self.conditions.iter().all(|c| c.evaluate(context))
    }

    pub fn execute_actions(
        &self,
        context: &mut HashMap<String, serde_json::Value>,
    ) -> Vec<ActionResult> {
        let mut results = Vec::new();
        for action in &self.actions {
            results.extend(action.execute(context));
        }
        results
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    pub version: u32,
    pub rule: Rule,
    pub change_note: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

impl RuleVersion {
    pub fn new(rule: Rule, version: u32, change_note: impl Into<String>) -> Self {
        Self {
            version,
            rule,
            change_note: change_note.into(),
            created_at: Utc::now(),
            created_by: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Created,
    Updated,
    Activated,
    Deactivated,
    Deprecated,
    Archived,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleChange {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub change_type: ChangeType,
    pub from_version: Option<u32>,
    pub to_version: Option<u32>,
    pub description: String,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl RuleChange {
    pub fn new(
        rule_id: Uuid,
        change_type: ChangeType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id,
            change_type,
            from_version: None,
            to_version: None,
            description: description.into(),
            changed_at: Utc::now(),
            changed_by: None,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_ids: Vec<Uuid>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl RuleSet {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            rule_ids: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule_id: Uuid) {
        if !self.rule_ids.contains(&rule_id) {
            self.rule_ids.push(rule_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_rule(&mut self, rule_id: Uuid) {
        self.rule_ids.retain(|id| id != &rule_id);
        self.updated_at = Utc::now();
    }

    pub fn contains(&self, rule_id: Uuid) -> bool {
        self.rule_ids.contains(&rule_id)
    }

    pub fn rule_count(&self) -> usize {
        self.rule_ids.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleEngineError {
    RuleNotFound(String),
    RuleSetNotFound(String),
    RuleAlreadyExists(String),
    RuleSetAlreadyExists(String),
    VersionNotFound(String),
    InvalidStatusTransition(String),
    CircularDependency(String),
    DependencyNotFound(String),
    EvaluationFailed(String),
}

impl fmt::Display for RuleEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuleNotFound(id) => write!(f, "Rule not found: {id}"),
            Self::RuleSetNotFound(id) => write!(f, "Rule set not found: {id}"),
            Self::RuleAlreadyExists(id) => write!(f, "Rule already exists: {id}"),
            Self::RuleSetAlreadyExists(id) => {
                write!(f, "Rule set already exists: {id}")
            }
            Self::VersionNotFound(id) => write!(f, "Version not found: {id}"),
            Self::InvalidStatusTransition(msg) => {
                write!(f, "Invalid status transition: {msg}")
            }
            Self::CircularDependency(id) => {
                write!(f, "Circular dependency detected: {id}")
            }
            Self::DependencyNotFound(id) => {
                write!(f, "Dependency not found: {id}")
            }
            Self::EvaluationFailed(msg) => {
                write!(f, "Evaluation failed: {msg}")
            }
        }
    }
}

impl std::error::Error for RuleEngineError {}

pub type RuleEngineResult<T> = Result<T, RuleEngineError>;

#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub matched: bool,
    pub action_results: Vec<ActionResult>,
    pub evaluation_order: usize,
}

#[derive(Debug, Clone)]
pub struct RuleEngine {
    rules: HashMap<Uuid, Rule>,
    rule_versions: HashMap<Uuid, Vec<RuleVersion>>,
    current_versions: HashMap<Uuid, u32>,
    rule_sets: HashMap<Uuid, RuleSet>,
    changes: Vec<RuleChange>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            rule_versions: HashMap::new(),
            current_versions: HashMap::new(),
            rule_sets: HashMap::new(),
            changes: Vec::new(),
        }
    }

    pub fn register_rule(&mut self, rule: Rule) -> RuleEngineResult<Uuid> {
        let rule_id = rule.id;
        if self.rules.contains_key(&rule_id) {
            return Err(RuleEngineError::RuleAlreadyExists(rule_id.to_string()));
        }

        let initial_version = RuleVersion::new(rule.clone(), 1, "Initial version");
        self.rules.insert(rule_id, rule.clone());
        self.rule_versions.insert(rule_id, vec![initial_version]);
        self.current_versions.insert(rule_id, 1);

        self.log_change(
            rule_id,
            ChangeType::Created,
            format!("Rule '{}' created", rule.name),
            None,
            Some(1),
        );

        Ok(rule_id)
    }

    pub fn get_rule(&self, rule_id: Uuid) -> RuleEngineResult<&Rule> {
        self.rules
            .get(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))
    }

    pub fn update_rule(&mut self, rule_id: Uuid, updated_rule: Rule) -> RuleEngineResult<u32> {
        let current_version = self
            .current_versions
            .get(&rule_id)
            .copied()
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;

        let new_version_num = current_version + 1;
        let new_version = RuleVersion::new(
            updated_rule.clone(),
            new_version_num,
            format!("Updated to version {new_version_num}"),
        );

        let versions = self
            .rule_versions
            .get_mut(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;
        versions.push(new_version);

        self.rules.insert(rule_id, updated_rule);
        self.current_versions.insert(rule_id, new_version_num);

        self.log_change(
            rule_id,
            ChangeType::Updated,
            format!("Rule updated to version {new_version_num}"),
            Some(current_version),
            Some(new_version_num),
        );

        Ok(new_version_num)
    }

    pub fn rollback_to_version(&mut self, rule_id: Uuid, version: u32) -> RuleEngineResult<()> {
        let versions = self
            .rule_versions
            .get(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;

        let target_version = versions
            .iter()
            .find(|v| v.version == version)
            .cloned()
            .ok_or_else(|| {
                RuleEngineError::VersionNotFound(format!(
                    "Version {version} for rule {rule_id}"
                ))
            })?;

        let current_version = self
            .current_versions
            .get(&rule_id)
            .copied()
            .unwrap_or(0);

        self.rules.insert(rule_id, target_version.rule);
        self.current_versions.insert(rule_id, version);

        self.log_change(
            rule_id,
            ChangeType::RolledBack,
            format!("Rolled back to version {version}"),
            Some(current_version),
            Some(version),
        );

        Ok(())
    }

    pub fn get_current_version(&self, rule_id: Uuid) -> RuleEngineResult<u32> {
        self.current_versions
            .get(&rule_id)
            .copied()
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))
    }

    pub fn get_versions(&self, rule_id: Uuid) -> RuleEngineResult<&Vec<RuleVersion>> {
        self.rule_versions
            .get(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))
    }

    pub fn activate_rule(&mut self, rule_id: Uuid) -> RuleEngineResult<()> {
        let rule = self
            .rules
            .get_mut(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;

        match rule.status {
            RuleStatus::Draft | RuleStatus::Deprecated => {
                rule.status = RuleStatus::Active;
                rule.updated_at = Utc::now();
                self.log_change(
                    rule_id,
                    ChangeType::Activated,
                    "Rule activated".to_string(),
                    None,
                    None,
                );
                Ok(())
            }
            _ => Err(RuleEngineError::InvalidStatusTransition(format!(
                "Cannot activate rule with status {}",
                rule.status
            ))),
        }
    }

    pub fn deactivate_rule(&mut self, rule_id: Uuid) -> RuleEngineResult<()> {
        let rule = self
            .rules
            .get_mut(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;

        if rule.status == RuleStatus::Active {
            rule.status = RuleStatus::Draft;
            rule.updated_at = Utc::now();
            self.log_change(
                rule_id,
                ChangeType::Deactivated,
                "Rule deactivated".to_string(),
                None,
                None,
            );
            Ok(())
        } else {
            Err(RuleEngineError::InvalidStatusTransition(format!(
                "Cannot deactivate rule with status {}",
                rule.status
            )))
        }
    }

    pub fn deprecate_rule(&mut self, rule_id: Uuid) -> RuleEngineResult<()> {
        let rule = self
            .rules
            .get_mut(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;

        if rule.status == RuleStatus::Active {
            rule.status = RuleStatus::Deprecated;
            rule.updated_at = Utc::now();
            self.log_change(
                rule_id,
                ChangeType::Deprecated,
                "Rule deprecated".to_string(),
                None,
                None,
            );
            Ok(())
        } else {
            Err(RuleEngineError::InvalidStatusTransition(format!(
                "Cannot deprecate rule with status {}",
                rule.status
            )))
        }
    }

    pub fn archive_rule(&mut self, rule_id: Uuid) -> RuleEngineResult<()> {
        let rule = self
            .rules
            .get_mut(&rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(rule_id.to_string()))?;

        if rule.status == RuleStatus::Deprecated || rule.status == RuleStatus::Draft {
            rule.status = RuleStatus::Archived;
            rule.updated_at = Utc::now();
            self.log_change(
                rule_id,
                ChangeType::Archived,
                "Rule archived".to_string(),
                None,
                None,
            );
            Ok(())
        } else {
            Err(RuleEngineError::InvalidStatusTransition(format!(
                "Cannot archive rule with status {}",
                rule.status
            )))
        }
    }

    pub fn create_rule_set(&mut self, rule_set: RuleSet) -> RuleEngineResult<Uuid> {
        let id = rule_set.id;
        if self.rule_sets.contains_key(&id) {
            return Err(RuleEngineError::RuleSetAlreadyExists(id.to_string()));
        }
        self.rule_sets.insert(id, rule_set);
        Ok(id)
    }

    pub fn get_rule_set(&self, rule_set_id: Uuid) -> RuleEngineResult<&RuleSet> {
        self.rule_sets
            .get(&rule_set_id)
            .ok_or_else(|| RuleEngineError::RuleSetNotFound(rule_set_id.to_string()))
    }

    pub fn get_rules_by_set(&self, rule_set_id: Uuid) -> RuleEngineResult<Vec<&Rule>> {
        let rule_set = self.get_rule_set(rule_set_id)?;
        let mut rules: Vec<&Rule> = rule_set
            .rule_ids
            .iter()
            .filter_map(|id| self.rules.get(id))
            .collect();
        rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        Ok(rules)
    }

    pub fn add_rule_to_set(&mut self, rule_set_id: Uuid, rule_id: Uuid) -> RuleEngineResult<()> {
        let rule_set = self
            .rule_sets
            .get_mut(&rule_set_id)
            .ok_or_else(|| RuleEngineError::RuleSetNotFound(rule_set_id.to_string()))?;

        if !self.rules.contains_key(&rule_id) {
            return Err(RuleEngineError::RuleNotFound(rule_id.to_string()));
        }

        rule_set.add_rule(rule_id);
        Ok(())
    }

    pub fn remove_rule_from_set(
        &mut self,
        rule_set_id: Uuid,
        rule_id: Uuid,
    ) -> RuleEngineResult<()> {
        let rule_set = self
            .rule_sets
            .get_mut(&rule_set_id)
            .ok_or_else(|| RuleEngineError::RuleSetNotFound(rule_set_id.to_string()))?;
        rule_set.remove_rule(rule_id);
        Ok(())
    }

    pub fn evaluate_rule(
        &self,
        rule_id: Uuid,
        context: &mut HashMap<String, serde_json::Value>,
    ) -> RuleEngineResult<RuleEvaluationResult> {
        let rule = self.get_rule(rule_id)?;
        let matched = rule.is_active() && rule.evaluate_conditions(context);
        let action_results = if matched {
            rule.execute_actions(context)
        } else {
            Vec::new()
        };

        Ok(RuleEvaluationResult {
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            matched,
            action_results,
            evaluation_order: 0,
        })
    }

    pub fn evaluate_rules(
        &self,
        rule_ids: &[Uuid],
        context: &mut HashMap<String, serde_json::Value>,
    ) -> RuleEngineResult<Vec<RuleEvaluationResult>> {
        let mut active_rules: Vec<&Rule> = rule_ids
            .iter()
            .filter_map(|id| self.rules.get(id))
            .filter(|r| r.is_active())
            .collect();

        active_rules.sort_by_key(|r| std::cmp::Reverse(r.priority));

        let sorted_ids: Vec<Uuid> = active_rules.iter().map(|r| r.id).collect();
        self.check_dependencies(&sorted_ids)?;

        let mut results = Vec::new();
        for (idx, rule) in active_rules.iter().enumerate() {
            let matched = rule.evaluate_conditions(context);
            let action_results = if matched {
                rule.execute_actions(context)
            } else {
                Vec::new()
            };
            results.push(RuleEvaluationResult {
                rule_id: rule.id,
                rule_name: rule.name.clone(),
                matched,
                action_results,
                evaluation_order: idx + 1,
            });
        }

        Ok(results)
    }

    pub fn evaluate_rule_set(
        &self,
        rule_set_id: Uuid,
        context: &mut HashMap<String, serde_json::Value>,
    ) -> RuleEngineResult<Vec<RuleEvaluationResult>> {
        let rule_set = self.get_rule_set(rule_set_id)?;
        self.evaluate_rules(&rule_set.rule_ids, context)
    }

    fn check_dependencies(&self, rule_ids: &[Uuid]) -> RuleEngineResult<()> {
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        for &id in rule_ids {
            self.dfs_check_cycle(id, &mut visited, &mut in_stack)?;
        }
        Ok(())
    }

    fn dfs_check_cycle(
        &self,
        rule_id: Uuid,
        visited: &mut std::collections::HashSet<Uuid>,
        in_stack: &mut std::collections::HashSet<Uuid>,
    ) -> RuleEngineResult<()> {
        if in_stack.contains(&rule_id) {
            return Err(RuleEngineError::CircularDependency(rule_id.to_string()));
        }
        if visited.contains(&rule_id) {
            return Ok(());
        }

        visited.insert(rule_id);
        in_stack.insert(rule_id);

        if let Some(rule) = self.rules.get(&rule_id) {
            for &dep in &rule.dependencies {
                if !self.rules.contains_key(&dep) {
                    return Err(RuleEngineError::DependencyNotFound(dep.to_string()));
                }
                self.dfs_check_cycle(dep, visited, in_stack)?;
            }
        }

        in_stack.remove(&rule_id);
        Ok(())
    }

    fn log_change(
        &mut self,
        rule_id: Uuid,
        change_type: ChangeType,
        description: String,
        from_version: Option<u32>,
        to_version: Option<u32>,
    ) {
        let mut change = RuleChange::new(rule_id, change_type, description);
        change.from_version = from_version;
        change.to_version = to_version;
        self.changes.push(change);
    }

    pub fn get_change_history(&self, rule_id: Uuid) -> Vec<&RuleChange> {
        self.changes
            .iter()
            .filter(|c| c.rule_id == rule_id)
            .collect()
    }

    pub fn get_all_changes(&self) -> &[RuleChange] {
        &self.changes
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn rule_set_count(&self) -> usize {
        self.rule_sets.len()
    }

    pub fn list_rules(&self) -> Vec<&Rule> {
        self.rules.values().collect()
    }

    pub fn active_rules(&self) -> Vec<&Rule> {
        self.rules.values().filter(|r| r.is_active()).collect()
    }

    pub fn get_rules_by_type(&self, rule_type: &RuleType) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|r| r.rule_type == *rule_type)
            .collect()
    }

    pub fn get_rules_by_status(&self, status: &RuleStatus) -> Vec<&Rule> {
        self.rules
            .values()
            .filter(|r| r.status == *status)
            .collect()
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_rule(name: &str, priority: i32) -> Rule {
        let mut rule = Rule::new(name, "Test rule", RuleType::Validation)
            .with_priority(priority);
        rule.add_condition(RuleCondition::new(
            "amount",
            ConditionOperator::GreaterThan,
            json!(100),
        ));
        rule.add_action(RuleAction::new(
            ActionType::SetValue,
            Some("approved".to_string()),
            json!(true),
        ));
        rule
    }

    #[test]
    fn test_create_rule() {
        let rule = Rule::new("test_rule", "A test rule", RuleType::Validation);

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.description, "A test rule");
        assert_eq!(rule.rule_type, RuleType::Validation);
        assert_eq!(rule.status, RuleStatus::Draft);
        assert_eq!(rule.priority, 0);
        assert!(rule.conditions.is_empty());
        assert!(rule.actions.is_empty());
        assert!(rule.dependencies.is_empty());
    }

    #[test]
    fn test_register_rule() {
        let mut engine = RuleEngine::new();
        let rule = create_test_rule("test_rule", 0);
        let rule_id = rule.id;

        let result = engine.register_rule(rule);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), rule_id);
        assert_eq!(engine.rule_count(), 1);
        assert_eq!(engine.get_current_version(rule_id).unwrap(), 1);
    }

    #[test]
    fn test_register_duplicate_rule() {
        let mut engine = RuleEngine::new();
        let rule = create_test_rule("test_rule", 0);
        let rule_clone = rule.clone();

        assert!(engine.register_rule(rule).is_ok());
        let result = engine.register_rule(rule_clone);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuleEngineError::RuleAlreadyExists(_)
        ));
    }

    #[test]
    fn test_evaluate_rule_matched() {
        let mut engine = RuleEngine::new();
        let mut rule = create_test_rule("amount_check", 0);
        rule.status = RuleStatus::Active;
        let rule_id = rule.id;
        engine.register_rule(rule).unwrap();

        let mut context = HashMap::new();
        context.insert("amount".to_string(), json!(200));

        let result = engine.evaluate_rule(rule_id, &mut context).unwrap();
        assert!(result.matched);
        assert_eq!(context.get("approved").unwrap(), &json!(true));
    }

    #[test]
    fn test_evaluate_rule_not_matched() {
        let mut engine = RuleEngine::new();
        let mut rule = create_test_rule("amount_check", 0);
        rule.status = RuleStatus::Active;
        let rule_id = rule.id;
        engine.register_rule(rule).unwrap();

        let mut context = HashMap::new();
        context.insert("amount".to_string(), json!(50));

        let result = engine.evaluate_rule(rule_id, &mut context).unwrap();
        assert!(!result.matched);
        assert!(!context.contains_key("approved"));
    }

    #[test]
    fn test_version_rollback() {
        let mut engine = RuleEngine::new();
        let mut rule = create_test_rule("test_rule", 0);
        let rule_id = rule.id;
        engine.register_rule(rule.clone()).unwrap();

        rule.description = "Updated description".to_string();
        rule.priority = 10;
        let new_version = engine.update_rule(rule_id, rule).unwrap();
        assert_eq!(new_version, 2);
        assert_eq!(engine.get_current_version(rule_id).unwrap(), 2);

        assert!(engine.rollback_to_version(rule_id, 1).is_ok());
        assert_eq!(engine.get_current_version(rule_id).unwrap(), 1);

        let rolled_back = engine.get_rule(rule_id).unwrap();
        assert_eq!(rolled_back.priority, 0);
        assert_eq!(rolled_back.description, "Test rule");
    }

    #[test]
    fn test_rule_set_operations() {
        let mut engine = RuleEngine::new();
        let rule1 = create_test_rule("rule1", 1);
        let rule2 = create_test_rule("rule2", 2);
        let id1 = rule1.id;
        let id2 = rule2.id;
        engine.register_rule(rule1).unwrap();
        engine.register_rule(rule2).unwrap();

        let rule_set = RuleSet::new("pricing_rules", "Pricing validation rules");
        let set_id = rule_set.id;
        engine.create_rule_set(rule_set).unwrap();
        assert_eq!(engine.rule_set_count(), 1);

        assert!(engine.add_rule_to_set(set_id, id1).is_ok());
        assert!(engine.add_rule_to_set(set_id, id2).is_ok());

        let rule_list = engine.get_rules_by_set(set_id).unwrap();
        assert_eq!(rule_list.len(), 2);
        assert_eq!(rule_list[0].name, "rule2");
        assert_eq!(rule_list[1].name, "rule1");

        assert!(engine.remove_rule_from_set(set_id, id1).is_ok());
        assert_eq!(engine.get_rule_set(set_id).unwrap().rule_count(), 1);
    }

    #[test]
    fn test_activate_deactivate_rule() {
        let mut engine = RuleEngine::new();
        let rule = create_test_rule("test_rule", 0);
        let rule_id = rule.id;
        engine.register_rule(rule).unwrap();

        assert!(engine.get_rule(rule_id).unwrap().is_draft());

        assert!(engine.activate_rule(rule_id).is_ok());
        assert!(engine.get_rule(rule_id).unwrap().is_active());

        assert!(engine.deactivate_rule(rule_id).is_ok());
        assert!(engine.get_rule(rule_id).unwrap().is_draft());
    }

    #[test]
    fn test_rule_priority_sorting() {
        let mut engine = RuleEngine::new();
        let mut rule1 = create_test_rule("low", 1);
        rule1.status = RuleStatus::Active;
        let mut rule2 = create_test_rule("medium", 5);
        rule2.status = RuleStatus::Active;
        let mut rule3 = create_test_rule("high", 10);
        rule3.status = RuleStatus::Active;

        let id1 = rule1.id;
        let id2 = rule2.id;
        let id3 = rule3.id;

        engine.register_rule(rule1).unwrap();
        engine.register_rule(rule2).unwrap();
        engine.register_rule(rule3).unwrap();

        let mut context = HashMap::new();
        context.insert("amount".to_string(), json!(200));

        let results = engine
            .evaluate_rules(&[id1, id2, id3], &mut context)
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].rule_name, "high");
        assert_eq!(results[0].evaluation_order, 1);
        assert_eq!(results[1].rule_name, "medium");
        assert_eq!(results[2].rule_name, "low");
    }

    #[test]
    fn test_rule_dependencies() {
        let mut engine = RuleEngine::new();
        let mut base_rule = create_test_rule("base", 10);
        base_rule.status = RuleStatus::Active;
        let base_id = base_rule.id;
        engine.register_rule(base_rule).unwrap();

        let mut dependent_rule = create_test_rule("dependent", 5);
        dependent_rule.status = RuleStatus::Active;
        dependent_rule.add_dependency(base_id);
        let dep_id = dependent_rule.id;
        engine.register_rule(dependent_rule).unwrap();

        let mut context = HashMap::new();
        context.insert("amount".to_string(), json!(200));

        let results = engine
            .evaluate_rules(&[dep_id, base_id], &mut context)
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_change_history() {
        let mut engine = RuleEngine::new();
        let rule = create_test_rule("test_rule", 0);
        let rule_id = rule.id;
        engine.register_rule(rule).unwrap();

        engine.activate_rule(rule_id).unwrap();
        engine.deactivate_rule(rule_id).unwrap();

        let history = engine.get_change_history(rule_id);
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].change_type, ChangeType::Created);
        assert_eq!(history[1].change_type, ChangeType::Activated);
        assert_eq!(history[2].change_type, ChangeType::Deactivated);
    }

    #[test]
    fn test_multiple_rules_evaluation() {
        let mut engine = RuleEngine::new();

        let mut rule1 = Rule::new("discount_10", "10% discount", RuleType::Calculation)
            .with_priority(1);
        rule1.status = RuleStatus::Active;
        rule1.add_condition(RuleCondition::new(
            "amount",
            ConditionOperator::GreaterThan,
            json!(100),
        ));
        rule1.add_action(RuleAction::new(
            ActionType::SetValue,
            Some("discount".to_string()),
            json!(0.1),
        ));
        let id1 = rule1.id;
        engine.register_rule(rule1).unwrap();

        let mut rule2 = Rule::new("discount_20", "20% discount", RuleType::Calculation)
            .with_priority(10);
        rule2.status = RuleStatus::Active;
        rule2.add_condition(RuleCondition::new(
            "amount",
            ConditionOperator::GreaterThan,
            json!(500),
        ));
        rule2.add_action(RuleAction::new(
            ActionType::SetValue,
            Some("discount".to_string()),
            json!(0.2),
        ));
        let id2 = rule2.id;
        engine.register_rule(rule2).unwrap();

        let mut context_high = HashMap::new();
        context_high.insert("amount".to_string(), json!(600));
        let results_high = engine
            .evaluate_rules(&[id1, id2], &mut context_high)
            .unwrap();
        assert_eq!(results_high.len(), 2);
        assert_eq!(context_high.get("discount").unwrap(), &json!(0.1));

        let mut context_low = HashMap::new();
        context_low.insert("amount".to_string(), json!(200));
        let results_low = engine
            .evaluate_rules(&[id1, id2], &mut context_low)
            .unwrap();
        assert_eq!(results_low[0].rule_name, "discount_20");
        assert!(!results_low[0].matched);
        assert!(results_low[1].matched);
    }

    #[test]
    fn test_rule_status_transitions() {
        let mut engine = RuleEngine::new();
        let rule = create_test_rule("test_rule", 0);
        let rule_id = rule.id;
        engine.register_rule(rule).unwrap();

        assert!(engine.get_rule(rule_id).unwrap().is_draft());
        assert!(engine.activate_rule(rule_id).is_ok());
        assert!(engine.get_rule(rule_id).unwrap().is_active());

        assert!(engine.deprecate_rule(rule_id).is_ok());
        assert!(engine.get_rule(rule_id).unwrap().is_deprecated());

        assert!(engine.activate_rule(rule_id).is_ok());
        assert!(engine.get_rule(rule_id).unwrap().is_active());

        engine.deprecate_rule(rule_id).unwrap();
        assert!(engine.archive_rule(rule_id).is_ok());
        assert!(engine.get_rule(rule_id).unwrap().is_archived());

        let rule2 = create_test_rule("draft_rule", 0);
        let id2 = rule2.id;
        engine.register_rule(rule2).unwrap();
        assert!(engine.archive_rule(id2).is_ok());
        assert!(engine.get_rule(id2).unwrap().is_archived());
    }

    #[test]
    fn test_condition_operators() {
        let context: HashMap<String, serde_json::Value> = vec![
            ("name".to_string(), json!("hello world")),
            ("count".to_string(), json!(42)),
            ("tag".to_string(), json!("rust")),
        ]
        .into_iter()
        .collect();

        assert!(RuleCondition::new(
            "count",
            ConditionOperator::Equal,
            json!(42)
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "count",
            ConditionOperator::NotEqual,
            json!(10)
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "count",
            ConditionOperator::GreaterThan,
            json!(40)
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "count",
            ConditionOperator::LessThan,
            json!(50)
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "count",
            ConditionOperator::GreaterThanOrEqual,
            json!(42)
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "count",
            ConditionOperator::LessThanOrEqual,
            json!(42)
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "name",
            ConditionOperator::Contains,
            json!("hello")
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "name",
            ConditionOperator::NotContains,
            json!("xyz")
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "name",
            ConditionOperator::StartsWith,
            json!("hello")
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "name",
            ConditionOperator::EndsWith,
            json!("world")
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "tag",
            ConditionOperator::In,
            json!(["rust", "go"])
        )
        .evaluate(&context));
        assert!(RuleCondition::new(
            "tag",
            ConditionOperator::NotIn,
            json!(["python", "java"])
        )
        .evaluate(&context));
    }

    #[test]
    fn test_rule_set_evaluation() {
        let mut engine = RuleEngine::new();
        let mut rule1 = create_test_rule("rule1", 1);
        rule1.status = RuleStatus::Active;
        let mut rule2 = create_test_rule("rule2", 2);
        rule2.status = RuleStatus::Active;
        let id1 = rule1.id;
        let id2 = rule2.id;
        engine.register_rule(rule1).unwrap();
        engine.register_rule(rule2).unwrap();

        let rule_set = RuleSet::new("test_set", "Test rule set");
        let set_id = rule_set.id;
        engine.create_rule_set(rule_set).unwrap();
        engine.add_rule_to_set(set_id, id1).unwrap();
        engine.add_rule_to_set(set_id, id2).unwrap();

        let mut context = HashMap::new();
        context.insert("amount".to_string(), json!(200));

        let results = engine.evaluate_rule_set(set_id, &mut context).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_invalid_status_transitions() {
        let mut engine = RuleEngine::new();
        let rule = create_test_rule("test_rule", 0);
        let rule_id = rule.id;
        engine.register_rule(rule).unwrap();

        assert!(engine.deprecate_rule(rule_id).is_err());
        assert!(engine.deactivate_rule(rule_id).is_err());

        engine.activate_rule(rule_id).unwrap();
        assert!(engine.activate_rule(rule_id).is_err());

        engine.deprecate_rule(rule_id).unwrap();
        engine.archive_rule(rule_id).unwrap();
        assert!(engine.activate_rule(rule_id).is_err());
    }

    #[test]
    fn test_rule_status_display() {
        assert_eq!(RuleStatus::Draft.to_string(), "Draft");
        assert_eq!(RuleStatus::Active.to_string(), "Active");
        assert_eq!(RuleStatus::Deprecated.to_string(), "Deprecated");
        assert_eq!(RuleStatus::Archived.to_string(), "Archived");
    }

    #[test]
    fn test_rule_type_display() {
        assert_eq!(RuleType::Validation.to_string(), "Validation");
        assert_eq!(RuleType::Calculation.to_string(), "Calculation");
        assert_eq!(RuleType::Transformation.to_string(), "Transformation");
        assert_eq!(RuleType::Decision.to_string(), "Decision");
        assert_eq!(RuleType::Notification.to_string(), "Notification");
    }

    #[test]
    fn test_default_values() {
        let status = RuleStatus::default();
        assert_eq!(status, RuleStatus::Draft);

        let rule_type = RuleType::default();
        assert_eq!(rule_type, RuleType::Validation);

        let engine = RuleEngine::default();
        assert_eq!(engine.rule_count(), 0);
        assert_eq!(engine.rule_set_count(), 0);
    }
}
