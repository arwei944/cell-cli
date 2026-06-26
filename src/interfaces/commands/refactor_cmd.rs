use crate::application::refactor_service::RefactorService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_refactor(args: RefactorArgs) -> CellResult<()> {
    let mut service = RefactorService::new();

    match args.sub {
        RefactorSub::Analyze { path } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let result = service.analyze_code_smells(&p)?;

            println!("📊 代码异味分析结果");
            println!("{}", "─".repeat(60));
            println!("  扫描路径: {}", p);
            println!("  发现异味: {}", result.summary.total_smells);
            println!("  生成建议: {}", result.summary.total_proposals);
            println!("  ├── Critical: {}", result.summary.critical_count);
            println!("  ├── Major: {}", result.summary.major_count);
            println!("  ├── Minor: {}", result.summary.minor_count);
            println!("  └── Info: {}", result.summary.info_count);
            println!("{}", "─".repeat(60));

            for proposal in &result.proposals {
                service.add_proposal(proposal.clone());
            }

            println!("\n已将建议保存，使用 `cell refactor list` 查看");
        }
        RefactorSub::List { severity } => {
            let all_proposals = service.list_refactor_proposals();
            let filtered: Vec<_> = if let Some(sev) = severity {
                let sev_lower = sev.to_lowercase();
                all_proposals.into_iter()
                    .filter(|p| format!("{:?}", p.severity).to_lowercase() == sev_lower)
                    .collect()
            } else {
                all_proposals
            };
            println!("{}", service.format_proposals(&filtered));
        }
        RefactorSub::Apply { id } => {
            service.generate_execution_plan(&id)?;
            service.apply_refactor(&id)?;
            let detail = service.get_proposal_detail(&id)?;
            println!("✅ 重构已应用");
            println!("{}", service.format_proposal_detail(&detail));
        }
    }

    Ok(())
}
