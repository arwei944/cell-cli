use crate::application::decision_engine::DecisionEngineService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_decide(args: DecideArgs) -> CellResult<()> {
    let service = DecisionEngineService::new();
    let project_path = ".";
    let agent_id = args.agent.unwrap_or_else(|| "unknown".to_string());

    match args.sub {
        DecideSub::Make { title, context } => {
            let decision = service.make_decision(
                project_path,
                &title,
                context.as_deref().unwrap_or(""),
                &agent_id,
            )?;
            println!("{}", service.format_decision(&decision));

            if decision.needs_human_review {
                println!("\n⚠️  此决策需要人工审查确认");
            }
        }
        DecideSub::List { pending } => {
            let report = service.list_decisions(project_path, pending)?;
            println!("{}", service.format_report(&report));
        }
        DecideSub::Show { id } => {
            let report = service.list_decisions(project_path, false)?;
            match report.decisions.iter().find(|d| d.id == id) {
                Some(decision) => {
                    println!("{}", service.format_decision(decision));
                }
                None => {
                    println!("\n❌ 未找到决策: {}", id);
                }
            }
        }
        DecideSub::Rules {} => {
            println!("\n📋 决策规则\n");
            for (i, rule) in service.get_rules().iter().enumerate() {
                let status = if rule.enabled { "✅" } else { "🚫" };
                let sev = match rule.severity {
                    crate::application::decision_engine::DecisionSeverity::Low => "🟢",
                    crate::application::decision_engine::DecisionSeverity::Medium => "🟡",
                    crate::application::decision_engine::DecisionSeverity::High => "🟠",
                    crate::application::decision_engine::DecisionSeverity::Critical => "🔴",
                };
                println!("  {}. {} {} [{}] {}", i + 1, status, sev, rule.id, rule.name);
                println!("     条件: {}", rule.condition);
                println!("     动作: {}\n", rule.action);
            }
        }
    }

    Ok(())
}
