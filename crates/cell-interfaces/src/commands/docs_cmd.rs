use cell_application::docs_generator_service::{DocsGeneratorService, DocsConfig, DocsFormat};
use cell_domain::errors::CellResult;
use crate::cli::{DocsArgs, DocsSub};

pub fn cmd_docs(args: DocsArgs) -> CellResult<()> {
    let service = DocsGeneratorService::new();
    let project_path = ".";
    let format = match args.format.to_lowercase().as_str() {
        "markdown" | "md" => DocsFormat::Markdown,
        "html" => DocsFormat::Html,
        "pdf" => DocsFormat::Pdf,
        "openapi" => DocsFormat::OpenApi,
        "all" => DocsFormat::All,
        _ => DocsFormat::Markdown,
    };

    let config = DocsConfig {
        output_dir: args.output.clone(),
        format,
        include_private: args.include_private,
        include_tests: args.include_tests,
    };

    match args.sub {
        DocsSub::Generate {} => {
            println!("\n📚 生成所有文档...\n");
            let docs = service.generate_all(project_path, &config)?;
            
            println!("  已生成文档:");
            for d in &docs {
                println!("    • {} ({})", d.name, d.format.label());
                println!("      路径: {}", d.path);
            }
            println!();
        }
        DocsSub::Architecture {} => {
            println!("\n📚 生成架构文档...\n");
            let docs = service.generate_architecture_docs(project_path, &config)?;
            
            for d in &docs {
                println!("  • {} ({})", d.name, d.format.label());
                println!("    路径: {}", d.path);
                println!("    预览:");
                for line in d.content_preview.lines().take(5) {
                    println!("      {line}");
                }
            }
            println!();
        }
        DocsSub::Api {} => {
            println!("\n📚 生成 API 文档...\n");
            let docs = service.generate_api_docs(project_path, &config)?;
            
            for d in &docs {
                println!("  • {} ({})", d.name, d.format.label());
                println!("    路径: {}", d.path);
            }
            println!();
        }
        DocsSub::Decisions {} => {
            println!("\n📚 生成决策记录文档...\n");
            let docs = service.generate_decision_docs(project_path, &config)?;
            
            for d in &docs {
                println!("  • {} ({})", d.name, d.format.label());
                println!("    路径: {}", d.path);
                println!("    预览:");
                for line in d.content_preview.lines().take(5) {
                    println!("      {line}");
                }
            }
            println!();
        }
        DocsSub::Serve { port } => {
            println!("\n📚 文档服务\n");
            println!("  端口: {port}");
            println!("  访问: http://localhost:{port}");
            println!();
            println!("  提示: 请先运行 `cell docs generate` 生成文档");
            println!();
        }
    }

    Ok(())
}