use cell_domain::errors::CellResult;
use cell_domain::plugin_system::{
    PluginId, PluginInstance, PluginManager, PluginManifest,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub kind: String,
    pub status: String,
    pub loaded_at: Option<String>,
    pub activated_at: Option<String>,
    pub error: Option<String>,
}

impl From<&PluginInstance> for PluginInfo {
    fn from(instance: &PluginInstance) -> Self {
        Self {
            id: instance.manifest.id.0.clone(),
            name: instance.manifest.name.clone(),
            version: instance.manifest.version.clone(),
            description: instance.manifest.description.clone(),
            author: instance.manifest.author.clone(),
            kind: instance.manifest.kind.label().to_string(),
            status: instance.status.label().to_string(),
            loaded_at: instance.loaded_at.clone(),
            activated_at: instance.activated_at.clone(),
            error: instance.last_error.clone(),
        }
    }
}

pub struct PluginService {
    manager: PluginManager,
}

impl PluginService {
    pub fn new() -> Self {
        Self {
            manager: PluginManager::new(env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.manager.list().iter().map(|i| PluginInfo::from(*i)).collect()
    }

    pub fn load_plugin(&mut self, path: &str) -> CellResult<PluginInfo> {
        let manifest = Self::read_manifest(path)?;
        self.manager.register(manifest.clone())?;
        self.manager.load(&manifest.id)?;
        let instance = self.manager.get(&manifest.id).ok_or_else(|| {
            cell_domain::errors::CellError::NotFound(format!("Plugin {} not found after loading", manifest.id.0))
        })?;
        Ok(PluginInfo::from(instance))
    }

    pub fn activate_plugin(&mut self, id: &str) -> CellResult<PluginInfo> {
        let plugin_id = PluginId(id.to_string());
        self.manager.activate(&plugin_id)?;
        let instance = self.manager.get(&plugin_id).ok_or_else(|| {
            cell_domain::errors::CellError::NotFound(format!("Plugin {id} not found"))
        })?;
        Ok(PluginInfo::from(instance))
    }

    pub fn deactivate_plugin(&mut self, id: &str) -> CellResult<PluginInfo> {
        let plugin_id = PluginId(id.to_string());
        self.manager.deactivate(&plugin_id)?;
        let instance = self.manager.get(&plugin_id).ok_or_else(|| {
            cell_domain::errors::CellError::NotFound(format!("Plugin {id} not found"))
        })?;
        Ok(PluginInfo::from(instance))
    }

    pub fn get_plugin_status(&self, id: &str) -> CellResult<PluginInfo> {
        let plugin_id = PluginId(id.to_string());
        let instance = self.manager.get(&plugin_id).ok_or_else(|| {
            cell_domain::errors::CellError::NotFound(format!("Plugin {id} not found"))
        })?;
        Ok(PluginInfo::from(instance))
    }

    fn read_manifest(path: &str) -> CellResult<PluginManifest> {
        let manifest_path = Path::new(path);
        let content = std::fs::read_to_string(manifest_path)?;
        
        let ext = manifest_path.extension().and_then(|e| e.to_str());
        match ext {
            Some("json") => serde_json::from_str(&content).map_err(cell_domain::errors::CellError::Serde),
            Some("yaml" | "yml") => serde_yaml::from_str(&content).map_err(cell_domain::errors::CellError::Yaml),
            Some("toml") => toml::from_str(&content).map_err(cell_domain::errors::CellError::Toml),
            _ => Ok(serde_json::from_str(&content).map_err(cell_domain::errors::CellError::Serde)?),
        }
    }

    pub fn format_plugin_list(&self, plugins: &[PluginInfo]) -> String {
        let mut output = String::new();

        output.push_str("\n📦 插件列表\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        if plugins.is_empty() {
            output.push_str("   暂无已加载的插件\n\n");
        } else {
            for plugin in plugins {
                let status_icon = match plugin.status.as_str() {
                    "Active" => "✅",
                    "Inactive" => "⚪",
                    "Loading" => "⏳",
                    "Error" => "🔴",
                    _ => "📦",
                };

                output.push_str(&format!(
                    "{} {} ({})\n",
                    status_icon, plugin.name, plugin.id
                ));
                output.push_str(&format!("   版本: {}\n", plugin.version));
                output.push_str(&format!("   类型: {}\n", plugin.kind));
                output.push_str(&format!("   状态: {}\n", plugin.status));
                output.push_str(&format!("   描述: {}\n", plugin.description));

                if let Some(error) = &plugin.error {
                    output.push_str(&format!("   ❌ 错误: {error}\n"));
                }

                output.push('\n');
            }
        }

        output.push_str("💡 提示: 使用 `cell plugin load <path>` 加载插件\n");

        output
    }

    pub fn format_plugin_status(&self, info: &PluginInfo) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n📦 插件状态 - {}\n", info.name));
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!("ID:          {}\n", info.id));
        output.push_str(&format!("名称:        {}\n", info.name));
        output.push_str(&format!("版本:        {}\n", info.version));
        output.push_str(&format!("作者:        {}\n", info.author));
        output.push_str(&format!("类型:        {}\n", info.kind));
        output.push_str(&format!("状态:        {}\n", info.status));
        output.push_str(&format!("描述:        {}\n", info.description));

        if let Some(loaded_at) = &info.loaded_at {
            output.push_str(&format!("加载时间:    {loaded_at}\n"));
        }

        if let Some(activated_at) = &info.activated_at {
            output.push_str(&format!("激活时间:    {activated_at}\n"));
        }

        if let Some(error) = &info.error {
            output.push_str(&format!("\n❌ 错误信息:\n{error}"));
        }

        output
    }
}

impl Default for PluginService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_domain::plugin_system::{Permission, PluginKind, PluginManifest};

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
    fn test_list_plugins_empty() {
        let service = PluginService::new();
        let plugins = service.list_plugins();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_load_and_list_plugin() {
        let mut service = PluginService::new();
        let manifest = create_test_manifest("test-plugin", "Test Plugin");
        let json = serde_json::to_string(&manifest).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");
        std::fs::write(&manifest_path, json).unwrap();

        let result = service.load_plugin(manifest_path.to_str().unwrap());
        assert!(result.is_ok());

        let plugins = service.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "test-plugin");
        assert_eq!(plugins[0].name, "Test Plugin");
        assert_eq!(plugins[0].status, "Inactive");
    }

    #[test]
    fn test_activate_plugin() {
        let mut service = PluginService::new();
        let manifest = create_test_manifest("test-plugin", "Test Plugin");
        let json = serde_json::to_string(&manifest).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");
        std::fs::write(&manifest_path, json).unwrap();

        service.load_plugin(manifest_path.to_str().unwrap()).unwrap();
        let result = service.activate_plugin("test-plugin");
        assert!(result.is_ok());

        let status = service.get_plugin_status("test-plugin").unwrap();
        assert_eq!(status.status, "Active");
    }

    #[test]
    fn test_deactivate_plugin() {
        let mut service = PluginService::new();
        let manifest = create_test_manifest("test-plugin", "Test Plugin");
        let json = serde_json::to_string(&manifest).unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");
        std::fs::write(&manifest_path, json).unwrap();

        service.load_plugin(manifest_path.to_str().unwrap()).unwrap();
        service.activate_plugin("test-plugin").unwrap();

        let result = service.deactivate_plugin("test-plugin");
        assert!(result.is_ok());

        let status = service.get_plugin_status("test-plugin").unwrap();
        assert_eq!(status.status, "Inactive");
    }

    #[test]
    fn test_get_plugin_status_not_found() {
        let service = PluginService::new();
        let result = service.get_plugin_status("non-existent");
        assert!(result.is_err());
    }
}
