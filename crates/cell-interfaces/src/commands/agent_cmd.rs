use cell_application::agent_protocol_service::{AgentProtocolService, AgentRole};
use cell_domain::errors::CellResult;
use crate::cli::{AgentArgs, AgentSub, AgentTaskSub};

pub fn cmd_agent(args: AgentArgs) -> CellResult<()> {
    let service = AgentProtocolService::new();
    let project_path = ".";

    match args.sub {
        AgentSub::Register { name, role } => {
            let role_enum = match role.to_lowercase().as_str() {
                "architect" => AgentRole::Architect,
                "developer" => AgentRole::Developer,
                "tester" => AgentRole::Tester,
                "reviewer" => AgentRole::Reviewer,
                "coordinator" => AgentRole::Coordinator,
                _ => AgentRole::Developer,
            };
            
            let capabilities = vec!["entropy_check".to_string(), "arch_validate".to_string(), "code_generation".to_string()];
            let agent = service.register_agent(project_path, &name, role_enum, capabilities)?;
            
            println!("\n✅ Agent 已注册\n");
            println!("  ID: {}", agent.id);
            println!("  名称: {}", agent.name);
            println!("  角色: {}", agent.role.label());
            println!("  状态: {}", agent.status.label());
            println!();
        }
        AgentSub::List {} => {
            let agents = service.list_agents(project_path)?;
            
            println!("\n🤖 Agent 列表\n");
            if agents.is_empty() {
                println!("  暂无 Agent");
            } else {
                println!("  {:<36} {:<15} {:<8} {:<20}", "ID", "名称", "角色", "状态");
                println!("  {}", "-".repeat(80));
                for a in &agents {
                    println!("  {:<36} {:<15} {:<8} {:<20}", a.id, a.name, a.role.label(), a.status.label());
                }
            }
            println!();
        }
        AgentSub::Status { id } => {
            let agent = service.get_agent_status(project_path, &id)?;
            
            println!("\n🤖 Agent 状态\n");
            println!("  ID: {}", agent.id);
            println!("  名称: {}", agent.name);
            println!("  角色: {}", agent.role.label());
            println!("  状态: {}", agent.status.label());
            println!("  心跳: {}", agent.last_heartbeat);
            if let Some(task) = &agent.current_task {
                println!("  当前任务: {task}");
            }
            println!("  能力: {}", agent.capabilities.join(", "));
            println!();
        }
        AgentSub::Task { sub } => {
            match sub {
                AgentTaskSub::Create { name, description } => {
                    let task = service.create_task(project_path, &name, &description, vec![])?;
                    println!("\n✅ 任务已创建\n");
                    println!("  ID: {}", task.id);
                    println!("  名称: {}", task.name);
                    println!("  状态: {}", task.status.label());
                    println!();
                }
                AgentTaskSub::Assign { task_id, agent_id } => {
                    service.assign_task(project_path, &task_id, &agent_id)?;
                    println!("\n✅ 任务已分配\n");
                    println!("  任务: {task_id}");
                    println!("  Agent: {agent_id}");
                    println!();
                }
                AgentTaskSub::Complete { task_id, success } => {
                    service.complete_task(project_path, &task_id, success, "完成", vec![])?;
                    println!("\n✅ 任务已完成\n");
                    println!("  任务: {task_id}");
                    println!("  结果: {}", if success { "成功" } else { "失败" });
                    println!();
                }
                AgentTaskSub::List {} => {
                    let tasks = service.list_tasks(project_path)?;
                    
                    println!("\n📋 任务列表\n");
                    if tasks.is_empty() {
                        println!("  暂无任务");
                    } else {
                        println!("  {:<36} {:<20} {:<10} {:<36}", "ID", "名称", "状态", "分配给");
                        println!("  {}", "-".repeat(100));
                        for t in &tasks {
                            let assigned = t.assigned_to.as_deref().unwrap_or("-");
                            println!("  {:<36} {:<20} {:<10} {:<36}", t.id, t.name, t.status.label(), assigned);
                        }
                    }
                    println!();
                }
            }
        }
        AgentSub::Delegate { task_id, agent_id } => {
            let task = service.delegate_task(project_path, &task_id, &agent_id)?;
            println!("\n✅ 任务已委托\n");
            println!("  任务: {}", task.name);
            println!("  Agent: {agent_id}");
            println!();
        }
        AgentSub::Handoff { from, to, task_id, notes } => {
            let handoff = service.create_handoff(project_path, &from, to.as_deref(), &task_id, &notes)?;
            println!("\n✅ 交接包已创建\n");
            println!("  来自: {}", handoff.from_agent);
            println!("  目标: {}", handoff.to_agent.as_deref().unwrap_or("未指定"));
            println!("  任务: {}", handoff.task_id);
            println!("  备注: {}", handoff.context.notes);
            println!();
        }
        AgentSub::Heartbeat { id } => {
            service.heartbeat(project_path, &id)?;
            println!("\n✅ 心跳已更新: {id}\n");
        }
    }

    Ok(())
}