use cell_application::agent_profile_service::AgentProfileService;
use cell_domain::errors::CellResult;
use crate::cli::{AgentProfileArgs, AgentProfileSub};

pub fn cmd_agent_profile(args: AgentProfileArgs) -> CellResult<()> {
    let service = AgentProfileService::new();
    let project_path = ".";

    match args.sub {
        AgentProfileSub::Show { id } => {
            let agent_id = id
                .or(args.agent)
                .unwrap_or_else(|| "default".to_string());
            let profile = service.get_profile(project_path, &agent_id)?;
            println!("{}", service.format_profile(&profile));
        }
        AgentProfileSub::List {} => {
            let profiles = service.list_profiles(project_path)?;
            println!("\n🤖 Agent 列表 (共 {} 个)\n", profiles.len());
            for (i, p) in profiles.iter().enumerate() {
                println!(
                    "  {}. {} [{}] - {:.1} 分",
                    i + 1,
                    p.agent_name,
                    p.role,
                    p.overall_score
                );
            }
            println!();
        }
        AgentProfileSub::Rank {} => {
            let rankings = service.get_ranking(project_path)?;
            println!("{}", service.format_ranking(&rankings));
        }
        AgentProfileSub::Record {
            agent,
            success,
            on_time,
            duration,
        } => {
            service.update_task_completion(
                project_path,
                &agent,
                success,
                on_time,
                duration.unwrap_or(0),
            )?;
            println!(
                "\n✅ 已记录任务完成: agent={agent}, success={success}, on_time={on_time}"
            );
        }
    }

    Ok(())
}
