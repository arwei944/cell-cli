use crate::application::code_review_service::CodeReviewService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_review(args: ReviewArgs) -> CellResult<()> {
    let service = CodeReviewService::new();
    let project_path = ".";
    let deep = args.deep;

    println!("\n🔍 代码审查中{}...\n", if deep { " (深度模式)" } else { "" });
    
    let result = service.review_code(project_path, deep)?;
    let report = service.generate_review_report(&result);
    println!("{}", report);

    if !result.passed {
        std::process::exit(1);
    }

    Ok(())
}
