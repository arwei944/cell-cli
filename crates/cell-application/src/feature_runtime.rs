use cell_domain::errors::{CellError, CellResult};
use cell_domain::extension::{ExtensionRegistry, ExtensionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureLifecycleState {
    Draft,
    Active,
    Mounted,
    Suspended,
    Retired,
}

impl FeatureLifecycleState {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Draft => "草稿 - 功能开发中，未发布",
            Self::Active => "活跃 - 功能已发布，可挂载",
            Self::Mounted => "已挂载 - 功能正在运行",
            Self::Suspended => "已暂停 - 功能已暂停使用",
            Self::Retired => "已退役 - 功能已废弃，待删除",
        }
    }

    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Active) |
(Self::Active | Self::Suspended, Self::Mounted | Self::Retired) |
(Self::Mounted, Self::Suspended | Self::Retired) |
(Self::Retired, Self::Draft)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUnit {
    pub name: String,
    pub description: String,
    pub version: String,
    pub state: FeatureLifecycleState,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub extensions: Vec<ExtensionType>,
    pub metadata: HashMap<String, String>,
}

impl FeatureUnit {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            name: name.into(),
            description: description.into(),
            version: "0.1.0".to_string(),
            state: FeatureLifecycleState::Draft,
            owner: "system".to_string(),
            created_at: now,
            updated_at: now,
            dependencies: vec![],
            extensions: vec![],
            metadata: HashMap::new(),
        }
    }

    pub fn transition_to(&mut self, next: FeatureLifecycleState) -> CellResult<()> {
        if !self.state.can_transition_to(&next) {
            return Err(CellError::Config(format!(
                "Invalid state transition: {:?} -> {:?}",
                self.state, next
            )));
        }
        self.state = next;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountLogEntry {
    pub feature_name: String,
    pub action: MountAction,
    pub phase: MountPhase,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountAction {
    Mount,
    Unmount,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MountPhase {
    Prepare,
    Validate,
    Activate,
    Commit,
    Rollback,
}

impl MountPhase {
    pub fn order(&self) -> u8 {
        match self {
            Self::Prepare => 1,
            Self::Validate => 2,
            Self::Activate => 3,
            Self::Commit => 4,
            Self::Rollback => 0,
        }
    }
}

pub struct FeatureRuntime {
    features: HashMap<String, FeatureUnit>,
    pub extension_registry: ExtensionRegistry,
    mount_log: Vec<MountLogEntry>,
    mounted: Vec<String>,
}

impl FeatureRuntime {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
            extension_registry: ExtensionRegistry::new(),
            mount_log: Vec::new(),
            mounted: Vec::new(),
        }
    }

    pub fn create_feature(&mut self, name: impl Into<String>, description: impl Into<String>) -> CellResult<&FeatureUnit> {
        let name = name.into();
        if self.features.contains_key(&name) {
            return Err(CellError::AlreadyExists(format!("Feature '{name}' already exists")));
        }
        let feature = FeatureUnit::new(name.clone(), description);
        self.features.insert(name.clone(), feature);
        Ok(self.features.get(&name).unwrap())
    }

    pub fn get_feature(&self, name: &str) -> Option<&FeatureUnit> {
        self.features.get(name)
    }

    pub fn list_features(&self) -> Vec<&FeatureUnit> {
        let mut features: Vec<&FeatureUnit> = self.features.values().collect();
        features.sort_by(|a, b| a.name.cmp(&b.name));
        features
    }

    pub fn list_mounted(&self) -> Vec<&str> {
        self.mounted.iter().map(std::string::String::as_str).collect()
    }

    pub fn transition_feature(&mut self, name: &str, next: FeatureLifecycleState) -> CellResult<()> {
        let feature = self.features.get_mut(name)
            .ok_or_else(|| CellError::NotFound(format!("Feature '{name}' not found")))?;
        feature.transition_to(next)
    }

    pub fn mount(&mut self, name: &str) -> CellResult<()> {
        if self.mounted.contains(&name.to_string()) {
            return Err(CellError::Config(format!("Feature '{name}' is already mounted")));
        }

        let feature = self.features.get(name)
            .ok_or_else(|| CellError::NotFound(format!("Feature '{name}' not found")))?;

        if feature.state != FeatureLifecycleState::Active && feature.state != FeatureLifecycleState::Suspended {
            return Err(CellError::Config(format!(
                "Feature '{}' cannot be mounted in state {:?}",
                name, feature.state
            )));
        }

        let result = self.execute_mount_phases(name, MountAction::Mount);
        
        if result.is_ok() {
            self.mounted.push(name.to_string());
            if let Some(f) = self.features.get_mut(name) {
                let _ = f.transition_to(FeatureLifecycleState::Mounted);
            }
        }

        result
    }

    pub fn unmount(&mut self, name: &str) -> CellResult<()> {
        if !self.mounted.contains(&name.to_string()) {
            return Err(CellError::Config(format!("Feature '{name}' is not mounted")));
        }

        let result = self.execute_mount_phases(name, MountAction::Unmount);
        
        if result.is_ok() {
            self.mounted.retain(|n| n != name);
            if let Some(f) = self.features.get_mut(name) {
                let _ = f.transition_to(FeatureLifecycleState::Active);
            }
        }

        result
    }

    fn execute_mount_phases(&mut self, name: &str, action: MountAction) -> CellResult<()> {
        let phases = [
            MountPhase::Prepare,
            MountPhase::Validate,
            MountPhase::Activate,
            MountPhase::Commit,
        ];

        let mut completed_phases = Vec::new();

        for phase in &phases {
            let success = self.run_phase(name, &action, phase);
            
            self.mount_log.push(MountLogEntry {
                feature_name: name.to_string(),
                action: action.clone(),
                phase: phase.clone(),
                timestamp: Utc::now(),
                success,
                message: if success {
                    format!("{phase:?} phase completed")
                } else {
                    format!("{phase:?} phase failed")
                },
            });

            if success {
                completed_phases.push(phase.clone());
            } else {
                self.rollback(name, &action, &completed_phases);
                return Err(CellError::Config(format!(
                    "Failed to {action:?} feature '{name}' at {phase:?} phase"
                )));
            }
        }

        Ok(())
    }

    fn run_phase(&self, _name: &str, _action: &MountAction, _phase: &MountPhase) -> bool {
        true
    }

    fn rollback(&mut self, name: &str, action: &MountAction, completed_phases: &[MountPhase]) {
        for phase in completed_phases.iter().rev() {
            self.mount_log.push(MountLogEntry {
                feature_name: name.to_string(),
                action: action.clone(),
                phase: MountPhase::Rollback,
                timestamp: Utc::now(),
                success: true,
                message: format!("Rolled back {phase:?} phase"),
            });
        }
    }

    pub fn retire(&mut self, name: &str) -> CellResult<()> {
        if self.mounted.contains(&name.to_string()) {
            self.unmount(name)?;
        }
        self.transition_feature(name, FeatureLifecycleState::Retired)
    }

    pub fn mount_log(&self) -> &[MountLogEntry] {
        &self.mount_log
    }

    pub fn save_state(&self, path: &Path) -> CellResult<()> {
        let state = FeatureRuntimeState {
            features: self.features.values().cloned().collect(),
            mounted: self.mounted.clone(),
            mount_log: self.mount_log.clone(),
        };
        let yaml = serde_yaml::to_string(&state)
            .map_err(|e| CellError::Config(format!("Failed to serialize state: {e}")))?;
        std::fs::write(path, yaml)
            .map_err(|e| CellError::Config(format!("Failed to write state file: {e}")))?;
        Ok(())
    }

    pub fn load_state(path: &Path) -> CellResult<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| CellError::Config(format!("Failed to read state file: {e}")))?;
        let state: FeatureRuntimeState = serde_yaml::from_str(&content)
            .map_err(|e| CellError::Config(format!("Failed to parse state file: {e}")))?;
        
        let mut features = HashMap::new();
        for f in state.features {
            features.insert(f.name.clone(), f);
        }

        Ok(Self {
            features,
            extension_registry: ExtensionRegistry::new(),
            mount_log: state.mount_log,
            mounted: state.mounted,
        })
    }
}

impl Default for FeatureRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeatureRuntimeState {
    features: Vec<FeatureUnit>,
    mounted: Vec<String>,
    mount_log: Vec<MountLogEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_lifecycle_transitions() {
        assert!(FeatureLifecycleState::Draft.can_transition_to(&FeatureLifecycleState::Active));
        assert!(FeatureLifecycleState::Active.can_transition_to(&FeatureLifecycleState::Mounted));
        assert!(FeatureLifecycleState::Mounted.can_transition_to(&FeatureLifecycleState::Suspended));
        assert!(FeatureLifecycleState::Suspended.can_transition_to(&FeatureLifecycleState::Mounted));
        assert!(FeatureLifecycleState::Mounted.can_transition_to(&FeatureLifecycleState::Retired));
        
        assert!(!FeatureLifecycleState::Draft.can_transition_to(&FeatureLifecycleState::Mounted));
        assert!(!FeatureLifecycleState::Retired.can_transition_to(&FeatureLifecycleState::Mounted));
    }

    #[test]
    fn test_feature_unit_transition() {
        let mut feature = FeatureUnit::new("test", "Test feature");
        assert_eq!(feature.state, FeatureLifecycleState::Draft);
        
        assert!(feature.transition_to(FeatureLifecycleState::Active).is_ok());
        assert_eq!(feature.state, FeatureLifecycleState::Active);
        
        assert!(feature.transition_to(FeatureLifecycleState::Draft).is_err());
    }

    #[test]
    fn test_runtime_create_feature() {
        let mut rt = FeatureRuntime::new();
        let feature = rt.create_feature("test-feature", "A test feature").unwrap();
        assert_eq!(feature.name, "test-feature");
        assert_eq!(feature.state, FeatureLifecycleState::Draft);
        
        assert!(rt.create_feature("test-feature", "Duplicate").is_err());
    }

    #[test]
    fn test_runtime_mount_unmount() {
        let mut rt = FeatureRuntime::new();
        rt.create_feature("feature-a", "Feature A").unwrap();
        rt.transition_feature("feature-a", FeatureLifecycleState::Active).unwrap();
        
        assert!(rt.mount("feature-a").is_ok());
        assert_eq!(rt.list_mounted().len(), 1);
        
        assert!(rt.mount("feature-a").is_err());
        
        assert!(rt.unmount("feature-a").is_ok());
        assert_eq!(rt.list_mounted().len(), 0);
    }

    #[test]
    fn test_runtime_retire() {
        let mut rt = FeatureRuntime::new();
        rt.create_feature("old-feature", "Old feature").unwrap();
        rt.transition_feature("old-feature", FeatureLifecycleState::Active).unwrap();
        rt.mount("old-feature").unwrap();
        
        assert!(rt.retire("old-feature").is_ok());
        let feature = rt.get_feature("old-feature").unwrap();
        assert_eq!(feature.state, FeatureLifecycleState::Retired);
        assert_eq!(rt.list_mounted().len(), 0);
    }

    #[test]
    fn test_mount_log() {
        let mut rt = FeatureRuntime::new();
        rt.create_feature("log-test", "Log test").unwrap();
        rt.transition_feature("log-test", FeatureLifecycleState::Active).unwrap();
        
        rt.mount("log-test").unwrap();
        assert!(rt.mount_log().len() >= 4);
        
        rt.unmount("log-test").unwrap();
        assert!(rt.mount_log().len() >= 8);
    }

    #[test]
    fn test_extension_registry_in_runtime() {
        let rt = FeatureRuntime::new();
        let list = rt.extension_registry.list_by_type(&ExtensionType::Validation);
        assert!(list.is_empty());
    }
}
