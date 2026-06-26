use crate::application::saga_service::SagaService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_saga(args: SagaArgs) -> CellResult<()> {
    let service = SagaService::new();
    match args.sub {
        SagaSub::Create { name } => {
            service.create_saga(&name)?;
            println!("\n✅ 创建 Saga: {}\n", name);
        }
        SagaSub::List {} => {
            println!("\n📋 Saga 列表\n");
            let sagas = service.list_sagas();
            if sagas.is_empty() {
                println!("  (无)");
            } else {
                for s in sagas {
                    println!("  • {}", s);
                }
            }
        }
    }
    Ok(())
}
