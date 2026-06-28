use cell_adapters::file_decision_store::FileDecisionStore;
use cell_adapters::file_evolution_store::FileEvolutionStore;
use cell_adapters::file_handoff_exporter::FileHandoffExporter;
use cell_adapters::file_progress_store::FileProgressStore;
use cell_application::dev_env_service::DevEnvService;
use cell_application::dev_workflow_service::DevWorkflowService;
use cell_application::onboarding_service::OnboardingService;
use cell_domain::errors::CellResult;
use crate::cli::{DevArgs, DevSub};

pub fn cmd_dev(args: DevArgs) -> CellResult<()> {
    let progress_store = FileProgressStore::new();
    let evolution_store = FileEvolutionStore::new();
    let handoff_exporter = FileHandoffExporter::new();
    let decision_store = FileDecisionStore;
    
    let service = DevWorkflowService::new(
        progress_store,
        evolution_store,
        handoff_exporter,
        decision_store,
    );
    let project_path = ".";

    match args.sub {
        DevSub::Bootstrap {} => {
            let onboarding = OnboardingService::new();
            let result = onboarding.run_onboarding(project_path)?;
            println!("{}", onboarding.format_onboarding_result(&result));

            if result.all_successful {
                println!("\n🚀 开发环境已就绪！现在你可以:");
                println!("   • cell dev status - 查看当前状态");
                println!("   • cell dev next - 获取下一步建议");
                println!("   • cell task list - 查看待处理任务");
            }
        }
        DevSub::Doctor {} => {
            let onboarding = OnboardingService::new();
            let report = onboarding.check_environment(project_path)?;
            println!("{}", onboarding.format_doctor_report(&report));
        }
        DevSub::Start { name, description } => {
            let result = service.start_task(project_path, &name, description.as_deref())?;
            println!("{}", service.format_phase_result(&result));
            println!("\n💡 下一步: 运行 `cell dev design` 进入设计阶段");
        }
        DevSub::Design {} => {
            let result = service.design_phase(project_path)?;
            println!("{}", service.format_phase_result(&result));
            println!("\n💡 下一步: 开始编码，随时运行 `cell dev checkpoint` 检查进度");
        }
        DevSub::Checkpoint { message } => {
            let msg = message.unwrap_or_else(|| "代码检查点".to_string());
            let result = service.code_checkpoint(project_path, &msg)?;
            println!("{}", service.format_phase_result(&result));
        }
        DevSub::Verify { deep } => {
            let result = service.verify_phase(project_path, deep)?;
            println!("{}", service.format_phase_result(&result));
            if result.success {
                println!("\n💡 下一步: 运行 `cell dev handoff` 生成交接包");
            }
        }
        DevSub::Handoff { message } => {
            let result = service.handoff_phase(project_path, message.as_deref())?;
            println!("{}", service.format_phase_result(&result));
            println!("\n🎉 开发工作流完成！交接包已生成。");
        }
        DevSub::Status {} => {
            let env_service = DevEnvService::new();
            let status = env_service.get_status(project_path)?;
            println!("{}", env_service.format_status(&status));
        }
        DevSub::Decision { title, context, decision } => {
            let store = FileDecisionStore;
            service.record_decision(
                project_path,
                store,
                &title,
                &context.unwrap_or_default(),
                &decision.unwrap_or_default(),
            )?;
            println!("\n✅ 决策已记录: {title}\n");
        }
        DevSub::Next {} => {
            let env_service = DevEnvService::new();
            let suggestions = env_service.get_next_suggestions(project_path)?;
            println!("{}", env_service.format_suggestions(&suggestions));
        }
        DevSub::Context {} => {
            let env_service = DevEnvService::new();
            let snapshot = env_service.generate_context_snapshot(project_path)?;
            println!("{}", env_service.format_context_snapshot(&snapshot));
        }
        DevSub::Reset { scope } => {
            let scope_str = scope.unwrap_or_else(|| "all".to_string());
            println!("⚠️  即将重置开发环境 (范围: {scope_str})");
            println!("   这将清除所有运行时数据，但不会影响你的代码。");
            println!("   确认继续吗？(按 Enter 继续，Ctrl+C 取消)");

            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);

            let env_service = DevEnvService::new();
            env_service.reset_environment(project_path, &scope_str)?;
            println!("\n✅ 环境已重置");
        }
    }

    Ok(())
}
