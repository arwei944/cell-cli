use crate::application::tool_policy_service::{ToolPolicyService, AgentRole};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_tool_policy(args: ToolPolicyArgs) -> CellResult<()> {
    let service = ToolPolicyService::new();
    let project_path = ".";

    let role_filter = match args.role.as_deref() {
        Some("architect") => Some(AgentRole::Architect),
        Some("developer") => Some(AgentRole::Developer),
        Some("tester") => Some(AgentRole::Tester),
        Some("reviewer") => Some(AgentRole::Reviewer),
        Some("observer") => Some(AgentRole::Observer),
        _ => None,
    };

    match args.sub {
        ToolPolicySub::List {} => {
            let report = service.list_tools(project_path)?;
            println!("{}", service.format_report(&report));

            if let Some(role) = role_filter {
                println!("\n🎭 当前角色: {:?}", role);
                let available = service.get_tools(Some(&role));
                println!("  可用工具数: {}", available.len());
            }
        }
        ToolPolicySub::Show { id } => {
            match service.get_tool(&id) {
                Some(tool) => {
                    println!("{}", service.format_tool_detail(tool));
                }
                None => {
                    println!("\n❌ 未找到工具: {}", id);
                }
            }
        }
        ToolPolicySub::Check { tool_id, role } => {
            let role_enum = match role.as_str() {
                "architect" => AgentRole::Architect,
                "developer" => AgentRole::Developer,
                "tester" => AgentRole::Tester,
                "reviewer" => AgentRole::Reviewer,
                "observer" => AgentRole::Observer,
                _ => {
                    println!("\n❌ 未知角色: {}", role);
                    return Ok(());
                }
            };

            match service.check_tool_access(&tool_id, &role_enum) {
                Ok(()) => {
                    println!("\n✅ 角色 {:?} 有权使用工具 {}", role_enum, tool_id);
                }
                Err(msg) => {
                    println!("\n❌ {}", msg);
                }
            }
        }
        ToolPolicySub::Record { tool_id, agent, duration, success } => {
            service.record_usage(
                project_path,
                &tool_id,
                &agent,
                duration.unwrap_or(0),
                success,
                None,
            )?;
            println!(
                "\n✅ 已记录工具使用: {} by {} ({})",
                tool_id,
                agent,
                if success { "成功" } else { "失败" }
            );
        }
    }

    Ok(())
}
