use cell_application::arch_advisor::ArchitectureAdvisor;
use cell_application::arch_fixer::ArchitectureFixer;
use cell_application::arch_linter::ArchitectureLinter;
use cell_application::arch_service::{ArchitectureRules, validate_architecture};
use cell_application::arch_visualizer::{ArchitectureVisualizer, OutputFormat, VisualizationOptions};
use cell_application::dependency_analyzer::DependencyAnalyzer;
use cell_application::impact_analysis_service::ImpactAnalysisService;
use cell_domain::errors::CellResult;
use crate::cli::{ArchArgs, ArchSub};

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
            let vis = ArchitectureVisualizer::new();
            let options = VisualizationOptions {
                output_format: OutputFormat::Mermaid,
                include_violations: true,
                include_stats: true,
            };
            let diagram = vis.visualize(".", options)?;
            println!("{diagram}");
            
            if let Some(out_path) = output {
                std::fs::write(&out_path, &diagram)?;
                println!("已输出到: {out_path}");
            }
        }
        ArchSub::Lint { fix, deep: _deep, json } => {
            if fix {
                let fixer = ArchitectureFixer::new();
                let result = fixer.fix(".", false)?;
                println!("{}", fixer.format_result(&result));
            } else {
                let linter = ArchitectureLinter::new();
                let result = linter.lint(std::path::Path::new("."));
                
                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("{}", linter.format_result(&result));
                }
                
                if result.error_count > 0 {
                    std::process::exit(1);
                }
            }
        }
        ArchSub::Rules {} => {
            let linter = ArchitectureLinter::new();
            let rules = linter.list_rules();
            
            println!("\n📋 架构 Lint 规则列表\n{}", "─".repeat(50));
            println!("  共 {} 条规则\n", rules.len());
            
            let mut current_cat = String::new();
            for rule in rules {
                let cat = format!("{:?}", rule.category);
                if cat != current_cat {
                    println!("\n  📂 {cat}:");
                    current_cat = cat;
                }
                
                let icon = if rule.enabled { "✅" } else { "⚪" };
                let sev = match rule.severity {
                    cell_application::arch_linter::LintSeverity::Error => "🔴 Error",
                    cell_application::arch_linter::LintSeverity::Warning => "🟡 Warning",
                    cell_application::arch_linter::LintSeverity::Info => "🟢 Info",
                };
                
                println!("    {} [{}] {} - {} ({})", 
                    icon, rule.id, rule.name, rule.description, sev);
            }
            println!("\n{}", "─".repeat(50));
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
            let advisor = ArchitectureAdvisor::new();
            let result = advisor.advise(".")?;
            println!("{}", advisor.format_result(&result));
        }
    }
    Ok(())
}
