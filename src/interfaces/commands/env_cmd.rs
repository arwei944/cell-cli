use crate::application::multi_env_config_service::{MultiEnvConfigService, Environment};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_env(args: EnvArgs) -> CellResult<()> {
    let service = MultiEnvConfigService::new();
    let project_path = ".";

    match args.sub {
        EnvSub::Create { name } => {
            let env_enum = match name.to_lowercase().as_str() {
                "dev" => Environment::Dev,
                "staging" => Environment::Staging,
                "prod" => Environment::Prod,
                _ => Environment::Custom(name.clone()),
            };
            
            let config = service.create_environment(project_path, &env_enum)?;
            
            println!("\n✅ 环境已创建\n");
            println!("  名称: {}", config.name);
            println!("  路径: {}", config.base_path);
            println!();
        }
        EnvSub::List {} => {
            let envs = service.list_environments(project_path)?;
            
            println!("\n🌍 环境列表\n");
            if envs.is_empty() {
                println!("  暂无环境配置");
            } else {
                for e in &envs {
                    println!("  • {} ({})", e.name, e.config_values.len());
                    println!("    路径: {}", e.base_path);
                    println!("    创建: {}", e.created_at);
                }
            }
            println!();
        }
        EnvSub::Set { env, key, value } => {
            service.set_config(project_path, &env, &key, &value)?;
            println!("\n✅ 配置已设置\n");
            println!("  环境: {}", env);
            println!("  键: {}", key);
            println!("  值: {}", value);
            println!();
        }
        EnvSub::Get { env, key } => {
            let value = service.get_config(project_path, &env, &key)?;
            
            println!("\n📝 配置值\n");
            println!("  环境: {}", env);
            println!("  键: {}", key);
            println!("  值: {}", value.as_deref().unwrap_or("未设置"));
            println!();
        }
        EnvSub::Diff { base, target } => {
            let report = service.diff_environments(project_path, &base, &target)?;
            
            println!("\n📊 配置差异报告\n");
            println!("  基线环境: {}", report.base_env);
            println!("  目标环境: {}", report.target_env);
            println!("  差异数量: {}", report.drift_count);
            println!();
            
            if report.drift_count > 0 {
                println!("  差异详情:");
                for diff in &report.diffs {
                    if diff.diff_type != crate::application::multi_env_config_service::DiffType::Same {
                        let icon = match diff.diff_type {
                            crate::application::multi_env_config_service::DiffType::Added => "+",
                            crate::application::multi_env_config_service::DiffType::Removed => "-",
                            crate::application::multi_env_config_service::DiffType::Changed => "~",
                            _ => " ",
                        };
                        println!("    {} {} ({})", icon, diff.key, diff.diff_type.label());
                    }
                }
                println!();
            }
        }
        EnvSub::Drift {} => {
            let reports = service.detect_drift(project_path)?;
            
            println!("\n🔍 配置漂移检测\n");
            if reports.is_empty() {
                println!("  ✅ 未检测到漂移");
            } else {
                for r in &reports {
                    println!("\n  {} -> {}:", r.base_env, r.target_env);
                    println!("    漂移数量: {}", r.drift_count);
                }
            }
            println!();
        }
        EnvSub::Sync { from, to } => {
            service.sync_config(project_path, &from, &to)?;
            println!("\n✅ 配置已同步\n");
            println!("  从: {}", from);
            println!("  到: {}", to);
            println!();
        }
    }

    Ok(())
}