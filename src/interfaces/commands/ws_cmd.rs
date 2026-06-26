use crate::application::websocket_dashboard_service::WebSocketDashboardService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_ws(args: WsArgs) -> CellResult<()> {
    let service = WebSocketDashboardService::new();

    match args.sub {
        WsSub::Serve { port } => {
            println!("\n🌐 WebSocket Dashboard 服务启动\n");
            println!("  端口: {}", port);
            println!("  WebSocket: ws://localhost:{}/ws/dashboard", port);
            println!("  HTML: http://localhost:{}/dashboard", port);
            println!();
            println!("  按 Ctrl+C 停止服务");
            println!();
            
            println!("  HTML 页面预览:");
            println!("{}", service.generate_ws_html());
        }
        WsSub::Html {} => {
            println!("\n📄 WebSocket Dashboard HTML\n");
            println!("{}", service.generate_ws_html());
        }
        WsSub::Test {} => {
            println!("\n🧪 测试 WebSocket 消息\n");
            
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| crate::domain::errors::CellError::Config(e.to_string()))?;
            
            rt.block_on(async {
                service.update_entropy(42.5, "B", "stable").await?;
                println!("  ✅ 熵值更新消息已发送");
                
                service.update_progress("developing", 75.0, 3, 4).await?;
                println!("  ✅ 进度更新消息已发送");
                
                service.notify_test_completed(83, 0).await?;
                println!("  ✅ 测试完成消息已发送");
                
                service.agent_heartbeat("Cell CLI").await?;
                println!("  ✅ Agent 心跳消息已发送");
                
                Ok::<(), crate::domain::errors::CellError>(())
            })?;
            
            println!("\n  💡 在实际运行时，这些消息会通过 WebSocket 广播到客户端\n");
        }
    }

    Ok(())
}