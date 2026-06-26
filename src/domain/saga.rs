use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SagaState {
    Pending,
    Running,
    Compensating,
    Completed,
    Failed,
    Cancelled,
}

impl Default for SagaState {
    fn default() -> Self {
        SagaState::Pending
    }
}

impl fmt::Display for SagaState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SagaState::Pending => write!(f, "Pending"),
            SagaState::Running => write!(f, "Running"),
            SagaState::Compensating => write!(f, "Compensating"),
            SagaState::Completed => write!(f, "Completed"),
            SagaState::Failed => write!(f, "Failed"),
            SagaState::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep<S: Clone> {
    pub name: String,
    pub execute: S,
    pub compensate: Option<S>,
    pub retry_policy: RetryPolicy,
    pub timeout_ms: u64,
}

impl<S: Clone> SagaStep<S> {
    pub fn new(name: impl Into<String>, execute: S) -> Self {
        Self {
            name: name.into(),
            execute,
            compensate: None,
            retry_policy: RetryPolicy::default(),
            timeout_ms: 30000,
        }
    }

    pub fn with_compensate(mut self, compensate: S) -> Self {
        self.compensate = Some(compensate);
        self
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn has_compensation(&self) -> bool {
        self.compensate.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32 - 1);
        delay.min(self.max_delay_ms as f64) as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaDefinition<S: Clone> {
    pub name: String,
    pub steps: Vec<SagaStep<S>>,
    pub timeout_ms: u64,
}

impl<S: Clone> SagaDefinition<S> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            timeout_ms: 300000,
        }
    }

    pub fn add_step(&mut self, step: SagaStep<S>) -> &mut Self {
        self.steps.push(step);
        self
    }

    pub fn step(&mut self, name: impl Into<String>, execute: S) -> &mut Self {
        self.steps.push(SagaStep::new(name, execute));
        self
    }

    pub fn validate(&self) -> Result<(), SagaError> {
        if self.steps.is_empty() {
            return Err(SagaError::InvalidDefinition("Saga must have at least one step".to_string()));
        }

        for (i, step) in self.steps.iter().enumerate() {
            if step.name.is_empty() {
                return Err(SagaError::InvalidDefinition(format!("Step {} has empty name", i)));
            }
            if step.timeout_ms == 0 {
                return Err(SagaError::InvalidDefinition(format!("Step '{}' has zero timeout", step.name)));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaExecution {
    pub id: String,
    pub saga_name: String,
    pub state: SagaState,
    pub current_step: usize,
    pub completed_steps: Vec<CompletedStep>,
    pub compensated_steps: Vec<CompensatedStep>,
    pub context: HashMap<String, String>,
    pub started_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
}

impl SagaExecution {
    pub fn new(id: impl Into<String>, saga_name: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            saga_name: saga_name.into(),
            state: SagaState::Pending,
            current_step: 0,
            completed_steps: Vec::new(),
            compensated_steps: Vec::new(),
            context: HashMap::new(),
            started_at: now.clone(),
            updated_at: now,
            error_message: None,
        }
    }

    pub fn set_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.context.insert(key.into(), value.into());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }

    pub fn mark_step_completed(&mut self, step_name: &str, result: HashMap<String, String>) {
        self.completed_steps.push(CompletedStep {
            step_name: step_name.to_string(),
            completed_at: chrono::Utc::now().to_rfc3339(),
            result,
        });
        self.current_step += 1;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn mark_step_compensated(&mut self, step_name: &str, compensation_result: HashMap<String, String>) {
        self.compensated_steps.push(CompensatedStep {
            step_name: step_name.to_string(),
            compensated_at: chrono::Utc::now().to_rfc3339(),
            compensation_result,
        });
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.state = SagaState::Failed;
        self.error_message = Some(error.into());
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn mark_completed(&mut self) {
        self.state = SagaState::Completed;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn mark_compensating(&mut self) {
        self.state = SagaState::Compensating;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn is_completed(&self) -> bool {
        self.state == SagaState::Completed
    }

    pub fn is_failed(&self) -> bool {
        self.state == SagaState::Failed
    }

    pub fn can_compensate(&self) -> bool {
        !self.completed_steps.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedStep {
    pub step_name: String,
    pub completed_at: String,
    pub result: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensatedStep {
    pub step_name: String,
    pub compensated_at: String,
    pub compensation_result: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SagaError {
    InvalidDefinition(String),
    StepExecutionFailed(String),
    CompensationFailed(String),
    Timeout(String),
    Cancelled(String),
}

impl fmt::Display for SagaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SagaError::InvalidDefinition(msg) => write!(f, "Invalid saga definition: {}", msg),
            SagaError::StepExecutionFailed(msg) => write!(f, "Step execution failed: {}", msg),
            SagaError::CompensationFailed(msg) => write!(f, "Compensation failed: {}", msg),
            SagaError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            SagaError::Cancelled(msg) => write!(f, "Cancelled: {}", msg),
        }
    }
}

impl std::error::Error for SagaError {}

pub struct SagaOrchestrator<S: Clone> {
    definition: SagaDefinition<S>,
    executions: HashMap<String, SagaExecution>,
}

impl<S: Clone> SagaOrchestrator<S> {
    pub fn new(definition: SagaDefinition<S>) -> Result<Self, SagaError> {
        definition.validate()?;
        Ok(Self {
            definition,
            executions: HashMap::new(),
        })
    }

    pub fn start(&mut self, execution_id: impl Into<String>) -> &mut SagaExecution {
        let id = execution_id.into();
        let execution = SagaExecution::new(&id, &self.definition.name);
        self.executions.insert(id.clone(), execution);
        self.executions.get_mut(&id).unwrap()
    }

    pub fn get_execution(&self, id: &str) -> Option<&SagaExecution> {
        self.executions.get(id)
    }

    pub fn get_execution_mut(&mut self, id: &str) -> Option<&mut SagaExecution> {
        self.executions.get_mut(id)
    }

    pub fn list_executions(&self) -> Vec<&SagaExecution> {
        self.executions.values().collect()
    }

    pub fn generate_code(&self, step_type: &str) -> String {
        match step_type {
            "rust" => self.generate_rust_code(),
            "typescript" => self.generate_typescript_code(),
            _ => self.generate_rust_code(),
        }
    }

    fn generate_rust_code(&self) -> String {
        let steps: Vec<String> = self.definition.steps.iter()
            .map(|s| {
                format!(
                    r#"    SagaStep::new("{}", |_| {{
        // Execute step logic
        Ok(HashMap::new())
    }})"#,
                    s.name
                )
            })
            .collect();

        format!(
            r#"use cell_architecture::saga::{{SagaDefinition, SagaStep, SagaOrchestrator}};

pub fn create_{}_saga() -> SagaOrchestrator<fn(&HashMap<String, String>) -> Result<HashMap<String, String>, String>> {{
    let mut saga = SagaDefinition::new("{}");

    {}
    
    SagaOrchestrator::new(saga).expect("Invalid saga definition")
}}
"#,
            self.definition.name.to_lowercase().replace(' ', "_"),
            self.definition.name,
            steps.join(",\n")
        )
    }

    fn generate_typescript_code(&self) -> String {
        let steps: Vec<String> = self.definition.steps.iter()
            .map(|s| format!(
                r#"  {{
    name: "{}",
    execute: async (ctx) => {{ /* execute */ }},
    compensate: {} /* compensate */
  }}"#,
                s.name,
                if s.has_compensation() { "async (ctx) => { /* compensate */ }" } else { "undefined" }
            ))
            .collect();

        format!(
            r#"export interface SagaContext {{
  [key: string]: string;
}}

export interface SagaStep {{
  name: string;
  execute: (ctx: SagaContext) => Promise<SagaContext>;
  compensate?: (ctx: SagaContext) => Promise<SagaContext>;
}}

export interface SagaDefinition {{
  name: string;
  steps: SagaStep[];
}}

export const {}Saga: SagaDefinition = {{
  name: "{}",
  steps: [
{}
  ]
}};
"#,
            self.definition.name.to_lowercase().replace(' ', "_"),
            self.definition.name,
            steps.join(",\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestStep = fn(&HashMap<String, String>) -> Result<HashMap<String, String>, String>;

    fn dummy_step(_ctx: &HashMap<String, String>) -> Result<HashMap<String, String>, String> {
        Ok(HashMap::new())
    }

    #[test]
    fn test_saga_definition_creation() {
        let mut saga = SagaDefinition::<TestStep>::new("TestSaga");
        saga.step("step1", dummy_step);
        saga.step("step2", dummy_step);

        assert_eq!(saga.steps.len(), 2);
        assert_eq!(saga.name, "TestSaga");
    }

    #[test]
    fn test_saga_validation_empty() {
        let saga = SagaDefinition::<TestStep>::new("EmptySaga");
        let result = saga.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_saga_validation_valid() {
        let mut saga = SagaDefinition::<TestStep>::new("ValidSaga");
        saga.step("step1", dummy_step);
        let result = saga.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_saga_execution_lifecycle() {
        let mut execution = SagaExecution::new("exec-1", "TestSaga");

        assert_eq!(execution.state, SagaState::Pending);

        execution.mark_step_completed("step1", HashMap::new());
        assert_eq!(execution.current_step, 1);

        execution.mark_step_completed("step2", HashMap::new());
        assert_eq!(execution.current_step, 2);

        execution.mark_completed();
        assert!(execution.is_completed());
        assert!(!execution.is_failed());
    }

    #[test]
    fn test_saga_execution_failure() {
        let mut execution = SagaExecution::new("exec-1", "TestSaga");
        execution.mark_step_completed("step1", HashMap::new());

        execution.mark_failed("Something went wrong");

        assert!(execution.is_failed());
        assert_eq!(execution.error_message.as_ref().unwrap(), "Something went wrong");
    }

    #[test]
    fn test_saga_execution_compensation() {
        let mut execution = SagaExecution::new("exec-1", "TestSaga");
        execution.mark_step_completed("step1", HashMap::new());
        execution.mark_step_completed("step2", HashMap::new());
        execution.mark_compensating();

        execution.mark_step_compensated("step2", HashMap::new());
        assert_eq!(execution.compensated_steps.len(), 1);

        execution.mark_step_compensated("step1", HashMap::new());
        assert_eq!(execution.compensated_steps.len(), 2);
    }

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        };

        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);
        let delay3 = policy.calculate_delay(3);

        assert!(delay1 <= 100);
        assert!(delay2 <= 200);
        assert!(delay3 <= 400);
    }

    #[test]
    fn test_saga_step_with_compensation() {
        let step = SagaStep::new("test-step", dummy_step)
            .with_compensate(dummy_step);

        assert!(step.has_compensation());
    }

    #[test]
    fn test_saga_state_display() {
        assert_eq!(SagaState::Pending.to_string(), "Pending");
        assert_eq!(SagaState::Running.to_string(), "Running");
        assert_eq!(SagaState::Completed.to_string(), "Completed");
        assert_eq!(SagaState::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_saga_orchestrator() {
        let mut saga = SagaDefinition::<TestStep>::new("OrchTestSaga");
        saga.step("step1", dummy_step);
        saga.step("step2", dummy_step);

        let orchestrator = SagaOrchestrator::new(saga).unwrap();

        let executions = orchestrator.list_executions();
        assert!(executions.is_empty());
    }

    #[test]
    fn test_saga_code_generation() {
        let mut saga = SagaDefinition::<TestStep>::new("OrderSaga");
        saga.step("reserve_inventory", dummy_step);
        saga.step("process_payment", dummy_step);
        saga.step("confirm_order", dummy_step);

        let orchestrator = SagaOrchestrator::new(saga).unwrap();
        let code = orchestrator.generate_code("rust");

        assert!(code.contains("OrderSaga"));
        assert!(code.contains("reserve_inventory"));
        assert!(code.contains("process_payment"));
    }

    #[test]
    fn test_saga_context() {
        let mut execution = SagaExecution::new("exec-1", "TestSaga");

        execution.set_context("order_id", "12345");
        execution.set_context("user_id", "67890");

        assert_eq!(execution.get_context("order_id"), Some(&"12345".to_string()));
        assert_eq!(execution.get_context("user_id"), Some(&"67890".to_string()));
        assert_eq!(execution.get_context("nonexistent"), None);
    }

    #[test]
    fn test_can_compensate() {
        let mut execution = SagaExecution::new("exec-1", "TestSaga");
        assert!(!execution.can_compensate());

        execution.mark_step_completed("step1", HashMap::new());
        assert!(execution.can_compensate());
    }
}
