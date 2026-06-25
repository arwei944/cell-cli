use crate::adapters::template_engine::TemplateEngine;
use crate::application::generate_service::GenerateService;
use crate::application::init_service::{InitInput, run_init};
use crate::domain::cell_spec::{AdapterKind, PortKind};
use crate::domain::errors::{CellError, CellResult};
use crate::interfaces::cli::*;
use std::path::Path;

pub fn cmd_init(args: InitArgs) -> CellResult<()> {
    tracing::info!("Initializing cell project: {:?}", args.name);
    let input = InitInput {
        name: args.name,
        path: args.path,
        template: args.template,
        yes: args.yes,
        force: args.force,
    };
    let path = run_init(&input)?;
    println!("Created Cell project at {}", path);
    Ok(())
}

pub fn cmd_generate(args: GenerateArgs) -> CellResult<()> {
    let engine = TemplateEngine::new();
    let service = GenerateService::new(engine);
    match args.sub {
        GenerateSub::Cell { name, output, spec, force } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let files = if let Some(spec_path) = spec {
                service.generate_cell_from_spec(&spec_path, &out, force)?
            } else {
                let output_dir = Path::new(&out).join(&name);
                service.generate_cell_from_name(&name, output_dir.to_str().unwrap_or(&out), force)?
            };
            println!("Generated {} files:", files.len());
            for f in &files {
                println!("  - {}", f);
            }
        }
        GenerateSub::Port { name, kind, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let port_kind = parse_port_kind(kind.as_deref().unwrap_or("usecase"))?;
            let file = service.generate_port(&name, port_kind, &out)?;
            println!("Generated port: {}", file);
        }
        GenerateSub::Adapter { name, kind, port, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let adapter_kind = parse_adapter_kind(kind.as_deref().unwrap_or("inmemory"))?;
            let port_name = port.unwrap_or_else(|| "UnknownPort".to_string());
            let file = service.generate_adapter(&name, adapter_kind, &port_name, &out)?;
            println!("Generated adapter: {}", file);
        }
    }
    Ok(())
}

pub fn cmd_validate(_args: ValidateArgs) -> CellResult<()> {
    println!("cell validate");
    Ok(())
}

fn parse_port_kind(s: &str) -> CellResult<PortKind> {
    match s.to_lowercase().as_str() {
        "usecase" | "use_case" | "use-case" => Ok(PortKind::UseCase),
        "query" => Ok(PortKind::Query),
        "repository" | "repo" => Ok(PortKind::Repository),
        "gateway" => Ok(PortKind::Gateway),
        "publisher" | "pub" => Ok(PortKind::Publisher),
        "subscriber" | "sub" => Ok(PortKind::Subscriber),
        _ => Err(CellError::Config(format!(
            "Unknown port kind: {}. Valid: usecase, query, repository, gateway, publisher, subscriber",
            s
        ))),
    }
}

fn parse_adapter_kind(s: &str) -> CellResult<AdapterKind> {
    match s.to_lowercase().as_str() {
        "inmemory" | "in_memory" | "memory" => Ok(AdapterKind::InMemory),
        "postgres" | "pg" | "postgresql" => Ok(AdapterKind::Postgres),
        "redis" => Ok(AdapterKind::Redis),
        "http" | "http_client" => Ok(AdapterKind::Http),
        "grpc" => Ok(AdapterKind::Grpc),
        "kafka" => Ok(AdapterKind::Kafka),
        "nats" => Ok(AdapterKind::Nats),
        "file" => Ok(AdapterKind::File),
        "mock" => Ok(AdapterKind::Mock),
        _ => Err(CellError::Config(format!(
            "Unknown adapter kind: {}. Valid: inmemory, postgres, redis, http, grpc, kafka, nats, file, mock",
            s
        ))),
    }
}
