use cell_domain::errors::{CellError, CellResult};
use cell_domain::feature::{FeatureStatus, FeatureUnit, MountPoint};
use std::collections::HashMap;

/// Feature Unit 生命周期状态流转验证
/// Design -> Development -> Testing -> Staging -> Production -> (Deprecated -> Retired)
const VALID_TRANSITIONS: &[(FeatureStatus, FeatureStatus)] = &[
    (FeatureStatus::Design, FeatureStatus::Development),
    (FeatureStatus::Development, FeatureStatus::Testing),
    (FeatureStatus::Testing, FeatureStatus::Staging),
    (FeatureStatus::Staging, FeatureStatus::Production),
    (FeatureStatus::Production, FeatureStatus::Deprecated),
    (FeatureStatus::Deprecated, FeatureStatus::Retired),
];

/// Feature Unit 服务
pub struct FeatureService {
    features: HashMap<String, FeatureUnit>,
}

impl FeatureService {
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }

    /// 创建功能单元
    pub fn create_feature(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        _owner: Option<String>,
    ) -> CellResult<FeatureUnit> {
        let name = name.into();
        if self.features.contains_key(&name) {
            return Err(CellError::AlreadyExists(format!("Feature '{name}' already exists")));
        }

        let feature = FeatureUnit {
            id: uuid::Uuid::new_v4(),
            name: name.clone(),
            description: description.into(),
            version: "0.1.0".to_string(),
            status: FeatureStatus::Design,
            extension_points: Vec::new(),
            dependencies: Vec::new(),
            mounts: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        self.features.insert(name, feature.clone());
        Ok(feature)
    }

    /// 获取功能单元
    pub fn get_feature(&self, name: &str) -> CellResult<&FeatureUnit> {
        self.features.get(name)
            .ok_or_else(|| CellError::NotFound(format!("Feature '{name}' not found")))
    }

    /// 列出所有功能单元
    pub fn list_features(&self) -> Vec<&FeatureUnit> {
        self.features.values().collect()
    }

    /// 更新功能单元状态
    pub fn transition_status(
        &mut self,
        name: &str,
        new_status: FeatureStatus,
    ) -> CellResult<FeatureUnit> {
        let feature = self.features.get_mut(name)
            .ok_or_else(|| CellError::NotFound(format!("Feature '{name}' not found")))?;

        let old_status = feature.status.clone();
        if !can_transition(&old_status, &new_status) {
            return Err(CellError::Config(format!(
                "Cannot transition from {old_status:?} to {new_status:?}"
            )));
        }

        feature.status = new_status;
        feature.updated_at = chrono::Utc::now();

        Ok(feature.clone())
    }

    /// 挂载功能单元
    pub fn mount_feature(
        &mut self,
        name: &str,
        host: impl Into<String>,
        extension_point: impl Into<String>,
        priority: i32,
    ) -> CellResult<()> {
        let feature = self.features.get_mut(name)
            .ok_or_else(|| CellError::NotFound(format!("Feature '{name}' not found")))?;

        feature.mounts.push(MountPoint {
            host: host.into(),
            extension_point: extension_point.into(),
            priority,
        });

        Ok(())
    }

    /// 卸载功能单元
    pub fn unmount_feature(&mut self, name: &str, host: &str) -> CellResult<()> {
        let feature = self.features.get_mut(name)
            .ok_or_else(|| CellError::NotFound(format!("Feature '{name}' not found")))?;

        feature.mounts.retain(|m| m.host != host);
        Ok(())
    }

    /// 格式化功能单元信息
    pub fn format_feature(&self, feature: &FeatureUnit) -> String {
        format!(
            "  📦 {} (v{})\n     状态: {:?}\n     描述: {}\n     挂载点: {}",
            feature.name,
            feature.version,
            feature.status,
            feature.description,
            feature.mounts.len()
        )
    }

    /// 格式化列表
    pub fn format_list(&self) -> String {
        let mut o = String::new();
        o.push_str("📋 Feature Units\n");
        o.push_str(&"=".repeat(60));
        o.push_str("\n\n");

        let features = self.list_features();
        if features.is_empty() {
            o.push_str("(no features)\n");
            return o;
        }

        for f in &features {
            let status_icon = match f.status {
                FeatureStatus::Design => "📝",
                FeatureStatus::Development => "🔨",
                FeatureStatus::Testing => "🧪",
                FeatureStatus::Staging => "🚀",
                FeatureStatus::Production => "✅",
                FeatureStatus::Deprecated => "⚠️",
                FeatureStatus::Retired => "💤",
            };
            o.push_str(&format!(
                "  {} {} (v{}) - {:?}\n",
                status_icon, f.name, f.version, f.status
            ));
        }

        o.push_str(&format!("\nTotal: {}\n", features.len()));
        o
    }
}

impl Default for FeatureService {
    fn default() -> Self {
        Self::new()
    }
}

/// 检查状态转换是否合法
fn can_transition(from: &FeatureStatus, to: &FeatureStatus) -> bool {
    if from == to {
        return true;
    }
    VALID_TRANSITIONS.contains(&(from.clone(), to.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_feature() {
        let mut service = FeatureService::new();
        let f = service.create_feature("test", "desc", Some("owner".to_string())).unwrap();
        assert_eq!(f.status, FeatureStatus::Design);
        assert_eq!(f.version, "0.1.0");
    }

    #[test]
    fn test_create_duplicate_fails() {
        let mut service = FeatureService::new();
        service.create_feature("test", "desc", None).unwrap();
        let result = service.create_feature("test", "desc2", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(can_transition(&FeatureStatus::Design, &FeatureStatus::Development));
        assert!(can_transition(&FeatureStatus::Development, &FeatureStatus::Testing));
        assert!(can_transition(&FeatureStatus::Testing, &FeatureStatus::Staging));
        assert!(can_transition(&FeatureStatus::Production, &FeatureStatus::Deprecated));
        assert!(can_transition(&FeatureStatus::Deprecated, &FeatureStatus::Retired));
        assert!(!can_transition(&FeatureStatus::Design, &FeatureStatus::Production));
    }

    #[test]
    fn test_transition_status() {
        let mut service = FeatureService::new();
        service.create_feature("test", "desc", None).unwrap();
        let f = service.transition_status("test", FeatureStatus::Development).unwrap();
        assert_eq!(f.status, FeatureStatus::Development);
    }
}
