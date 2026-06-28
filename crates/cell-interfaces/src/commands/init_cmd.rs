use cell_adapters::template_engine::TemplateEngine;
use cell_application::arch_linter::{ArchitectureLinter, LintResult};
use cell_application::generate_service::{GenerateOptions, GenerateService};
use cell_application::init_service::{InitInput, run_init};
use cell_domain::cell_spec::{AdapterKind, PortKind};
use cell_domain::errors::{CellError, CellResult};
use crate::cli::{InitArgs, GenerateArgs, GenerateSub, ValidateArgs};
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
    println!("Created Cell project at {path}");
    Ok(())
}

pub fn cmd_generate(args: GenerateArgs) -> CellResult<()> {
    let engine = TemplateEngine::new();
    let options = GenerateOptions {
        telemetry: !args.no_telemetry,
        cell_name: None,
    };
    let service = GenerateService::new(engine).with_options(options);
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
                println!("  - {f}");
            }
        }
        GenerateSub::Port { name, kind, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let port_kind = parse_port_kind(kind.as_deref().unwrap_or("usecase"))?;
            let file = service.generate_port(&name, port_kind, &out)?;
            println!("Generated port: {file}");
        }
        GenerateSub::Adapter { name, kind, port, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let adapter_kind = parse_adapter_kind(kind.as_deref().unwrap_or("inmemory"))?;
            let port_name = port.unwrap_or_else(|| "UnknownPort".to_string());
            let file = service.generate_adapter(&name, adapter_kind, &port_name, &out)?;
            println!("Generated adapter: {file}");
        }
        GenerateSub::Entity { name, fields, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_entity(&name, fields.as_deref(), &out)?;
            println!("Generated entity: {file}");
        }
        GenerateSub::ValueObject { name, fields, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_value_object(&name, fields.as_deref(), &out)?;
            println!("Generated value object: {file}");
        }
        GenerateSub::Aggregate { name, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_aggregate(&name, &out)?;
            println!("Generated aggregate: {file}");
        }
        GenerateSub::DomainEvent { name, fields, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_domain_event(&name, fields.as_deref(), &out)?;
            println!("Generated domain event: {file}");
        }
        GenerateSub::DomainService { name, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_domain_service(&name, &out)?;
            println!("Generated domain service: {file}");
        }
        GenerateSub::Repository { name, entity, output } => {
            let out = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_repository(&name, &entity, &out)?;
            println!("Generated repository: {file}");
        }
        GenerateSub::Usecase { name, input, output, impl_, output_dir } => {
            let out = output_dir.unwrap_or_else(|| ".".to_string());
            let files = service.generate_usecase(&name, input.as_deref(), output.as_deref(), impl_, &out)?;
            println!("Generated {} usecase files:", files.len());
            for f in &files {
                println!("  - {f}");
            }
        }
        GenerateSub::K8s { .. } | GenerateSub::Health { .. } => {
            return Err(cell_domain::errors::CellError::Config(
                "Use 'cell generate k8s' or 'cell generate health' via the new generate command".to_string()
            ));
        }
    }
    Ok(())
}

pub fn cmd_validate(args: ValidateArgs) -> CellResult<()> {
    let path = args.path.unwrap_or_else(|| ".".to_string());
    let linter = ArchitectureLinter::new();
    let result = linter.lint(Path::new(&path));

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_validate_report(&result);
    }

    if result.error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn print_validate_report(result: &LintResult) {
    println!("\n{}", "=".repeat(60));
    println!("  📐 Cell Architecture Validation Report");
    println!("{}", "=".repeat(60));
    
    println!("\n  📁 Scanned files: {}", result.total_files);
    println!("  📊 Total violations: {}", result.total_violations);
    println!("     ❌ Errors:   {}", result.error_count);
    println!("     ⚠️  Warnings: {}", result.warning_count);
    println!("     ℹ️  Info:     {}", result.info_count);

    if !result.by_category.is_empty() {
        println!("\n  📂 By category:");
        let mut cats: Vec<_> = result.by_category.iter().collect();
        cats.sort_by_key(|(k, _)| (*k).clone());
        for (cat, count) in cats {
            println!("     {cat}: {count}");
        }
    }

    if result.violations.is_empty() {
        println!("\n  ✅ All checks passed! Architecture is healthy.");
    } else {
        println!("\n  🔍 Violations:");
        println!("  {}", "─".repeat(56));
        
        let mut sorted = result.violations.clone();
        sorted.sort_by(|a, b| {
            let sev_order = |s: &cell_application::arch_linter::LintSeverity| match s {
                cell_application::arch_linter::LintSeverity::Error => 0,
                cell_application::arch_linter::LintSeverity::Warning => 1,
                cell_application::arch_linter::LintSeverity::Info => 2,
            };
            sev_order(&a.severity).cmp(&sev_order(&b.severity))
        });

        for v in &sorted {
            let icon = match v.severity {
                cell_application::arch_linter::LintSeverity::Error => "❌",
                cell_application::arch_linter::LintSeverity::Warning => "⚠️ ",
                cell_application::arch_linter::LintSeverity::Info => "ℹ️ ",
            };
            println!("\n  {} [{}] {}:{}", icon, v.rule_id, v.file, v.line);
            println!("     {}", v.message);
            if let Some(sug) = &v.suggestion {
                println!("     💡 Suggestion: {sug}");
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    
    if result.error_count > 0 {
        println!("  ❌ VALIDATION FAILED - {} error(s)", result.error_count);
    } else if result.warning_count > 0 {
        println!("  ⚠️  PASSED with {} warning(s)", result.warning_count);
    } else {
        println!("  ✅ VALIDATION PASSED");
    }
    println!("{}\n", "=".repeat(60));
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
            "Unknown port kind: {s}. Valid: usecase, query, repository, gateway, publisher, subscriber"
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
            "Unknown adapter kind: {s}. Valid: inmemory, postgres, redis, http, grpc, kafka, nats, file, mock"
        ))),
    }
}
