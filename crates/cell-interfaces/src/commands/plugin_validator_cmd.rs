use cell_application::plugin_validator_service::PluginValidatorService;
use cell_domain::errors::CellResult;
use crate::cli::{PluginValidatorArgs, PluginValidatorSub};

pub fn cmd_plugin_validate(args: PluginValidatorArgs) -> CellResult<()> {
    let service = PluginValidatorService::new();

    match args.sub {
        PluginValidatorSub::Validate { path, json } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let result = service.validate_plugin(&p)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", service.format_validation_result(&result));
            }

            if !result.is_valid() {
                std::process::exit(1);
            }
        }
        PluginValidatorSub::Scan { path, json } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let result = service.scan_plugin(&p)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", service.format_scan_result(&result));
            }
        }
        PluginValidatorSub::Audit { path, json } => {
            let p = path.unwrap_or_else(|| ".".to_string());
            let result = service.audit_plugin(&p)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("{}", service.format_audit_result(&result));
            }

            if result.summary.risk_level == cell_application::plugin_validator_service::RiskLevel::Critical {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}