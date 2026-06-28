use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub last_heartbeat: String,
    pub current_task: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Architect,
    Developer,
    Tester,
    Reviewer,
    Coordinator,
}

impl AgentRole {
    pub fn label(&self) -> &str {
        match self {
            Self::Architect => "架构师",
            Self::Developer => "开发者",
            Self::Tester => "测试者",
            Self::Reviewer => "审查者",
            Self::Coordinator => "协调者",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Busy,
    Waiting,
    Offline,
}

impl AgentStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Idle => "空闲",
            Self::Busy => "忙碌",
            Self::Waiting => "等待",
            Self::Offline => "离线",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub dependencies: Vec<String>,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn label(&self) -> &str {
        match self {
            Self::Pending => "待处理",
            Self::Assigned => "已分配",
            Self::InProgress => "进行中",
            Self::Completed => "已完成",
            Self::Failed => "失败",
            Self::Cancelled => "取消",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub artifacts: Vec<String>,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPackage {
    pub from_agent: String,
    pub to_agent: Option<String>,
    pub task_id: String,
    pub context: HandoffContext,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffContext {
    pub architecture_snapshot: Option<serde_json::Value>,
    pub pending_decisions: Vec<String>,
    pub active_features: Vec<String>,
    pub recent_files: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProtocol {
    pub agents: Vec<AgentInfo>,
    pub tasks: Vec<Task>,
    pub handoffs: Vec<HandoffPackage>,
}

pub struct AgentProtocolService;

impl AgentProtocolService {
    pub fn new() -> Self {
        Self
    }

    pub fn register_agent(&self, project_path: &str, name: &str, role: AgentRole, capabilities: Vec<String>) -> CellResult<AgentInfo> {
        let mut protocol = self.load_protocol(project_path)?;
        
        let id = uuid::Uuid::new_v4().to_string();
        let agent = AgentInfo {
            id,
            name: name.to_string(),
            role,
            status: AgentStatus::Idle,
            last_heartbeat: chrono::Utc::now().to_rfc3339(),
            current_task: None,
            capabilities,
        };

        protocol.agents.push(agent.clone());
        self.save_protocol(project_path, &protocol)?;
        Ok(agent)
    }

    pub fn unregister_agent(&self, project_path: &str, agent_id: &str) -> CellResult<()> {
        let mut protocol = self.load_protocol(project_path)?;
        protocol.agents.retain(|a| a.id != agent_id);
        self.save_protocol(project_path, &protocol)?;
        Ok(())
    }

    pub fn heartbeat(&self, project_path: &str, agent_id: &str) -> CellResult<()> {
        let mut protocol = self.load_protocol(project_path)?;
        for agent in &mut protocol.agents {
            if agent.id == agent_id {
                agent.last_heartbeat = chrono::Utc::now().to_rfc3339();
            }
        }
        self.save_protocol(project_path, &protocol)?;
        Ok(())
    }

    pub fn list_agents(&self, project_path: &str) -> CellResult<Vec<AgentInfo>> {
        let protocol = self.load_protocol(project_path)?;
        Ok(protocol.agents)
    }

    pub fn create_task(&self, project_path: &str, name: &str, description: &str, dependencies: Vec<String>) -> CellResult<Task> {
        let mut protocol = self.load_protocol(project_path)?;
        
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let task = Task {
            id,
            name: name.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            assigned_to: None,
            created_at: now.clone(),
            updated_at: now,
            dependencies,
            result: None,
        };

        protocol.tasks.push(task.clone());
        self.save_protocol(project_path, &protocol)?;
        Ok(task)
    }

    pub fn assign_task(&self, project_path: &str, task_id: &str, agent_id: &str) -> CellResult<()> {
        let mut protocol = self.load_protocol(project_path)?;
        
        for task in &mut protocol.tasks {
            if task.id == task_id {
                task.assigned_to = Some(agent_id.to_string());
                task.status = TaskStatus::Assigned;
                task.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }
        
        for agent in &mut protocol.agents {
            if agent.id == agent_id {
                agent.current_task = Some(task_id.to_string());
                agent.status = AgentStatus::Busy;
            }
        }

        self.save_protocol(project_path, &protocol)?;
        Ok(())
    }

    pub fn delegate_task(&self, project_path: &str, task_id: &str, agent_id: &str) -> CellResult<Task> {
        self.assign_task(project_path, task_id, agent_id)?;
        let protocol = self.load_protocol(project_path)?;
        protocol.tasks.into_iter().find(|t| t.id == task_id)
            .ok_or_else(|| cell_domain::errors::CellError::Config(format!("Task {task_id} not found")))
    }

    pub fn start_task(&self, project_path: &str, task_id: &str) -> CellResult<()> {
        let mut protocol = self.load_protocol(project_path)?;
        
        for task in &mut protocol.tasks {
            if task.id == task_id {
                task.status = TaskStatus::InProgress;
                task.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }

        self.save_protocol(project_path, &protocol)?;
        Ok(())
    }

    pub fn complete_task(&self, project_path: &str, task_id: &str, success: bool, output: &str, artifacts: Vec<String>) -> CellResult<()> {
        let mut protocol = self.load_protocol(project_path)?;
        
        for task in &mut protocol.tasks {
            if task.id == task_id {
                task.status = if success { TaskStatus::Completed } else { TaskStatus::Failed };
                task.result = Some(TaskResult {
                    success,
                    output: output.to_string(),
                    artifacts: artifacts.clone(),
                    completed_at: chrono::Utc::now().to_rfc3339(),
                });
                task.updated_at = chrono::Utc::now().to_rfc3339();
            }
        }

        for agent in &mut protocol.agents {
            if agent.current_task.as_deref() == Some(task_id) {
                agent.current_task = None;
                agent.status = AgentStatus::Idle;
            }
        }

        self.save_protocol(project_path, &protocol)?;
        Ok(())
    }

    pub fn list_tasks(&self, project_path: &str) -> CellResult<Vec<Task>> {
        let protocol = self.load_protocol(project_path)?;
        Ok(protocol.tasks)
    }

    pub fn create_handoff(&self, project_path: &str, from_agent: &str, to_agent: Option<&str>, task_id: &str, notes: &str) -> CellResult<HandoffPackage> {
        let mut protocol = self.load_protocol(project_path)?;
        
        let handoff = HandoffPackage {
            from_agent: from_agent.to_string(),
            to_agent: to_agent.map(std::string::ToString::to_string),
            task_id: task_id.to_string(),
            context: HandoffContext {
                architecture_snapshot: None,
                pending_decisions: Vec::new(),
                active_features: Vec::new(),
                recent_files: Vec::new(),
                notes: notes.to_string(),
            },
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        protocol.handoffs.push(handoff.clone());
        self.save_protocol(project_path, &protocol)?;
        Ok(handoff)
    }

    pub fn list_handoffs(&self, project_path: &str) -> CellResult<Vec<HandoffPackage>> {
        let protocol = self.load_protocol(project_path)?;
        Ok(protocol.handoffs)
    }

    pub fn get_agent_status(&self, project_path: &str, agent_id: &str) -> CellResult<AgentInfo> {
        let protocol = self.load_protocol(project_path)?;
        protocol.agents.into_iter().find(|a| a.id == agent_id)
            .ok_or_else(|| cell_domain::errors::CellError::Config(format!("Agent {agent_id} not found")))
    }

    fn protocol_path(project_path: &str) -> std::path::PathBuf {
        Path::new(project_path).join(".cell/agent_protocol.json")
    }

    fn load_protocol(&self, project_path: &str) -> CellResult<AgentProtocol> {
        let path = Self::protocol_path(project_path);
        if !path.exists() {
            return Ok(AgentProtocol {
                agents: Vec::new(),
                tasks: Vec::new(),
                handoffs: Vec::new(),
            });
        }
        let content = std::fs::read_to_string(&path)?;
        let protocol: AgentProtocol = serde_json::from_str(&content)
            .map_err(|e| cell_domain::errors::CellError::Config(format!("Invalid protocol file: {e}")))?;
        Ok(protocol)
    }

    fn save_protocol(&self, project_path: &str, protocol: &AgentProtocol) -> CellResult<()> {
        let path = Self::protocol_path(project_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(protocol)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for AgentProtocolService {
    fn default() -> Self {
        Self::new()
    }
}