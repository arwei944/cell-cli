use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    System,
    Team,
    Cell,
    Feature,
}

impl AccountType {
    pub fn label(&self) -> &str {
        match self {
            Self::System => "System",
            Self::Team => "Team",
            Self::Cell => "Cell",
            Self::Feature => "Feature",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyAccount {
    pub id: AccountId,
    pub name: String,
    pub account_type: AccountType,
    pub balance: f64,
    pub credit_limit: f64,
    pub parent_account: Option<AccountId>,
    pub allocated_budget: f64,
    pub consumed: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl EntropyAccount {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        account_type: AccountType,
        initial_balance: f64,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: AccountId(id.into()),
            name: name.into(),
            account_type,
            balance: initial_balance,
            credit_limit: 0.0,
            parent_account: None,
            allocated_budget: initial_balance,
            consumed: 0.0,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_credit_limit(mut self, limit: f64) -> Self {
        self.credit_limit = limit;
        self
    }

    pub fn with_parent(mut self, parent: AccountId) -> Self {
        self.parent_account = Some(parent);
        self
    }

    pub fn available_balance(&self) -> f64 {
        self.balance + self.credit_limit
    }

    pub fn utilization_rate(&self) -> f64 {
        if self.allocated_budget == 0.0 {
            return 0.0;
        }
        (self.consumed / self.allocated_budget) * 100.0
    }

    pub fn is_over_budget(&self) -> bool {
        self.consumed > self.allocated_budget
    }

    pub fn deposit(&mut self, amount: f64) -> Result<f64, BankError> {
        if amount < 0.0 {
            return Err(BankError::InvalidAmount("Cannot deposit negative amount".to_string()));
        }
        self.balance += amount;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(self.balance)
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<f64, BankError> {
        if amount < 0.0 {
            return Err(BankError::InvalidAmount("Cannot withdraw negative amount".to_string()));
        }
        if self.balance - amount < -self.credit_limit {
            return Err(BankError::InsufficientFunds(format!(
                "Balance {} with credit limit {} is less than required {}",
                self.balance, self.credit_limit, amount
            )));
        }
        self.balance -= amount;
        self.consumed += amount;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(self.balance)
    }

    pub fn transfer_to(&mut self, to: &mut Self, amount: f64) -> Result<(), BankError> {
        if amount < 0.0 {
            return Err(BankError::InvalidAmount("Cannot transfer negative amount".to_string()));
        }
        if self.balance < amount {
            return Err(BankError::InsufficientFunds(format!(
                "Insufficient funds: balance {} < required {}",
                self.balance, amount
            )));
        }

        self.balance -= amount;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        to.balance += amount;
        to.updated_at = chrono::Utc::now().to_rfc3339();

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: Option<AccountId>,
    pub to: Option<AccountId>,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub description: String,
    pub timestamp: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdraw,
    Transfer,
    Allocation,
    Consumed,
    Adjustment,
}

impl TransactionType {
    pub fn label(&self) -> &str {
        match self {
            Self::Deposit => "Deposit",
            Self::Withdraw => "Withdraw",
            Self::Transfer => "Transfer",
            Self::Allocation => "Allocation",
            Self::Consumed => "Consumed",
            Self::Adjustment => "Adjustment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BankError {
    AccountNotFound(String),
    InsufficientFunds(String),
    InvalidAmount(String),
    DuplicateAccount(String),
    InvalidOperation(String),
}

impl std::fmt::Display for BankError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound(id) => write!(f, "Account not found: {id}"),
            Self::InsufficientFunds(msg) => write!(f, "Insufficient funds: {msg}"),
            Self::InvalidAmount(msg) => write!(f, "Invalid amount: {msg}"),
            Self::DuplicateAccount(id) => write!(f, "Account already exists: {id}"),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
        }
    }
}

impl std::error::Error for BankError {}

pub struct EntropyBank {
    accounts: HashMap<AccountId, EntropyAccount>,
    transactions: Vec<Transaction>,
    next_tx_id: u64,
}

impl EntropyBank {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: Vec::new(),
            next_tx_id: 1,
        }
    }

    pub fn create_account(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        account_type: AccountType,
        initial_balance: f64,
    ) -> Result<&EntropyAccount, BankError> {
        let id_str = id.into();
        let account_id = AccountId(id_str.clone());

        if self.accounts.contains_key(&account_id) {
            return Err(BankError::DuplicateAccount(id_str));
        }

        let account = EntropyAccount::new(id_str, name, account_type, initial_balance);
        self.accounts.insert(account_id.clone(), account);

        self.record_transaction(
            None,
            Some(account_id.clone()),
            initial_balance,
            TransactionType::Deposit,
            "Initial deposit".to_string(),
        );

        Ok(self.accounts.get(&account_id).unwrap())
    }

    pub fn get_account(&self, id: &AccountId) -> Option<&EntropyAccount> {
        self.accounts.get(id)
    }

    pub fn list_accounts(&self) -> Vec<&EntropyAccount> {
        self.accounts.values().collect()
    }

    pub fn accounts_by_type(&self, account_type: &AccountType) -> Vec<&EntropyAccount> {
        self.accounts
            .values()
            .filter(|a| &a.account_type == account_type)
            .collect()
    }

    pub fn deposit(&mut self, account_id: &AccountId, amount: f64, description: &str) -> Result<f64, BankError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or_else(|| BankError::AccountNotFound(account_id.0.clone()))?;

        let result = account.deposit(amount)?;

        self.record_transaction(
            None,
            Some(account_id.clone()),
            amount,
            TransactionType::Deposit,
            description.to_string(),
        );

        Ok(result)
    }

    pub fn withdraw(&mut self, account_id: &AccountId, amount: f64, description: &str) -> Result<f64, BankError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or_else(|| BankError::AccountNotFound(account_id.0.clone()))?;

        let result = account.withdraw(amount)?;

        self.record_transaction(
            Some(account_id.clone()),
            None,
            amount,
            TransactionType::Withdraw,
            description.to_string(),
        );

        Ok(result)
    }

    pub fn transfer(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        amount: f64,
        description: &str,
    ) -> Result<(), BankError> {
        if from == to {
            return Err(BankError::InvalidOperation("Cannot transfer to same account".to_string()));
        }

        let from_account = self.accounts.get(from).cloned()
            .ok_or_else(|| BankError::AccountNotFound(from.0.clone()))?;
        let _to_account = self.accounts.get(to)
            .ok_or_else(|| BankError::AccountNotFound(to.0.clone()))?;

        if from_account.balance < amount {
            return Err(BankError::InsufficientFunds(format!(
                "Account {} has {} but needs {}",
                from.0, from_account.balance, amount
            )));
        }

        {
            let from_acc = self.accounts.get_mut(from).unwrap();
            from_acc.balance -= amount;
            from_acc.updated_at = chrono::Utc::now().to_rfc3339();
        }

        {
            let to_acc = self.accounts.get_mut(to).unwrap();
            to_acc.balance += amount;
            to_acc.updated_at = chrono::Utc::now().to_rfc3339();
        }

        self.record_transaction(
            Some(from.clone()),
            Some(to.clone()),
            amount,
            TransactionType::Transfer,
            description.to_string(),
        );

        Ok(())
    }

    pub fn allocate_budget(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        amount: f64,
    ) -> Result<(), BankError> {
        self.transfer(from, to, amount, "Budget allocation")?;

        if let Some(acc) = self.accounts.get_mut(to) {
            acc.allocated_budget += amount;
        }

        Ok(())
    }

    pub fn record_consumption(
        &mut self,
        account_id: &AccountId,
        amount: f64,
        reason: &str,
    ) -> Result<(), BankError> {
        self.withdraw(account_id, amount, &format!("Consumption: {reason}"))?;
        Ok(())
    }

    pub fn total_balance(&self) -> f64 {
        self.accounts.values().map(|a| a.balance).sum()
    }

    pub fn total_consumed(&self) -> f64 {
        self.accounts.values().map(|a| a.consumed).sum()
    }

    pub fn get_transactions(&self, account_id: Option<&AccountId>) -> Vec<&Transaction> {
        account_id.map_or_else(
            || self.transactions.iter().collect(),
            |id| {
                self.transactions
                    .iter()
                    .filter(|t| {
                        t.from.as_ref().is_some_and(|a| a == id)
                            || t.to.as_ref().is_some_and(|a| a == id)
                    })
                    .collect()
            },
        )
    }

    pub fn over_budget_accounts(&self) -> Vec<&EntropyAccount> {
        self.accounts
            .values()
            .filter(|a| a.is_over_budget())
            .collect()
    }

    pub fn top_consumers(&self, n: usize) -> Vec<&EntropyAccount> {
        let mut accounts: Vec<&EntropyAccount> = self.accounts.values().collect();
        accounts.sort_by(|a, b| b.consumed.partial_cmp(&a.consumed).unwrap_or(std::cmp::Ordering::Equal));
        accounts.into_iter().take(n).collect()
    }

    fn record_transaction(
        &mut self,
        from: Option<AccountId>,
        to: Option<AccountId>,
        amount: f64,
        tx_type: TransactionType,
        description: String,
    ) {
        let tx = Transaction {
            id: format!("tx-{}", self.next_tx_id),
            from,
            to,
            amount,
            transaction_type: tx_type,
            description,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        self.transactions.push(tx);
        self.next_tx_id += 1;
    }
}

impl Default for EntropyBank {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let mut bank = EntropyBank::new();
        let account = bank.create_account(
            "team-a",
            "Team A",
            AccountType::Team,
            1000.0,
        ).unwrap();

        assert_eq!(account.balance, 1000.0);
        assert_eq!(account.account_type, AccountType::Team);
        assert_eq!(account.allocated_budget, 1000.0);
    }

    #[test]
    fn test_duplicate_account() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc-1", "Account 1", AccountType::Cell, 100.0).unwrap();
        let result = bank.create_account("acc-1", "Account 1", AccountType::Cell, 100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_deposit() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc-1", "Test", AccountType::Cell, 100.0).unwrap();
        let balance = bank.deposit(&AccountId("acc-1".to_string()), 50.0, "test deposit").unwrap();
        assert_eq!(balance, 150.0);
    }

    #[test]
    fn test_withdraw() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc-1", "Test", AccountType::Cell, 100.0).unwrap();
        let balance = bank.withdraw(&AccountId("acc-1".to_string()), 30.0, "test withdraw").unwrap();
        assert_eq!(balance, 70.0);
    }

    #[test]
    fn test_withdraw_insufficient() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc-1", "Test", AccountType::Cell, 50.0).unwrap();
        let result = bank.withdraw(&AccountId("acc-1".to_string()), 100.0, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut bank = EntropyBank::new();
        bank.create_account("from", "From", AccountType::Cell, 100.0).unwrap();
        bank.create_account("to", "To", AccountType::Cell, 50.0).unwrap();

        bank.transfer(
            &AccountId("from".to_string()),
            &AccountId("to".to_string()),
            30.0,
            "test transfer",
        ).unwrap();

        let from_acc = bank.get_account(&AccountId("from".to_string())).unwrap();
        let to_acc = bank.get_account(&AccountId("to".to_string())).unwrap();

        assert_eq!(from_acc.balance, 70.0);
        assert_eq!(to_acc.balance, 80.0);
    }

    #[test]
    fn test_transfer_insufficient() {
        let mut bank = EntropyBank::new();
        bank.create_account("from", "From", AccountType::Cell, 10.0).unwrap();
        bank.create_account("to", "To", AccountType::Cell, 50.0).unwrap();

        let result = bank.transfer(
            &AccountId("from".to_string()),
            &AccountId("to".to_string()),
            30.0,
            "test",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_transfer_same_account() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc", "Test", AccountType::Cell, 100.0).unwrap();

        let result = bank.transfer(
            &AccountId("acc".to_string()),
            &AccountId("acc".to_string()),
            10.0,
            "test",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_total_balance() {
        let mut bank = EntropyBank::new();
        bank.create_account("a", "A", AccountType::Cell, 100.0).unwrap();
        bank.create_account("b", "B", AccountType::Cell, 200.0).unwrap();

        assert_eq!(bank.total_balance(), 300.0);
    }

    #[test]
    fn test_utilization_rate() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc", "Test", AccountType::Cell, 100.0).unwrap();
        bank.record_consumption(&AccountId("acc".to_string()), 30.0, "entropy increase").unwrap();

        let acc = bank.get_account(&AccountId("acc".to_string())).unwrap();
        assert_eq!(acc.utilization_rate(), 30.0);
    }

    #[test]
    fn test_over_budget() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc", "Test", AccountType::Cell, 100.0).unwrap();

        let account = bank.get_account(&AccountId("acc".to_string())).unwrap();
        assert!(!account.is_over_budget());
    }

    #[test]
    fn test_account_not_found() {
        let bank = EntropyBank::new();
        let result = bank.get_account(&AccountId("nonexistent".to_string()));
        assert!(result.is_none());
    }

    #[test]
    fn test_transaction_history() {
        let mut bank = EntropyBank::new();
        bank.create_account("acc", "Test", AccountType::Cell, 100.0).unwrap();
        bank.deposit(&AccountId("acc".to_string()), 50.0, "test").unwrap();

        let txs = bank.get_transactions(Some(&AccountId("acc".to_string())));
        assert!(txs.len() >= 2);
    }

    #[test]
    fn test_top_consumers() {
        let mut bank = EntropyBank::new();
        bank.create_account("low", "Low", AccountType::Cell, 1000.0).unwrap();
        bank.create_account("high", "High", AccountType::Cell, 1000.0).unwrap();

        bank.record_consumption(&AccountId("high".to_string()), 500.0, "test").unwrap();
        bank.record_consumption(&AccountId("low".to_string()), 100.0, "test").unwrap();

        let top = bank.top_consumers(2);
        assert_eq!(top[0].name, "High");
        assert_eq!(top[1].name, "Low");
    }

    #[test]
    fn test_accounts_by_type() {
        let mut bank = EntropyBank::new();
        bank.create_account("team-1", "Team 1", AccountType::Team, 100.0).unwrap();
        bank.create_account("cell-1", "Cell 1", AccountType::Cell, 100.0).unwrap();
        bank.create_account("cell-2", "Cell 2", AccountType::Cell, 100.0).unwrap();

        let teams = bank.accounts_by_type(&AccountType::Team);
        let cells = bank.accounts_by_type(&AccountType::Cell);

        assert_eq!(teams.len(), 1);
        assert_eq!(cells.len(), 2);
    }

    #[test]
    fn test_allocate_budget() {
        let mut bank = EntropyBank::new();
        bank.create_account("parent", "Parent", AccountType::Team, 1000.0).unwrap();
        bank.create_account("child", "Child", AccountType::Cell, 0.0).unwrap();

        bank.allocate_budget(
            &AccountId("parent".to_string()),
            &AccountId("child".to_string()),
            200.0,
        ).unwrap();

        let child = bank.get_account(&AccountId("child".to_string())).unwrap();
        assert_eq!(child.allocated_budget, 200.0);
        assert_eq!(child.balance, 200.0);
    }

    #[test]
    fn test_available_balance_with_credit() {
        let account = EntropyAccount::new("test", "Test", AccountType::Cell, 100.0)
            .with_credit_limit(50.0);

        assert_eq!(account.available_balance(), 150.0);
    }

    #[test]
    fn test_total_consumed() {
        let mut bank = EntropyBank::new();
        bank.create_account("a", "A", AccountType::Cell, 100.0).unwrap();
        bank.create_account("b", "B", AccountType::Cell, 100.0).unwrap();

        bank.record_consumption(&AccountId("a".to_string()), 20.0, "test").unwrap();
        bank.record_consumption(&AccountId("b".to_string()), 30.0, "test").unwrap();

        assert_eq!(bank.total_consumed(), 50.0);
    }
}
