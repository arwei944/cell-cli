use cell_adapters::template_engine::TemplateEngine;
use cell_application::generate_service::{GenerateOptions, GenerateService};
use cell_domain::cell_spec::{AdapterKind, PortKind};
use cell_domain::errors::CellResult;
use crate::cli::{GenerateArgs, GenerateSub};

pub fn cmd_generate(args: GenerateArgs) -> CellResult<()> {
    let engine = TemplateEngine::new();
    let options = GenerateOptions {
        telemetry: !args.no_telemetry,
        cell_name: None,
    };
    let service = GenerateService::new(engine).with_options(options);

    match args.sub {
        GenerateSub::Cell { name, output, spec, force } => {
            let out_dir = output.unwrap_or_else(|| name.clone());
            
            if let Some(spec_path) = spec {
                let files = service.generate_cell_from_spec(&spec_path, &out_dir, force)?;
                println!("\n✅ 从规格文件生成 Cell: {name}\n");
                println!("生成的文件:");
                for f in files {
                    println!("   • {f}");
                }
            } else {
                let files = service.generate_cell_from_name(&name, &out_dir, force)?;
                println!("\n✅ 生成 Cell: {name}\n");
                println!("生成的文件:");
                for f in files {
                    println!("   • {f}");
                }
            }
        }
        GenerateSub::Port { name, kind, output } => {
            let port_kind = kind.map(|k| parse_port_kind(&k)).transpose()?.unwrap_or(PortKind::UseCase);
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let port_kind_clone = port_kind.clone();
            
            let file = service.generate_port(&name, port_kind, &out_dir)?;
            println!("\n✅ 生成 Port: {name} ({port_kind_clone:?})\n");
            println!("输出文件: {file}");
        }
        GenerateSub::Adapter { name, kind, port, output } => {
            let adapter_kind = kind.map(|k| parse_adapter_kind(&k)).transpose()?.unwrap_or(AdapterKind::InMemory);
            let port_name = port.unwrap_or_default();
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let adapter_kind_clone = adapter_kind.clone();
            
            let file = service.generate_adapter(&name, adapter_kind, &port_name, &out_dir)?;
            println!("\n✅ 生成 Adapter: {name} ({adapter_kind_clone:?})\n");
            println!("输出文件: {file}");
            if !port_name.is_empty() {
                println!("实现的 Port: {port_name}");
            }
        }
        GenerateSub::Entity { name, fields, output } => {
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_entity(&name, fields.as_deref(), &out_dir)?;
            println!("\n✅ 生成 Entity: {name}\n");
            println!("输出文件: {file}");
        }
        GenerateSub::ValueObject { name, fields, output } => {
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_value_object(&name, fields.as_deref(), &out_dir)?;
            println!("\n✅ 生成 ValueObject: {name}\n");
            println!("输出文件: {file}");
        }
        GenerateSub::Aggregate { name, output } => {
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_aggregate(&name, &out_dir)?;
            println!("\n✅ 生成 Aggregate: {name}\n");
            println!("输出文件: {file}");
        }
        GenerateSub::DomainEvent { name, fields, output } => {
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_domain_event(&name, fields.as_deref(), &out_dir)?;
            println!("\n✅ 生成 DomainEvent: {name}\n");
            println!("输出文件: {file}");
        }
        GenerateSub::DomainService { name, output } => {
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_domain_service(&name, &out_dir)?;
            println!("\n✅ 生成 DomainService: {name}\n");
            println!("输出文件: {file}");
        }
        GenerateSub::Repository { name, entity, output } => {
            let out_dir = output.unwrap_or_else(|| ".".to_string());
            let file = service.generate_repository(&name, &entity, &out_dir)?;
            println!("\n✅ 生成 Repository: {name}\n");
            println!("输出文件: {file}");
            println!("管理实体: {entity}");
        }
        GenerateSub::Usecase { name, input, output, impl_, output_dir } => {
            let out_dir = output_dir.unwrap_or_else(|| ".".to_string());
            let files = service.generate_usecase(&name, input.as_deref(), output.as_deref(), impl_, &out_dir)?;
            println!("\n✅ 生成 UseCase: {name}\n");
            println!("生成的文件:");
            for f in files {
                println!("   • {f}");
            }
        }
        GenerateSub::K8s { name, image, port, replicas, service_type, namespace, no_hpa, output } => {
            generate_k8s_manifests(&name, &image, port, replicas, service_type.as_deref(), namespace.as_deref(), no_hpa, output.as_deref())?;
        }
        GenerateSub::Health { output } => {
            generate_health_handler(output.as_deref())?;
        }
    }

    Ok(())
}

fn parse_port_kind(s: &str) -> CellResult<PortKind> {
    match s.to_lowercase().as_str() {
        "usecase" | "use-case" | "uc" => Ok(PortKind::UseCase),
        "query" | "q" => Ok(PortKind::Query),
        "repository" | "repo" => Ok(PortKind::Repository),
        "gateway" | "g" => Ok(PortKind::Gateway),
        "publisher" | "pub" => Ok(PortKind::Publisher),
        "subscriber" | "sub" => Ok(PortKind::Subscriber),
        _ => Err(cell_domain::errors::CellError::Config(format!(
            "Unknown port kind: {s}. Valid: usecase, query, repository, gateway, publisher, subscriber"
        ))),
    }
}

fn parse_adapter_kind(s: &str) -> CellResult<AdapterKind> {
    match s.to_lowercase().as_str() {
        "inmemory" | "in-memory" | "mem" => Ok(AdapterKind::InMemory),
        "postgres" | "pg" => Ok(AdapterKind::Postgres),
        "redis" => Ok(AdapterKind::Redis),
        "http" => Ok(AdapterKind::Http),
        "grpc" => Ok(AdapterKind::Grpc),
        "kafka" => Ok(AdapterKind::Kafka),
        "nats" => Ok(AdapterKind::Nats),
        "file" => Ok(AdapterKind::File),
        "mock" => Ok(AdapterKind::Mock),
        _ => Err(cell_domain::errors::CellError::Config(format!(
            "Unknown adapter kind: {s}. Valid: inmemory, postgres, redis, http, grpc, kafka, nats, file, mock"
        ))),
    }
}

fn generate_k8s_manifests(
    name: &str,
    image: &str,
    port: Option<u16>,
    replicas: Option<u32>,
    service_type: Option<&str>,
    namespace: Option<&str>,
    no_hpa: bool,
    output: Option<&str>,
) -> CellResult<()> {
    use cell_domain::k8s_deployment::{K8sDeploymentConfig, ServiceType};

    let mut config = K8sDeploymentConfig::new(name, image);

    if let Some(p) = port {
        config.port = p;
    }
    if let Some(r) = replicas {
        config.replicas = r;
    }
    if let Some(ns) = namespace {
        config.namespace = ns.to_string();
    }
    if let Some(st) = service_type {
        config.service_type = match st.to_lowercase().as_str() {
            "clusterip" | "cluster-ip" => ServiceType::ClusterIP,
            "nodeport" | "node-port" => ServiceType::NodePort,
            "loadbalancer" | "load-balancer" => ServiceType::LoadBalancer,
            _ => return Err(cell_domain::errors::CellError::Config(format!(
                "Invalid service type: {st}. Valid: ClusterIP, NodePort, LoadBalancer"
            ))),
        };
    }
    if no_hpa {
        config.hpa.enabled = false;
    }

    let yaml = config.generate_all_yaml();

    if let Some(out) = output {
        std::fs::write(out, &yaml)?;
        println!("\n✅ K8s manifests generated: {out}\n");
    } else {
        println!("\n📦 K8s Manifests for {name}:\n");
        println!("{}", "=".repeat(60));
        println!("{yaml}");
        println!("{}", "=".repeat(60));
    }

    println!("\nGenerated resources:");
    println!("   • Deployment ({})", config.replicas);
    println!("   • Service ({:?})", config.service_type);
    if config.hpa.enabled {
        println!("   • HPA (min: {}, max: {})", config.hpa.min_replicas, config.hpa.max_replicas);
    }
    println!("   • Health probes: startup + liveness + readiness");

    Ok(())
}

fn generate_health_handler(output: Option<&str>) -> CellResult<()> {
    use cell_domain::k8s_deployment::K8sDeploymentConfig;

    let config = K8sDeploymentConfig::new("temp", "temp:latest");
    let code = config.generate_health_handler_rs();

    if let Some(out) = output {
        std::fs::write(out, &code)?;
        println!("\n✅ Health handler generated: {out}\n");
    } else {
        println!("\n💚 Health Check Handler:\n");
        println!("{}", "=".repeat(60));
        println!("{code}");
        println!("{}", "=".repeat(60));
    }

    println!("\nFeatures:");
    println!("   • GET /health endpoint");
    println!("   • Status + version + uptime");
    println!("   • Axum-compatible router");

    Ok(())
}
