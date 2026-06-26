use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionRecord {
    pub id: Uuid,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub status: DecisionStatus,
    pub category: DecisionCategory,
    pub alternatives: Vec<Alternative>,
    pub consequences: Vec<Consequence>,
    pub rationale: String,
    pub related_tasks: Vec<String>,
    pub related_files: Vec<String>,
    pub tags: Vec<String>,
    pub made_by: Option<String>,
    pub made_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub superseded_by: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DecisionCategory {
    Architecture,
    Technology,
    Process,
    Tooling,
    Design,
    Testing,
    Deployment,
    Security,
    Performance,
    Other,
}

impl DecisionCategory {
    pub fn label(&self) -> &str {
        match self {
            DecisionCategory::Architecture => "架构决策",
            DecisionCategory::Technology => "技术选型",
            DecisionCategory::Process => "流程决策",
            DecisionCategory::Tooling => "工具决策",
            DecisionCategory::Design => "设计决策",
            DecisionCategory::Testing => "测试决策",
            DecisionCategory::Deployment => "部署决策",
            DecisionCategory::Security => "安全决策",
            DecisionCategory::Performance => "性能决策",
            DecisionCategory::Other => "其他决策",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "arch" | "architecture" => Some(DecisionCategory::Architecture),
            "tech" | "technology" => Some(DecisionCategory::Technology),
            "process" => Some(DecisionCategory::Process),
            "tool" | "tooling" => Some(DecisionCategory::Tooling),
            "design" => Some(DecisionCategory::Design),
            "test" | "testing" => Some(DecisionCategory::Testing),
            "deploy" | "deployment" => Some(DecisionCategory::Deployment),
            "security" | "sec" => Some(DecisionCategory::Security),
            "perf" | "performance" => Some(DecisionCategory::Performance),
            "other" => Some(DecisionCategory::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DecisionStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

impl DecisionStatus {
    pub fn label(&self) -> &str {
        match self {
            DecisionStatus::Proposed => "提议中",
            DecisionStatus::Accepted => "已采纳",
            DecisionStatus::Rejected => "已拒绝",
            DecisionStatus::Deprecated => "已废弃",
            DecisionStatus::Superseded => "已替代",
        }
    }
}

impl std::fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionStatus::Proposed => write!(f, "Proposed"),
            DecisionStatus::Accepted => write!(f, "Accepted"),
            DecisionStatus::Rejected => write!(f, "Rejected"),
            DecisionStatus::Deprecated => write!(f, "Deprecated"),
            DecisionStatus::Superseded => write!(f, "Superseded"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alternative {
    pub name: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Consequence {
    pub description: String,
    pub impact: ImpactLevel,
    pub certainty: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ImpactLevel {
    PositiveHigh,
    PositiveMedium,
    PositiveLow,
    Neutral,
    NegativeLow,
    NegativeMedium,
    NegativeHigh,
}

impl ImpactLevel {
    pub fn label(&self) -> &str {
        match self {
            ImpactLevel::PositiveHigh => "强正面",
            ImpactLevel::PositiveMedium => "中正面",
            ImpactLevel::PositiveLow => "弱正面",
            ImpactLevel::Neutral => "中性",
            ImpactLevel::NegativeLow => "弱负面",
            ImpactLevel::NegativeMedium => "中负面",
            ImpactLevel::NegativeHigh => "强负面",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "++" | "positive-high" | "strong-positive" => Some(ImpactLevel::PositiveHigh),
            "+" | "positive-medium" | "medium-positive" => Some(ImpactLevel::PositiveMedium),
            "+-" | "positive-low" | "weak-positive" => Some(ImpactLevel::PositiveLow),
            "0" | "neutral" => Some(ImpactLevel::Neutral),
            "-+" | "negative-low" | "weak-negative" => Some(ImpactLevel::NegativeLow),
            "-" | "negative-medium" | "medium-negative" => Some(ImpactLevel::NegativeMedium),
            "--" | "negative-high" | "strong-negative" => Some(ImpactLevel::NegativeHigh),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecisionMetrics {
    pub total_decisions: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub superseded_count: usize,
    pub by_category: HashMap<DecisionCategory, usize>,
    pub last_7_days: usize,
    pub last_30_days: usize,
}

impl DecisionRecord {
    pub fn new(
        title: &str,
        context: &str,
        decision: &str,
        rationale: &str,
        category: DecisionCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            context: context.to_string(),
            decision: decision.to_string(),
            status: DecisionStatus::Accepted,
            category,
            alternatives: Vec::new(),
            consequences: Vec::new(),
            rationale: rationale.to_string(),
            related_tasks: Vec::new(),
            related_files: Vec::new(),
            tags: Vec::new(),
            made_by: None,
            made_at: now,
            updated_at: now,
            revoked_at: None,
            superseded_by: None,
            metadata: HashMap::new(),
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("# ADR-{}: {}", simple_id(&self.id), self.title));
        lines.push(String::new());
        lines.push(format!("**状态**: {}", self.status.label()));
        lines.push(format!("**分类**: {}", self.category.label()));
        lines.push(format!("**日期**: {}", self.made_at.format("%Y-%m-%d %H:%M:%S")));
        if let Some(by) = &self.made_by {
            lines.push(format!("**决策者**: {}", by));
        }
        lines.push(String::new());

        lines.push("## 背景".to_string());
        lines.push(self.context.clone());
        lines.push(String::new());

        lines.push("## 决策".to_string());
        lines.push(self.decision.clone());
        lines.push(String::new());

        lines.push("## 决策理由".to_string());
        lines.push(self.rationale.clone());
        lines.push(String::new());

        if !self.alternatives.is_empty() {
            lines.push("## 备选方案".to_string());
            for (i, alt) in self.alternatives.iter().enumerate() {
                lines.push(format!("### 方案 {}: {}", i + 1, alt.name));
                lines.push(alt.description.clone());
                if !alt.pros.is_empty() {
                    lines.push("**优点**:".to_string());
                    for pro in &alt.pros {
                        lines.push(format!("- {}", pro));
                    }
                }
                if !alt.cons.is_empty() {
                    lines.push("**缺点**:".to_string());
                    for con in &alt.cons {
                        lines.push(format!("- {}", con));
                    }
                }
                lines.push(String::new());
            }
        }

        if !self.consequences.is_empty() {
            lines.push("## 影响与后果".to_string());
            for c in &self.consequences {
                lines.push(format!("- [{}] {}", c.impact.label(), c.description));
            }
            lines.push(String::new());
        }

        if !self.tags.is_empty() {
            lines.push("## 标签".to_string());
            lines.push(self.tags.join(", "));
            lines.push(String::new());
        }

        if !self.related_files.is_empty() {
            lines.push("## 相关文件".to_string());
            for f in &self.related_files {
                lines.push(format!("- {}", f));
            }
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

pub fn simple_id(uuid: &Uuid) -> String {
    uuid.to_string()
        .split('-')
        .next()
        .unwrap_or("")
        .to_string()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_decision() {
        let d = DecisionRecord::new(
            "选择 Rust 作为开发语言",
            "项目需要高性能和内存安全",
            "使用 Rust 2024 开发",
            "Rust 提供内存安全且性能接近 C++",
            DecisionCategory::Technology,
        );
        assert_eq!(d.title, "选择 Rust 作为开发语言");
        assert_eq!(d.status, DecisionStatus::Accepted);
        assert_eq!(d.category, DecisionCategory::Technology);
    }

    #[test]
    fn test_decision_category_from_str() {
        assert_eq!(
            DecisionCategory::from_str("arch"),
            Some(DecisionCategory::Architecture)
        );
        assert_eq!(
            DecisionCategory::from_str("technology"),
            Some(DecisionCategory::Technology)
        );
        assert_eq!(DecisionCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_impact_level_from_str() {
        assert_eq!(
            ImpactLevel::from_str("++"),
            Some(ImpactLevel::PositiveHigh)
        );
        assert_eq!(ImpactLevel::from_str("0"), Some(ImpactLevel::Neutral));
        assert_eq!(
            ImpactLevel::from_str("--"),
            Some(ImpactLevel::NegativeHigh)
        );
    }

    #[test]
    fn test_decision_to_markdown() {
        let d = DecisionRecord::new(
            "测试决策",
            "测试背景",
            "测试决策内容",
            "测试理由",
            DecisionCategory::Architecture,
        );
        let md = d.to_markdown();
        assert!(md.contains("测试决策"));
        assert!(md.contains("背景"));
        assert!(md.contains("决策"));
        assert!(md.contains("决策理由"));
    }
}
