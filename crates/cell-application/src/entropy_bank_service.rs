use cell_domain::errors::{CellError, CellResult};
use cell_domain::feature::EntropyBankAccount;

/// 熵值银行服务
pub struct EntropyBankService;

impl EntropyBankService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_account(&self, owner: &str) -> CellResult<EntropyBankAccount> {
        Err(CellError::NotFound(format!("熵值银行账户 '{owner}' 不存在")))
    }

    pub fn deposit(&self, owner: &str, amount: f64, reason: &str) -> CellResult<()> {
        println!("💰 存入熵值: {amount} 给 {owner}，原因: {reason}");
        Ok(())
    }

    pub fn withdraw(&self, owner: &str, amount: f64, reason: &str) -> CellResult<()> {
        println!("💸 支取熵值: {amount} 从 {owner}，原因: {reason}");
        Ok(())
    }

    pub fn format_account(&self, account: &EntropyBankAccount) -> String {
        format!(
            "🏦 熵值银行账户: {}\n  余额: {:.1}\n  存入总计: {:.1}\n  支取总计: {:.1}",
            account.owner, account.balance, account.total_deposited, account.total_withdrawn
        )
    }
}

impl Default for EntropyBankService {
    fn default() -> Self {
        Self::new()
    }
}
