use crate::domain::errors::CellResult;
use crate::domain::saga::SagaDefinition;

/// Saga 服务
pub struct SagaService;

impl SagaService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_saga(&self, name: impl Into<String>) -> CellResult<SagaDefinition<String>> {
        let saga = SagaDefinition::new(name);
        Ok(saga)
    }

    pub fn list_sagas(&self) -> Vec<String> {
        vec!["示例 Saga: 订单支付".to_string()]
    }
}

impl Default for SagaService {
    fn default() -> Self {
        Self::new()
    }
}
