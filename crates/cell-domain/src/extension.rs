use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub type ExtensionResult<T> = Result<T, ExtensionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionError {
    NotFound(String),
    ExecutionFailed(String),
    ValidationFailed(String),
    Timeout,
    FallbackFailed(String),
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Extension not found: {name}"),
            Self::ExecutionFailed(msg) => write!(f, "Extension execution failed: {msg}"),
            Self::ValidationFailed(msg) => write!(f, "Extension validation failed: {msg}"),
            Self::Timeout => write!(f, "Extension execution timeout"),
            Self::FallbackFailed(msg) => write!(f, "Extension fallback failed: {msg}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    Validation,
    Calculation,
    Notification,
    Export,
    Transformation,
}

impl ExtensionType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Validation => "验证扩展点 - 用于自定义验证规则",
            Self::Calculation => "计算扩展点 - 用于自定义计算逻辑",
            Self::Notification => "通知扩展点 - 用于自定义通知渠道",
            Self::Export => "导出扩展点 - 用于自定义导出格式",
            Self::Transformation => "转换扩展点 - 用于自定义数据转换",
        }
    }
}

pub trait ExtensionPoint: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn extension_type(&self) -> ExtensionType;
    fn priority(&self) -> i32 { 0 }
    fn description(&self) -> &'static str { "" }
    fn is_enabled(&self) -> bool { true }
}

pub trait ValidationExtension: ExtensionPoint {
    fn validate(&self, context: &ExtensionContext) -> ExtensionResult<ValidationResult>;
    fn fallback(&self, _context: &ExtensionContext, _error: &ExtensionError) -> ExtensionResult<ValidationResult> {
        Err(ExtensionError::FallbackFailed("No fallback available".to_string()))
    }
}

pub trait CalculationExtension: ExtensionPoint {
    fn calculate(&self, input: f64, context: &ExtensionContext) -> ExtensionResult<f64>;
    fn fallback(&self, _input: f64, _context: &ExtensionContext, _error: &ExtensionError) -> ExtensionResult<f64> {
        Err(ExtensionError::FallbackFailed("No fallback available".to_string()))
    }
}

pub trait NotificationExtension: ExtensionPoint {
    fn notify(&self, message: &str, context: &ExtensionContext) -> ExtensionResult<bool>;
    fn fallback(&self, _message: &str, _context: &ExtensionContext, _error: &ExtensionError) -> ExtensionResult<bool> {
        Err(ExtensionError::FallbackFailed("No fallback available".to_string()))
    }
}

pub trait ExportExtension: ExtensionPoint {
    fn export(&self, data: &serde_json::Value, context: &ExtensionContext) -> ExtensionResult<Vec<u8>>;
    fn fallback(&self, _data: &serde_json::Value, _context: &ExtensionContext, _error: &ExtensionError) -> ExtensionResult<Vec<u8>> {
        Err(ExtensionError::FallbackFailed("No fallback available".to_string()))
    }
}

pub trait TransformationExtension: ExtensionPoint {
    fn transform(&self, input: serde_json::Value, context: &ExtensionContext) -> ExtensionResult<serde_json::Value>;
    fn fallback(&self, _input: serde_json::Value, _context: &ExtensionContext, _error: &ExtensionError) -> ExtensionResult<serde_json::Value> {
        Err(ExtensionError::FallbackFailed("No fallback available".to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
            warnings: vec![],
        }
    }

    pub fn warn(msg: impl Into<String>) -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![msg.into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionContext {
    pub feature_name: Option<String>,
    pub cell_name: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ExtensionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_feature(mut self, name: impl Into<String>) -> Self {
        self.feature_name = Some(name.into());
        self
    }

    pub fn with_cell(mut self, name: impl Into<String>) -> Self {
        self.cell_name = Some(name.into());
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ExtensionStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub fallback_calls: u64,
    pub total_duration_ms: u64,
}


pub struct ExtensionRegistry {
    validations: Vec<Arc<dyn ValidationExtension>>,
    calculations: Vec<Arc<dyn CalculationExtension>>,
    notifications: Vec<Arc<dyn NotificationExtension>>,
    exports: Vec<Arc<dyn ExportExtension>>,
    transformations: Vec<Arc<dyn TransformationExtension>>,
    stats: HashMap<String, ExtensionStats>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            validations: Vec::new(),
            calculations: Vec::new(),
            notifications: Vec::new(),
            exports: Vec::new(),
            transformations: Vec::new(),
            stats: HashMap::new(),
        }
    }

    pub fn register_validation(&mut self, ext: Arc<dyn ValidationExtension>) {
        self.validations.push(ext);
        self.validations.sort_by_key(|e| std::cmp::Reverse(e.priority()));
    }

    pub fn register_calculation(&mut self, ext: Arc<dyn CalculationExtension>) {
        self.calculations.push(ext);
        self.calculations.sort_by_key(|e| std::cmp::Reverse(e.priority()));
    }

    pub fn register_notification(&mut self, ext: Arc<dyn NotificationExtension>) {
        self.notifications.push(ext);
        self.notifications.sort_by_key(|e| std::cmp::Reverse(e.priority()));
    }

    pub fn register_export(&mut self, ext: Arc<dyn ExportExtension>) {
        self.exports.push(ext);
        self.exports.sort_by_key(|e| std::cmp::Reverse(e.priority()));
    }

    pub fn register_transformation(&mut self, ext: Arc<dyn TransformationExtension>) {
        self.transformations.push(ext);
        self.transformations.sort_by_key(|e| std::cmp::Reverse(e.priority()));
    }

    pub fn list_by_type(&self, ext_type: &ExtensionType) -> Vec<String> {
        match ext_type {
            ExtensionType::Validation => self.validations.iter().map(|e| e.name().to_string()).collect(),
            ExtensionType::Calculation => self.calculations.iter().map(|e| e.name().to_string()).collect(),
            ExtensionType::Notification => self.notifications.iter().map(|e| e.name().to_string()).collect(),
            ExtensionType::Export => self.exports.iter().map(|e| e.name().to_string()).collect(),
            ExtensionType::Transformation => self.transformations.iter().map(|e| e.name().to_string()).collect(),
        }
    }

    pub fn get_stats(&self, name: &str) -> Option<&ExtensionStats> {
        self.stats.get(name)
    }

    fn record_call(&mut self, name: &str, success: bool, fallback: bool, _duration_ms: u64) {
        let stats = self.stats.entry(name.to_string()).or_default();
        stats.total_calls += 1;
        if success {
            stats.successful_calls += 1;
        } else {
            stats.failed_calls += 1;
        }
        if fallback {
            stats.fallback_calls += 1;
        }
    }

    pub fn execute_validation_chain(&mut self, context: &ExtensionContext) -> ExtensionResult<ValidationResult> {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut at_least_one_ran = false;

        let exts: Vec<Arc<dyn ValidationExtension>> = self.validations
            .iter()
            .filter(|e| e.is_enabled())
            .cloned()
            .collect();

        for ext in &exts {
            at_least_one_ran = true;
            match ext.validate(context) {
                Ok(result) => {
                    all_errors.extend(result.errors);
                    all_warnings.extend(result.warnings);
                    self.record_call(ext.name(), true, false, 0);
                }
                Err(e) => {
                    if let Ok(result) = ext.fallback(context, &e) {
                        all_errors.extend(result.errors);
                        all_warnings.extend(result.warnings);
                        self.record_call(ext.name(), true, true, 0);
                    } else {
                        self.record_call(ext.name(), false, false, 0);
                        all_errors.push(format!("Validation extension '{}' failed: {}", ext.name(), e));
                    }
                }
            }
        }

        if !at_least_one_ran {
            return Ok(ValidationResult::ok());
        }

        Ok(ValidationResult {
            valid: all_errors.is_empty(),
            errors: all_errors,
            warnings: all_warnings,
        })
    }

    pub fn execute_calculation_chain(&mut self, input: f64, context: &ExtensionContext) -> ExtensionResult<f64> {
        let exts: Vec<Arc<dyn CalculationExtension>> = self.calculations
            .iter()
            .filter(|e| e.is_enabled())
            .cloned()
            .collect();

        let mut last_result = input;
        let mut at_least_one_ran = false;

        for ext in &exts {
            at_least_one_ran = true;
            match ext.calculate(last_result, context) {
                Ok(result) => {
                    last_result = result;
                    self.record_call(ext.name(), true, false, 0);
                }
                Err(e) => {
                    match ext.fallback(last_result, context, &e) {
                        Ok(result) => {
                            last_result = result;
                            self.record_call(ext.name(), true, true, 0);
                        }
                        Err(fe) => {
                            self.record_call(ext.name(), false, false, 0);
                            return Err(fe);
                        }
                    }
                }
            }
        }

        if !at_least_one_ran {
            return Ok(input);
        }

        Ok(last_result)
    }

    pub fn execute_notification_chain(&mut self, message: &str, context: &ExtensionContext) -> ExtensionResult<bool> {
        let exts: Vec<Arc<dyn NotificationExtension>> = self.notifications
            .iter()
            .filter(|e| e.is_enabled())
            .cloned()
            .collect();

        let mut all_succeeded = true;
        let mut at_least_one_ran = false;

        for ext in &exts {
            at_least_one_ran = true;
            match ext.notify(message, context) {
                Ok(success) => {
                    if !success {
                        all_succeeded = false;
                    }
                    self.record_call(ext.name(), true, false, 0);
                }
                Err(e) => {
                    if let Ok(success) = ext.fallback(message, context, &e) {
                        if !success {
                            all_succeeded = false;
                        }
                        self.record_call(ext.name(), true, true, 0);
                    } else {
                        all_succeeded = false;
                        self.record_call(ext.name(), false, false, 0);
                    }
                }
            }
        }

        if !at_least_one_ran {
            return Ok(false);
        }

        Ok(all_succeeded)
    }

    pub fn all_stats(&self) -> &HashMap<String, ExtensionStats> {
        &self.stats
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestValidation {
        name: String,
        should_pass: bool,
    }

    impl ExtensionPoint for TestValidation {
        fn name(&self) -> &str { &self.name }
        fn extension_type(&self) -> ExtensionType { ExtensionType::Validation }
        fn priority(&self) -> i32 { 0 }
    }

    impl ValidationExtension for TestValidation {
        fn validate(&self, _context: &ExtensionContext) -> ExtensionResult<ValidationResult> {
            if self.should_pass {
                Ok(ValidationResult::ok())
            } else {
                Ok(ValidationResult::error("test error"))
            }
        }
    }

    #[derive(Debug)]
    struct TestCalculation {
        name: String,
        multiplier: f64,
    }

    impl ExtensionPoint for TestCalculation {
        fn name(&self) -> &str { &self.name }
        fn extension_type(&self) -> ExtensionType { ExtensionType::Calculation }
    }

    impl CalculationExtension for TestCalculation {
        fn calculate(&self, input: f64, _context: &ExtensionContext) -> ExtensionResult<f64> {
            Ok(input * self.multiplier)
        }
    }

    #[test]
    fn test_extension_registry_new() {
        let registry = ExtensionRegistry::new();
        assert!(registry.validations.is_empty());
        assert!(registry.calculations.is_empty());
    }

    #[test]
    fn test_register_and_list_validation() {
        let mut registry = ExtensionRegistry::new();
        let ext = Arc::new(TestValidation { name: "test".to_string(), should_pass: true });
        registry.register_validation(ext);
        let list = registry.list_by_type(&ExtensionType::Validation);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "test");
    }

    #[test]
    fn test_calculation_chain() {
        let mut registry = ExtensionRegistry::new();
        registry.register_calculation(Arc::new(TestCalculation { name: "double".to_string(), multiplier: 2.0 }));
        registry.register_calculation(Arc::new(TestCalculation { name: "triple".to_string(), multiplier: 3.0 }));
        
        let context = ExtensionContext::new();
        let result = registry.execute_calculation_chain(5.0, &context).unwrap();
        assert_eq!(result, 30.0); // 5 * 2 * 3 = 30
    }

    #[test]
    fn test_validation_chain_with_errors() {
        let mut registry = ExtensionRegistry::new();
        registry.register_validation(Arc::new(TestValidation { name: "pass".to_string(), should_pass: true }));
        registry.register_validation(Arc::new(TestValidation { name: "fail".to_string(), should_pass: false }));
        
        let context = ExtensionContext::new();
        let result = registry.execute_validation_chain(&context).unwrap();
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_extension_context_builder() {
        let ctx = ExtensionContext::new()
            .with_feature("test-feature")
            .with_cell("test-cell")
            .with_meta("key", "value");
        
        assert_eq!(ctx.feature_name.as_deref(), Some("test-feature"));
        assert_eq!(ctx.cell_name.as_deref(), Some("test-cell"));
        assert_eq!(ctx.metadata.get("key").unwrap(), "value");
    }

    #[test]
    fn test_extension_types() {
        assert_eq!(ExtensionType::Validation.description(), "验证扩展点 - 用于自定义验证规则");
        assert_eq!(ExtensionType::Calculation.description(), "计算扩展点 - 用于自定义计算逻辑");
        assert_eq!(ExtensionType::Notification.description(), "通知扩展点 - 用于自定义通知渠道");
        assert_eq!(ExtensionType::Export.description(), "导出扩展点 - 用于自定义导出格式");
        assert_eq!(ExtensionType::Transformation.description(), "转换扩展点 - 用于自定义数据转换");
    }
}
