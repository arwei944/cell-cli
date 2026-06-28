use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPhase {
    Init,
    Planning,
    Implementation,
    Verification,
    Handoff,
    Complete,
}

impl WorkflowPhase {
    pub fn order(&self) -> u8 {
        match self {
            Self::Init => 0,
            Self::Planning => 1,
            Self::Implementation => 2,
            Self::Verification => 3,
            Self::Handoff => 4,
            Self::Complete => 5,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Init => "init",
            Self::Planning => "planning",
            Self::Implementation => "implementation",
            Self::Verification => "verification",
            Self::Handoff => "handoff",
            Self::Complete => "complete",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Init => "🚀 初始化",
            Self::Planning => "📋 规划设计",
            Self::Implementation => "💻 开发实现",
            Self::Verification => "✅ 验证测试",
            Self::Handoff => "📦 交接交付",
            Self::Complete => "🎉 完成",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Init => Some(Self::Planning),
            Self::Planning => Some(Self::Implementation),
            Self::Implementation => Some(Self::Verification),
            Self::Verification => Some(Self::Handoff),
            Self::Handoff => Some(Self::Complete),
            Self::Complete => None,
        }
    }

    pub fn required_gates(&self) -> Vec<&str> {
        match self {
            Self::Init => vec!["task_defined", "baseline_captured"],
            Self::Planning => vec!["architecture_reviewed", "decisions_recorded"],
            Self::Implementation => vec!["code_checkpoint_passed", "entropy_controlled"],
            Self::Verification => vec!["tests_passing", "arch_compliant", "entropy_gate_passed"],
            Self::Handoff => vec!["handoff_generated", "progress_complete"],
            Self::Complete => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_name: String,
    pub passed: bool,
    pub detail: String,
    pub measured_at: String,
    pub metrics: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from: WorkflowPhase,
    pub to: WorkflowPhase,
    pub agent_id: String,
    pub transitioned_at: String,
    pub gates_passed: Vec<GateResult>,
    pub gates_failed: Vec<GateResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflow_id: Uuid,
    pub task_name: String,
    pub task_description: String,
    pub current_phase: WorkflowPhase,
    pub registered_agents: Vec<AgentInfo>,
    pub active_agent: Option<String>,
    pub transitions: Vec<PhaseTransition>,
    pub gates: HashMap<String, GateResult>,
    pub baseline: WorkflowBaseline,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowBaseline {
    pub entropy_score: Option<f64>,
    pub arch_violations: Option<usize>,
    pub test_count: Option<usize>,
    pub test_coverage: Option<f64>,
    pub captured_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCommand {
    pub action: WorkflowAction,
    pub agent_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowAction {
    RegisterAgent,
    Start,
    AdvancePhase,
    RecordGate,
    GetStatus,
    GetNextPhase,
    CheckGates,
    ListPhases,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub success: bool,
    pub workflow_id: Option<Uuid>,
    pub current_phase: Option<WorkflowPhase>,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub errors: Vec<String>,
}

impl WorkflowState {
    pub fn new(task_name: &str, task_description: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            workflow_id: Uuid::new_v4(),
            task_name: task_name.to_string(),
            task_description: task_description.to_string(),
            current_phase: WorkflowPhase::Init,
            registered_agents: Vec::new(),
            active_agent: None,
            transitions: Vec::new(),
            gates: HashMap::new(),
            baseline: WorkflowBaseline::default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn can_advance_to(&self, target: &WorkflowPhase) -> bool {
        if let Some(next) = self.current_phase.next() {
            next == *target || target.order() > self.current_phase.order() && target.order() <= next.order()
        } else {
            false
        }
    }

    pub fn current_gates(&self) -> Vec<&str> {
        self.current_phase.required_gates()
    }

    pub fn all_current_gates_passed(&self) -> bool {
        self.current_gates().iter().all(|gate| {
            self.gates.get(*gate).is_some_and(|g| g.passed)
        })
    }

    pub fn register_agent(&mut self, agent: AgentInfo) {
        if !self.registered_agents.iter().any(|a| a.id == agent.id) {
            self.registered_agents.push(agent);
        }
    }

    pub fn record_gate(&mut self, gate: GateResult) {
        self.gates.insert(gate.gate_name.clone(), gate);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn advance_phase(&mut self, agent_id: &str, gates_passed: Vec<GateResult>, gates_failed: Vec<GateResult>) -> Option<WorkflowPhase> {
        if let Some(next_phase) = self.current_phase.next() {
            let transition = PhaseTransition {
                from: self.current_phase.clone(),
                to: next_phase.clone(),
                agent_id: agent_id.to_string(),
                transitioned_at: chrono::Utc::now().to_rfc3339(),
                gates_passed,
                gates_failed,
            };
            self.transitions.push(transition);
            self.current_phase = next_phase.clone();
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Some(next_phase)
        } else {
            None
        }
    }
}

impl WorkflowResponse {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            workflow_id: None,
            current_phase: None,
            message: message.to_string(),
            data: None,
            errors: Vec::new(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            workflow_id: None,
            current_phase: None,
            message: message.to_string(),
            data: None,
            errors: vec![message.to_string()],
        }
    }

    pub fn with_state(mut self, state: &WorkflowState) -> Self {
        self.workflow_id = Some(state.workflow_id);
        self.current_phase = Some(state.current_phase.clone());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}
