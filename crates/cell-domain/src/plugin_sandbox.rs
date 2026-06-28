use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    pub fn matches(&self, required: &Self) -> bool {
        if self == &Self::All || required == &Self::All {
            return true;
        }
        self == required
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    Whitelist,
    Blacklist,
}

impl PermissionMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Whitelist => "whitelist",
            Self::Blacklist => "blacklist",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimit {
    pub memory_bytes: u64,
    pub execution_time_ms: u64,
    pub max_call_count: u64,
    pub cpu_percent: u32,
}

impl Default for ResourceLimit {
    fn default() -> Self {
        Self {
            memory_bytes: 128 * 1024 * 1024,
            execution_time_ms: 5000,
            max_call_count: 1000,
            cpu_percent: 50,
        }
    }
}

impl ResourceLimit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_memory(mut self, bytes: u64) -> Self {
        self.memory_bytes = bytes;
        self
    }

    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }

    pub fn with_max_calls(mut self, count: u64) -> Self {
        self.max_call_count = count;
        self
    }

    pub fn with_cpu(mut self, percent: u32) -> Self {
        self.cpu_percent = percent;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub id: String,
    pub name: String,
    pub permission_mode: PermissionMode,
    pub permissions: Vec<Permission>,
    pub resource_limit: ResourceLimit,
    pub created_at: String,
    pub updated_at: String,
}

impl SandboxPolicy {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            name: name.into(),
            permission_mode: PermissionMode::Whitelist,
            permissions: Vec::new(),
            resource_limit: ResourceLimit::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_permission_mode(mut self, mode: PermissionMode) -> Self {
        self.permission_mode = mode;
        self
    }

    pub fn with_permissions(mut self, perms: Vec<Permission>) -> Self {
        self.permissions = perms;
        self
    }

    pub fn with_resource_limit(mut self, limit: ResourceLimit) -> Self {
        self.resource_limit = limit;
        self
    }

    pub fn add_permission(&mut self, perm: Permission) {
        if !self.permissions.contains(&perm) {
            self.permissions.push(perm);
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    pub fn remove_permission(&mut self, perm: &Permission) {
        self.permissions.retain(|p| p != perm);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn check_permission(&self, required: &Permission) -> bool {
        let has_perm = self
            .permissions
            .iter()
            .any(|p| p.matches(required));

        match self.permission_mode {
            PermissionMode::Whitelist => has_perm,
            PermissionMode::Blacklist => !has_perm,
        }
    }

    pub fn update_resource_limit(&mut self, limit: ResourceLimit) {
        self.resource_limit = limit;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SandboxStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Destroyed,
}

impl SandboxStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Created => "Created",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Stopped => "Stopped",
            Self::Destroyed => "Destroyed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SandboxStats {
    pub total_calls: u64,
    pub total_duration_ms: u64,
    pub peak_memory_bytes: u64,
    pub error_count: u64,
    pub last_called_at: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    pub id: String,
    pub plugin_id: String,
    pub policy_id: String,
    pub status: SandboxStatus,
    pub stats: SandboxStats,
    pub current_memory_bytes: u64,
    pub created_at: String,
    pub started_at: Option<String>,
    pub destroyed_at: Option<String>,
}

impl Sandbox {
    pub fn new(id: impl Into<String>, plugin_id: impl Into<String>, policy_id: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            plugin_id: plugin_id.into(),
            policy_id: policy_id.into(),
            status: SandboxStatus::Created,
            stats: SandboxStats::default(),
            current_memory_bytes: 0,
            created_at: now,
            started_at: None,
            destroyed_at: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == SandboxStatus::Running
    }

    pub fn is_destroyed(&self) -> bool {
        self.status == SandboxStatus::Destroyed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditLogAction {
    SandboxCreated,
    SandboxDestroyed,
    PluginExecuted,
    PermissionGranted,
    PermissionDenied,
    ResourceLimitExceeded,
    PolicyUpdated,
}

impl AuditLogAction {
    pub fn label(&self) -> &str {
        match self {
            Self::SandboxCreated => "sandbox_created",
            Self::SandboxDestroyed => "sandbox_destroyed",
            Self::PluginExecuted => "plugin_executed",
            Self::PermissionGranted => "permission_granted",
            Self::PermissionDenied => "permission_denied",
            Self::ResourceLimitExceeded => "resource_limit_exceeded",
            Self::PolicyUpdated => "policy_updated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub sandbox_id: String,
    pub plugin_id: String,
    pub action: AuditLogAction,
    pub details: String,
    pub timestamp: String,
    pub success: bool,
}

impl AuditLogEntry {
    pub fn new(
        sandbox_id: impl Into<String>,
        plugin_id: impl Into<String>,
        action: AuditLogAction,
        details: impl Into<String>,
        success: bool,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sandbox_id: sandbox_id.into(),
            plugin_id: plugin_id.into(),
            action,
            details: details.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            success,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    NotFound(String),
    AlreadyExists(String),
    InvalidState(String),
    PermissionDenied(String),
    ResourceLimitExceeded(String),
    ExecutionTimeout(String),
    MemoryLimitExceeded(String),
    CallLimitExceeded(String),
    PolicyNotFound(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Sandbox not found: {id}"),
            Self::AlreadyExists(id) => write!(f, "Sandbox already exists: {id}"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
            Self::ResourceLimitExceeded(msg) => write!(f, "Resource limit exceeded: {msg}"),
            Self::ExecutionTimeout(msg) => write!(f, "Execution timeout: {msg}"),
            Self::MemoryLimitExceeded(msg) => write!(f, "Memory limit exceeded: {msg}"),
            Self::CallLimitExceeded(msg) => write!(f, "Call limit exceeded: {msg}"),
            Self::PolicyNotFound(id) => write!(f, "Policy not found: {id}"),
        }
    }
}

impl std::error::Error for SandboxError {}

pub struct SandboxManager {
    sandboxes: HashMap<String, Sandbox>,
    policies: HashMap<String, SandboxPolicy>,
    audit_logs: Vec<AuditLogEntry>,
    max_audit_logs: usize,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            sandboxes: HashMap::new(),
            policies: HashMap::new(),
            audit_logs: Vec::new(),
            max_audit_logs: 10000,
        }
    }

    pub fn with_max_audit_logs(mut self, max: usize) -> Self {
        self.max_audit_logs = max;
        self
    }

    pub fn add_policy(&mut self, policy: SandboxPolicy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    pub fn get_policy(&self, id: &str) -> Option<&SandboxPolicy> {
        self.policies.get(id)
    }

    pub fn update_policy(&mut self, policy_id: &str, limit: ResourceLimit) -> Result<(), SandboxError> {
        let policy = self
            .policies
            .get_mut(policy_id)
            .ok_or_else(|| SandboxError::PolicyNotFound(policy_id.to_string()))?;

        policy.update_resource_limit(limit);

        self.add_audit_log(
            "system",
            "system",
            AuditLogAction::PolicyUpdated,
            format!("Policy {policy_id} updated"),
            true,
        );

        Ok(())
    }

    pub fn create_sandbox(
        &mut self,
        sandbox_id: impl Into<String>,
        plugin_id: impl Into<String>,
        policy_id: impl Into<String>,
    ) -> Result<&Sandbox, SandboxError> {
        let id = sandbox_id.into();
        let plugin = plugin_id.into();
        let policy = policy_id.into();

        if self.sandboxes.contains_key(&id) {
            return Err(SandboxError::AlreadyExists(id));
        }

        if !self.policies.contains_key(&policy) {
            return Err(SandboxError::PolicyNotFound(policy));
        }

        let sandbox = Sandbox::new(id.clone(), plugin.clone(), policy.clone());
        self.sandboxes.insert(id.clone(), sandbox);

        self.add_audit_log(
            &id,
            &plugin,
            AuditLogAction::SandboxCreated,
            format!("Sandbox created with policy {policy}"),
            true,
        );

        Ok(self.sandboxes.get(&id).unwrap())
    }

    pub fn destroy_sandbox(&mut self, sandbox_id: &str) -> Result<(), SandboxError> {
        let sandbox = self
            .sandboxes
            .get_mut(sandbox_id)
            .ok_or_else(|| SandboxError::NotFound(sandbox_id.to_string()))?;

        if sandbox.is_destroyed() {
            return Err(SandboxError::InvalidState(format!(
                "Sandbox {sandbox_id} is already destroyed"
            )));
        }

        let plugin_id = sandbox.plugin_id.clone();
        sandbox.status = SandboxStatus::Destroyed;
        sandbox.destroyed_at = Some(chrono::Utc::now().to_rfc3339());

        self.add_audit_log(
            sandbox_id,
            &plugin_id,
            AuditLogAction::SandboxDestroyed,
            "Sandbox destroyed".to_string(),
            true,
        );

        Ok(())
    }

    pub fn get_sandbox(&self, id: &str) -> Option<&Sandbox> {
        self.sandboxes.get(id)
    }

    pub fn list_sandboxes(&self) -> Vec<&Sandbox> {
        self.sandboxes.values().collect()
    }

    pub fn check_permission(&mut self, sandbox_id: &str, permission: &Permission) -> Result<bool, SandboxError> {
        let (plugin_id, allowed) = {
            let sandbox = self
                .sandboxes
                .get(sandbox_id)
                .ok_or_else(|| SandboxError::NotFound(sandbox_id.to_string()))?;

            if sandbox.is_destroyed() {
                return Err(SandboxError::InvalidState(format!(
                    "Sandbox {sandbox_id} is destroyed"
                )));
            }

            let policy = self
                .policies
                .get(&sandbox.policy_id)
                .ok_or_else(|| SandboxError::PolicyNotFound(sandbox.policy_id.clone()))?;

            (sandbox.plugin_id.clone(), policy.check_permission(permission))
        };

        self.add_audit_log(
            sandbox_id,
            &plugin_id,
            if allowed {
                AuditLogAction::PermissionGranted
            } else {
                AuditLogAction::PermissionDenied
            },
            format!("Permission {} check: {}", permission.label(), allowed),
            allowed,
        );

        Ok(allowed)
    }

    pub fn execute_plugin(
        &mut self,
        sandbox_id: &str,
        duration_ms: u64,
        memory_used_bytes: u64,
    ) -> Result<(), SandboxError> {
        let (policy_id, plugin_id, resource_limit, current_calls) = {
            let sandbox = self
                .sandboxes
                .get(sandbox_id)
                .ok_or_else(|| SandboxError::NotFound(sandbox_id.to_string()))?;

            if sandbox.is_destroyed() {
                return Err(SandboxError::InvalidState(format!(
                    "Sandbox {sandbox_id} is destroyed"
                )));
            }

            let policy = self
                .policies
                .get(&sandbox.policy_id)
                .ok_or_else(|| SandboxError::PolicyNotFound(sandbox.policy_id.clone()))?;

            (
                sandbox.policy_id.clone(),
                sandbox.plugin_id.clone(),
                policy.resource_limit.clone(),
                sandbox.stats.total_calls,
            )
        };

        if current_calls >= resource_limit.max_call_count {
            self.add_audit_log(
                sandbox_id,
                &plugin_id,
                AuditLogAction::ResourceLimitExceeded,
                format!(
                    "Call limit exceeded: {}/{}",
                    current_calls, resource_limit.max_call_count
                ),
                false,
            );
            return Err(SandboxError::CallLimitExceeded(format!(
                "Max calls {} reached",
                resource_limit.max_call_count
            )));
        }

        if duration_ms > resource_limit.execution_time_ms {
            self.add_audit_log(
                sandbox_id,
                &plugin_id,
                AuditLogAction::ResourceLimitExceeded,
                format!(
                    "Execution timeout: {}ms / {}ms",
                    duration_ms, resource_limit.execution_time_ms
                ),
                false,
            );
            return Err(SandboxError::ExecutionTimeout(format!(
                "Execution {}ms exceeds limit {}ms",
                duration_ms, resource_limit.execution_time_ms
            )));
        }

        if memory_used_bytes > resource_limit.memory_bytes {
            self.add_audit_log(
                sandbox_id,
                &plugin_id,
                AuditLogAction::ResourceLimitExceeded,
                format!(
                    "Memory limit exceeded: {} / {} bytes",
                    memory_used_bytes, resource_limit.memory_bytes
                ),
                false,
            );
            return Err(SandboxError::MemoryLimitExceeded(format!(
                "Memory {} bytes exceeds limit {} bytes",
                memory_used_bytes, resource_limit.memory_bytes
            )));
        }

        let sandbox = self.sandboxes.get_mut(sandbox_id).unwrap();
        sandbox.stats.total_calls += 1;
        sandbox.stats.total_duration_ms += duration_ms;
        sandbox.stats.last_called_at = Some(chrono::Utc::now().to_rfc3339());
        sandbox.current_memory_bytes = memory_used_bytes;

        if memory_used_bytes > sandbox.stats.peak_memory_bytes {
            sandbox.stats.peak_memory_bytes = memory_used_bytes;
        }

        if sandbox.status == SandboxStatus::Created {
            sandbox.status = SandboxStatus::Running;
            sandbox.started_at = Some(chrono::Utc::now().to_rfc3339());
        }

        self.add_audit_log(
            sandbox_id,
            &plugin_id,
            AuditLogAction::PluginExecuted,
            format!(
                "Executed in {duration_ms}ms, using {memory_used_bytes} bytes"
            ),
            true,
        );

        let _ = policy_id;
        Ok(())
    }

    fn add_audit_log(
        &mut self,
        sandbox_id: &str,
        plugin_id: &str,
        action: AuditLogAction,
        details: String,
        success: bool,
    ) {
        let entry = AuditLogEntry::new(sandbox_id, plugin_id, action, details, success);
        self.audit_logs.push(entry);

        if self.audit_logs.len() > self.max_audit_logs {
            let overflow = self.audit_logs.len() - self.max_audit_logs;
            self.audit_logs.drain(0..overflow);
        }
    }

    pub fn audit_logs(&self) -> &[AuditLogEntry] {
        &self.audit_logs
    }

    pub fn audit_logs_by_sandbox(&self, sandbox_id: &str) -> Vec<&AuditLogEntry> {
        self.audit_logs
            .iter()
            .filter(|e| e.sandbox_id == sandbox_id)
            .collect()
    }

    pub fn audit_logs_by_action(&self, action: &AuditLogAction) -> Vec<&AuditLogEntry> {
        self.audit_logs
            .iter()
            .filter(|e| &e.action == action)
            .collect()
    }

    pub fn running_sandboxes(&self) -> Vec<&Sandbox> {
        self.sandboxes
            .values()
            .filter(|s| s.is_running())
            .collect()
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy() -> SandboxPolicy {
        SandboxPolicy::new("test-policy", "Test Policy")
            .with_permission_mode(PermissionMode::Whitelist)
            .with_permissions(vec![
                Permission::ReadFileSystem,
                Permission::AccessArchitectureData,
            ])
            .with_resource_limit(
                ResourceLimit::new()
                    .with_memory(64 * 1024 * 1024)
                    .with_execution_time(1000)
                    .with_max_calls(10)
                    .with_cpu(30),
            )
    }

    fn create_test_manager() -> SandboxManager {
        let mut manager = SandboxManager::new();
        manager.add_policy(create_test_policy());
        manager
    }

    #[test]
    fn test_create_sandbox() {
        let mut manager = create_test_manager();
        let result = manager.create_sandbox("sb-1", "plugin-1", "test-policy");

        assert!(result.is_ok());
        let sandbox = result.unwrap();
        assert_eq!(sandbox.id, "sb-1");
        assert_eq!(sandbox.plugin_id, "plugin-1");
        assert_eq!(sandbox.policy_id, "test-policy");
        assert_eq!(sandbox.status, SandboxStatus::Created);
    }

    #[test]
    fn test_create_duplicate_sandbox_fails() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();
        let result = manager.create_sandbox("sb-1", "plugin-2", "test-policy");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::AlreadyExists(_)));
    }

    #[test]
    fn test_create_sandbox_with_invalid_policy_fails() {
        let mut manager = SandboxManager::new();
        let result = manager.create_sandbox("sb-1", "plugin-1", "nonexistent-policy");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::PolicyNotFound(_)));
    }

    #[test]
    fn test_destroy_sandbox() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        let result = manager.destroy_sandbox("sb-1");
        assert!(result.is_ok());

        let sandbox = manager.get_sandbox("sb-1").unwrap();
        assert!(sandbox.is_destroyed());
        assert!(sandbox.destroyed_at.is_some());
    }

    #[test]
    fn test_destroy_nonexistent_sandbox_fails() {
        let mut manager = create_test_manager();
        let result = manager.destroy_sandbox("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::NotFound(_)));
    }

    #[test]
    fn test_permission_check_whitelist_allowed() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        let result = manager.check_permission("sb-1", &Permission::ReadFileSystem);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_permission_check_whitelist_denied() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        let result = manager.check_permission("sb-1", &Permission::WriteFileSystem);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_permission_check_blacklist_mode() {
        let mut manager = SandboxManager::new();
        let policy = SandboxPolicy::new("blacklist-policy", "Blacklist Policy")
            .with_permission_mode(PermissionMode::Blacklist)
            .with_permissions(vec![Permission::NetworkAccess]);
        manager.add_policy(policy);
        manager.create_sandbox("sb-1", "plugin-1", "blacklist-policy").unwrap();

        let allowed = manager.check_permission("sb-1", &Permission::ReadFileSystem).unwrap();
        assert!(allowed);

        let denied = manager.check_permission("sb-1", &Permission::NetworkAccess).unwrap();
        assert!(!denied);
    }

    #[test]
    fn test_permission_all_matches_everything() {
        let mut manager = SandboxManager::new();
        let policy = SandboxPolicy::new("all-policy", "All Permissions")
            .with_permission_mode(PermissionMode::Whitelist)
            .with_permissions(vec![Permission::All]);
        manager.add_policy(policy);
        manager.create_sandbox("sb-1", "plugin-1", "all-policy").unwrap();

        assert!(manager.check_permission("sb-1", &Permission::ReadFileSystem).unwrap());
        assert!(manager.check_permission("sb-1", &Permission::NetworkAccess).unwrap());
        assert!(manager.check_permission("sb-1", &Permission::ExecuteCommand).unwrap());
    }

    #[test]
    fn test_execute_plugin_success() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        let result = manager.execute_plugin("sb-1", 100, 1024 * 1024);
        assert!(result.is_ok());

        let sandbox = manager.get_sandbox("sb-1").unwrap();
        assert_eq!(sandbox.stats.total_calls, 1);
        assert_eq!(sandbox.stats.total_duration_ms, 100);
        assert_eq!(sandbox.status, SandboxStatus::Running);
    }

    #[test]
    fn test_execute_plugin_timeout() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        let result = manager.execute_plugin("sb-1", 2000, 1024);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::ExecutionTimeout(_)));
    }

    #[test]
    fn test_execute_plugin_memory_exceeded() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        let result = manager.execute_plugin("sb-1", 100, 128 * 1024 * 1024);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::MemoryLimitExceeded(_)));
    }

    #[test]
    fn test_execute_plugin_call_limit_exceeded() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        for i in 0..10 {
            let result = manager.execute_plugin("sb-1", 50, 1024);
            assert!(result.is_ok(), "Call {} should succeed", i + 1);
        }

        let result = manager.execute_plugin("sb-1", 50, 1024);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::CallLimitExceeded(_)));
    }

    #[test]
    fn test_execute_on_destroyed_sandbox_fails() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();
        manager.destroy_sandbox("sb-1").unwrap();

        let result = manager.execute_plugin("sb-1", 50, 1024);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::InvalidState(_)));
    }

    #[test]
    fn test_audit_log_records_every_call() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        manager.execute_plugin("sb-1", 100, 1024).unwrap();
        manager.execute_plugin("sb-1", 200, 2048).unwrap();

        let logs = manager.audit_logs_by_sandbox("sb-1");
        assert!(logs.len() >= 2);

        let exec_logs = manager.audit_logs_by_action(&AuditLogAction::PluginExecuted);
        assert_eq!(exec_logs.len(), 2);
    }

    #[test]
    fn test_audit_log_includes_permission_checks() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        manager.check_permission("sb-1", &Permission::ReadFileSystem).unwrap();
        manager.check_permission("sb-1", &Permission::WriteFileSystem).unwrap();

        let granted = manager.audit_logs_by_action(&AuditLogAction::PermissionGranted);
        let denied = manager.audit_logs_by_action(&AuditLogAction::PermissionDenied);

        assert_eq!(granted.len(), 1);
        assert_eq!(denied.len(), 1);
    }

    #[test]
    fn test_policy_update() {
        let mut manager = create_test_manager();

        let new_limit = ResourceLimit::new()
            .with_memory(256 * 1024 * 1024)
            .with_execution_time(5000)
            .with_max_calls(100)
            .with_cpu(80);

        let result = manager.update_policy("test-policy", new_limit);
        assert!(result.is_ok());

        let policy = manager.get_policy("test-policy").unwrap();
        assert_eq!(policy.resource_limit.memory_bytes, 256 * 1024 * 1024);
        assert_eq!(policy.resource_limit.execution_time_ms, 5000);
        assert_eq!(policy.resource_limit.max_call_count, 100);
        assert_eq!(policy.resource_limit.cpu_percent, 80);
    }

    #[test]
    fn test_policy_update_nonexistent_fails() {
        let mut manager = create_test_manager();
        let result = manager.update_policy("nonexistent", ResourceLimit::new());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SandboxError::PolicyNotFound(_)));
    }

    #[test]
    fn test_multiple_sandboxes_isolated() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();
        manager.create_sandbox("sb-2", "plugin-2", "test-policy").unwrap();

        manager.execute_plugin("sb-1", 100, 1024).unwrap();
        manager.execute_plugin("sb-1", 200, 2048).unwrap();
        manager.execute_plugin("sb-2", 50, 512).unwrap();

        let sb1 = manager.get_sandbox("sb-1").unwrap();
        let sb2 = manager.get_sandbox("sb-2").unwrap();

        assert_eq!(sb1.stats.total_calls, 2);
        assert_eq!(sb1.stats.total_duration_ms, 300);
        assert_eq!(sb2.stats.total_calls, 1);
        assert_eq!(sb2.stats.total_duration_ms, 50);

        let logs1 = manager.audit_logs_by_sandbox("sb-1");
        let logs2 = manager.audit_logs_by_sandbox("sb-2");
        assert!(logs1.len() > logs2.len());
    }

    #[test]
    fn test_running_sandboxes_filter() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();
        manager.create_sandbox("sb-2", "plugin-2", "test-policy").unwrap();

        assert_eq!(manager.running_sandboxes().len(), 0);

        manager.execute_plugin("sb-1", 100, 1024).unwrap();
        assert_eq!(manager.running_sandboxes().len(), 1);

        manager.execute_plugin("sb-2", 100, 1024).unwrap();
        assert_eq!(manager.running_sandboxes().len(), 2);
    }

    #[test]
    fn test_sandbox_stats_peak_memory() {
        let mut manager = create_test_manager();
        manager.create_sandbox("sb-1", "plugin-1", "test-policy").unwrap();

        manager.execute_plugin("sb-1", 100, 1024).unwrap();
        manager.execute_plugin("sb-1", 100, 4096).unwrap();
        manager.execute_plugin("sb-1", 100, 2048).unwrap();

        let sandbox = manager.get_sandbox("sb-1").unwrap();
        assert_eq!(sandbox.stats.peak_memory_bytes, 4096);
    }

    #[test]
    fn test_permission_label_consistency() {
        assert_eq!(Permission::ReadFileSystem.label(), "read:fs");
        assert_eq!(Permission::WriteFileSystem.label(), "write:fs");
        assert_eq!(Permission::All.label(), "*");
    }

    #[test]
    fn test_resource_limit_default_values() {
        let limit = ResourceLimit::default();
        assert_eq!(limit.memory_bytes, 128 * 1024 * 1024);
        assert_eq!(limit.execution_time_ms, 5000);
        assert_eq!(limit.max_call_count, 1000);
        assert_eq!(limit.cpu_percent, 50);
    }
}
