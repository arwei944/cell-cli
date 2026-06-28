use cell_application::ab_test_service::ABTestService;
use cell_domain::ab_experiment::ExperimentType;
use cell_domain::errors::CellResult;
use crate::cli::{AbArgs, AbSub};

pub fn cmd_ab(args: AbArgs) -> CellResult<()> {
    let mut service = ABTestService::new();

    match args.sub {
        AbSub::Create { name, experiment_type } => {
            let exp_type = parse_experiment_type(experiment_type.as_deref())?;
            let variants = vec![
                ("control".to_string(), 50),
                ("treatment".to_string(), 50),
            ];

            let result = service.create_experiment(name, exp_type, variants)?;
            println!("{}", service.format_result(&service.get_experiment_result(&result.name)?));
            println!("\n✅ A/B experiment created successfully.");
            println!("   提示: 使用 `cell ab start {}` 开始实验", result.name);
        }
        AbSub::List {} => {
            let experiments = service.list_experiments();
            println!("{}", service.format_list(&experiments));
        }
        AbSub::Start { name } => {
            let _result = service.start_experiment(&name)?;
            println!("{}", service.format_result(&service.get_experiment_result(&name)?));
            println!("\n🚀 A/B experiment started successfully.");
        }
        AbSub::Pause { name } => {
            let _result = service.pause_experiment(&name)?;
            println!("{}", service.format_result(&service.get_experiment_result(&name)?));
            println!("\n⏸️  A/B experiment paused successfully.");
        }
        AbSub::Result { name } => {
            let result = service.get_experiment_result(&name)?;
            println!("{}", service.format_result(&result));
        }
        AbSub::End { name } => {
            let _result = service.end_experiment(&name)?;
            println!("{}", service.format_result(&service.get_experiment_result(&name)?));
            println!("\n🏁 A/B experiment ended successfully.");
        }
    }

    Ok(())
}

fn parse_experiment_type(s: Option<&str>) -> CellResult<ExperimentType> {
    let s = s.unwrap_or("feature");
    match s.to_lowercase().as_str() {
        "ui" => Ok(ExperimentType::UI),
        "algorithm" => Ok(ExperimentType::Algorithm),
        "feature" => Ok(ExperimentType::Feature),
        "price" => Ok(ExperimentType::Price),
        _ => Err(cell_domain::errors::CellError::Validation(format!(
            "Unknown experiment type: {s}. Valid: ui, algorithm, feature, price"
        ))),
    }
}
