use cell_application::plugin_sandbox_service::PluginSandboxService;
use cell_domain::errors::CellResult;
use crate::cli::{SandboxArgs, SandboxSub};

pub fn cmd_sandbox(args: SandboxArgs) -> CellResult<()> {
    let service = PluginSandboxService::new();

    match args.sub {
        SandboxSub::Create { name } => {
            let result = service.create_sandbox(&name)?;
            println!("✅ 沙箱创建成功");
            println!("   ID: {}", result.id);
            println!("   插件: {}", result.plugin_id);
            println!("   状态: {}", result.status);
            println!("   创建时间: {}", result.created_at);
        }
        SandboxSub::List {} => {
            let sandboxes = service.list_sandboxes()?;
            
            if sandboxes.is_empty() {
                println!("📭 暂无沙箱");
                return Ok(());
            }

            println!("\n📋 沙箱列表");
            println!("{}", "─".repeat(60));
            println!("{:<20} {:<15} {:<10} {:<10}", "名称", "插件", "状态", "调用次数");
            println!("{}", "─".repeat(60));

            for sb in &sandboxes {
                println!("{:<20} {:<15} {:<10} {:<10}", sb.id, sb.plugin_id, sb.status, sb.total_calls);
            }

            println!("{}", "─".repeat(60));
            println!("  共 {} 个沙箱", sandboxes.len());
        }
        SandboxSub::Limits { name } => {
            let limits = service.get_sandbox_limits(&name)?;
            println!("\n📊 沙箱资源限制: {name}");
            println!("{}", "─".repeat(40));
            println!("  内存限制: {} MB", limits.memory_bytes / 1024 / 1024);
            println!("  执行超时: {} ms", limits.execution_time_ms);
            println!("  最大调用数: {}", limits.max_call_count);
            println!("  CPU占比: {}%", limits.cpu_percent);
            println!("{}", "─".repeat(40));
        }
        SandboxSub::Exec { name, cmd } => {
            let cmd_str = cmd.join(" ");
            let result = service.exec_in_sandbox(&name, &cmd_str)?;
            
            if result.success {
                println!("✅ 命令执行成功");
                if let Some(output) = result.output {
                    println!("   输出: {output}");
                }
                println!("   耗时: {} ms", result.duration_ms);
                println!("   内存使用: {} KB", result.memory_used_bytes / 1024);
            } else {
                println!("❌ 命令执行失败");
                if let Some(error) = result.error {
                    println!("   错误: {error}");
                }
            }
        }
    }

    Ok(())
}
