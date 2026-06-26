use crate::adapters::file_decision_store::FileDecisionStore;
use crate::adapters::file_evolution_store::FileEvolutionStore;
use crate::adapters::file_handoff_exporter::FileHandoffExporter;
use crate::adapters::file_progress_store::FileProgressStore;
use crate::application::workflow_engine::WorkflowEngine;
use crate::domain::errors::CellResult;
use crate::domain::workflow::*;
use crate::interfaces::cli::*;
use std::collections::HashMap;

const DEFAULT_AGENT_ID: &str = "cell-cli";
const DEFAULT_AGENT_NAME: &str = "Cell CLI";

pub fn cmd_workflow(args: WorkflowArgs) -> CellResult<()> {
    let progress_store = FileProgressStore::new();
    let engine = WorkflowEngine::<
        FileProgressStore,
        FileEvolutionStore,
        FileHandoffExporter,
        FileDecisionStore,
    >::new(progress_store);

    let project_path = ".";
    let agent_id = args.agent_id.as_deref().unwrap_or(DEFAULT_AGENT_ID);

    let response = match args.sub {
        WorkflowSub::Start { name, description } => {
            let desc = description.unwrap_or_default();
            engine.start_workflow(project_path, &name, &desc, agent_id)?
        }
        WorkflowSub::Status {} => {
            engine.get_status(project_path)?
        }
        WorkflowSub::Check {} => {
            engine.run_phase_checks(project_path, agent_id)?
        }
        WorkflowSub::Advance {} => {
            engine.advance_phase(project_path, agent_id)?
        }
        WorkflowSub::Next {} => {
            engine.get_next_phase(project_path)?
        }
        WorkflowSub::Phases {} => {
            engine.execute_command(project_path, WorkflowCommand {
                action: WorkflowAction::ListPhases,
                agent_id: agent_id.to_string(),
                payload: serde_json::Value::Null,
            })?
        }
        WorkflowSub::Register { id, name, version } => {
            let agent = AgentInfo {
                id: id.clone(),
                name: name.unwrap_or(id.clone()),
                version: version.unwrap_or_else(|| "1.0.0".to_string()),
                capabilities: vec!["code_generation".to_string(), "testing".to_string()],
                registered_at: chrono::Utc::now().to_rfc3339(),
            };
            engine.register_agent(project_path, agent)?
        }
        WorkflowSub::Gate { name, passed, detail } => {
            let gate = GateResult {
                gate_name: name,
                passed,
                detail: detail.unwrap_or_default(),
                measured_at: chrono::Utc::now().to_rfc3339(),
                metrics: HashMap::new(),
            };
            engine.execute_command(project_path, WorkflowCommand {
                action: WorkflowAction::RecordGate,
                agent_id: agent_id.to_string(),
                payload: serde_json::to_value(gate)?,
            })?
        }
        WorkflowSub::Exec { command: cmd } => {
            let workflow_cmd: WorkflowCommand = serde_json::from_str(&cmd)
                .map_err(|e| crate::domain::errors::CellError::Config(format!("Invalid command JSON: {}", e)))?;
            engine.execute_command(project_path, workflow_cmd)?
        }
        WorkflowSub::Abort {} => {
            engine.execute_command(project_path, WorkflowCommand {
                action: WorkflowAction::Abort,
                agent_id: agent_id.to_string(),
                payload: serde_json::Value::Null,
            })?
        }
    };

    let format_str = args.format.unwrap_or_else(|| "text".to_string());
    match format_str.to_lowercase().as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        "yaml" | "yml" => {
            println!("{}", serde_yaml::to_string(&response)?);
        }
        _ => {
            print_text_response(&response);
        }
    }

    Ok(())
}

fn print_text_response(response: &WorkflowResponse) {
    let icon = if response.success { "✅" } else { "❌" };
    println!("\n{} {}\n", icon, response.message);

    if let Some(phase) = &response.current_phase {
        println!("  当前阶段: {}", phase.display_name());
    }

    if let Some(data) = &response.data {
        if let Some(gate_status) = data.get("gate_status").and_then(|v| v.as_object()) {
            println!("\n  门禁状态:");
            for (name, passed) in gate_status {
                let icon = if passed.as_bool().unwrap_or(false) { "✅" } else { "❌" };
                println!("    {} {}", icon, name);
            }
        }

        if let Some(all_passed) = data.get("all_gates_passed").and_then(|v| v.as_bool()) {
            if all_passed {
                println!("\n  🎉 所有门禁已通过，可以推进到下一阶段！");
            }
        }

        if let Some(next) = data.get("next_phase").and_then(|v| v.as_str()) {
            println!("\n  下一阶段: {}", next);
            if let Some(can_advance) = data.get("can_advance").and_then(|v| v.as_bool()) {
                if can_advance {
                    println!("  可推进: ✅ 是");
                } else {
                    println!("  可推进: ❌ 否（需先通过门禁）");
                }
            }
        }

        if let Some(phases) = data.get("phases").and_then(|v| v.as_array()) {
            println!("\n  工作流阶段:");
            for phase in phases {
                let label = phase.get("phase").and_then(|v| v.as_str()).unwrap_or("");
                let display = phase.get("display").and_then(|v| v.as_str()).unwrap_or("");
                let gates_vec: Vec<&str> = phase
                    .get("required_gates")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|g| g.as_str()).collect())
                    .unwrap_or_default();
                println!("    {} ({})", display, label);
                for gate in &gates_vec {
                    println!("      └─ {}", gate);
                }
            }
        }
    }

    if !response.errors.is_empty() {
        println!("\n  错误:");
        for err in &response.errors {
            println!("    • {}", err);
        }
    }

    println!();
}
