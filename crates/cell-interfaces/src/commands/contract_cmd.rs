use cell_application::contract_service::ContractService;
use cell_domain::errors::CellResult;
use crate::cli::{ContractArgs, ContractSub};

pub fn cmd_contract(args: ContractArgs) -> CellResult<()> {
    let service = ContractService::new();
    match args.sub {
        ContractSub::Create { id, provider, consumer, port } => {
            service.create_contract(&id, &provider, &consumer, &port)?;
            println!("\n✅ 创建契约: {id} ({provider} -> {consumer}:{port})\n");
        }
        ContractSub::List {} => {
            println!("\n📋 契约列表\n");
            let contracts = service.list_contracts();
            if contracts.is_empty() {
                println!("  (无)");
            } else {
                for c in contracts {
                    println!("  • {c}");
                }
            }
        }
    }
    Ok(())
}
