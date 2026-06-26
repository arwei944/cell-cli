use crate::application::self_healing_service::{
    SelfHealingService, AnomalyType, AnomalySeverity,
};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_self_heal(args: SelfHealArgs) -> CellResult<()> {
    let service = SelfHealingService::new();
    let project_path = ".";

    match args.sub {
        SelfHealSub::Status {} => {
            let report = service.generate_healing_report(project_path)?;
            println!("{}", service.format_report(&report));
        }
        SelfHealSub::Detect {} => {
            let anomalies = service.detect_anomalies(project_path)?;
            if anomalies.is_empty() {
                println!("\n✅ 未检测到异常，系统运行正常");
            } else {
                println!("\n⚠️  检测到 {} 个活跃异常\n", anomalies.len());
                for a in &anomalies {
                    let sev_icon = match a.severity {
                        AnomalySeverity::Fatal => "🔴",
                        AnomalySeverity::Critical => "🟠",
                        AnomalySeverity::Warning => "🟡",
                        AnomalySeverity::Info => "ℹ️",
                    };
                    println!("  {} [{}] {}", sev_icon, a.id, a.description);
                }
                println!();
            }
        }
        SelfHealSub::Recover { id } => {
            match service.attempt_recovery(project_path, &id)? {
                Some(action) => {
                    println!("\n🩹 恢复操作已执行\n");
                    println!("  类型: {:?}", action.action_type);
                    println!("  描述: {}", action.description);
                    println!(
                        "  结果: {}",
                        if action.success { "✅ 成功" } else { "❌ 失败" }
                    );
                    if let Some(result) = &action.result {
                        println!("  详情: {}", result);
                    }
                    println!();
                }
                None => {
                    println!("\n❌ 未找到异常: {}", id);
                }
            }
        }
        SelfHealSub::Report {
            description,
            severity,
            agent,
        } => {
            let sev = match severity.as_deref() {
                Some("fatal") => AnomalySeverity::Fatal,
                Some("critical") => AnomalySeverity::Critical,
                Some("warning") => AnomalySeverity::Warning,
                _ => AnomalySeverity::Info,
            };
            let id = service.report_anomaly(
                project_path,
                AnomalyType::Unknown,
                sev,
                &description,
                agent.as_deref(),
                None,
            )?;
            println!("\n✅ 已报告异常: {}", id);
        }
        SelfHealSub::Escalate { id } => {
            let anomalies = service.detect_anomalies(project_path)?;
            match anomalies.iter().find(|a| a.id == id) {
                Some(anomaly) => {
                    println!("{}", service.generate_human_intervention_report(anomaly));
                }
                None => {
                    println!("\n❌ 未找到异常: {}", id);
                }
            }
        }
    }

    Ok(())
}
