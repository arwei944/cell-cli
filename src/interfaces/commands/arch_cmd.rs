use crate::application::arch_service::{ArchitectureRules, validate_architecture};
use crate::application::dependency_analyzer::DependencyAnalyzer;
use crate::application::impact_analysis_service::ImpactAnalysisService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_arch(args: ArchArgs) -> CellResult<()> {
    match args.sub {
        ArchSub::Validate { path } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let rules = ArchitectureRules::default();
            let result = validate_architecture(std::path::Path::new(&p), &rules);
            println!("{}", serde_json::to_string_pretty(&result)?);
            if !result.passed { std::process::exit(1); }
        }
        ArchSub::Visualize { output } => {
            println!("arch visualize, output: {:?}", output);
        }
        ArchSub::Lint { fix } => {
            println!("arch lint, fix: {}", fix);
        }
        ArchSub::Overview {} => {
            let analyzer = DependencyAnalyzer::new(".");
            let graph = analyzer.analyze()?;
            println!("{}", analyzer.format_summary(&graph));
        }
        ArchSub::Graph {} => {
            let analyzer = DependencyAnalyzer::new(".");
            let graph = analyzer.analyze()?;
            println!("{}", analyzer.generate_mermaid(&graph));
            println!("\n提示: 可将上述代码复制到 mermaid.live 查看可视化图表");
        }
        ArchSub::Impact { base, path } => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let service = ImpactAnalysisService::new(&path);
            let analysis = service.analyze(base.as_deref())?;
            println!("{}", service.format_report(&analysis));
        }
        ArchSub::Advise {} => {
            println!("arch advise");
        }
    }
    Ok(())
}
