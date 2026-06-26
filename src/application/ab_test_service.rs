use crate::domain::ab_experiment::{ABTestManager, Experiment, ExperimentType};
use crate::domain::errors::{CellError, CellResult};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub experiment_type: String,
    pub variant_count: usize,
    pub total_users: u64,
    pub traffic_percentage: f64,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>,
    pub started_at: Option<DateTime<chrono::Utc>>,
    pub ended_at: Option<DateTime<chrono::Utc>>,
}

impl From<&Experiment> for ExperimentSummary {
    fn from(exp: &Experiment) -> Self {
        let total_users: u64 = exp.metrics.values().map(|m| m.total_users).sum();
        Self {
            id: exp.id.to_string(),
            name: exp.name.clone(),
            description: exp.description.clone(),
            status: exp.status.label().to_string(),
            experiment_type: exp.experiment_type.label().to_string(),
            variant_count: exp.variants.len(),
            total_users,
            traffic_percentage: exp.traffic_percentage,
            created_at: exp.created_at,
            updated_at: exp.updated_at,
            started_at: exp.started_at,
            ended_at: exp.ended_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantResult {
    pub id: String,
    pub name: String,
    pub is_control: bool,
    pub weight: u32,
    pub total_users: u64,
    pub conversions: u64,
    pub conversion_rate: f64,
    pub revenue: f64,
    pub revenue_per_user: f64,
    pub click_count: u64,
    pub impression_count: u64,
    pub click_through_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub summary: ExperimentSummary,
    pub variants: Vec<VariantResult>,
}

pub struct ABTestService {
    manager: ABTestManager,
    name_to_id: HashMap<String, Uuid>,
}

impl ABTestService {
    pub fn new() -> Self {
        Self {
            manager: ABTestManager::new(),
            name_to_id: HashMap::new(),
        }
    }

    pub fn create_experiment(
        &mut self,
        name: impl Into<String>,
        experiment_type: ExperimentType,
        variants: Vec<(String, u32)>,
    ) -> CellResult<ExperimentSummary> {
        let name = name.into();
        if self.name_to_id.contains_key(&name) {
            return Err(CellError::Validation(format!(
                "Experiment '{}' already exists",
                name
            )));
        }
        if variants.len() < 2 {
            return Err(CellError::Validation("At least 2 variants required".to_string()));
        }
        let control_name = variants.first().unwrap().0.clone();
        let experiment = self
            .manager
            .create_experiment(
                &name,
                "Created via CLI",
                experiment_type,
                100.0,
                &control_name,
                variants,
            )
            .map_err(|e| CellError::Validation(e.to_string()))?;
        self.name_to_id.insert(name, experiment.id);
        Ok(experiment.into())
    }

    pub fn end_experiment(&mut self, name: impl AsRef<str>) -> CellResult<ExperimentSummary> {
        let id = self.name_to_id.get(name.as_ref())
            .ok_or_else(|| CellError::NotFound(format!("Experiment '{}' not found", name.as_ref())))?;
        let experiment = self.manager.end_experiment(*id)
            .map_err(|e| CellError::Validation(e.to_string()))?;
        Ok(experiment.into())
    }

    pub fn list_experiments(&self) -> Vec<ExperimentSummary> {
        self.manager
            .list_experiments()
            .iter()
            .map(|e| ExperimentSummary::from(*e))
            .collect()
    }

    pub fn start_experiment(&mut self, name: impl AsRef<str>) -> CellResult<ExperimentSummary> {
        let id = self.name_to_id.get(name.as_ref())
            .ok_or_else(|| CellError::NotFound(format!("Experiment '{}' not found", name.as_ref())))?;
        let experiment = self.manager.start_experiment(*id)
            .map_err(|e| CellError::Validation(e.to_string()))?;
        Ok(experiment.into())
    }

    pub fn pause_experiment(&mut self, name: impl AsRef<str>) -> CellResult<ExperimentSummary> {
        let id = self.name_to_id.get(name.as_ref())
            .ok_or_else(|| CellError::NotFound(format!("Experiment '{}' not found", name.as_ref())))?;
        let experiment = self.manager.pause_experiment(*id)
            .map_err(|e| CellError::Validation(e.to_string()))?;
        Ok(experiment.into())
    }

    pub fn get_experiment_result(&self, name: impl AsRef<str>) -> CellResult<ExperimentResult> {
        let id = self.name_to_id.get(name.as_ref())
            .ok_or_else(|| CellError::NotFound(format!("Experiment '{}' not found", name.as_ref())))?;
        let experiment = self.manager.get_experiment(*id)
            .ok_or_else(|| CellError::NotFound(format!("Experiment '{}' not found", name.as_ref())))?;
        let summary: ExperimentSummary = experiment.into();
        let experiment = self.manager.get_experiment(*id).unwrap();
        let variants: Vec<VariantResult> = experiment.variants.iter()
            .map(|v| {
                let metrics = experiment.metrics.get(&v.id).cloned().unwrap_or_default();
                VariantResult {
                    id: v.id.to_string(),
                    name: v.name.clone(),
                    is_control: v.is_control,
                    weight: v.weight,
                    total_users: metrics.total_users,
                    conversions: metrics.conversions,
                    conversion_rate: metrics.conversion_rate(),
                    revenue: metrics.revenue,
                    revenue_per_user: metrics.revenue_per_user(),
                    click_count: metrics.click_count,
                    impression_count: metrics.impression_count,
                    click_through_rate: metrics.click_through_rate(),
                }
            })
            .collect();
        Ok(ExperimentResult { summary, variants })
    }

    pub fn assign_variant(&mut self, user_id: impl AsRef<str>, experiment_id: impl AsRef<str>) -> CellResult<String> {
        let experiment_uuid = Uuid::parse_str(experiment_id.as_ref())
            .map_err(|e| CellError::Validation(format!("Invalid experiment ID: {}", e)))?;
        let variant = self.manager.get_variant(experiment_uuid, user_id.as_ref())
            .map_err(|e| CellError::Validation(e.to_string()))?;
        match variant {
            Some(v) => Ok(v.name.clone()),
            None => Err(CellError::Validation("Experiment is not running".to_string())),
        }
    }

    pub fn format_list(&self, experiments: &[ExperimentSummary]) -> String {
        if experiments.is_empty() {
            return "No A/B experiments found.".to_string();
        }
        let mut output = String::new();
        output.push_str("\n📋 A/B Experiments:\n");
        output.push_str(&format!("{}", "─".repeat(70)));
        output.push_str("\n   NAME               STATUS          TYPE          VARIANTS   USERS    TRAFFIC\n");
        output.push_str(&format!("{}", "─".repeat(70)));
        for exp in experiments {
            let status_icon = match exp.status.as_str() {
                "Running" => "🟢",
                "Paused" => "🟡",
                "Draft" => "⚪",
                "Completed" => "🔵",
                "Archived" => "⚫",
                _ => "⚪",
            };
            output.push_str(&format!(
                "\n   {:<18} {} {:<12} {:<12} {:<9} {:<7} {:>6}%\n",
                exp.name, status_icon, exp.status, exp.experiment_type, exp.variant_count, exp.total_users, exp.traffic_percentage
            ));
        }
        output.push_str(&format!("\n{}", "─".repeat(70)));
        output.push_str(&format!("\n   Total: {} experiments\n", experiments.len()));
        output
    }

    pub fn format_result(&self, result: &ExperimentResult) -> String {
        let mut output = String::new();
        output.push_str(&format!("\n📊 {} ({})\n", result.summary.name, result.summary.id));
        output.push_str(&format!("{}", "─".repeat(50)));
        output.push_str(&format!("\n   Status:         {}\n", result.summary.status));
        output.push_str(&format!("   Type:           {}\n", result.summary.experiment_type));
        output.push_str(&format!("   Total Users:    {}\n", result.summary.total_users));
        output.push_str(&format!("   Traffic:        {}%\n", result.summary.traffic_percentage));
        output.push_str(&format!("   Created:        {}\n", result.summary.created_at.format("%Y-%m-%d %H:%M:%S")));
        if let Some(started) = result.summary.started_at {
            output.push_str(&format!("   Started:        {}\n", started.format("%Y-%m-%d %H:%M:%S")));
        }
        output.push_str("\n   Variants:\n");
        output.push_str(&format!("{}", "─".repeat(50)));
        output.push_str("\n      NAME          TYPE     USERS   CONV_RATE\n");
        output.push_str(&format!("{}", "─".repeat(50)));
        for variant in &result.variants {
            let v_type = if variant.is_control { "Control" } else { "Variant" };
            let conv_rate = format!("{:.2}%", variant.conversion_rate * 100.0);
            output.push_str(&format!(
                "\n      {:<14} {:<8} {:<6} {:>10}\n",
                variant.name, v_type, variant.total_users, conv_rate
            ));
        }
        output
    }
}

impl Default for ABTestService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_experiment() {
        let mut service = ABTestService::new();
        let variants = vec![("control".to_string(), 50), ("variant-a".to_string(), 50)];
        let result = service.create_experiment("test-exp", ExperimentType::Feature, variants);
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.name, "test-exp");
        assert_eq!(summary.status, "Draft");
        assert_eq!(summary.variant_count, 2);
    }

    #[test]
    fn test_create_experiment_duplicate() {
        let mut service = ABTestService::new();
        let variants = vec![("control".to_string(), 50), ("v1".to_string(), 50)];
        service.create_experiment("test-exp", ExperimentType::Feature, variants.clone()).unwrap();
        let result = service.create_experiment("test-exp", ExperimentType::Feature, variants);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_experiments() {
        let mut service = ABTestService::new();
        service.create_experiment("exp1", ExperimentType::Feature, vec![("c".to_string(), 50), ("v1".to_string(), 50)]).unwrap();
        service.create_experiment("exp2", ExperimentType::Feature, vec![("c".to_string(), 30), ("v2".to_string(), 70)]).unwrap();
        let exps = service.list_experiments();
        assert_eq!(exps.len(), 2);
    }

    #[test]
    fn test_start_pause_experiment() {
        let mut service = ABTestService::new();
        service.create_experiment("test-exp", ExperimentType::Feature, vec![("c".to_string(), 50), ("v1".to_string(), 50)]).unwrap();
        let started = service.start_experiment("test-exp");
        assert!(started.is_ok());
        assert_eq!(started.unwrap().status, "Running");
        let paused = service.pause_experiment("test-exp");
        assert!(paused.is_ok());
        assert_eq!(paused.unwrap().status, "Paused");
    }

    #[test]
    fn test_get_experiment_result() {
        let mut service = ABTestService::new();
        service.create_experiment("test-exp", ExperimentType::Feature, vec![("c".to_string(), 50), ("v1".to_string(), 50)]).unwrap();
        service.start_experiment("test-exp").unwrap();
        let result = service.get_experiment_result("test-exp");
        assert!(result.is_ok());
        let exp_result = result.unwrap();
        assert_eq!(exp_result.summary.name, "test-exp");
        assert_eq!(exp_result.variants.len(), 2);
    }

    #[test]
    fn test_end_experiment() {
        let mut service = ABTestService::new();
        service.create_experiment("test-exp", ExperimentType::Feature, vec![("c".to_string(), 50), ("v1".to_string(), 50)]).unwrap();
        service.start_experiment("test-exp").unwrap();
        let ended = service.end_experiment("test-exp");
        assert!(ended.is_ok());
        assert_eq!(ended.unwrap().status, "Completed");
    }
}
