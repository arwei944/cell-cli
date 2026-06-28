use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditActionType {
    ToolCall,
    FileRead,
    FileWrite,
    FileDelete,
    CommandExec,
    Decision,
    TaskStart,
    TaskComplete,
    GitCommit,
    GitPush,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
    Blocked,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: String,
    pub agent_id: String,
    pub action_type: AuditActionType,
    pub action: String,
    pub result: AuditResult,
    pub details: Option<String>,
    pub duration_ms: Option<u64>,
    pub task_id: Option<String>,
    pub phase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub agent_id: Option<String>,
    pub action_type: Option<AuditActionType>,
    pub result: Option<AuditResult>,
    pub task_id: Option<String>,
    pub from_time: Option<String>,
    pub to_time: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub total_entries: usize,
    pub by_agent: std::collections::HashMap<String, usize>,
    pub by_action: std::collections::HashMap<String, usize>,
    pub by_result: std::collections::HashMap<String, usize>,
    pub entries: Vec<AuditLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResult {
    pub file: String,
    pub line: Option<u32>,
    pub last_modified_by: Option<String>,
    pub last_modified_at: Option<String>,
    pub related_commits: Vec<String>,
    pub related_tasks: Vec<String>,
}

pub struct AuditService;

impl AuditService {
    pub fn new() -> Self {
        Self
    }

    pub fn log_action(
        &self,
        project_path: &str,
        agent_id: &str,
        action_type: AuditActionType,
        action: &str,
        result: AuditResult,
        details: Option<&str>,
        duration_ms: Option<u64>,
        task_id: Option<&str>,
        phase: Option<&str>,
    ) -> CellResult<String> {
        let entry = AuditLogEntry {
            id: format!("audit-{}", uuid::Uuid::new_v4().simple()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent_id: agent_id.to_string(),
            action_type,
            action: action.to_string(),
            result,
            details: details.map(std::string::ToString::to_string),
            duration_ms,
            task_id: task_id.map(std::string::ToString::to_string),
            phase: phase.map(std::string::ToString::to_string),
        };

        self.save_entry(project_path, &entry)?;
        Ok(entry.id)
    }

    fn save_entry(&self, project_path: &str, entry: &AuditLogEntry) -> CellResult<()> {
        use std::io::Write;
        let audit_dir = Path::new(project_path).join(".cell").join("audit");
        std::fs::create_dir_all(&audit_dir)?;

        let date_str = chrono::Utc::now().format("%Y%m%d").to_string();
        let file_path = audit_dir.join(format!("audit-{date_str}.jsonl"));

        let line = serde_json::to_string(entry)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        writeln!(file, "{line}")?;

        Ok(())
    }

    pub fn query(&self, project_path: &str, query: AuditQuery) -> CellResult<AuditReport> {
        let audit_dir = Path::new(project_path).join(".cell").join("audit");
        let mut all_entries: Vec<AuditLogEntry> = Vec::new();

        if audit_dir.exists() {
            for entry in std::fs::read_dir(&audit_dir)? {
                let entry = entry?;
                if entry.path().extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    let content = std::fs::read_to_string(entry.path())?;
                    for line in content.lines() {
                        if let Ok(log_entry) = serde_json::from_str::<AuditLogEntry>(line) {
                            all_entries.push(log_entry);
                        }
                    }
                }
            }
        }

        let filtered: Vec<AuditLogEntry> = all_entries
            .into_iter()
            .filter(|e| {
                if let Some(agent) = &query.agent_id
                    && &e.agent_id != agent {
                        return false;
                    }
                if let Some(action_type) = &query.action_type
                    && &e.action_type != action_type {
                        return false;
                    }
                if let Some(result) = &query.result
                    && &e.result != result {
                        return false;
                    }
                if let Some(task_id) = &query.task_id
                    && e.task_id.as_deref() != Some(task_id.as_str()) {
                        return false;
                    }
                true
            })
            .take(query.limit)
            .collect();

        let mut by_agent = std::collections::HashMap::new();
        let mut by_action = std::collections::HashMap::new();
        let mut by_result = std::collections::HashMap::new();

        for e in &filtered {
            *by_agent.entry(e.agent_id.clone()).or_insert(0) += 1;
            *by_action
                .entry(format!("{:?}", e.action_type))
                .or_insert(0) += 1;
            *by_result.entry(format!("{:?}", e.result)).or_insert(0) += 1;
        }

        Ok(AuditReport {
            total_entries: filtered.len(),
            by_agent,
            by_action,
            by_result,
            entries: filtered,
        })
    }

    pub fn trace_file(&self, project_path: &str, file_path: &str) -> CellResult<TraceResult> {
        let mut related_commits = Vec::new();
        let mut last_modified_by = None;
        let mut last_modified_at = None;

        let blame_output = std::process::Command::new("git")
            .args(["blame", "--line-porcelain", file_path])
            .current_dir(project_path)
            .output();

        if let Ok(output) = blame_output
            && output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.starts_with("author ") {
                        last_modified_by = Some(line.trim_start_matches("author ").to_string());
                    }
                    if line.starts_with("author-time ")
                        && let Ok(ts) = line.trim_start_matches("author-time ").parse::<i64>() {
                            let dt = chrono::DateTime::from_timestamp(ts, 0);
                            last_modified_at = dt.map(|d| d.to_rfc3339());
                        }
                    if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit())
                        && !related_commits.contains(&line.to_string()) {
                            related_commits.push(line.to_string());
                        }
                }
            }

        Ok(TraceResult {
            file: file_path.to_string(),
            line: None,
            last_modified_by,
            last_modified_at,
            related_commits,
            related_tasks: Vec::new(),
        })
    }

    pub fn format_report(&self, report: &AuditReport) -> String {
        let mut output = String::new();

        output.push_str("\n📜 审计日志报告\n\n");
        output.push_str(&format!("  总记录数: {}\n", report.total_entries));

        output.push_str("\n  按 Agent:\n");
        for (agent, count) in &report.by_agent {
            output.push_str(&format!("    {agent}: {count}\n"));
        }

        output.push_str("\n  按操作类型:\n");
        for (action, count) in &report.by_action {
            output.push_str(&format!("    {action}: {count}\n"));
        }

        output.push_str("\n  按结果:\n");
        for (result, count) in &report.by_result {
            output.push_str(&format!("    {result}: {count}\n"));
        }

        if !report.entries.is_empty() {
            output.push_str("\n  最近记录:\n\n");
            for (i, entry) in report.entries.iter().take(20).enumerate() {
                let result_icon = match entry.result {
                    AuditResult::Success => "✅",
                    AuditResult::Failure => "❌",
                    AuditResult::Blocked => "🚫",
                    AuditResult::Warning => "⚠️",
                };
                output.push_str(&format!(
                    "  {}. {} [{}] {} - {}\n",
                    i + 1,
                    result_icon,
                    entry.agent_id,
                    entry.action,
                    entry.timestamp
                ));
            }
        }

        output
    }

    pub fn format_trace(&self, trace: &TraceResult) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n🔍 文件追溯: {}\n\n", trace.file));

        if let Some(author) = &trace.last_modified_by {
            output.push_str(&format!("  最后修改者: {author}\n"));
        }
        if let Some(time) = &trace.last_modified_at {
            output.push_str(&format!("  最后修改时间: {time}\n"));
        }

        if !trace.related_commits.is_empty() {
            output.push_str(&format!(
                "\n  相关提交 ({} 个):\n",
                trace.related_commits.len()
            ));
            for commit in trace.related_commits.iter().take(10) {
                output.push_str(&format!("    {}\n", &commit[..8]));
            }
        }

        if !trace.related_tasks.is_empty() {
            output.push_str(&format!(
                "\n  相关任务 ({} 个):\n",
                trace.related_tasks.len()
            ));
            for task in &trace.related_tasks {
                output.push_str(&format!("    {task}\n"));
            }
        }

        output
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_service_new() {
        let service = AuditService::new();
        let _ = service;
    }

    #[test]
    fn test_log_action() {
        let service = AuditService::new();
        let id = service
            .log_action(
                ".",
                "test-agent",
                AuditActionType::ToolCall,
                "test action",
                AuditResult::Success,
                None,
                Some(100),
                None,
                None,
            )
            .unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_query_audit() {
        let service = AuditService::new();
        let query = AuditQuery {
            agent_id: None,
            action_type: None,
            result: None,
            task_id: None,
            from_time: None,
            to_time: None,
            limit: 10,
        };
        let report = service.query(".", query).unwrap();
        // 审计记录可能存在也可能不存在，不做断言
        let _ = report;
    }

    #[test]
    fn test_trace_file() {
        let service = AuditService::new();
        let result = service.trace_file(".", "Cargo.toml").unwrap();
        assert_eq!(result.file, "Cargo.toml");
    }
}
