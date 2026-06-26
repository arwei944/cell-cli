use crate::application::service_mesh_service::ServiceMeshService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_mesh(args: MeshArgs) -> CellResult<()> {
    let service = ServiceMeshService::new();

    match args.sub {
        MeshSub::Generate { name, namespace, versions, gateway, output } => {
            let ns = namespace.unwrap_or_else(|| "default".to_string());
            let versions_vec = versions.unwrap_or_else(|| vec!["v1".to_string()]);
            let vers: Vec<&str> = versions_vec.iter().map(|s| s.as_str()).collect();
            
            let config = service.generate_istio_config_with_options(&name, &ns, &vers, gateway.as_deref())?;
            
            if let Some(out_path) = output {
                std::fs::write(&out_path, &config.yaml)?;
                println!("✅ Istio 配置已生成到: {}", out_path);
            } else {
                println!("{}", config.yaml);
            }
        }
        MeshSub::Validate { path } => {
            let result = service.validate_istio_config(&path)?;
            println!("{}", service.format_validation_result(&result));
            
            if !result.valid {
                std::process::exit(1);
            }
        }
        MeshSub::Diff { old, new } => {
            let diff = service.diff_configs(&old, &new)?;
            println!("{}", service.format_diff(&diff));
        }
    }

    Ok(())
}