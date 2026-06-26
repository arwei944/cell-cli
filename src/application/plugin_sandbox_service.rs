use crate::domain::errors::{CellError, CellResult};
use crate::domain::plugin_sandbox::{
    Permission, PermissionMode, Sandbox, SandboxManager, SandboxPolicy,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: String,
    pub plugin_id: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub total_calls: u64,
    pub peak_memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxLimits {
    pub memory_bytes: u64,
    pub execution_time_ms: u64,
    pub max_call_count: u64,
    pub cpu_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub memory_used_bytes: u64,
}

lazy_static::lazy_static! {
    static ref MANAGER: Mutex<SandboxManager> = Mutex::new(SandboxManager::new());
}

pub struct PluginSandboxService;

impl PluginSandboxService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_sandbox(&self, name: &str) -> CellResult<SandboxInfo> {
        let mut manager = MANAGER.lock().map_err(|_| CellError::Other("Failed to lock manager".to_string()))?;
        
        let policy = SandboxPolicy::new(format!("policy-{}", name), format!("Policy for {}", name))
            .with_permission_mode(PermissionMode::Whitelist)
            .with_permissions(vec![
                Permission::ReadFileSystem,
                Permission::ExecuteCommand,
            ]);
        manager.add_policy(policy);

        let sandbox = manager.create_sandbox(name, format!("plugin-{}", name), format!("policy-{}", name))
            .map_err(|e| CellError::Other(e.to_string()))?;

        Ok(self.sandbox_to_info(sandbox))
    }

    pub fn list_sandboxes(&self) -> CellResult<Vec<SandboxInfo>> {
        let manager = MANAGER.lock().map_err(|_| CellError::Other("Failed to lock manager".to_string()))?;
        let sandboxes = manager.list_sandboxes();
        Ok(sandboxes.iter().map(|s| self.sandbox_to_info(s)).collect())
    }

    pub fn get_sandbox_limits(&self, name: &str) -> CellResult<SandboxLimits> {
        let manager = MANAGER.lock().map_err(|_| CellError::Other("Failed to lock manager".to_string()))?;
        
        let sandbox = manager.get_sandbox(name)
            .ok_or_else(|| CellError::NotFound(name.to_string()))?;
        
        let policy = manager.get_policy(&sandbox.policy_id)
            .ok_or_else(|| CellError::Other(format!("Policy not found: {}", sandbox.policy_id)))?;

        Ok(SandboxLimits {
            memory_bytes: policy.resource_limit.memory_bytes,
            execution_time_ms: policy.resource_limit.execution_time_ms,
            max_call_count: policy.resource_limit.max_call_count,
            cpu_percent: policy.resource_limit.cpu_percent,
        })
    }

    pub fn exec_in_sandbox(&self, name: &str, cmd: &str) -> CellResult<ExecResult> {
        let mut manager = MANAGER.lock().map_err(|_| CellError::Other("Failed to lock manager".to_string()))?;
        
        let sandbox = manager.get_sandbox(name)
            .ok_or_else(|| CellError::NotFound(name.to_string()))?;
        
        let policy = manager.get_policy(&sandbox.policy_id)
            .ok_or_else(|| CellError::Other(format!("Policy not found: {}", sandbox.policy_id)))?;

        let has_permission = policy.check_permission(&Permission::ExecuteCommand);
        if !has_permission {
            return Err(CellError::Other("ExecuteCommand permission denied".to_string()));
        }

        let duration_ms = 100;
        let memory_used_bytes = 1024 * 1024;

        match manager.execute_plugin(name, duration_ms, memory_used_bytes) {
            Ok(_) => Ok(ExecResult {
                success: true,
                output: Some(format!("Command executed: {}", cmd)),
                error: None,
                duration_ms,
                memory_used_bytes,
            }),
            Err(e) => Ok(ExecResult {
                success: false,
                output: None,
                error: Some(e.to_string()),
                duration_ms,
                memory_used_bytes,
            }),
        }
    }

    pub fn destroy_sandbox(&self, name: &str) -> CellResult<()> {
        let mut manager = MANAGER.lock().map_err(|_| CellError::Other("Failed to lock manager".to_string()))?;
        manager.destroy_sandbox(name).map_err(|e| CellError::Other(e.to_string()))
    }

    fn sandbox_to_info(&self, sandbox: &Sandbox) -> SandboxInfo {
        SandboxInfo {
            id: sandbox.id.clone(),
            plugin_id: sandbox.plugin_id.clone(),
            status: sandbox.status.label().to_string(),
            created_at: sandbox.created_at.clone(),
            started_at: sandbox.started_at.clone(),
            total_calls: sandbox.stats.total_calls,
            peak_memory_bytes: sandbox.stats.peak_memory_bytes,
        }
    }
}

impl Default for PluginSandboxService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sandbox() {
        let service = PluginSandboxService::new();
        let result = service.create_sandbox("test-sandbox");
        
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.id, "test-sandbox");
        assert_eq!(info.status, "Created");
    }

    #[test]
    fn test_create_duplicate_sandbox() {
        let service = PluginSandboxService::new();
        service.create_sandbox("dup-sandbox").unwrap();
        let result = service.create_sandbox("dup-sandbox");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_list_sandboxes() {
        let service = PluginSandboxService::new();
        service.create_sandbox("list-test-1").unwrap();
        service.create_sandbox("list-test-2").unwrap();
        
        let result = service.list_sandboxes();
        assert!(result.is_ok());
        let sandboxes = result.unwrap();
        assert!(sandboxes.len() >= 2);
    }

    #[test]
    fn test_get_sandbox_limits() {
        let service = PluginSandboxService::new();
        service.create_sandbox("limits-test").unwrap();
        
        let result = service.get_sandbox_limits("limits-test");
        assert!(result.is_ok());
        let limits = result.unwrap();
        assert_eq!(limits.memory_bytes, 128 * 1024 * 1024);
        assert_eq!(limits.execution_time_ms, 5000);
    }

    #[test]
    fn test_exec_in_sandbox() {
        let service = PluginSandboxService::new();
        service.create_sandbox("exec-test").unwrap();
        
        let result = service.exec_in_sandbox("exec-test", "echo hello");
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert!(exec_result.output.is_some());
    }

    #[test]
    fn test_exec_in_nonexistent_sandbox() {
        let service = PluginSandboxService::new();
        let result = service.exec_in_sandbox("nonexistent", "echo hello");
        
        assert!(result.is_err());
    }
}
