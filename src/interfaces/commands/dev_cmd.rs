use crate::adapters::web_dashboard::start_dashboard_server;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;
use clap::CommandFactory;
use clap_complete::generate;

pub fn cmd_dashboard(args: DashboardArgs) -> CellResult<()> {
    let path = args.path.unwrap_or_else(|| ".".to_string());
    let port = args.port;

    if !args.no_open {
        let url = format!("http://localhost:{}", port);
        println!("🌐 正在打开仪表盘: {}", url);
        let _ = webbrowser::open(&url);
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { start_dashboard_server(&path, port).await })?;
    Ok(())
}

pub fn cmd_tools(args: ToolsArgs) -> CellResult<()> {
    match args.sub {
        ToolsSub::Enable { path } => handle_tools_enable(path.unwrap_or_else(|| ".".to_string())),
        ToolsSub::Status { path } => handle_tools_status(path.unwrap_or_else(|| ".".to_string())),
    }
}

fn handle_tools_enable(project_path: String) -> CellResult<()> {
    let cell_dir = std::path::Path::new(&project_path).join(".cell");
    let dirs = ["decisions", "progress", "evolution", "handoffs", "logs"];

    println!("🛠️  正在启用开发辅助工具...");
    for dir in &dirs {
        let dir_path = cell_dir.join(dir);
        std::fs::create_dir_all(&dir_path)?;
        println!("  ✅ {dir}/");
    }

    println!("\n🎉 所有辅助工具已启用！");
    println!("\n可用工具:");
    println!("  📊 cell dashboard    - 启动 Web 仪表盘");
    println!("  📝 cell progress     - 进度记录与追踪");
    println!("  💡 cell decision     - 决策记录管理");
    println!("  🧬 cell evolve       - 自进化系统");
    println!("  🤝 cell handoff      - 零漂移交接");
    println!("\n提示: 运行 cell dashboard 启动可视化仪表盘");
    Ok(())
}

fn handle_tools_status(project_path: String) -> CellResult<()> {
    let cell_dir = std::path::Path::new(&project_path).join(".cell");

    println!("🛠️  开发辅助工具状态");
    println!("项目路径: {}",
        std::fs::canonicalize(&project_path)
            .map(|p| p.display().to_string())
            .unwrap_or(project_path)
    );
    println!();

    let tools = [
        ("进度追踪", "progress", "cell progress"),
        ("决策记录", "decisions", "cell decision"),
        ("自进化系统", "evolution", "cell evolve"),
        ("交接工具", "handoffs", "cell handoff"),
        ("仪表盘", "-", "cell dashboard"),
    ];

    for (name, dir, cmd) in &tools {
        let status = if *dir == "-" || cell_dir.join(dir).exists() {
            "✅ 已启用"
        } else {
            "❌ 未启用"
        };
        println!("  {name}: {status}  ({cmd})");
    }
    Ok(())
}

pub fn cmd_completions(args: CompletionsArgs) -> CellResult<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}
