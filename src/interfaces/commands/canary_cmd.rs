use crate::application::canary_service::CanaryService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_canary(args: CanaryArgs) -> CellResult<()> {
    let mut service = CanaryService::new();

    match args.sub {
        CanarySub::Create { name, old_version, new_version } => {
            let status = service.create_canary(name, old_version.unwrap_or_else(|| "v1.0.0".to_string()), new_version.unwrap_or_else(|| "v2.0.0".to_string()))?;
            println!("{}", service.format_status(&status));
            println!("\n✅ Canary release created successfully.");
            println!("   提示: 使用 `cell canary start {}` 开始发布", status.name);
        }
        CanarySub::List {} => {
            let canaries = service.list_canaries();
            println!("{}", service.format_list(&canaries));
        }
        CanarySub::Promote { name } => {
            let status = service.promote_canary(name)?;
            println!("{}", service.format_status(&status));
            println!("\n✅ Canary release promoted successfully.");
        }
        CanarySub::Rollback { name, reason } => {
            let status = service.rollback_canary(name, reason.unwrap_or_else(|| "Manual rollback".to_string()))?;
            println!("{}", service.format_status(&status));
            println!("\n🔄 Canary release rolled back successfully.");
        }
        CanarySub::Status { name } => {
            let status = service.get_canary_status(name)?;
            println!("{}", service.format_status(&status));
        }
        CanarySub::Start { name } => {
            let status = service.start_canary(name)?;
            println!("{}", service.format_status(&status));
            println!("\n🚀 Canary release started successfully.");
            println!("   当前流量: {}% -> {}%", status.old_version, status.new_version);
        }
    }

    Ok(())
}