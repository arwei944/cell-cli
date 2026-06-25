use crate::application::entropy_service::run_entropy_check;
use crate::application::incremental_entropy_service::IncrementalEntropyService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_entropy(args: EntropyArgs, format: Option<String>) -> CellResult<()> {
    match args.sub {
        EntropySub::Check { path } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let result = run_entropy_check(&p)?;
            output_entropy_report(&result, format.as_deref());
        }
        EntropySub::Delta { path, full: _ } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let service = IncrementalEntropyService::new();
            let result = service.run(&p)?;
            println!("{}", service.format_result(&result));
        }
        EntropySub::Gate { threshold, path } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let result = run_entropy_check(&p)?;
            let thresh = threshold.unwrap_or(50.0);
            output_entropy_report(&result, format.as_deref());
            println!();
            if result.overall_score > thresh {
                println!("❌ 熵值门禁失败！");
                println!("   当前熵值: {:.2} > 阈值: {:.2}", result.overall_score, thresh);
                println!("   请关注高风险文件并进行优化。");
                std::process::exit(1);
            } else {
                println!("✅ 熵值门禁通过！");
                println!("   当前熵值: {:.2} ≤ 阈值: {:.2}", result.overall_score, thresh);
            }
        }
        EntropySub::Trend {} => {
            println!("📈 熵值趋势分析功能开发中...");
            println!("   该功能将在后续版本中提供历史熵值趋势追踪。");
        }
    }
    Ok(())
}

pub fn cmd_feature(args: FeatureArgs) -> CellResult<()> {
    println!("cell feature: {:?}", args.sub);
    Ok(())
}

fn output_entropy_report(report: &crate::domain::entropy::EntropyReport, format: Option<&str>) {
    match format {
        Some("json") => {
            println!("{}", serde_json::to_string_pretty(report).unwrap_or_default());
        }
        Some("yaml") => {
            println!("{}", serde_yaml::to_string(report).unwrap_or_default());
        }
        _ => {
            println!("{}", report.to_pretty_text());
        }
    }
}
