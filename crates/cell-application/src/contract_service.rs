use cell_domain::errors::CellResult;
use cell_domain::contract::Contract;

/// 契约服务
pub struct ContractService;

impl ContractService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_contract(
        &self,
        id: impl Into<String>,
        provider: impl Into<String>,
        consumer: impl Into<String>,
        port: impl Into<String>,
    ) -> CellResult<Contract> {
        Ok(Contract::new(id, provider, consumer, port))
    }

    pub fn list_contracts(&self) -> Vec<String> {
        vec!["示例契约: UserAPI".to_string()]
    }
}

impl Default for ContractService {
    fn default() -> Self {
        Self::new()
    }
}
