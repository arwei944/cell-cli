use crate::adapters::file_decision_store::FileDecisionStore;
use crate::application::decision_service::DecisionService;
use crate::domain::decision::{DecisionCategory, DecisionStatus};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_adr(args: AdrArgs) -> CellResult<()> {
    let store = FileDecisionStore::new();
    let service = DecisionService::new(store);

    match args.sub {
        AdrSub::New { title, status } => {
            let status = status.map(|s| parse_status(&s)).transpose()?.unwrap_or(DecisionStatus::Proposed);
            let status_label = status.label().to_string();
            let category = DecisionCategory::Architecture;
            
            let record = service.record_decision(
                ".",
                &title,
                "",
                "",
                "",
                category,
                None,
            )?;
            
            service.update_status(".", &record.id.simple().to_string(), status)?;
            
            println!("\n✅ 创建 ADR: {}\n", record.title);
            println!("ID: {}", record.id.simple());
            println!("状态: {}", status_label);
            println!("\n提示: 请编辑 .cell/decisions/{}.json 添加详细内容", record.id.simple());
        }
        AdrSub::List {} => {
            let decisions = service.list_decisions(".", None, None)?;
            
            println!("\n📋 ADR 列表\n");
            if decisions.is_empty() {
                println!("暂无 ADR 记录");
                println!("\n使用 `cell adr new <title>` 创建新的 ADR");
                return Ok(());
            }

            println!("{:<15} {:<40} {:<15} {:<20}", "ID", "标题", "状态", "创建时间");
            println!("────────────────────────────────────────────────────────────────────");
            
            for d in decisions {
                println!("{:<15} {:<40} {:<15} {:<20}", 
                    d.id.simple(), 
                    d.title, 
                    d.status.label(),
                    d.made_at
                );
            }
        }
        AdrSub::Show { id } => {
            if let Some(decision) = service.get_decision(".", &id)? {
                println!("{}", decision.to_markdown());
            } else {
                println!("❌ 未找到 ADR: {}", id);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn parse_status(s: &str) -> CellResult<DecisionStatus> {
    match s.to_lowercase().as_str() {
        "proposed" | "propose" => Ok(DecisionStatus::Proposed),
        "accepted" | "accept" => Ok(DecisionStatus::Accepted),
        "deprecated" | "deprecate" => Ok(DecisionStatus::Deprecated),
        "rejected" | "reject" => Ok(DecisionStatus::Rejected),
        "superseded" | "supersede" => Ok(DecisionStatus::Superseded),
        _ => Err(crate::domain::errors::CellError::Config(format!(
            "Unknown status: {}. Valid: proposed, accepted, deprecated, rejected, superseded", s
        ))),
    }
}
