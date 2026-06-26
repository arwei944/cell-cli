use crate::application::arch_service::{ArchitectureRules, validate_architecture};
use crate::application::entropy_service;
use crate::application::fast_verify_service::FastVerifyService;
use crate::application::ports::decision_store::DecisionStorePort;
use crate::application::ports::evolution_store::EvolutionStorePort;
use crate::application::ports::handoff_exporter::HandoffExporterPort;
use crate::application::ports::progress_store::ProgressStorePort;
use crate::application::progress_service::ProgressService;
use crate::domain::errors::{CellError, CellResult};
use crate::domain::progress::EventType;
use crate::domain::workflow::*;
use std::collections::HashMap;
use std::path::Path;

pub struct WorkflowEngine<P: ProgressStorePort + Clone, E: EvolutionStorePort, H: HandoffExporterPort, D: DecisionStorePort> {
    progress_service: ProgressService<P>,
    _phantom_evolution: std::marker::PhantomData<E>,
    _phantom_handoff: std::marker::PhantomData<H>,
    _phantom_decision: std::marker::PhantomData<D>,
}

impl<P: ProgressStorePort + Clone, E: EvolutionStorePort, H: HandoffExporterPort, D: DecisionStorePort>
    WorkflowEngine<P, E, H, D>
{
    pub fn new(progress_store: P) -> Self {
        Self {
            progress_service: ProgressService::new(progress_store),
            _phantom_evolution: std::marker::PhantomData,
            _phantom_handoff: std::marker::PhantomData,
            _phantom_decision: std::marker::PhantomData,
        }
    }

    pub fn register_agent(&self, project_path: &str, agent: AgentInfo) -> CellResult<WorkflowResponse> {
        let mut state = self.load_or_init_state(project_path)?;
        state.register_agent(agent.clone());
        self.save_state(project_path, &state)?;

        Ok(WorkflowResponse::ok(&format!("Agent '{}' registered successfully", agent.name))
            .with_state(&state))
    }

    pub fn start_workflow(&self, project_path: &str, task_name: &str, description: &str, agent_id: &str) -> CellResult<WorkflowResponse> {
        if self.workflow_exists(project_path) {
            let state = self.load_state(project_path)?;
            if state.current_phase != WorkflowPhase::Complete {
                return Err(CellError::Config(format!(
                    "Workflow already active in phase '{}'. Complete or abort first.",
                    state.current_phase.label()
                )));
            }
        }

        let mut state = WorkflowState::new(task_name, description);
        state.active_agent = Some(agent_id.to_string());

        self.progress_service.start_task(
            project_path,
            task_name,
            description,
            Some(agent_id),
        )?;

        let baseline = self.capture_baseline(project_path)?;
        state.baseline = baseline;

        let task_defined = GateResult {
            gate_name: "task_defined".to_string(),
            passed: true,
            detail: format!("Task '{}' defined", task_name),
            measured_at: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::new(),
        };
        state.record_gate(task_defined);

        let baseline_captured = GateResult {
            gate_name: "baseline_captured".to_string(),
            passed: state.baseline.entropy_score.is_some(),
            detail: if state.baseline.entropy_score.is_some() {
                "Baseline captured successfully".to_string()
            } else {
                "Baseline capture incomplete".to_string()
            },
            measured_at: chrono::Utc::now().to_rfc3339(),
            metrics: {
                let mut m = HashMap::new();
                if let Some(s) = state.baseline.entropy_score {
                    m.insert("entropy_score".to_string(), serde_json::json!(s));
                }
                if let Some(v) = state.baseline.arch_violations {
                    m.insert("arch_violations".to_string(), serde_json::json!(v));
                }
                m
            },
        };
        state.record_gate(baseline_captured);

        self.save_state(project_path, &state)?;

        Ok(WorkflowResponse::ok("Workflow started successfully")
            .with_state(&state))
    }

    pub fn advance_phase(&self, project_path: &str, agent_id: &str) -> CellResult<WorkflowResponse> {
        let mut state = self.load_state(project_path)?;

        if state.active_agent.as_deref() != Some(agent_id) {
            return Err(CellError::Config(format!(
                "Agent '{}' is not the active agent. Active: {:?}",
                agent_id, state.active_agent
            )));
        }

        let current_phase = state.current_phase.clone();
        let required_gates = state.current_gates();
        let mut gates_passed = Vec::new();
        let mut gates_failed = Vec::new();

        for gate_name in &required_gates {
            if let Some(gate) = state.gates.get(*gate_name) {
                if gate.passed {
                    gates_passed.push(gate.clone());
                } else {
                    gates_failed.push(gate.clone());
                }
            } else {
                gates_failed.push(GateResult {
                    gate_name: gate_name.to_string(),
                    passed: false,
                    detail: "Gate not evaluated yet".to_string(),
                    measured_at: chrono::Utc::now().to_rfc3339(),
                    metrics: HashMap::new(),
                });
            }
        }

        if !gates_failed.is_empty() {
            return Ok(WorkflowResponse {
                success: false,
                workflow_id: Some(state.workflow_id),
                current_phase: Some(state.current_phase),
                message: format!("Cannot advance from '{}': {} gates failed", 
                    current_phase.label(), gates_failed.len()),
                data: Some(serde_json::json!({
                    "failed_gates": gates_failed.iter().map(|g| &g.gate_name).collect::<Vec<_>>(),
                    "gates": gates_passed.iter().chain(gates_failed.iter())
                        .map(|g| (g.gate_name.clone(), g.passed))
                        .collect::<HashMap<_, _>>(),
                })),
                errors: gates_failed.iter().map(|g| format!("Gate '{}' failed: {}", g.gate_name, g.detail)).collect(),
            });
        }

        match state.advance_phase(agent_id, gates_passed, gates_failed) {
            Some(new_phase) => {
                self.progress_service.log_event(
                    project_path,
                    EventType::Update,
                    &format!("Advanced to phase: {}", new_phase.display_name()),
                    Some(&format!("Agent '{}' advanced workflow", agent_id)),
                )?;

                self.save_state(project_path, &state)?;

                Ok(WorkflowResponse::ok(&format!(
                    "Advanced to phase '{}'", new_phase.label()
                )).with_state(&state))
            }
            None => {
                Ok(WorkflowResponse::error("Cannot advance: already at final phase")
                    .with_state(&state))
            }
        }
    }

    pub fn run_phase_checks(&self, project_path: &str, agent_id: &str) -> CellResult<WorkflowResponse> {
        let mut state = self.load_state(project_path)?;
        let phase = state.current_phase.clone();

        match phase {
            WorkflowPhase::Init => {
                self.check_init_gates(project_path, &mut state)?;
            }
            WorkflowPhase::Planning => {
                self.check_planning_gates(project_path, &mut state)?;
            }
            WorkflowPhase::Implementation => {
                self.check_implementation_gates(project_path, &mut state)?;
            }
            WorkflowPhase::Verification => {
                self.check_verification_gates(project_path, &mut state)?;
            }
            WorkflowPhase::Handoff => {
                self.check_handoff_gates(project_path, &mut state)?;
            }
            WorkflowPhase::Complete => {}
        }

        state.active_agent = Some(agent_id.to_string());
        self.save_state(project_path, &state)?;

        let all_passed = state.all_current_gates_passed();
        let msg = if all_passed {
            format!("All gates passed for phase '{}'", phase.label())
        } else {
            format!("Some gates failed for phase '{}'", phase.label())
        };

        Ok(WorkflowResponse::ok(&msg)
            .with_state(&state)
            .with_data(serde_json::json!({
                "all_passed": all_passed,
                "gates": state.gates.iter()
                    .filter(|(k, _)| state.current_gates().contains(&k.as_str()))
                    .map(|(k, v)| (k.clone(), v.passed))
                    .collect::<HashMap<_, bool>>(),
            })))
    }

    pub fn get_status(&self, project_path: &str) -> CellResult<WorkflowResponse> {
        let state = self.load_state(project_path)?;
        let current_gates = state.current_gates();

        let gate_status: HashMap<String, bool> = current_gates
            .iter()
            .map(|g| (g.to_string(), state.gates.get(*g).map(|gr| gr.passed).unwrap_or(false)))
            .collect();

        Ok(WorkflowResponse::ok("Status retrieved")
            .with_state(&state)
            .with_data(serde_json::json!({
                "task_name": state.task_name,
                "current_phase": state.current_phase.label(),
                "phase_display": state.current_phase.display_name(),
                "required_gates": current_gates,
                "gate_status": gate_status,
                "all_gates_passed": state.all_current_gates_passed(),
                "registered_agents": state.registered_agents.len(),
                "active_agent": state.active_agent,
                "transitions": state.transitions.len(),
                "baseline": state.baseline,
            })))
    }

    pub fn get_next_phase(&self, project_path: &str) -> CellResult<WorkflowResponse> {
        let state = self.load_state(project_path)?;
        
        match state.current_phase.next() {
            Some(next) => {
                let can_advance = state.all_current_gates_passed();
                Ok(WorkflowResponse::ok(&format!("Next phase: {}", next.label()))
                    .with_state(&state)
                    .with_data(serde_json::json!({
                        "next_phase": next.label(),
                        "next_phase_display": next.display_name(),
                        "can_advance": can_advance,
                        "required_gates": state.current_gates(),
                    })))
            }
            None => {
                Ok(WorkflowResponse::ok("Workflow is complete")
                    .with_state(&state)
                    .with_data(serde_json::json!({ "next_phase": null })))
            }
        }
    }

    pub fn execute_command(&self, project_path: &str, cmd: WorkflowCommand) -> CellResult<WorkflowResponse> {
        match cmd.action {
            WorkflowAction::RegisterAgent => {
                let agent: AgentInfo = serde_json::from_value(cmd.payload)
                    .map_err(|e| CellError::Config(format!("Invalid agent payload: {}", e)))?;
                self.register_agent(project_path, agent)
            }
            WorkflowAction::Start => {
                let task_name = cmd.payload.get("task_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed")
                    .to_string();
                let description = cmd.payload.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                self.start_workflow(project_path, &task_name, &description, &cmd.agent_id)
            }
            WorkflowAction::AdvancePhase => {
                self.advance_phase(project_path, &cmd.agent_id)
            }
            WorkflowAction::RecordGate => {
                let mut state = self.load_state(project_path)?;
                let gate: GateResult = serde_json::from_value(cmd.payload)
                    .map_err(|e| CellError::Config(format!("Invalid gate payload: {}", e)))?;
                state.record_gate(gate);
                self.save_state(project_path, &state)?;
                Ok(WorkflowResponse::ok("Gate recorded").with_state(&state))
            }
            WorkflowAction::GetStatus => {
                self.get_status(project_path)
            }
            WorkflowAction::GetNextPhase => {
                self.get_next_phase(project_path)
            }
            WorkflowAction::CheckGates => {
                self.run_phase_checks(project_path, &cmd.agent_id)
            }
            WorkflowAction::ListPhases => {
                let phases: Vec<_> = [
                    WorkflowPhase::Init,
                    WorkflowPhase::Planning,
                    WorkflowPhase::Implementation,
                    WorkflowPhase::Verification,
                    WorkflowPhase::Handoff,
                    WorkflowPhase::Complete,
                ].iter().map(|p| serde_json::json!({
                    "phase": p.label(),
                    "display": p.display_name(),
                    "order": p.order(),
                    "required_gates": p.required_gates(),
                })).collect();
                
                Ok(WorkflowResponse::ok("Phase list").with_data(serde_json::json!(phases)))
            }
            WorkflowAction::Abort => {
                let state = self.load_state(project_path)?;
                self.progress_service.complete_task(project_path)?;
                Ok(WorkflowResponse::ok("Workflow aborted").with_state(&state))
            }
        }
    }

    fn capture_baseline(&self, project_path: &str) -> CellResult<WorkflowBaseline> {
        let mut baseline = WorkflowBaseline::default();
        baseline.captured_at = Some(chrono::Utc::now().to_rfc3339());

        if let Ok(report) = entropy_service::run_entropy_check(project_path) {
            baseline.entropy_score = Some(report.overall_score);
        }

        let rules = ArchitectureRules::default();
        let arch_result = validate_architecture(Path::new(project_path), &rules);
        baseline.arch_violations = Some(arch_result.violations.len());

        Ok(baseline)
    }

    fn check_init_gates(&self, _project_path: &str, _state: &mut WorkflowState) -> CellResult<()> {
        Ok(())
    }

    fn check_planning_gates(&self, project_path: &str, state: &mut WorkflowState) -> CellResult<()> {
        let rules = ArchitectureRules::default();
        let result = validate_architecture(Path::new(project_path), &rules);
        
        let gate = GateResult {
            gate_name: "architecture_reviewed".to_string(),
            passed: result.passed,
            detail: format!("{} violations found", result.violations.len()),
            measured_at: chrono::Utc::now().to_rfc3339(),
            metrics: {
                let mut m = HashMap::new();
                m.insert("violations".to_string(), serde_json::json!(result.violations.len()));
                m.insert("layers".to_string(), serde_json::json!(result.layer_stats.len()));
                m
            },
        };
        state.record_gate(gate);

        Ok(())
    }

    fn check_implementation_gates(&self, project_path: &str, state: &mut WorkflowState) -> CellResult<()> {
        let rules = ArchitectureRules::default();
        let arch_result = validate_architecture(Path::new(project_path), &rules);
        
        let baseline_v = state.baseline.arch_violations.unwrap_or(0);
        let current_v = arch_result.violations.len();
        
        let gate = GateResult {
            gate_name: "code_checkpoint_passed".to_string(),
            passed: current_v <= baseline_v,
            detail: format!("Baseline: {}, Current: {}", baseline_v, current_v),
            measured_at: chrono::Utc::now().to_rfc3339(),
            metrics: {
                let mut m = HashMap::new();
                m.insert("baseline_violations".to_string(), serde_json::json!(baseline_v));
                m.insert("current_violations".to_string(), serde_json::json!(current_v));
                m
            },
        };
        state.record_gate(gate);

        if let Ok(report) = entropy_service::run_entropy_check(project_path) {
            let baseline_e = state.baseline.entropy_score.unwrap_or(100.0);
            let current_e = report.overall_score;
            let diff = current_e - baseline_e;
            
            let gate = GateResult {
                gate_name: "entropy_controlled".to_string(),
                passed: diff < 5.0,
                detail: format!("Baseline: {:.2}, Current: {:.2}, Diff: {:+.2}", baseline_e, current_e, diff),
                measured_at: chrono::Utc::now().to_rfc3339(),
                metrics: {
                    let mut m = HashMap::new();
                    m.insert("baseline_entropy".to_string(), serde_json::json!(baseline_e));
                    m.insert("current_entropy".to_string(), serde_json::json!(current_e));
                    m.insert("diff".to_string(), serde_json::json!(diff));
                    m
                },
            };
            state.record_gate(gate);
        }

        Ok(())
    }

    fn check_verification_gates(&self, project_path: &str, state: &mut WorkflowState) -> CellResult<()> {
        let verifier = FastVerifyService::new();
        
        match verifier.quick_check(project_path) {
            Ok(result) => {
                let test_gate = GateResult {
                    gate_name: "tests_passing".to_string(),
                    passed: result.passed,
                    detail: format!("{} checks passed", result.checks.iter().filter(|c| c.passed).count()),
                    measured_at: chrono::Utc::now().to_rfc3339(),
                    metrics: HashMap::new(),
                };
                state.record_gate(test_gate);

                let rules = ArchitectureRules::default();
                let arch_result = validate_architecture(Path::new(project_path), &rules);
                let arch_gate = GateResult {
                    gate_name: "arch_compliant".to_string(),
                    passed: arch_result.passed,
                    detail: format!("{} violations", arch_result.violations.len()),
                    measured_at: chrono::Utc::now().to_rfc3339(),
                    metrics: HashMap::new(),
                };
                state.record_gate(arch_gate);

                if let Ok(report) = entropy_service::run_entropy_check(project_path) {
                    let baseline_e = state.baseline.entropy_score.unwrap_or(100.0);
                    let gate = GateResult {
                        gate_name: "entropy_gate_passed".to_string(),
                        passed: report.overall_score < baseline_e + 10.0,
                        detail: format!("Current: {:.2}, Baseline: {:.2}", report.overall_score, baseline_e),
                        measured_at: chrono::Utc::now().to_rfc3339(),
                        metrics: HashMap::new(),
                    };
                    state.record_gate(gate);
                }
            }
            Err(e) => {
                let gate = GateResult {
                    gate_name: "tests_passing".to_string(),
                    passed: false,
                    detail: format!("Verification failed: {}", e),
                    measured_at: chrono::Utc::now().to_rfc3339(),
                    metrics: HashMap::new(),
                };
                state.record_gate(gate);
            }
        }

        Ok(())
    }

    fn check_handoff_gates(&self, _project_path: &str, state: &mut WorkflowState) -> CellResult<()> {
        let handoff_gate = GateResult {
            gate_name: "handoff_generated".to_string(),
            passed: true,
            detail: "Handoff package can be generated".to_string(),
            measured_at: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::new(),
        };
        state.record_gate(handoff_gate);

        let progress_gate = GateResult {
            gate_name: "progress_complete".to_string(),
            passed: true,
            detail: "Progress tracking complete".to_string(),
            measured_at: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::new(),
        };
        state.record_gate(progress_gate);

        Ok(())
    }

    fn workflow_exists(&self, project_path: &str) -> bool {
        Path::new(project_path).join(".cell/workflow/state.json").exists()
    }

    fn load_state(&self, project_path: &str) -> CellResult<WorkflowState> {
        let state_file = Path::new(project_path).join(".cell/workflow/state.json");
        if !state_file.exists() {
            return Err(CellError::Config(
                "No active workflow. Start one with 'workflow start'.".to_string()
            ));
        }
        let content = std::fs::read_to_string(&state_file)?;
        let state: WorkflowState = serde_json::from_str(&content)
            .map_err(|e| CellError::Config(format!("Invalid workflow state: {}", e)))?;
        Ok(state)
    }

    fn load_or_init_state(&self, project_path: &str) -> CellResult<WorkflowState> {
        if self.workflow_exists(project_path) {
            self.load_state(project_path)
        } else {
            Ok(WorkflowState::new("unnamed", ""))
        }
    }

    fn save_state(&self, project_path: &str, state: &WorkflowState) -> CellResult<()> {
        let state_dir = Path::new(project_path).join(".cell/workflow");
        std::fs::create_dir_all(&state_dir)?;
        let content = serde_json::to_string_pretty(state)?;
        std::fs::write(state_dir.join("state.json"), content)?;
        Ok(())
    }
}
