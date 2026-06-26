use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CellId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRequest {
    pub port_id: PortId,
    pub method: String,
    pub parameters: HashMap<String, String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResponse {
    pub status: u16,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExpectation {
    pub status_code: Option<u16>,
    pub body_schema: Option<String>,
    pub headers_required: Vec<String>,
    pub timeout_ms: u64,
}

impl Default for ContractExpectation {
    fn default() -> Self {
        Self {
            status_code: Some(200),
            body_schema: None,
            headers_required: Vec::new(),
            timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: String,
    pub provider_cell: CellId,
    pub consumer_cell: CellId,
    pub port_id: PortId,
    pub request: ContractRequest,
    pub expectation: ContractExpectation,
    pub description: String,
}

impl Contract {
    pub fn new(
        id: impl Into<String>,
        provider: impl Into<String>,
        consumer: impl Into<String>,
        port: impl Into<String>,
    ) -> Self {
        let port_id = PortId(port.into());
        Self {
            id: id.into(),
            provider_cell: CellId(provider.into()),
            consumer_cell: CellId(consumer.into()),
            port_id: port_id.clone(),
            request: ContractRequest {
                port_id,
                method: "POST".to_string(),
                parameters: HashMap::new(),
                headers: HashMap::new(),
            },
            expectation: ContractExpectation::default(),
            description: String::new(),
        }
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.request.method = method.into();
        self
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.request.parameters.insert(key.into(), value.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.request.headers.insert(key.into(), value.into());
        self
    }

    pub fn expect_status(mut self, status: u16) -> Self {
        self.expectation.status_code = Some(status);
        self
    }

    pub fn expect_header(mut self, header: impl Into<String>) -> Self {
        self.expectation.headers_required.push(header.into());
        self
    }

    pub fn validate_response(&self, response: &ContractResponse) -> ContractValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if let Some(expected_status) = self.expectation.status_code {
            if response.status != expected_status {
                errors.push(format!(
                    "Status mismatch: expected {}, got {}",
                    expected_status, response.status
                ));
            }
        }

        for required_header in &self.expectation.headers_required {
            if !response.headers.contains_key(required_header) {
                errors.push(format!("Missing required header: {}", required_header));
            }
        }

        for header in response.headers.keys() {
            let is_expected = self.expectation.headers_required.contains(header);
            if !is_expected && header.starts_with("X-") {
                warnings.push(format!("Unexpected header: {}", header));
            }
        }

        if self.request.headers.contains_key("Content-Type")
            && !response.headers.contains_key("Content-Type")
        {
            warnings.push("Request had Content-Type but response did not".to_string());
        }

        ContractValidationResult {
            passed: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTestCase {
    pub name: String,
    pub contract: Contract,
    pub mock_response: ContractResponse,
    pub expected_result: ContractTestExpectation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractTestExpectation {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTestResult {
    pub test_name: String,
    pub contract_id: String,
    pub passed: bool,
    pub actual_result: ContractValidationResult,
    pub execution_time_ms: u64,
    pub executed_at: String,
}

impl ContractTestResult {
    pub fn new(test: &ContractTestCase, result: ContractValidationResult, time_ms: u64) -> Self {
        Self {
            test_name: test.name.clone(),
            contract_id: test.contract.id.clone(),
            passed: result.passed == (test.expected_result == ContractTestExpectation::Pass),
            actual_result: result,
            execution_time_ms: time_ms,
            executed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

pub struct ContractRegistry {
    contracts: HashMap<String, Contract>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }

    pub fn register(&mut self, contract: Contract) -> Result<(), ContractError> {
        if self.contracts.contains_key(&contract.id) {
            return Err(ContractError::DuplicateContract(contract.id.clone()));
        }
        self.contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Contract> {
        self.contracts.get(id)
    }

    pub fn get_by_provider(&self, cell: &CellId) -> Vec<&Contract> {
        self.contracts
            .values()
            .filter(|c| &c.provider_cell == cell)
            .collect()
    }

    pub fn get_by_consumer(&self, cell: &CellId) -> Vec<&Contract> {
        self.contracts
            .values()
            .filter(|c| &c.consumer_cell == cell)
            .collect()
    }

    pub fn list_contracts(&self) -> Vec<&Contract> {
        self.contracts.values().collect()
    }

    pub fn validate_compatibility(&self, cell_a: &CellId, cell_b: &CellId) -> CompatibilityReport {
        let contracts_a_to_b: Vec<&Contract> = self.contracts
            .values()
            .filter(|c| &c.provider_cell == cell_a && &c.consumer_cell == cell_b)
            .collect();

        let contracts_b_to_a: Vec<&Contract> = self.contracts
            .values()
            .filter(|c| &c.provider_cell == cell_b && &c.consumer_cell == cell_a)
            .collect();

        CompatibilityReport {
            cell_a: cell_a.clone(),
            cell_b: cell_b.clone(),
            contracts_a_to_b: contracts_a_to_b.len(),
            contracts_b_to_a: contracts_b_to_a.len(),
            known_issues: Vec::new(),
        }
    }
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub cell_a: CellId,
    pub cell_b: CellId,
    pub contracts_a_to_b: usize,
    pub contracts_b_to_a: usize,
    pub known_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    DuplicateContract(String),
    ContractNotFound(String),
    InvalidContract(String),
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractError::DuplicateContract(id) => write!(f, "Contract already exists: {}", id),
            ContractError::ContractNotFound(id) => write!(f, "Contract not found: {}", id),
            ContractError::InvalidContract(msg) => write!(f, "Invalid contract: {}", msg),
        }
    }
}

impl std::error::Error for ContractError {}

pub struct ContractTestRunner {
    registry: ContractRegistry,
}

impl ContractTestRunner {
    pub fn new() -> Self {
        Self {
            registry: ContractRegistry::new(),
        }
    }

    pub fn register_contract(&mut self, contract: Contract) -> Result<(), ContractError> {
        self.registry.register(contract)
    }

    pub fn run_test(&self, test: &ContractTestCase) -> ContractTestResult {
        let start = std::time::Instant::now();
        let validation_result = test.contract.validate_response(&test.mock_response);
        let elapsed = start.elapsed().as_millis() as u64;

        ContractTestResult::new(test, validation_result, elapsed)
    }

    pub fn run_all_tests(&self, tests: &[ContractTestCase]) -> ContractTestSuiteResult {
        let results: Vec<ContractTestResult> = tests
            .iter()
            .map(|t| self.run_test(t))
            .collect();

        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;

        ContractTestSuiteResult {
            total: results.len(),
            passed,
            failed,
            results,
        }
    }
}

impl Default for ContractTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTestSuiteResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<ContractTestResult>,
}

impl ContractTestSuiteResult {
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_contract() -> Contract {
        Contract::new("test-contract-1", "provider-cell", "consumer-cell", "user-port")
            .with_method("GET")
            .expect_status(200)
            .expect_header("Content-Type")
    }

    #[test]
    fn test_contract_creation() {
        let contract = create_test_contract();
        assert_eq!(contract.id, "test-contract-1");
        assert_eq!(contract.provider_cell.0, "provider-cell");
        assert_eq!(contract.consumer_cell.0, "consumer-cell");
        assert_eq!(contract.request.method, "GET");
    }

    #[test]
    fn test_contract_validation_pass() {
        let contract = create_test_contract();
        let response = ContractResponse {
            status: 200,
            body: Some("{}".to_string()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string())
            ].into_iter().collect(),
        };

        let result = contract.validate_response(&response);
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_contract_validation_status_mismatch() {
        let contract = create_test_contract();
        let response = ContractResponse {
            status: 500,
            body: Some("error".to_string()),
            headers: HashMap::new(),
        };

        let result = contract.validate_response(&response);
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.contains("Status mismatch")));
    }

    #[test]
    fn test_contract_validation_missing_header() {
        let contract = create_test_contract();
        let response = ContractResponse {
            status: 200,
            body: Some("{}".to_string()),
            headers: HashMap::new(),
        };

        let result = contract.validate_response(&response);
        assert!(!result.passed);
        assert!(result.errors.iter().any(|e| e.contains("Missing required header")));
    }

    #[test]
    fn test_contract_registry() {
        let mut registry = ContractRegistry::new();
        let contract = create_test_contract();

        registry.register(contract.clone()).unwrap();

        let found = registry.get("test-contract-1");
        assert!(found.is_some());

        let by_provider = registry.get_by_provider(&CellId("provider-cell".to_string()));
        assert_eq!(by_provider.len(), 1);
    }

    #[test]
    fn test_contract_registry_duplicate() {
        let mut registry = ContractRegistry::new();
        let contract = create_test_contract();

        registry.register(contract.clone()).unwrap();
        let result = registry.register(contract);

        assert!(result.is_err());
    }

    #[test]
    fn test_contract_test_runner() {
        let runner = ContractTestRunner::new();

        let contract = create_test_contract();
        let response = ContractResponse {
            status: 200,
            body: Some("{}".to_string()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string())
            ].into_iter().collect(),
        };

        let test_case = ContractTestCase {
            name: "test case 1".to_string(),
            contract,
            mock_response: response,
            expected_result: ContractTestExpectation::Pass,
        };

        let result = runner.run_test(&test_case);
        assert!(result.passed);
    }

    #[test]
    fn test_test_suite_results() {
        let suite = ContractTestSuiteResult {
            total: 10,
            passed: 8,
            failed: 2,
            results: Vec::new(),
        };

        assert_eq!(suite.success_rate(), 80.0);
        assert!(!suite.all_passed());
    }

    #[test]
    fn test_compatibility_report() {
        let registry = ContractRegistry::new();

        let report = registry.validate_compatibility(
            &CellId("cell-a".to_string()),
            &CellId("cell-b".to_string()),
        );

        assert_eq!(report.cell_a.0, "cell-a");
        assert_eq!(report.cell_b.0, "cell-b");
    }

    #[test]
    fn test_contract_with_params() {
        let contract = Contract::new("param-contract", "p", "c", "p")
            .with_param("user_id", "123")
            .with_param("action", "get");

        assert_eq!(contract.request.parameters.get("user_id"), Some(&"123".to_string()));
        assert_eq!(contract.request.parameters.get("action"), Some(&"get".to_string()));
    }

    #[test]
    fn test_test_result_timing() {
        let suite = ContractTestSuiteResult {
            total: 5,
            passed: 5,
            failed: 0,
            results: vec![
                ContractTestResult {
                    test_name: "t1".to_string(),
                    contract_id: "c1".to_string(),
                    passed: true,
                    actual_result: ContractValidationResult {
                        passed: true,
                        errors: vec![],
                        warnings: vec![],
                    },
                    execution_time_ms: 100,
                    executed_at: "2024-01-01T00:00:00Z".to_string(),
                }
            ],
        };

        assert!(suite.all_passed());
    }
}
