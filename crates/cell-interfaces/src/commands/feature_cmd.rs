use cell_application::feature_service::FeatureService;
use cell_domain::errors::CellResult;
use cell_domain::feature::{FeatureFlagConfig, FeatureFlagType};
use crate::cli::{FeatureArgs, FeatureSub, FlagAction};
use std::path::Path;

const FEATURE_FLAG_FILE: &str = "features.yaml";

pub fn cmd_feature(args: FeatureArgs) -> CellResult<()> {
    let mut service = FeatureService::new();
    match args.sub {
        FeatureSub::New { name, description, owner } => {
            let f = service.create_feature(
                name.clone(),
                description.unwrap_or_else(|| "无描述".to_string()),
                owner,
            )?;
            println!("\n✅ 创建新功能单元: {name}\n");
            println!("版本: v{}\n", f.version);
        }
        FeatureSub::Mount { name } => {
            service.mount_feature(&name, "default-host", "default", 0)?;
            println!("\n🔧 挂载功能单元: {name}\n");
            println!("功能单元已挂载到运行时环境");
        }
        FeatureSub::Unmount { name } => {
            service.unmount_feature(&name, "default-host")?;
            println!("\n🔧 卸载功能单元: {name}\n");
            println!("功能单元已从运行时环境卸载");
        }
        FeatureSub::Impact { name } => {
            println!("\n📊 功能单元影响分析: {name}\n");
            let feature = service.get_feature(&name)?;
            println!("影响分析结果:");
            println!("  • 挂载点数量: {}", feature.mounts.len());
            println!("  • 依赖数量: {}", feature.dependencies.len());
            println!("\n提示: 完整的影响分析功能将在后续版本中提供");
        }
        FeatureSub::List {} => {
            println!("{}", service.format_list());
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
                        println!("❌ 未知的 flag 类型: {filter_type}");
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
                println!("✅ 已启用功能开关: {name}");
            } else {
                println!("❌ 未找到功能开关: {name}");
            }
        }
        FlagAction::Disable { name } => {
            if config.set_enabled(&name, false) {
                save_flag_config(&flag_file, &config)?;
                println!("⏸️  已禁用功能开关: {name}");
            } else {
                println!("❌ 未找到功能开关: {name}");
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
                    println!("  灰度比例: {pct}%");
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
                println!("❌ 未找到功能开关: {name}");
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
