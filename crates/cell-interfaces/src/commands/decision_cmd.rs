use cell_adapters::file_decision_store::FileDecisionStore;
use cell_application::decision_service::DecisionService;
use cell_domain::decision::{DecisionCategory, DecisionStatus};
use cell_domain::errors::{CellError, CellResult};
use crate::cli::{AdrArgs, DecisionArgs, DecisionSub};

pub fn cmd_adr(args: AdrArgs) -> CellResult<()> {
    println!("cell adr: {:?}", args.sub);
    Ok(())
}

pub fn cmd_decision(args: DecisionArgs, format: Option<String>) -> CellResult<()> {
    let store = FileDecisionStore::new();
    let service = DecisionService::new(store);

    match args.sub {
        DecisionSub::New { title, context, decision, rationale, category, by } => {
            let cat = parse_decision_category(category.as_deref().unwrap_or("arch"))?;
            let record = service.record_decision(
                ".", &title,
                &context.unwrap_or_default(), &decision.unwrap_or_default(),
                &rationale.unwrap_or_default(), cat, by.as_deref()
            )?;
            println!("✅ 决策已记录: {}", record.title);
            println!("   ID: {}", record.id.simple());
            println!("   分类: {}", record.category.label());
        }
        DecisionSub::List { category, status } => {
            let cat = category.as_deref().map(parse_decision_category).transpose()?;
            let stat = status.as_deref().map(parse_decision_status).transpose()?;
            let decisions = service.list_decisions(".", cat, stat)?;
            print_decision_list(&decisions, format.as_deref());
        }
        DecisionSub::Show { id } => {
            match service.get_decision(".", &id)? {
                Some(d) => {
                    if format.as_deref() == Some("json") {
                        println!("{}", serde_json::to_string_pretty(&d)?);
                    } else {
                        println!("{}", d.to_markdown());
                    }
                }
                None => println!("❌ 未找到决策记录: {id}"),
            }
        }
        DecisionSub::Status { id, status } => {
            let stat = parse_decision_status(&status)?;
            let d = service.update_status(".", &id, stat)?;
            println!("✅ 决策状态已更新: {} -> {}", d.id.simple(), d.status.label());
        }
        DecisionSub::Alternative { id, name, description } => {
            let d = service.add_alternative(".", &id, &name, &description.unwrap_or_default(), Vec::new(), Vec::new())?;
            println!("✅ 备选方案已添加: {name}");
            let _ = d;
        }
        DecisionSub::Tag { id, tag } => {
            let d = service.add_tag(".", &id, &tag)?;
            println!("✅ 标签已添加: {tag}");
            let _ = d;
        }
        DecisionSub::Export { output } => {
            let out = output.unwrap_or_else(|| "decisions.md".to_string());
            let path = service.export_markdown(".", &out)?;
            println!("✅ 决策记录已导出: {path}");
        }
        DecisionSub::Metrics {} => {
            let metrics = service.get_metrics(".")?;
            if format.as_deref() == Some("json") {
                println!("{}", serde_json::to_string_pretty(&metrics)?);
            } else {
                print_decision_metrics(&metrics);
            }
        }
    }
    Ok(())
}

fn print_decision_list(decisions: &[cell_domain::decision::DecisionRecord], format: Option<&str>) {
    if format == Some("json") {
        println!("{}", serde_json::to_string_pretty(decisions).unwrap_or_default());
        return;
    }
    println!("📋 决策记录 (共 {} 条):\n", decisions.len());
    for d in decisions {
        let status_icon = match d.status {
            DecisionStatus::Proposed => "📝",
            DecisionStatus::Accepted => "✅",
            DecisionStatus::Rejected => "❌",
            DecisionStatus::Deprecated => "⚠️",
            DecisionStatus::Superseded => "🔄",
        };
        println!("  {} [{}] {} - {}",
            status_icon, d.id.simple(), d.title, d.category.label()
        );
        println!("     {}", d.made_at.format("%Y-%m-%d %H:%M"));
    }
}

fn print_decision_metrics(metrics: &cell_domain::decision::DecisionMetrics) {
    println!("📊 决策统计:");
    println!("   总决策数: {}", metrics.total_decisions);
    println!("   已采纳: {}", metrics.accepted_count);
    println!("   已拒绝: {}", metrics.rejected_count);
    println!("   已替代: {}", metrics.superseded_count);
    println!("   近7天: {}", metrics.last_7_days);
    println!("   近30天: {}", metrics.last_30_days);
    if !metrics.by_category.is_empty() {
        println!("\n   按分类:");
        for (cat, count) in &metrics.by_category {
            println!("     {}: {}", cat.label(), count);
        }
    }
}

fn parse_decision_category(s: &str) -> CellResult<DecisionCategory> {
    DecisionCategory::from_str(s).ok_or_else(|| CellError::Config(format!(
        "Unknown decision category: {s}. Valid: arch, tech, process, tool, design, test, deploy, security, perf, other"
    )))
}

fn parse_decision_status(s: &str) -> CellResult<DecisionStatus> {
    match s.to_lowercase().as_str() {
        "proposed" | "propose" => Ok(DecisionStatus::Proposed),
        "accepted" | "accept" => Ok(DecisionStatus::Accepted),
        "rejected" | "reject" => Ok(DecisionStatus::Rejected),
        "deprecated" | "deprecate" => Ok(DecisionStatus::Deprecated),
        "superseded" | "supersede" => Ok(DecisionStatus::Superseded),
        _ => Err(CellError::Config(format!(
            "Unknown decision status: {s}. Valid: proposed, accepted, rejected, deprecated, superseded"
        ))),
    }
}
