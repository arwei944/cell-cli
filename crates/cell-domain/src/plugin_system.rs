use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PluginId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    Registered,
    Loading,
    Active,
    Inactive,
    Error,
    Unloaded,
}

impl PluginStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Registered => "Registered",
            Self::Loading => "Loading",
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Error => "Error",
            Self::Unloaded => "Unloaded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginKind {
    Analyzer,
    Generator,
    Validator,
    Reporter,
    Command,
    Extension,
}

impl PluginKind {
    pub fn label(&self) -> &str {
        match self {
            Self::Analyzer => "Analyzer",
            Self::Generator => "Generator",
            Self::Validator => "Validator",
            Self::Reporter => "Reporter",
            Self::Command => "Command",
            Self::Extension => "Extension",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub kind: PluginKind,
    pub min_host_version: String,
    pub dependencies: Vec<PluginId>,
    pub optional_dependencies: Vec<PluginId>,
    pub entry_point: String,
    pub permissions: Vec<Permission>,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    ReadFileSystem,
    WriteFileSystem,
    ExecuteCommand,
    NetworkAccess,
    ModifyConfig,
    RegisterCommand,
    HookIntoLifecycle,
    AccessEntropyData,
    AccessArchitectureData,
    All,
}

impl Permission {
    pub fn label(&self) -> &str {
        match self {
            Self::ReadFileSystem => "read:fs",
            Self::WriteFileSystem => "write:fs",
            Self::ExecuteCommand => "execute:command",
            Self::NetworkAccess => "network:access",
            Self::ModifyConfig => "modify:config",
            Self::RegisterCommand => "register:command",
            Self::HookIntoLifecycle => "hook:lifecycle",
            Self::AccessEntropyData => "access:entropy",
            Self::AccessArchitectureData => "access:architecture",
            Self::All => "*",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstance {
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub loaded_at: Option<String>,
    pub activated_at: Option<String>,
    pub last_error: Option<String>,
    pub config: HashMap<String, String>,
    pub stats: PluginStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginStats {
    pub total_calls: u64,
    pub total_duration_ms: u64,
    pub error_count: u64,
    pub last_called_at: Option<String>,
}

impl PluginInstance {
    pub fn new(manifest: PluginManifest) -> Self {
        Self {
            manifest,
            status: PluginStatus::Registered,
            loaded_at: None,
            activated_at: None,
            last_error: None,
            config: HashMap::new(),
            stats: PluginStats::default(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == PluginStatus::Active
    }
}

pub enum PluginLifecycleEvent {
    PreInit,
    PostInit,
    PreAnalyze,
    PostAnalyze,
    PreGenerate,
    PostGenerate,
    PreValidate,
    PostValidate,
    Shutdown,
}

impl PluginLifecycleEvent {
    pub fn label(&self) -> &str {
        match self {
            Self::PreInit => "pre_init",
            Self::PostInit => "post_init",
            Self::PreAnalyze => "pre_analyze",
            Self::PostAnalyze => "post_analyze",
            Self::PreGenerate => "pre_generate",
            Self::PostGenerate => "post_generate",
            Self::PreValidate => "pre_validate",
            Self::PostValidate => "post_validate",
            Self::Shutdown => "shutdown",
        }
    }
}

#[allow(dead_code)]
pub struct PluginManager {
    plugins: HashMap<PluginId, PluginInstance>,
    load_order: Vec<PluginId>,
    hooks: HashMap<PluginLifecycleEvent, Vec<PluginId>>,
    host_version: String,
}

impl PluginManager {
    pub fn new(host_version: impl Into<String>) -> Self {
        Self {
            plugins: HashMap::new(),
            load_order: Vec::new(),
            hooks: HashMap::new(),
            host_version: host_version.into(),
        }
    }

    pub fn register(&mut self, manifest: PluginManifest) -> Result<(), PluginError> {
        let id = manifest.id.clone();

        if self.plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id.0));
        }

        if !self.version_compatible(&manifest.min_host_version) {
            return Err(PluginError::IncompatibleHost(format!(
                "Plugin requires host version {}, current is {}",
                manifest.min_host_version, self.host_version
            )));
        }

        let instance = PluginInstance::new(manifest);
        self.plugins.insert(id.clone(), instance);
        self.load_order.push(id);

        Ok(())
    }

    pub fn unregister(&mut self, id: &PluginId) -> Result<(), PluginError> {
        if !self.plugins.contains_key(id) {
            return Err(PluginError::NotFound(id.0.clone()));
        }

        let dependents = self.find_dependents(id);
        if !dependents.is_empty() {
            return Err(PluginError::HasDependents(format!(
                "Plugins depend on this: {:?}",
                dependents.iter().map(|p| p.0.clone()).collect::<Vec<_>>()
            )));
        }

        if let Some(instance) = self.plugins.get(id)
            && instance.is_active() {
                return Err(PluginError::InvalidState(format!(
                    "Plugin {} is still active", id.0
                )));
            }

        self.plugins.remove(id);
        self.load_order.retain(|x| x != id);

        Ok(())
    }

    pub fn load(&mut self, id: &PluginId) -> Result<(), PluginError> {
        {
            let plugin = self.plugins.get(id)
                .ok_or_else(|| PluginError::NotFound(id.0.clone()))?;

            if plugin.status != PluginStatus::Registered {
                return Err(PluginError::InvalidState(format!(
                    "Cannot load plugin in state {:?}",
                    plugin.status.label()
                )));
            }

            for dep_id in &plugin.manifest.dependencies {
                let dep = self.plugins.get(dep_id)
                    .ok_or_else(|| PluginError::MissingDependency(dep_id.0.clone()))?;

                if !dep.is_active() && dep.status != PluginStatus::Registered {
                    return Err(PluginError::DependencyNotLoaded(dep_id.0.clone()));
                }
            }
        }

        let plugin = self.plugins.get_mut(id).unwrap();
        plugin.status = PluginStatus::Loading;
        plugin.loaded_at = Some(chrono::Utc::now().to_rfc3339());
        plugin.status = PluginStatus::Inactive;

        Ok(())
    }

    pub fn activate(&mut self, id: &PluginId) -> Result<(), PluginError> {
        let deps: Vec<PluginId> = {
            let plugin = self.plugins.get(id)
                .ok_or_else(|| PluginError::NotFound(id.0.clone()))?;

            if plugin.status != PluginStatus::Inactive {
                return Err(PluginError::InvalidState(format!(
                    "Cannot activate plugin in state {:?}",
                    plugin.status.label()
                )));
            }

            plugin.manifest.dependencies.clone()
        };

        for dep_id in &deps {
            self.activate_if_inactive(dep_id)?;
        }

        let plugin = self.plugins.get_mut(id).unwrap();
        plugin.status = PluginStatus::Active;
        plugin.activated_at = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    fn activate_if_inactive(&mut self, id: &PluginId) -> Result<(), PluginError> {
        let status = self.plugins.get(id)
            .map(|p| p.status.clone())
            .ok_or_else(|| PluginError::MissingDependency(id.0.clone()))?;

        if status == PluginStatus::Inactive {
            self.activate(id)?;
        } else if status != PluginStatus::Active {
            return Err(PluginError::DependencyNotActive(id.0.clone()));
        }

        Ok(())
    }

    pub fn deactivate(&mut self, id: &PluginId) -> Result<(), PluginError> {
        let dependents = self.find_dependents(id);
        let active_dependents: Vec<_> = dependents.iter()
            .filter(|d| self.plugins.get(d).is_some_and(PluginInstance::is_active))
            .collect();

        if !active_dependents.is_empty() {
            return Err(PluginError::HasActiveDependents(format!(
                "Active plugins depend on this: {:?}",
                active_dependents.iter().map(|p| p.0.clone()).collect::<Vec<_>>()
            )));
        }

        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.0.clone()))?;

        if plugin.status != PluginStatus::Active {
            return Err(PluginError::InvalidState(format!(
                "Cannot deactivate plugin in state {:?}",
                plugin.status.label()
            )));
        }

        plugin.status = PluginStatus::Inactive;
        Ok(())
    }

    pub fn get(&self, id: &PluginId) -> Option<&PluginInstance> {
        self.plugins.get(id)
    }

    pub fn list(&self) -> Vec<&PluginInstance> {
        self.load_order
            .iter()
            .filter_map(|id| self.plugins.get(id))
            .collect()
    }

    pub fn active_plugins(&self) -> Vec<&PluginInstance> {
        self.plugins.values().filter(|p| p.is_active()).collect()
    }

    pub fn plugins_by_kind(&self, kind: &PluginKind) -> Vec<&PluginInstance> {
        self.plugins
            .values()
            .filter(|p| &p.manifest.kind == kind)
            .collect()
    }

    pub fn find_dependents(&self, id: &PluginId) -> Vec<PluginId> {
        let mut dependents = Vec::new();

        for plugin in self.plugins.values() {
            if plugin.manifest.dependencies.contains(id) {
                dependents.push(plugin.manifest.id.clone());
            }
        }

        dependents
    }

    pub fn validate_dependencies(&self) -> Vec<DependencyError> {
        let mut errors = Vec::new();

        for plugin in self.plugins.values() {
            for dep in &plugin.manifest.dependencies {
                if !self.plugins.contains_key(dep) {
                    errors.push(DependencyError::Missing {
                        plugin: plugin.manifest.id.clone(),
                        dependency: dep.clone(),
                    });
                }
            }
        }

        if self.check_circular_deps() == Ok(()) {
        } else {
            errors.push(DependencyError::Circular);
        }

        errors
    }

    fn check_circular_deps(&self) -> Result<(), PluginError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for plugin_id in &self.load_order {
            if !visited.contains(plugin_id)
                && self.dfs_cycle(plugin_id, &mut visited, &mut stack) {
                    return Err(PluginError::CircularDependency);
                }
        }

        Ok(())
    }

    fn dfs_cycle(
        &self,
        current: &PluginId,
        visited: &mut HashSet<PluginId>,
        stack: &mut HashSet<PluginId>,
    ) -> bool {
        visited.insert(current.clone());
        stack.insert(current.clone());

        if let Some(plugin) = self.plugins.get(current) {
            for dep in &plugin.manifest.dependencies {
                if !visited.contains(dep) {
                    if self.dfs_cycle(dep, visited, stack) {
                        return true;
                    }
                } else if stack.contains(dep) {
                    return true;
                }
            }
        }

        stack.remove(current);
        false
    }

    pub fn record_call(&mut self, id: &PluginId, duration_ms: u64, success: bool) {
        if let Some(plugin) = self.plugins.get_mut(id) {
            plugin.stats.total_calls += 1;
            plugin.stats.total_duration_ms += duration_ms;
            if !success {
                plugin.stats.error_count += 1;
            }
            plugin.stats.last_called_at = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    pub fn set_plugin_config(&mut self, id: &PluginId, key: &str, value: &str) -> Result<(), PluginError> {
        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.0.clone()))?;
        plugin.config.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn version_compatible(&self, min_version: &str) -> bool {
        let host_parts: Vec<u64> = self.host_version
            .split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        let min_parts: Vec<u64> = min_version
            .split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        for i in 0..3 {
            let host = host_parts.get(i).copied().unwrap_or(0);
            let min = min_parts.get(i).copied().unwrap_or(0);
            if host > min {
                return true;
            } else if host < min {
                return false;
            }
        }

        true
    }

    pub fn shutdown_all(&mut self) {
        let mut active_ids: Vec<PluginId> = self.plugins
            .values()
            .filter(|p| p.is_active())
            .map(|p| p.manifest.id.clone())
            .collect();

        active_ids.reverse();

        for id in &active_ids {
            if let Some(plugin) = self.plugins.get_mut(id) {
                plugin.status = PluginStatus::Inactive;
            }
        }
    }
}

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyError {
    Missing { plugin: PluginId, dependency: PluginId },
    Circular,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    NotFound(String),
    AlreadyRegistered(String),
    IncompatibleHost(String),
    MissingDependency(String),
    DependencyNotLoaded(String),
    DependencyNotActive(String),
    HasDependents(String),
    HasActiveDependents(String),
    InvalidState(String),
    CircularDependency,
    PermissionDenied(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Plugin not found: {id}"),
            Self::AlreadyRegistered(id) => write!(f, "Plugin already registered: {id}"),
            Self::IncompatibleHost(msg) => write!(f, "Incompatible host: {msg}"),
            Self::MissingDependency(dep) => write!(f, "Missing dependency: {dep}"),
            Self::DependencyNotLoaded(dep) => write!(f, "Dependency not loaded: {dep}"),
            Self::DependencyNotActive(dep) => write!(f, "Dependency not active: {dep}"),
            Self::HasDependents(msg) | Self::HasActiveDependents(msg) => write!(f, "{msg}"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            Self::CircularDependency => write!(f, "Circular dependency detected"),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest(id: &str, name: &str) -> PluginManifest {
        PluginManifest {
            id: PluginId(id.to_string()),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Test plugin {name}"),
            author: "Test Author".to_string(),
            kind: PluginKind::Analyzer,
            min_host_version: "0.1.0".to_string(),
            dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            entry_point: format!("plugin_{id}.wasm"),
            permissions: vec![Permission::ReadFileSystem],
            tags: vec!["test".to_string()],
            homepage: None,
            repository: None,
            license: None,
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new("1.0.0");
        assert_eq!(manager.list().len(), 0);
    }

    #[test]
    fn test_register_plugin() {
        let mut manager = PluginManager::new("1.0.0");
        let manifest = create_test_manifest("test-1", "Test Plugin");
        assert!(manager.register(manifest).is_ok());
        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn test_register_duplicate() {
        let mut manager = PluginManager::new("1.0.0");
        let manifest = create_test_manifest("test-1", "Test Plugin");
        manager.register(manifest.clone()).unwrap();
        let result = manager.register(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_incompatible_host_version() {
        let mut manager = PluginManager::new("0.0.1");
        let mut manifest = create_test_manifest("test-1", "Test");
        manifest.min_host_version = "1.0.0".to_string();
        let result = manager.register(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_plugin() {
        let mut manager = PluginManager::new("1.0.0");
        manager.register(create_test_manifest("test-1", "Test")).unwrap();
        assert!(manager.load(&PluginId("test-1".to_string())).is_ok());

        let plugin = manager.get(&PluginId("test-1".to_string())).unwrap();
        assert_eq!(plugin.status, PluginStatus::Inactive);
    }

    #[test]
    fn test_activate_plugin() {
        let mut manager = PluginManager::new("1.0.0");
        manager.register(create_test_manifest("test-1", "Test")).unwrap();
        manager.load(&PluginId("test-1".to_string())).unwrap();
        assert!(manager.activate(&PluginId("test-1".to_string())).is_ok());

        let plugin = manager.get(&PluginId("test-1".to_string())).unwrap();
        assert!(plugin.is_active());
    }

    #[test]
    fn test_deactivate_plugin() {
        let mut manager = PluginManager::new("1.0.0");
        manager.register(create_test_manifest("test-1", "Test")).unwrap();
        manager.load(&PluginId("test-1".to_string())).unwrap();
        manager.activate(&PluginId("test-1".to_string())).unwrap();
        assert!(manager.deactivate(&PluginId("test-1".to_string())).is_ok());

        let plugin = manager.get(&PluginId("test-1".to_string())).unwrap();
        assert!(!plugin.is_active());
    }

    #[test]
    fn test_plugin_with_dependency() {
        let mut manager = PluginManager::new("1.0.0");

        let dep = create_test_manifest("dep-1", "Dependency");
        manager.register(dep).unwrap();

        let mut dependent = create_test_manifest("plugin-1", "Dependent");
        dependent.dependencies = vec![PluginId("dep-1".to_string())];
        manager.register(dependent).unwrap();

        manager.load(&PluginId("dep-1".to_string())).unwrap();
        manager.activate(&PluginId("dep-1".to_string())).unwrap();
        manager.load(&PluginId("plugin-1".to_string())).unwrap();
        assert!(manager.activate(&PluginId("plugin-1".to_string())).is_ok());
    }

    #[test]
    fn test_missing_dependency() {
        let mut manager = PluginManager::new("1.0.0");

        let mut dependent = create_test_manifest("plugin-1", "Dependent");
        dependent.dependencies = vec![PluginId("missing".to_string())];

        manager.register(dependent).unwrap();
        let result = manager.load(&PluginId("plugin-1".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_dependents() {
        let mut manager = PluginManager::new("1.0.0");

        manager.register(create_test_manifest("dep-1", "Dependency")).unwrap();

        let mut p1 = create_test_manifest("plugin-1", "Plugin 1");
        p1.dependencies = vec![PluginId("dep-1".to_string())];
        manager.register(p1).unwrap();

        let dependents = manager.find_dependents(&PluginId("dep-1".to_string()));
        assert_eq!(dependents.len(), 1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut manager = PluginManager::new("1.0.0");

        let mut p1 = create_test_manifest("p1", "Plugin 1");
        p1.dependencies = vec![PluginId("p2".to_string())];
        manager.register(p1).unwrap();

        let mut p2 = create_test_manifest("p2", "Plugin 2");
        p2.dependencies = vec![PluginId("p1".to_string())];
        manager.register(p2).unwrap();

        let errors = manager.validate_dependencies();
        assert!(errors.iter().any(|e| matches!(e, DependencyError::Circular)));
    }

    #[test]
    fn test_active_plugins() {
        let mut manager = PluginManager::new("1.0.0");

        manager.register(create_test_manifest("p1", "Plugin 1")).unwrap();
        manager.register(create_test_manifest("p2", "Plugin 2")).unwrap();

        manager.load(&PluginId("p1".to_string())).unwrap();
        manager.activate(&PluginId("p1".to_string())).unwrap();

        assert_eq!(manager.active_plugins().len(), 1);
    }

    #[test]
    fn test_plugins_by_kind() {
        let mut manager = PluginManager::new("1.0.0");

        let mut p1 = create_test_manifest("p1", "Plugin 1");
        p1.kind = PluginKind::Generator;
        manager.register(p1).unwrap();

        let mut p2 = create_test_manifest("p2", "Plugin 2");
        p2.kind = PluginKind::Generator;
        manager.register(p2).unwrap();

        let p3 = create_test_manifest("p3", "Plugin 3");
        manager.register(p3).unwrap();

        assert_eq!(manager.plugins_by_kind(&PluginKind::Generator).len(), 2);
    }

    #[test]
    fn test_record_call_stats() {
        let mut manager = PluginManager::new("1.0.0");
        manager.register(create_test_manifest("p1", "Plugin 1")).unwrap();

        manager.record_call(&PluginId("p1".to_string()), 100, true);
        manager.record_call(&PluginId("p1".to_string()), 50, false);

        let plugin = manager.get(&PluginId("p1".to_string())).unwrap();
        assert_eq!(plugin.stats.total_calls, 2);
        assert_eq!(plugin.stats.total_duration_ms, 150);
        assert_eq!(plugin.stats.error_count, 1);
    }

    #[test]
    fn test_plugin_config() {
        let mut manager = PluginManager::new("1.0.0");
        manager.register(create_test_manifest("p1", "Plugin 1")).unwrap();

        manager.set_plugin_config(&PluginId("p1".to_string()), "key", "value").unwrap();

        let plugin = manager.get(&PluginId("p1".to_string())).unwrap();
        assert_eq!(plugin.config.get("key").unwrap(), "value");
    }

    #[test]
    fn test_unregister_plugin() {
        let mut manager = PluginManager::new("1.0.0");
        manager.register(create_test_manifest("p1", "Plugin 1")).unwrap();
        assert!(manager.unregister(&PluginId("p1".to_string())).is_ok());
        assert!(!manager.plugins.contains_key(&PluginId("p1".to_string())));
    }

    #[test]
    fn test_unregister_with_dependents_fails() {
        let mut manager = PluginManager::new("1.0.0");

        manager.register(create_test_manifest("dep", "Dep")).unwrap();

        let mut p1 = create_test_manifest("p1", "Plugin 1");
        p1.dependencies = vec![PluginId("dep".to_string())];
        manager.register(p1).unwrap();

        let result = manager.unregister(&PluginId("dep".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_deactivate_with_active_dependents_fails() {
        let mut manager = PluginManager::new("1.0.0");

        manager.register(create_test_manifest("dep", "Dep")).unwrap();
        manager.load(&PluginId("dep".to_string())).unwrap();
        manager.activate(&PluginId("dep".to_string())).unwrap();

        let mut p1 = create_test_manifest("p1", "Plugin 1");
        p1.dependencies = vec![PluginId("dep".to_string())];
        manager.register(p1).unwrap();
        manager.load(&PluginId("p1".to_string())).unwrap();
        manager.activate(&PluginId("p1".to_string())).unwrap();

        let result = manager.deactivate(&PluginId("dep".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_shutdown_all() {
        let mut manager = PluginManager::new("1.0.0");

        manager.register(create_test_manifest("p1", "Plugin 1")).unwrap();
        manager.register(create_test_manifest("p2", "Plugin 2")).unwrap();

        manager.load(&PluginId("p1".to_string())).unwrap();
        manager.activate(&PluginId("p1".to_string())).unwrap();
        manager.load(&PluginId("p2".to_string())).unwrap();
        manager.activate(&PluginId("p2".to_string())).unwrap();

        manager.shutdown_all();
        assert_eq!(manager.active_plugins().len(), 0);
    }

    #[test]
    fn test_version_compatible() {
        let manager = PluginManager::new("1.5.0");
        assert!(manager.version_compatible("1.0.0"));
        assert!(manager.version_compatible("1.5.0"));
        assert!(!manager.version_compatible("2.0.0"));
    }
}
