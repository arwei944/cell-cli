use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Architect,
    Developer,
    Tester,
    Reviewer,
    Observer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk_level: ToolRiskLevel,
    pub allowed_roles: Vec<AgentRole>,
    pub rate_limit_per_minute: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageRecord {
    pub tool_id: String,
    pub agent_id: String,
    pub timestamp: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicyReport {
    pub total_tools: usize,
    pub enabled_tools: usize,
    pub by_risk: std::collections::HashMap<String, usize>,
    pub by_category: std::collections::HashMap<String, usize>,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub tool_id: String,
    pub agent_id: String,
    pub action: String,
    pub timestamp: String,
    pub success: bool,
    pub details: Option<String>,
}

pub struct ToolPolicyService {
    tools: Vec<ToolDefinition>,
}

impl ToolPolicyService {
    pub fn new() -> Self {
        Self {
            tools: Self::default_tools(),
        }
    }

    fn default_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                id: "tool_read_file".to_string(),
                name: "Read File".to_string(),
                description: "读取文件内容".to_string(),
                category: "filesystem".to_string(),
                risk_level: ToolRiskLevel::Low,
                allowed_roles: vec![
                    AgentRole::Architect,
                    AgentRole::Developer,
                    AgentRole::Tester,
                    AgentRole::Reviewer,
                    AgentRole::Observer,
                ],
                rate_limit_per_minute: Some(60),
                enabled: true,
            },
            ToolDefinition {
                id: "tool_edit_file".to_string(),
                name: "Edit File".to_string(),
                description: "修改文件内容".to_string(),
                category: "filesystem".to_string(),
                risk_level: ToolRiskLevel::Medium,
                allowed_roles: vec![AgentRole::Architect, AgentRole::Developer],
                rate_limit_per_minute: Some(30),
                enabled: true,
            },
            ToolDefinition {
                id: "tool_write_file".to_string(),
                name: "Write File".to_string(),
                description: "创建新文件".to_string(),
                category: "filesystem".to_string(),
                risk_level: ToolRiskLevel::Medium,
                allowed_roles: vec![AgentRole::Architect, AgentRole::Developer],
                rate_limit_per_minute: Some(20),
                enabled: true,
            },
            ToolDefinition {
                id: "tool_delete_file".to_string(),
                name: "Delete File".to_string(),
                description: "删除文件".to_string(),
                category: "filesystem".to_string(),
                risk_level: ToolRiskLevel::High,
                allowed_roles: vec![AgentRole::Architect],
                rate_limit_per_minute: Some(5),
                enabled: true,
            },
            ToolDefinition {
                id: "tool_run_command".to_string(),
                name: "Run Command".to_string(),
                description: "执行 shell 命令".to_string(),
                category: "execution".to_string(),
                risk_level: ToolRiskLevel::High,
                allowed_roles: vec![AgentRole::Architect, AgentRole::Developer],
                rate_limit_per_minute: Some(10),
                enabled: true,
            },
            ToolDefinition {
                id: "tool_search".to_string(),
                name: "Search Codebase".to_string(),
                description: "搜索代码库".to_string(),
                category: "analysis".to_string(),
                risk_level: ToolRiskLevel::Low,
                allowed_roles: vec![
                    AgentRole::Architect,
                    AgentRole::Developer,
                    AgentRole::Tester,
                    AgentRole::Reviewer,
                    AgentRole::Observer,
                ],
                rate_limit_per_minute: Some(30),
                enabled: true,
            },
            ToolDefinition {
                id: "tool_git".to_string(),
                name: "Git Operations".to_string(),
                description: "Git 版本控制操作".to_string(),
                category: "vcs".to_string(),
                risk_level: ToolRiskLevel::Medium,
                allowed_roles: vec![AgentRole::Architect, AgentRole::Developer],
                rate_limit_per_minute: Some(20),
                enabled: true,
            },
            ToolDefinition {
                id: "mcp_browser".to_string(),
                name: "Integrated Browser".to_string(),
                description: "集成浏览器 MCP".to_string(),
                category: "mcp".to_string(),
                risk_level: ToolRiskLevel::Medium,
                allowed_roles: vec![AgentRole::Architect, AgentRole::Developer],
                rate_limit_per_minute: Some(15),
                enabled: true,
            },
            ToolDefinition {
                id: "cell_arch".to_string(),
                name: "Cell Architecture CLI".to_string(),
                description: "Cell 架构工具链命令行工具".to_string(),
                category: "cell".to_string(),
                risk_level: ToolRiskLevel::Low,
                allowed_roles: vec![
                    AgentRole::Architect,
                    AgentRole::Developer,
                    AgentRole::Tester,
                    AgentRole::Reviewer,
                ],
                rate_limit_per_minute: None,
                enabled: true,
            },
            ToolDefinition {
                id: "cell_entropy".to_string(),
                name: "Cell Entropy".to_string(),
                description: "熵值计算工具".to_string(),
                category: "cell".to_string(),
                risk_level: ToolRiskLevel::Low,
                allowed_roles: vec![
                    AgentRole::Architect,
                    AgentRole::Developer,
                    AgentRole::Tester,
                    AgentRole::Reviewer,
                    AgentRole::Observer,
                ],
                rate_limit_per_minute: None,
                enabled: true,
            },
        ]
    }

    pub fn check_tool_access(
        &self,
        tool_id: &str,
        agent_role: &AgentRole,
    ) -> Result<(), String> {
        let tool = self.tools.iter().find(|t| t.id == tool_id);

        match tool {
            None => Err(format!("工具不存在: {}", tool_id)),
            Some(tool) if !tool.enabled => Err(format!("工具已禁用: {}", tool_id)),
            Some(tool) if !tool.allowed_roles.contains(agent_role) => Err(format!(
                "角色 {:?} 无权使用工具 {}",
                agent_role, tool_id
            )),
            Some(_) => Ok(()),
        }
    }

    pub fn get_tools(&self, role: Option<&AgentRole>) -> Vec<&ToolDefinition> {
        self.tools
            .iter()
            .filter(|t| t.enabled)
            .filter(|t| role.map(|r| t.allowed_roles.contains(r)).unwrap_or(true))
            .collect()
    }

    pub fn get_tool(&self, tool_id: &str) -> Option<&ToolDefinition> {
        self.tools.iter().find(|t| t.id == tool_id)
    }

    pub fn list_tools(&self, project_path: &str) -> CellResult<ToolPolicyReport> {
        let tools = self.tools.clone();

        let mut by_risk = std::collections::HashMap::new();
        let mut by_category = std::collections::HashMap::new();

        for tool in &tools {
            *by_risk
                .entry(format!("{:?}", tool.risk_level))
                .or_insert(0) += 1;
            *by_category
                .entry(tool.category.clone())
                .or_insert(0) += 1;
        }

        let enabled_tools = tools.iter().filter(|t| t.enabled).count();

        let _ = project_path;

        Ok(ToolPolicyReport {
            total_tools: tools.len(),
            enabled_tools,
            by_risk,
            by_category,
            tools,
        })
    }

    pub fn record_usage(
        &self,
        project_path: &str,
        tool_id: &str,
        agent_id: &str,
        duration_ms: u64,
        success: bool,
        error_message: Option<&str>,
    ) -> CellResult<()> {
        let audit_dir = Path::new(project_path).join(".cell").join("audit").join("tools");
        std::fs::create_dir_all(&audit_dir)?;

        let record = ToolUsageRecord {
            tool_id: tool_id.to_string(),
            agent_id: agent_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms,
            success,
            error_message: error_message.map(|s| s.to_string()),
        };

        let date_str = chrono::Utc::now().format("%Y%m%d").to_string();
        let file_path = audit_dir.join(format!("usage-{}.jsonl", date_str));

        let line = serde_json::to_string(&record)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        use std::io::Write;
        writeln!(file, "{}", line)?;

        Ok(())
    }

    pub fn format_report(&self, report: &ToolPolicyReport) -> String {
        let mut output = String::new();

        output.push_str("\n🛠️  工具白名单报告\n\n");
        output.push_str(&format!("  总工具数: {}\n", report.total_tools));
        output.push_str(&format!("  已启用: {}\n", report.enabled_tools));

        output.push_str("\n  按风险等级:\n");
        for (risk, count) in &report.by_risk {
            output.push_str(&format!("    {}: {}\n", risk, count));
        }

        output.push_str("\n  按类别:\n");
        for (cat, count) in &report.by_category {
            output.push_str(&format!("    {}: {}\n", cat, count));
        }

        output.push_str("\n  工具列表:\n\n");

        for tool in &report.tools {
            let risk_icon = match tool.risk_level {
                ToolRiskLevel::Low => "🟢",
                ToolRiskLevel::Medium => "🟡",
                ToolRiskLevel::High => "🟠",
                ToolRiskLevel::Critical => "🔴",
            };
            let status = if tool.enabled { "✅" } else { "🚫" };

            output.push_str(&format!(
                "  {} {} {} - {}\n",
                status, risk_icon, tool.id, tool.name
            ));
            output.push_str(&format!("     类别: {} | 风险: {:?}\n", tool.category, tool.risk_level));
            output.push_str(&format!(
                "     允许角色: {:?}\n\n",
                tool.allowed_roles
                    .iter()
                    .map(|r| format!("{:?}", r))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        output
    }

    pub fn format_tool_detail(&self, tool: &ToolDefinition) -> String {
        let mut output = String::new();

        let risk_icon = match tool.risk_level {
            ToolRiskLevel::Low => "🟢",
            ToolRiskLevel::Medium => "🟡",
            ToolRiskLevel::High => "🟠",
            ToolRiskLevel::Critical => "🔴",
        };

        output.push_str(&format!("\n{} 工具详情: {}\n\n", risk_icon, tool.name));
        output.push_str(&format!("  ID: {}\n", tool.id));
        output.push_str(&format!("  描述: {}\n", tool.description));
        output.push_str(&format!("  类别: {}\n", tool.category));
        output.push_str(&format!("  风险等级: {:?}\n", tool.risk_level));
        output.push_str(&format!("  状态: {}\n", if tool.enabled { "已启用" } else { "已禁用" }));

        if let Some(limit) = tool.rate_limit_per_minute {
            output.push_str(&format!("  频率限制: {}/分钟\n", limit));
        }

        output.push_str(&format!(
            "  允许角色: {}\n",
            tool.allowed_roles
                .iter()
                .map(|r| format!("{:?}", r))
                .collect::<Vec<_>>()
                .join(", ")
        ));

        output
    }
}

impl Default for ToolPolicyService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_policy_new() {
        let service = ToolPolicyService::new();
        assert!(!service.get_tools(None).is_empty());
    }

    #[test]
    fn test_check_access_developer_read() {
        let service = ToolPolicyService::new();
        let result = service.check_tool_access("tool_read_file", &AgentRole::Developer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_access_observer_delete() {
        let service = ToolPolicyService::new();
        let result = service.check_tool_access("tool_delete_file", &AgentRole::Observer);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_access_nonexistent() {
        let service = ToolPolicyService::new();
        let result = service.check_tool_access("nonexistent", &AgentRole::Architect);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_tools() {
        let service = ToolPolicyService::new();
        let report = service.list_tools(".").unwrap();
        assert!(report.total_tools > 0);
    }

    #[test]
    fn test_get_tool() {
        let service = ToolPolicyService::new();
        let tool = service.get_tool("cell_arch");
        assert!(tool.is_some());
    }
}
