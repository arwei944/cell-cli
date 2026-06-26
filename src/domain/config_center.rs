use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::errors::{CellError, CellResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigType {
    String,
    Int,
    Float,
    Bool,
    Json,
    List,
}

impl ConfigType {
    pub fn validate(&self, value: &serde_json::Value) -> bool {
        match self {
            ConfigType::String => value.is_string(),
            ConfigType::Int => value.is_i64() || value.is_u64(),
            ConfigType::Float => value.is_f64() || value.is_i64(),
            ConfigType::Bool => value.is_boolean(),
            ConfigType::Json => value.is_object() || value.is_array(),
            ConfigType::List => value.is_array(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfigStatus {
    Active,
    Deprecated,
    Archived,
}

impl Default for ConfigStatus {
    fn default() -> Self {
        ConfigStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigEnvironment {
    Dev,
    Test,
    Staging,
    Prod,
}

impl ConfigEnvironment {
    pub fn parent(&self) -> Option<ConfigEnvironment> {
        match self {
            ConfigEnvironment::Dev => None,
            ConfigEnvironment::Test => Some(ConfigEnvironment::Dev),
            ConfigEnvironment::Staging => Some(ConfigEnvironment::Test),
            ConfigEnvironment::Prod => Some(ConfigEnvironment::Staging),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ConfigKey {
    pub key: String,
    pub service: Option<String>,
    pub environment: ConfigEnvironment,
}

impl ConfigKey {
    pub fn new(key: impl Into<String>, environment: ConfigEnvironment) -> Self {
        Self {
            key: key.into(),
            service: None,
            environment,
        }
    }

    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn env_key(&self) -> String {
        match &self.service {
            Some(svc) => format!("{:?}:{}:{}", self.environment, svc, self.key),
            None => format!("{:?}::{}", self.environment, self.key),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMeta {
    pub description: String,
    pub tags: Vec<String>,
    pub schema: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: ConfigKey,
    pub value: serde_json::Value,
    pub value_type: ConfigType,
    pub status: ConfigStatus,
    pub meta: ConfigMeta,
}

impl ConfigEntry {
    pub fn new(
        key: ConfigKey,
        value: serde_json::Value,
        value_type: ConfigType,
    ) -> CellResult<Self> {
        if !value_type.validate(&value) {
            return Err(CellError::Validation(format!(
                "Value does not match type {:?}",
                value_type
            )));
        }
        let now = chrono::Utc::now().to_rfc3339();
        Ok(Self {
            key,
            value,
            value_type,
            status: ConfigStatus::Active,
            meta: ConfigMeta {
                description: String::new(),
                tags: Vec::new(),
                schema: None,
                created_at: now.clone(),
                updated_at: now,
            },
        })
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.meta.description = desc.into();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.meta.tags = tags;
        self
    }

    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.meta.schema = Some(schema);
        self
    }

    pub fn validate_schema(&self) -> CellResult<bool> {
        if self.meta.schema.is_none() {
            return Ok(true);
        }
        if self.value_type != ConfigType::Json && self.value_type != ConfigType::List {
            return Ok(true);
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version: u64,
    pub entry: ConfigEntry,
    pub change_id: Uuid,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub id: Uuid,
    pub key: ConfigKey,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub change_type: String,
    pub author: String,
    pub reason: String,
    pub created_at: String,
}

impl ConfigChange {
    pub fn new(
        key: ConfigKey,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        change_type: impl Into<String>,
        author: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            key,
            old_value,
            new_value,
            change_type: change_type.into(),
            author: author.into(),
            reason: reason.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigCenter {
    entries: HashMap<String, Vec<ConfigVersion>>,
    changes: Vec<ConfigChange>,
}

impl ConfigCenter {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            changes: Vec::new(),
        }
    }

    fn entry_key(key: &ConfigKey) -> String {
        key.env_key()
    }

    pub fn register(
        &mut self,
        entry: ConfigEntry,
        author: impl Into<String>,
        reason: impl Into<String>,
    ) -> CellResult<u64> {
        let ek = Self::entry_key(&entry.key);
        if self.entries.contains_key(&ek) {
            return Err(CellError::AlreadyExists(format!(
                "Config already exists: {}",
                entry.key.key
            )));
        }

        let change = ConfigChange::new(
            entry.key.clone(),
            None,
            entry.value.clone(),
            "CREATE",
            author,
            reason,
        );
        let change_id = change.id;
        self.changes.push(change);

        let version = ConfigVersion {
            version: 1,
            entry,
            change_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.entries.insert(ek, vec![version]);
        Ok(1)
    }

    pub fn get(&self, key: &ConfigKey) -> Option<&ConfigEntry> {
        let ek = Self::entry_key(key);
        self.entries
            .get(&ek)
            .and_then(|v| v.last())
            .map(|v| &v.entry)
    }

    pub fn get_with_inheritance(&self, key: &ConfigKey) -> Option<ConfigEntry> {
        let mut current_env = Some(key.environment.clone());
        while let Some(env) = current_env {
            let lookup_key = ConfigKey {
                key: key.key.clone(),
                service: key.service.clone(),
                environment: env,
            };
            if let Some(entry) = self.get(&lookup_key) {
                return Some(entry.clone());
            }
            current_env = lookup_key.environment.parent();
        }

        if key.service.is_some() {
            let base_key = ConfigKey {
                key: key.key.clone(),
                service: None,
                environment: key.environment.clone(),
            };
            return self.get_with_inheritance(&base_key);
        }
        None
    }

    pub fn update(
        &mut self,
        key: &ConfigKey,
        new_value: serde_json::Value,
        author: impl Into<String>,
        reason: impl Into<String>,
    ) -> CellResult<u64> {
        let ek = Self::entry_key(key);
        let versions = self
            .entries
            .get_mut(&ek)
            .ok_or_else(|| CellError::NotFound(format!("Config not found: {}", key.key)))?;

        let latest = versions
            .last()
            .ok_or_else(|| CellError::NotFound("No versions".into()))?;

        if !latest.entry.value_type.validate(&new_value) {
            return Err(CellError::Validation(format!(
                "Value does not match type {:?}",
                latest.entry.value_type
            )));
        }

        let old_value = latest.entry.value.clone();
        let new_version_num = latest.version + 1;
        let change = ConfigChange::new(
            key.clone(),
            Some(old_value),
            new_value.clone(),
            "UPDATE",
            author,
            reason,
        );
        let change_id = change.id;
        self.changes.push(change);

        let mut new_entry = latest.entry.clone();
        new_entry.value = new_value;
        new_entry.meta.updated_at = chrono::Utc::now().to_rfc3339();

        versions.push(ConfigVersion {
            version: new_version_num,
            entry: new_entry,
            change_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        Ok(new_version_num)
    }

    pub fn rollback(
        &mut self,
        key: &ConfigKey,
        target_version: u64,
        author: impl Into<String>,
        reason: impl Into<String>,
    ) -> CellResult<u64> {
        let ek = Self::entry_key(key);
        let versions = self
            .entries
            .get_mut(&ek)
            .ok_or_else(|| CellError::NotFound(format!("Config not found: {}", key.key)))?;

        let target_entry = versions
            .iter()
            .find(|v| v.version == target_version)
            .ok_or_else(|| CellError::NotFound(format!("Version {} not found", target_version)))?
            .entry
            .clone();

        let latest = versions.last().unwrap();
        let new_version_num = latest.version + 1;

        let change = ConfigChange::new(
            key.clone(),
            Some(latest.entry.value.clone()),
            target_entry.value.clone(),
            "ROLLBACK",
            author,
            reason,
        );
        let change_id = change.id;
        self.changes.push(change);

        let mut rolled_entry = target_entry;
        rolled_entry.meta.updated_at = chrono::Utc::now().to_rfc3339();

        versions.push(ConfigVersion {
            version: new_version_num,
            entry: rolled_entry,
            change_id,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        Ok(new_version_num)
    }

    pub fn get_by_environment(&self, env: &ConfigEnvironment) -> Vec<&ConfigEntry> {
        self.entries
            .values()
            .filter_map(|v| v.last())
            .map(|v| &v.entry)
            .filter(|e| e.key.environment == *env)
            .collect()
    }

    pub fn search_by_tags(&self, tags: &[String]) -> Vec<&ConfigEntry> {
        self.entries
            .values()
            .filter_map(|v| v.last())
            .map(|v| &v.entry)
            .filter(|e| tags.iter().any(|t| e.meta.tags.contains(t)))
            .collect()
    }

    pub fn version_history(&self, key: &ConfigKey) -> CellResult<Vec<u64>> {
        let ek = Self::entry_key(key);
        self.entries
            .get(&ek)
            .map(|v| v.iter().map(|cv| cv.version).collect())
            .ok_or_else(|| CellError::NotFound(format!("Config not found: {}", key.key)))
    }

    pub fn change_history(&self, key: &ConfigKey) -> Vec<&ConfigChange> {
        self.changes.iter().filter(|c| c.key == *key).collect()
    }

    pub fn get_version(&self, key: &ConfigKey, version: u64) -> CellResult<&ConfigEntry> {
        let ek = Self::entry_key(key);
        self.entries
            .get(&ek)
            .and_then(|v| v.iter().find(|cv| cv.version == version))
            .map(|v| &v.entry)
            .ok_or_else(|| CellError::NotFound(format!("Version {} not found", version)))
    }
}

impl Default for ConfigCenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_key() -> ConfigKey {
        ConfigKey::new("db.url", ConfigEnvironment::Dev).with_service("user-svc")
    }

    fn test_entry() -> ConfigEntry {
        ConfigEntry::new(test_key(), json!("postgres://localhost/dev"), ConfigType::String)
            .unwrap()
            .with_description("DB URL")
            .with_tags(vec!["db".to_string(), "conn".to_string()])
    }

    #[test]
    fn test_config_type_validation() {
        assert!(ConfigType::String.validate(&json!("hi")));
        assert!(!ConfigType::String.validate(&json!(123)));
        assert!(ConfigType::Int.validate(&json!(42)));
        assert!(ConfigType::Float.validate(&json!(3.14)));
        assert!(ConfigType::Bool.validate(&json!(true)));
        assert!(ConfigType::Json.validate(&json!({"a":1})));
        assert!(ConfigType::List.validate(&json!([1,2])));
    }

    #[test]
    fn test_entry_creation() {
        let e = test_entry();
        assert_eq!(e.key.key, "db.url");
        assert_eq!(e.value_type, ConfigType::String);
        assert_eq!(e.status, ConfigStatus::Active);
        assert_eq!(e.meta.tags.len(), 2);
    }

    #[test]
    fn test_entry_type_mismatch() {
        let r = ConfigEntry::new(test_key(), json!(123), ConfigType::String);
        assert!(r.is_err());
    }

    #[test]
    fn test_register_and_get() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "admin", "init").unwrap();
        let got = c.get(&test_key()).unwrap();
        assert_eq!(got.value, json!("postgres://localhost/dev"));
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "admin", "1").unwrap();
        assert!(c.register(test_entry(), "admin", "2").is_err());
    }

    #[test]
    fn test_update_config() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "admin", "init").unwrap();
        let v = c.update(&test_key(), json!("new-url"), "admin", "update").unwrap();
        assert_eq!(v, 2);
        let got = c.get(&test_key()).unwrap();
        assert_eq!(got.value, json!("new-url"));
    }

    #[test]
    fn test_update_type_mismatch_fails() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "admin", "init").unwrap();
        assert!(c.update(&test_key(), json!(123), "admin", "bad").is_err());
    }

    #[test]
    fn test_rollback_version() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "admin", "v1").unwrap();
        c.update(&test_key(), json!("v2"), "admin", "v2").unwrap();
        c.update(&test_key(), json!("v3"), "admin", "v3").unwrap();
        let v = c.rollback(&test_key(), 1, "admin", "rb").unwrap();
        assert_eq!(v, 4);
        assert_eq!(c.get(&test_key()).unwrap().value, json!("postgres://localhost/dev"));
    }

    #[test]
    fn test_environment_isolation() {
        let mut c = ConfigCenter::new();
        let dk = ConfigKey::new("timeout", ConfigEnvironment::Dev);
        let pk = ConfigKey::new("timeout", ConfigEnvironment::Prod);
        c.register(ConfigEntry::new(dk.clone(), json!(30), ConfigType::Int).unwrap(), "a", "d").unwrap();
        c.register(ConfigEntry::new(pk.clone(), json!(60), ConfigType::Int).unwrap(), "a", "p").unwrap();
        assert_eq!(c.get(&dk).unwrap().value, json!(30));
        assert_eq!(c.get(&pk).unwrap().value, json!(60));
    }

    #[test]
    fn test_config_inheritance() {
        let mut c = ConfigCenter::new();
        let dk = ConfigKey::new("flag", ConfigEnvironment::Dev);
        c.register(ConfigEntry::new(dk, json!(true), ConfigType::Bool).unwrap(), "a", "d").unwrap();
        let tk = ConfigKey::new("flag", ConfigEnvironment::Test);
        let got = c.get_with_inheritance(&tk);
        assert!(got.is_some());
        assert_eq!(got.unwrap().value, json!(true));
    }

    #[test]
    fn test_service_level_override() {
        let mut c = ConfigCenter::new();
        let ek = ConfigKey::new("retries", ConfigEnvironment::Dev);
        c.register(ConfigEntry::new(ek, json!(3), ConfigType::Int).unwrap(), "a", "env").unwrap();
        let sk = ConfigKey::new("retries", ConfigEnvironment::Dev).with_service("sp");
        c.register(ConfigEntry::new(sk.clone(), json!(5), ConfigType::Int).unwrap(), "a", "svc").unwrap();
        assert_eq!(c.get(&sk).unwrap().value, json!(5));
    }

    #[test]
    fn test_search_by_tags() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "a", "init").unwrap();
        let tk = ConfigKey::new("timeout", ConfigEnvironment::Dev);
        let te = ConfigEntry::new(tk, json!(30), ConfigType::Int).unwrap()
            .with_tags(vec!["perf".to_string()]);
        c.register(te, "a", "t").unwrap();
        let r = c.search_by_tags(&["db".to_string()]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].key.key, "db.url");
    }

    #[test]
    fn test_version_history() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "a", "v1").unwrap();
        c.update(&test_key(), json!("v2"), "a", "v2").unwrap();
        c.update(&test_key(), json!("v3"), "a", "v3").unwrap();
        let h = c.version_history(&test_key()).unwrap();
        assert_eq!(h, vec![1, 2, 3]);
    }

    #[test]
    fn test_change_history() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "alice", "init").unwrap();
        c.update(&test_key(), json!("v2"), "bob", "u1").unwrap();
        c.update(&test_key(), json!("v3"), "carol", "u2").unwrap();
        let ch = c.change_history(&test_key());
        assert_eq!(ch.len(), 3);
        assert_eq!(ch[0].author, "alice");
        assert_eq!(ch[0].change_type, "CREATE");
        assert_eq!(ch[1].author, "bob");
        assert_eq!(ch[2].author, "carol");
    }

    #[test]
    fn test_get_specific_version() {
        let mut c = ConfigCenter::new();
        c.register(test_entry(), "a", "v1").unwrap();
        c.update(&test_key(), json!("v2"), "a", "v2").unwrap();
        let v1 = c.get_version(&test_key(), 1).unwrap();
        let v2 = c.get_version(&test_key(), 2).unwrap();
        assert_eq!(v1.value, json!("postgres://localhost/dev"));
        assert_eq!(v2.value, json!("v2"));
    }

    #[test]
    fn test_get_by_environment() {
        let mut c = ConfigCenter::new();
        let dk = ConfigKey::new("a", ConfigEnvironment::Dev);
        c.register(ConfigEntry::new(dk, json!(1), ConfigType::Int).unwrap(), "a", "d").unwrap();
        let tk = ConfigKey::new("b", ConfigEnvironment::Test);
        c.register(ConfigEntry::new(tk, json!(2), ConfigType::Int).unwrap(), "a", "t").unwrap();
        let de = c.get_by_environment(&ConfigEnvironment::Dev);
        assert_eq!(de.len(), 1);
        assert_eq!(de[0].key.key, "a");
    }

    #[test]
    fn test_env_parent_chain() {
        assert!(ConfigEnvironment::Dev.parent().is_none());
        assert_eq!(ConfigEnvironment::Test.parent(), Some(ConfigEnvironment::Dev));
        assert_eq!(ConfigEnvironment::Staging.parent(), Some(ConfigEnvironment::Test));
        assert_eq!(ConfigEnvironment::Prod.parent(), Some(ConfigEnvironment::Staging));
    }
}
