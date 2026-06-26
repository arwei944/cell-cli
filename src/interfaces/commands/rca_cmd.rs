use crate::application::rca_v2_service::RCAV2Service;
use crate::domain::errors::CellResult;
use crate::domain::rca_engine_v2::{RCASignal, SignalSeverity, SignalType};
use crate::interfaces::cli::*;
use chrono::Utc;
use uuid::Uuid;

pub fn cmd_rca(args: RcaArgs) -> CellResult<()> {
    let mut service = RCAV2Service::new();

    match args.sub {
        RcaSub::Analyze { signal } => {
            let rca_signal = RCASignal::new(
                Uuid::new_v4().to_string(),
                SignalType::Log,
                "cli",
                "unknown",
                signal,
                SignalSeverity::Warning,
                Utc::now(),
            );
            let record = service.analyze_signal(rca_signal)?;
            println!("{}", service.format_result(&record));
            println!("\n✅ RCA analysis completed.");
            println!("   分析 ID: {}", record.id);
        }
        RcaSub::List {} => {
            let analyses = service.list_analyses();
            println!("\n📋 RCA Analysis List\n{}", "─".repeat(60));
            if analyses.is_empty() {
                println!("  暂无分析记录");
            } else {
                println!("  共 {} 条记录\n", analyses.len());
                println!("  {:<36}  {:<15}  {:<10}  {:<10}  {}",
                    "ID", "Component", "Type", "Severity", "Root Cause");
                println!("  {}", "─".repeat(60));
                for summary in &analyses {
                    let root_cause = if summary.has_root_cause {
                        format!("✅ {:.1}%", summary.confidence.unwrap_or(0.0) * 100.0)
                    } else {
                        "❌".to_string()
                    };
                    println!("  {:<36}  {:<15}  {:<10}  {:<10}  {}",
                        summary.id.chars().take(36).collect::<String>(),
                        summary.signal_component.chars().take(15).collect::<String>(),
                        summary.signal_type.chars().take(10).collect::<String>(),
                        summary.signal_severity.chars().take(10).collect::<String>(),
                        root_cause);
                }
            }
            println!("{}", "─".repeat(60));
        }
        RcaSub::Detail { id } => {
            let record = service.get_analysis_detail(&id)?;
            println!("{}", service.format_result(&record));
        }
    }

    Ok(())
}
