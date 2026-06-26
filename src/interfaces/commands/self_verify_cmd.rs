use crate::application::self_verify_service::{SelfVerifyService, VerifyConfig};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_self_verify(args: SelfVerifyArgs) -> CellResult<()> {
    let config = VerifyConfig {
        max_attempts: args.attempts,
        run_arch_check: !args.no_arch,
        run_tests: !args.no_tests,
        run_entropy_check: !args.no_entropy,
        auto_fix: !args.no_fix,
    };

    let service = SelfVerifyService::with_config(config);
    let project_path = ".";

    println!("🔍 开始自我验证...");
    let result = service.run_self_verify(project_path)?;
    println!("{}", service.format_result(&result));

    if !result.passed && args.rollback {
        println!("\n⏪ 验证失败，尝试回滚到稳定版本...");
        let rolled = service.rollback_to_stable(project_path)?;
        if rolled {
            println!("✅ 已回滚到上一个稳定版本");
        } else {
            println!("⚠️  回滚失败或没有可回滚的版本");
        }
    }

    if result.passed {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
