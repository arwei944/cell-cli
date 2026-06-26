use crate::application::entropy_service::run_entropy_check;
use crate::application::entropy_trend_service::EntropyTrendService;
use crate::application::incremental_entropy_service::IncrementalEntropyService;
use crate::domain::errors::CellResult;
use crate::domain::feature::{FeatureFlagConfig, FeatureFlagType};
use crate::interfaces::cli::*;
use std::path::Path;

const FEATURE_FLAG_FILE: &str = "features.yaml";

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
            let service = EntropyTrendService::new();
            let trend = service.analyze(".")?;
            println!("{}", service.format_trend(&trend));
        }
    }
    Ok(())
}

pub fn cmd_feature(args: FeatureArgs) -> CellResult<()> {
    match args.sub {
        FeatureSub::New { name, description, owner: _owner } => {
            println!("创建功能单元: {}", name);
            if let Some(desc) = description {
                println!("  描述: {}", desc);
            }
            println!("  TODO: 实现 feature new 命令");
        }
        FeatureSub::Mount { name } => {
            println!("挂载功能单元: {}", name);
            println!("  TODO: 实现 feature mount 命令");
        }
        FeatureSub::Unmount { name } => {
            println!("卸载功能单元: {}", name);
            println!("  TODO: 实现 feature unmount 命令");
        }
        FeatureSub::Impact { name } => {
            println!("分析功能影响: {}", name);
            println!("  TODO: 实现 feature impact 命令");
        }
        FeatureSub::List {} => {
            println!("功能单元列表:");
            println!("  TODO: 实现 feature list 命令");
        }
        FeatureSub::Flag { action } => {
            handle_feature_flag(action)?;
        }
    }
    Ok(())
}

fn handle_feature_flag(action: FlagAction) -> CellResult<()> {
    let config_path = Path::new(".");
    let flag_file = config_path.join(FEATURE_FLAG_FILE);
    
    let mut config = if flag_file.exists() {
        let content = std::fs::read_to_string(&flag_file)?;
        FeatureFlagConfig::from_yaml(&content)?
    } else {
        FeatureFlagConfig::default()
    };

    match action {
        FlagAction::List { r#type } => {
            let flags = if let Some(filter_type) = r#type {
                let flag_type = match filter_type.as_str() {
                    "release" => FeatureFlagType::Release,
                    "ops" => FeatureFlagType::Ops,
                    "experiment" => FeatureFlagType::Experiment,
                    "permission" => FeatureFlagType::Permission,
                    _ => {
                        println!("❌ 未知的 flag 类型: {}", filter_type);
                        println!("   支持的类型: release, ops, experiment, permission");
                        return Ok(());
                    }
                };
                config.list_by_type(&flag_type)
            } else {
                config.flags.iter().collect()
            };

            println!("\n🚩 功能开关列表\n{}", "─".repeat(60));
            println!("  共 {} 个开关\n", flags.len());

            if flags.is_empty() {
                println!("  暂无功能开关");
            } else {
                println!("  {:<25} {:<12} {:<8} {:<20}", "名称", "类型", "状态", "描述");
                println!("  {}", "─".repeat(58));
                for flag in &flags {
                    let status = if flag.enabled { "✅ 启用" } else { "⏸️ 禁用" };
                    let type_str = format!("{:?}", flag.flag_type).to_lowercase();
                    println!("  {:<25} {:<12} {:<10} {:<20}", 
                        flag.name, 
                        type_str,
                        status,
                        if flag.description.len() > 18 { format!("{}...", &flag.description[..18]) } else { flag.description.clone() }
                    );
                }
            }
            println!();
        }
        FlagAction::Enable { name } => {
            if config.set_enabled(&name, true) {
                save_flag_config(&flag_file, &config)?;
                println!("✅ 已启用功能开关: {}", name);
            } else {
                println!("❌ 未找到功能开关: {}", name);
            }
        }
        FlagAction::Disable { name } => {
            if config.set_enabled(&name, false) {
                save_flag_config(&flag_file, &config)?;
                println!("⏸️  已禁用功能开关: {}", name);
            } else {
                println!("❌ 未找到功能开关: {}", name);
            }
        }
        FlagAction::Show { name } => {
            if let Some(flag) = config.get_flag(&name) {
                println!("\n🚩 功能开关详情\n{}", "─".repeat(60));
                println!("  名称: {}", flag.name);
                println!("  类型: {:?} ({})", flag.flag_type, flag.flag_type.description());
                println!("  状态: {}", if flag.enabled { "✅ 启用" } else { "⏸️ 禁用" });
                println!("  描述: {}", flag.description);
                if let Some(pct) = flag.percentage {
                    println!("  灰度比例: {}%", pct);
                }
                if !flag.target_users.is_empty() {
                    println!("  目标用户: {:?}", flag.target_users);
                }
                if !flag.target_groups.is_empty() {
                    println!("  目标组: {:?}", flag.target_groups);
                }
                println!("  创建时间: {}", flag.created_at);
                println!("  更新时间: {}", flag.updated_at);
                println!();
            } else {
                println!("❌ 未找到功能开关: {}", name);
            }
        }
    }

    Ok(())
}

fn save_flag_config(path: &Path, config: &FeatureFlagConfig) -> CellResult<()> {
    let yaml = config.to_yaml()?;
    std::fs::write(path, yaml)?;
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
