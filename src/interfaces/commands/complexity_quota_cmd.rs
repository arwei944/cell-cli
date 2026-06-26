use crate::application::complexity_quota_service::ComplexityQuotaService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_complexity_quota(args: ComplexityQuotaArgs) -> CellResult<()> {
    let service = ComplexityQuotaService::new();
    match args.sub {
        ComplexityQuotaSub::Status { name } => {
            println!("\n📊 查询复杂度配额: {}\n", name);
            match service.get_quota(&name) {
                Ok(quota) => println!("{}", service.format_quota(&quota)),
                Err(_) => println!("  配额不存在"),
            }
        }
        ComplexityQuotaSub::Check { name, required } => {
            let allowed = service.check_quota(&name, required)?;
            println!("\n🔍 检查复杂度配额: {}\n", name);
            println!("  需求: {:.1}", required);
            println!("  结果: {}", if allowed { "✅ 允许" } else { "❌ 超出配额" });
        }
    }
    Ok(())
}
