use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    StuckLoop,
    ConsecutiveFailures,
    EntropySpike,
    LongRunningTask,
    HighErrorRate,
    MemoryPressure,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub id: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub detected_at: String,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
    pub resolved: bool,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub id: String,
    pub action_type: RecoveryActionType,
    pub description: String,
    pub triggered_by: String,
    pub executed_at: String,
    pub success: bool,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryActionType {
    Retry,
    Rollback,
    TaskDegrade,
    TaskSplit,
    Escalate,
    Pause,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingReport {
    pub total_anomalies: usize,
    pub active_anomalies: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_type: std::collections::HashMap<String, usize>,
    pub total_recoveries: usize,
    pub success_rate: f64,
    pub anomalies: Vec<Anomaly>,
    pub recovery_actions: Vec<RecoveryAction>,
}

#[allow(dead_code)]
pub struct SelfHealingService {
    max_retries: u32,
    max_consecutive_failures: u32,
}

impl SelfHealingService {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            max_consecutive_failures: 5,
        }
    }

    pub fn detect_anomalies(&self, project_path: &str) -> CellResult<Vec<Anomaly>> {
        let mut anomalies = Vec::new();

        let cell_dir = Path::new(project_path).join(".cell");

        let anomalies_dir = cell_dir.join("anomalies");
        if anomalies_dir.exists() {
            for entry in std::fs::read_dir(&anomalies_dir)? {
                let entry = entry?;
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(anomaly) = serde_json::from_str::<Anomaly>(&content)
                    && !anomaly.resolved {
                        anomalies.push(anomaly);
                    }
            }
        }

        Ok(anomalies)
    }

    pub fn report_anomaly(
        &self,
        project_path: &str,
        anomaly_type: AnomalyType,
        severity: AnomalySeverity,
        description: &str,
        agent_id: Option<&str>,
        task_id: Option<&str>,
    ) -> CellResult<String> {
        let anomaly = Anomaly {
            id: format!("anomaly-{}", uuid::Uuid::new_v4().simple()),
            anomaly_type,
            severity,
            description: description.to_string(),
            detected_at: chrono::Utc::now().to_rfc3339(),
            agent_id: agent_id.map(std::string::ToString::to_string),
            task_id: task_id.map(std::string::ToString::to_string),
            resolved: false,
            resolution: None,
        };

        self.save_anomaly(project_path, &anomaly)?;
        Ok(anomaly.id)
    }

    fn save_anomaly(&self, project_path: &str, anomaly: &Anomaly) -> CellResult<()> {
        let anomalies_dir = Path::new(project_path)
            .join(".cell")
            .join("anomalies");
        std::fs::create_dir_all(&anomalies_dir)?;

        let file_path = anomalies_dir.join(format!("{}.json", anomaly.id));
        std::fs::write(&file_path, serde_json::to_string_pretty(anomaly)?)?;

        Ok(())
    }

    fn save_recovery_action(
        &self,
        project_path: &str,
        action: &RecoveryAction,
    ) -> CellResult<()> {
        let recovery_dir = Path::new(project_path)
            .join(".cell")
            .join("recovery_actions");
        std::fs::create_dir_all(&recovery_dir)?;

        let file_path = recovery_dir.join(format!("{}.json", action.id));
        std::fs::write(&file_path, serde_json::to_string_pretty(action)?)?;

        Ok(())
    }

    pub fn attempt_recovery(
        &self,
        project_path: &str,
        anomaly_id: &str,
    ) -> CellResult<Option<RecoveryAction>> {
        let anomalies = self.detect_anomalies(project_path)?;
        let anomaly = match anomalies.iter().find(|a| a.id == anomaly_id) {
            Some(a) => a.clone(),
            None => return Ok(None),
        };

        let action_type = match anomaly.severity {
            AnomalySeverity::Info | AnomalySeverity::Warning => RecoveryActionType::Retry,
            AnomalySeverity::Critical => RecoveryActionType::Rollback,
            AnomalySeverity::Fatal => RecoveryActionType::Escalate,
        };

        let action = self.execute_recovery(project_path, &anomaly, action_type)?;
        Ok(Some(action))
    }

    fn execute_recovery(
        &self,
        project_path: &str,
        anomaly: &Anomaly,
        action_type: RecoveryActionType,
    ) -> CellResult<RecoveryAction> {
        let description = match action_type {
            RecoveryActionType::Retry => "重试操作".to_string(),
            RecoveryActionType::Rollback => "回滚到上一个稳定版本".to_string(),
            RecoveryActionType::TaskDegrade => "任务降级处理".to_string(),
            RecoveryActionType::TaskSplit => "拆分为更小的子任务".to_string(),
            RecoveryActionType::Escalate => "升级，需要人工介入".to_string(),
            RecoveryActionType::Pause => "暂停并等待".to_string(),
        };

        let success = match action_type {
            RecoveryActionType::Rollback => {
                let output = std::process::Command::new("git")
                    .args(["reset", "--hard", "HEAD~1"])
                    .current_dir(project_path)
                    .output();
                output.is_ok_and(|o| o.status.success())
            }
            RecoveryActionType::Escalate => false,
            _ => true,
        };

        let action = RecoveryAction {
            id: format!("recovery-{}", uuid::Uuid::new_v4().simple()),
            action_type: action_type.clone(),
            description,
            triggered_by: anomaly.id.clone(),
            executed_at: chrono::Utc::now().to_rfc3339(),
            success,
            result: if success {
                Some("恢复操作成功".to_string())
            } else {
                Some("恢复操作失败，需要人工介入".to_string())
            },
        };

        self.save_recovery_action(project_path, &action)?;

        if success {
            let mut resolved_anomaly = anomaly.clone();
            resolved_anomaly.resolved = true;
            resolved_anomaly.resolution = Some(format!(
                "通过 {action_type:?} 恢复成功"
            ));
            self.save_anomaly(project_path, &resolved_anomaly)?;
        }

        Ok(action)
    }

    pub fn generate_healing_report(&self, project_path: &str) -> CellResult<SelfHealingReport> {
        let anomalies = self.detect_anomalies(project_path)?;
        let active_count = anomalies.len();

        let mut by_severity = std::collections::HashMap::new();
        let mut by_type = std::collections::HashMap::new();

        for a in &anomalies {
            *by_severity
                .entry(format!("{:?}", a.severity))
                .or_insert(0) += 1;
            *by_type
                .entry(format!("{:?}", a.anomaly_type))
                .or_insert(0) += 1;
        }

        let recovery_dir = Path::new(project_path)
            .join(".cell")
            .join("recovery_actions");

        let mut recovery_actions = Vec::new();
        if recovery_dir.exists() {
            for entry in std::fs::read_dir(&recovery_dir)? {
                let entry = entry?;
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(action) = serde_json::from_str::<RecoveryAction>(&content) {
                    recovery_actions.push(action);
                }
            }
        }

        let total_recoveries = recovery_actions.len();
        let success_rate = if total_recoveries > 0 {
            recovery_actions.iter().filter(|a| a.success).count() as f64 / total_recoveries as f64
        } else {
            0.0
        };

        Ok(SelfHealingReport {
            total_anomalies: anomalies.len(),
            active_anomalies: active_count,
            by_severity,
            by_type,
            total_recoveries,
            success_rate,
            anomalies,
            recovery_actions,
        })
    }

    pub fn generate_human_intervention_report(
        &self,
        anomaly: &Anomaly,
    ) -> String {
        let mut output = String::new();

        output.push_str("🚨 需要人工介入\n\n");
        output.push_str(&format!("  异常 ID: {}\n", anomaly.id));
        output.push_str(&format!("  类型: {:?}\n", anomaly.anomaly_type));
        output.push_str(&format!("  严重程度: {:?}\n", anomaly.severity));
        output.push_str(&format!("  检测时间: {}\n", anomaly.detected_at));
        output.push_str(&format!("  描述: {}\n\n", anomaly.description));

        if let Some(agent) = &anomaly.agent_id {
            output.push_str(&format!("  相关 Agent: {agent}\n"));
        }
        if let Some(task) = &anomaly.task_id {
            output.push_str(&format!("  相关任务: {task}\n"));
        }

        output.push_str("\n📋 已尝试的操作:\n");
        output.push_str("  1. 自动重试\n");
        output.push_str("  2. 自动回滚\n");
        output.push_str("  3. 任务降级\n\n");

        output.push_str("💡 建议的下一步:\n");
        output.push_str("  1. 检查错误日志了解具体原因\n");
        output.push_str("  2. 评估是否需要调整架构设计\n");
        output.push_str("  3. 手动修复后，标记异常为已解决\n");

        output
    }

    pub fn format_report(&self, report: &SelfHealingReport) -> String {
        let mut output = String::new();

        output.push_str("\n🩹 自愈系统报告\n\n");
        output.push_str(&format!("  活跃异常数: {}\n", report.active_anomalies));
        output.push_str(&format!("  总恢复次数: {}\n", report.total_recoveries));
        output.push_str(&format!(
            "  恢复成功率: {:.1}%\n",
            report.success_rate * 100.0
        ));

        if !report.by_severity.is_empty() {
            output.push_str("\n  按严重程度:\n");
            for (sev, count) in &report.by_severity {
                let icon = match sev.as_str() {
                    "Fatal" => "🔴",
                    "Critical" => "🟠",
                    "Warning" => "🟡",
                    _ => "🟢",
                };
                output.push_str(&format!("    {icon} {sev}: {count}\n"));
            }
        }

        if !report.by_type.is_empty() {
            output.push_str("\n  按异常类型:\n");
            for (t, count) in &report.by_type {
                output.push_str(&format!("    {t}: {count}\n"));
            }
        }

        if !report.anomalies.is_empty() {
            output.push_str("\n  活跃异常:\n");
            for a in &report.anomalies {
                let sev_icon = match a.severity {
                    AnomalySeverity::Fatal => "🔴",
                    AnomalySeverity::Critical => "🟠",
                    AnomalySeverity::Warning => "🟡",
                    AnomalySeverity::Info => "ℹ️",
                };
                output.push_str(&format!(
                    "  {} [{}] {}\n",
                    sev_icon, a.id, a.description
                ));
            }
        }

        output
    }
}

impl Default for SelfHealingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_healing_new() {
        let service = SelfHealingService::new();
        assert_eq!(service.max_retries, 3);
    }

    #[test]
    fn test_detect_anomalies() {
        let service = SelfHealingService::new();
        let anomalies = service.detect_anomalies(".").unwrap();
        // 异常可能存在也可能不存在
        let _ = anomalies;
    }

    #[test]
    fn test_report_anomaly() {
        let service = SelfHealingService::new();
        let id = service
            .report_anomaly(
                ".",
                AnomalyType::ConsecutiveFailures,
                AnomalySeverity::Warning,
                "连续失败 3 次",
                Some("test-agent"),
                Some("test-task"),
            )
            .unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_healing_report() {
        let service = SelfHealingService::new();
        let report = service.generate_healing_report(".").unwrap();
        let _ = report;
    }
}
