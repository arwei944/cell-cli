use cell_application::entropy_bank_service::EntropyBankService;
use cell_domain::errors::CellResult;
use crate::cli::{EntropyBankArgs, EntropyBankSub};

pub fn cmd_entropy_bank(args: EntropyBankArgs) -> CellResult<()> {
    let service = EntropyBankService::new();
    match args.sub {
        EntropyBankSub::Balance { owner } => {
            println!("\n🏦 查询熵值银行账户: {owner}\n");
            match service.get_account(&owner) {
                Ok(account) => println!("{}", service.format_account(&account)),
                Err(_) => println!("  账户不存在，余额: 0"),
            }
        }
        EntropyBankSub::Deposit { owner, amount, reason } => {
            service.deposit(&owner, amount, &reason)?;
            println!("✅ 存入成功");
        }
        EntropyBankSub::Withdraw { owner, amount, reason } => {
            service.withdraw(&owner, amount, &reason)?;
            println!("✅ 支取成功");
        }
    }
    Ok(())
}
