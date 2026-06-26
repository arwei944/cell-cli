use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDecision {
    pub id: String,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub rationale: String,
    pub alternatives: Vec<String>,
    pub severity: DecisionSeverity,
    pub confidence: DecisionConfidence,
    pub needs_human_review: bool,
    pub created_at: String,
    pub agent_id: String,
    pub related_adr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRule {
    pub id: String,
    pub name: String,
    pub condition: String,
    pub action: String,
    pub severity: DecisionSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEngineReport {
    pub total_decisions: usize,
    pub by_confidence: std::collections::HashMap<String, usize>,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub pending_review: usize,
    pub decisions: Vec<AutoDecision>,
}

pub struct DecisionEngineService {
    rules: Vec<DecisionRule>,
}

impl DecisionEngineService {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    fn default_rules() -> Vec<DecisionRule> {
        vec![
            DecisionRule {
                id: "rule-naming-convention".to_string(),
                name: "命名规范".to_string(),
                condition: "命名违规".to_string(),
                action: "自动修复命名，遵循项目约定".to_string(),
                severity: DecisionSeverity::Low,
                enabled: true,
            },
            DecisionRule {
                id: "rule-layer-dependency".to_string(),
                name: "分层依赖规则".to_string(),
                condition: "层依赖违规".to_string(),
                action: "自动调整 import 路径，遵循四层架构".to_string(),
                severity: DecisionSeverity::High,
                enabled: true,
            },
            DecisionRule {
                id: "rule-circular-dep".to_string(),
                name: "循环依赖".to_string(),
                condition: "循环依赖检测".to_string(),
                action: "标记为高优先级，需要人工审查".to_string(),
                severity: DecisionSeverity::Critical,
                enabled: true,
            },
            DecisionRule {
                id: "rule-new-module".to_string(),
                name: "新模块放置位置".to_string(),
                condition: "创建新模块".to_string(),
                action: "按四层架构放置到正确的层（Domain/Application/Adapters/Interfaces）".to_string(),
                severity: DecisionSeverity::Medium,
                enabled: true,
            },
            DecisionRule {
                id: "rule-test-coverage".to_string(),
                name: "测试覆盖率".to_string(),
                condition: "测试覆盖率下降".to_string(),
                action: "自动补充单元测试，维持覆盖率".to_string(),
                severity: DecisionSeverity::Medium,
                enabled: true,
            },
        ]
    }

    pub fn make_decision(
        &self,
        project_path: &str,
        title: &str,
        context: &str,
        agent_id: &str,
    ) -> CellResult<AutoDecision> {
        let (decision, rationale, confidence) = self.analyze_and_decide(title, context);

        let needs_review = matches!(confidence, DecisionConfidence::Low)
            || matches!(rationale.as_str(), "复杂架构决策，需要人工确认");

        let decision_obj = AutoDecision {
            id: format!("dec-{}", uuid::Uuid::new_v4().simple()),
            title: title.to_string(),
            context: context.to_string(),
            decision,
            rationale,
            alternatives: vec![],
            severity: Self::assess_severity(title, context),
            confidence,
            needs_human_review: needs_review,
            created_at: chrono::Utc::now().to_rfc3339(),
            agent_id: agent_id.to_string(),
            related_adr: None,
        };

        self.save_decision(project_path, &decision_obj)?;
        Ok(decision_obj)
    }

    fn analyze_and_decide(&self, title: &str, context: &str) -> (String, String, DecisionConfidence) {
        let title_lower = title.to_lowercase();
        let context_lower = context.to_lowercase();

        if title_lower.contains("命名") || context_lower.contains("naming") || title_lower.contains("命名") {
            (
                "遵循项目命名约定：模块名小写蛇形，结构体大驼峰，trait 用 -able 后缀".to_string(),
                "基于历史 ADR 和代码风格一致性".to_string(),
                DecisionConfidence::High,
            )
        } else if title_lower.contains("架构") || context_lower.contains("architecture") {
            (
                "遵循四层架构（Domain → Application → Adapters → Interfaces），依赖方向由外向内".to_string(),
                "基于 Cell Architecture 核心原则".to_string(),
                DecisionConfidence::High,
            )
        } else if title_lower.contains("依赖") || context_lower.contains("dependency") {
            (
                "通过依赖倒置原则，使用 Port/Adapter 模式解耦".to_string(),
                "基于整洁架构和依赖注入原则".to_string(),
                DecisionConfidence::Medium,
            )
        } else if title_lower.contains("测试") || context_lower.contains("test") {
            (
                "为新增代码补充单元测试，维持 70%+ 覆盖率".to_string(),
                "基于质量门禁要求".to_string(),
                DecisionConfidence::High,
            )
        } else if title_lower.contains("数据库") || context_lower.contains("database") {
            (
                "使用 Repository 模式抽象数据访问，具体实现放在 Adapters 层".to_string(),
                "基于持久化无关原则".to_string(),
                DecisionConfidence::Medium,
            )
        } else if title_lower.contains("性能") || context_lower.contains("performance") {
            (
                "先优化热点路径，使用并行处理，必要时引入缓存".to_string(),
                "基于性能优化最佳实践".to_string(),
                DecisionConfidence::Medium,
            )
        } else if title_lower.contains("安全") || context_lower.contains("security") {
            (
                "标记为高优先级，需要人工审查确认".to_string(),
                "安全相关决策必须人工确认".to_string(),
                DecisionConfidence::Low,
            )
        } else {
            (
                "建议参考历史 ADR 和现有代码风格，与团队确认后实施".to_string(),
                "不确定领域，需要更多上下文".to_string(),
                DecisionConfidence::Low,
            )
        }
    }

    fn assess_severity(title: &str, context: &str) -> DecisionSeverity {
        let text = format!("{} {}", title, context).to_lowercase();

        if text.contains("安全") || text.contains("security") || text.contains("循环依赖") {
            DecisionSeverity::Critical
        } else if text.contains("架构") || text.contains("architecture") || text.contains("分层") {
            DecisionSeverity::High
        } else if text.contains("性能") || text.contains("数据库") || text.contains("依赖") {
            DecisionSeverity::Medium
        } else {
            DecisionSeverity::Low
        }
    }

    fn save_decision(&self, project_path: &str, decision: &AutoDecision) -> CellResult<()> {
        let decisions_dir = Path::new(project_path)
            .join(".cell")
            .join("auto_decisions");
        std::fs::create_dir_all(&decisions_dir)?;

        let file_path = decisions_dir.join(format!("{}.json", decision.id));
        std::fs::write(&file_path, serde_json::to_string_pretty(decision)?)?;

        Ok(())
    }

    pub fn list_decisions(
        &self,
        project_path: &str,
        needs_review_only: bool,
    ) -> CellResult<DecisionEngineReport> {
        let decisions_dir = Path::new(project_path)
            .join(".cell")
            .join("auto_decisions");

        let mut decisions = Vec::new();

        if decisions_dir.exists() {
            for entry in std::fs::read_dir(&decisions_dir)? {
                let entry = entry?;
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(decision) = serde_json::from_str::<AutoDecision>(&content) {
                    if !needs_review_only || decision.needs_human_review {
                        decisions.push(decision);
                    }
                }
            }
        }

        let mut by_confidence = std::collections::HashMap::new();
        let mut by_severity = std::collections::HashMap::new();

        for d in &decisions {
            *by_confidence
                .entry(format!("{:?}", d.confidence))
                .or_insert(0) += 1;
            *by_severity
                .entry(format!("{:?}", d.severity))
                .or_insert(0) += 1;
        }

        let pending_review = decisions.iter().filter(|d| d.needs_human_review).count();

        decisions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(DecisionEngineReport {
            total_decisions: decisions.len(),
            by_confidence,
            by_severity,
            pending_review,
            decisions,
        })
    }

    pub fn get_rules(&self) -> &[DecisionRule] {
        &self.rules
    }

    pub fn format_decision(&self, decision: &AutoDecision) -> String {
        let mut output = String::new();

        let conf_icon = match decision.confidence {
            DecisionConfidence::High => "✅",
            DecisionConfidence::Medium => "🤔",
            DecisionConfidence::Low => "⚠️",
        };

        let sev_icon = match decision.severity {
            DecisionSeverity::Low => "🟢",
            DecisionSeverity::Medium => "🟡",
            DecisionSeverity::High => "🟠",
            DecisionSeverity::Critical => "🔴",
        };

        output.push_str(&format!("\n{} 决策: {}\n\n", sev_icon, decision.title));
        output.push_str(&format!("  ID: {}\n", decision.id));
        output.push_str(&format!("  置信度: {} {:?}\n", conf_icon, decision.confidence));
        output.push_str(&format!("  严重程度: {:?}\n", decision.severity));
        output.push_str(&format!(
            "  需要人工审查: {}\n",
            if decision.needs_human_review { "是 ⚠️" } else { "否" }
        ));
        output.push_str(&format!("  决策: {}\n", decision.decision));
        output.push_str(&format!("  理由: {}\n", decision.rationale));
        output.push_str(&format!("  Agent: {}\n", decision.agent_id));
        output.push_str(&format!("  时间: {}\n", decision.created_at));

        output
    }

    pub fn format_report(&self, report: &DecisionEngineReport) -> String {
        let mut output = String::new();

        output.push_str("\n🤖 自主决策报告\n\n");
        output.push_str(&format!("  总决策数: {}\n", report.total_decisions));
        output.push_str(&format!("  待人工审查: {}\n", report.pending_review));

        output.push_str("\n  按置信度:\n");
        for (level, count) in &report.by_confidence {
            output.push_str(&format!("    {}: {}\n", level, count));
        }

        output.push_str("\n  按严重程度:\n");
        for (level, count) in &report.by_severity {
            output.push_str(&format!("    {}: {}\n", level, count));
        }

        if !report.decisions.is_empty() {
            output.push_str("\n  最近决策:\n\n");
            for (i, d) in report.decisions.iter().take(10).enumerate() {
                let review_flag = if d.needs_human_review { " ⚠️" } else { "" };
                output.push_str(&format!(
                    "  {}. [{:?}] {}{}\n",
                    i + 1,
                    d.confidence,
                    d.title,
                    review_flag
                ));
            }
        }

        output
    }
}

impl Default for DecisionEngineService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_engine_new() {
        let engine = DecisionEngineService::new();
        assert!(!engine.get_rules().is_empty());
    }

    #[test]
    fn test_make_decision_naming() {
        let engine = DecisionEngineService::new();
        let decision = engine
            .make_decision(".", "命名规范", "变量命名不统一", "test-agent")
            .unwrap();
        assert_eq!(decision.confidence, DecisionConfidence::High);
        assert!(!decision.decision.is_empty());
    }

    #[test]
    fn test_make_decision_security() {
        let engine = DecisionEngineService::new();
        let decision = engine
            .make_decision(".", "安全方案", "数据加密方式选择", "test-agent")
            .unwrap();
        assert!(decision.needs_human_review);
    }

    #[test]
    fn test_list_decisions() {
        let engine = DecisionEngineService::new();
        let report = engine.list_decisions(".", false).unwrap();
        // 决策可能存在也可能不存在，不做断言
        let _ = report;
    }
}
