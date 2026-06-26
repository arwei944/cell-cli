use crate::application::arch_service::{ArchitectureRules, ValidationResult, Severity};
use crate::application::entropy_service;
use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    pub id: String,
    pub category: AdviceCategory,
    pub severity: AdviceSeverity,
    pub title: String,
    pub description: String,
    pub suggestion: String,
    pub affected_files: Vec<String>,
    pub effort: EffortEstimate,
    pub impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdviceCategory {
    ArchitectureDrift,
    Complexity,
    Coupling,
    TestCoverage,
    CodeQuality,
    Documentation,
    BestPractice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdviceSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortEstimate {
    Minutes,
    Hours,
    Days,
    Weeks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Transformational,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct AdviceResult {
    pub advices: Vec<Advice>,
    pub summary: AdviceSummary,
}

#[derive(Debug, Clone)]
pub struct AdviceSummary {
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub total_issues: usize,
    pub recommended_actions: Vec<String>,
}

pub struct ArchitectureAdvisor;

impl ArchitectureAdvisor {
    pub fn new() -> Self {
        Self
    }

    pub fn advise(&self, project_path: &str) -> CellResult<AdviceResult> {
        let root = Path::new(project_path);
        let rules = ArchitectureRules::default();
        let validation = super::arch_service::validate_architecture(root, &rules);
        
        let entropy_result = entropy_service::run_entropy_check(project_path);
        
        let mut advices = self.generate_architecture_advices(&validation);
        
        if let Ok(entropy) = entropy_result {
            advices.extend(self.generate_entropy_advices(&entropy));
        }
        
        let summary = self.generate_summary(&advices);
        
        Ok(AdviceResult { advices, summary })
    }

    fn generate_architecture_advices(&self, validation: &ValidationResult) -> Vec<Advice> {
        let mut advices = Vec::new();

        for (layer_name, stats) in &validation.layer_stats {
            if stats.violations > 0 {
                advices.push(Advice {
                    id: format!("arch-{}", layer_name),
                    category: AdviceCategory::ArchitectureDrift,
                    severity: if stats.violations > 3 { AdviceSeverity::Critical } else { AdviceSeverity::High },
                    title: format!("{}层存在架构违规", layer_name),
                    description: format!("{}层有{}个依赖违规，违反了分层架构原则", layer_name, stats.violations),
                    suggestion: format!("检查{}层的import语句，确保只依赖内层（domain -> application -> adapters -> interfaces）", layer_name),
                    affected_files: Vec::new(),
                    effort: if stats.violations > 5 { EffortEstimate::Hours } else { EffortEstimate::Minutes },
                    impact: ImpactLevel::High,
                });
            }

            if stats.external_deps > stats.internal_deps * 2 {
                advices.push(Advice {
                    id: format!("coupling-{}", layer_name),
                    category: AdviceCategory::Coupling,
                    severity: AdviceSeverity::Medium,
                    title: format!("{}层外部依赖过多", layer_name),
                    description: format!("{}层外部依赖({})远多于内部依赖({})，可能导致过度耦合", layer_name, stats.external_deps, stats.internal_deps),
                    suggestion: "考虑将部分依赖抽象为Port接口，减少直接依赖".to_string(),
                    affected_files: Vec::new(),
                    effort: EffortEstimate::Hours,
                    impact: ImpactLevel::Medium,
                });
            }
        }

        for violation in &validation.violations {
            if violation.severity == Severity::Error {
                advices.push(Advice {
                    id: format!("violation-{}", violation.file),
                    category: AdviceCategory::ArchitectureDrift,
                    severity: AdviceSeverity::Critical,
                    title: format!("非法层间依赖: {} -> {}", violation.from_module, violation.to_module),
                    description: violation.message.clone(),
                    suggestion: format!("移除{}对{}的依赖，通过Port接口或领域事件进行通信", violation.from_module, violation.to_module),
                    affected_files: vec![violation.file.clone()],
                    effort: EffortEstimate::Minutes,
                    impact: ImpactLevel::High,
                });
            }
        }

        advices
    }

    fn generate_entropy_advices(&self, entropy: &crate::domain::entropy::EntropyReport) -> Vec<Advice> {
        let mut advices = Vec::new();

        if entropy.overall_score > 60.0 {
            advices.push(Advice {
                id: "entropy-high".to_string(),
                category: AdviceCategory::Complexity,
                severity: if entropy.overall_score > 80.0 { AdviceSeverity::Critical } else { AdviceSeverity::High },
                title: "架构熵值过高".to_string(),
                description: format!("当前熵值{}超过健康阈值，系统复杂度偏高", entropy.overall_score),
                suggestion: "优先重构高风险文件，简化复杂逻辑，提升测试覆盖率".to_string(),
                affected_files: entropy.high_risk_files.clone(),
                effort: EffortEstimate::Days,
                impact: ImpactLevel::Transformational,
            });
        }

        if entropy.dimensions.complexity > 70.0 {
            advices.push(Advice {
                id: "complexity-high".to_string(),
                category: AdviceCategory::Complexity,
                severity: AdviceSeverity::High,
                title: "代码复杂度偏高".to_string(),
                description: format!("复杂度维度得分{}，可能存在深层嵌套和复杂逻辑", entropy.dimensions.complexity),
                suggestion: "拆分大型函数，使用策略模式替代复杂条件分支，降低嵌套深度".to_string(),
                affected_files: entropy.breakdown.iter()
                    .filter(|f| f.complexity_score > 60.0)
                    .map(|f| f.path.clone())
                    .collect(),
                effort: EffortEstimate::Hours,
                impact: ImpactLevel::High,
            });
        }

        if entropy.dimensions.coupling > 60.0 {
            advices.push(Advice {
                id: "coupling-high".to_string(),
                category: AdviceCategory::Coupling,
                severity: AdviceSeverity::Medium,
                title: "模块耦合度偏高".to_string(),
                description: format!("耦合维度得分{}，模块间依赖关系较紧密", entropy.dimensions.coupling),
                suggestion: "识别循环依赖，引入接口抽象，使用依赖注入降低耦合".to_string(),
                affected_files: Vec::new(),
                effort: EffortEstimate::Hours,
                impact: ImpactLevel::Medium,
            });
        }

        if entropy.dimensions.test < 30.0 {
            advices.push(Advice {
                id: "test-low".to_string(),
                category: AdviceCategory::TestCoverage,
                severity: AdviceSeverity::High,
                title: "测试覆盖率偏低".to_string(),
                description: format!("测试维度得分{}，建议增加单元测试和集成测试", entropy.dimensions.test),
                suggestion: "为核心业务逻辑添加单元测试，使用TDD方式开发新功能".to_string(),
                affected_files: Vec::new(),
                effort: EffortEstimate::Days,
                impact: ImpactLevel::High,
            });
        }

        if !entropy.high_risk_files.is_empty() {
            advices.push(Advice {
                id: "high-risk-files".to_string(),
                category: AdviceCategory::CodeQuality,
                severity: AdviceSeverity::High,
                title: format!("发现{}个高风险文件", entropy.high_risk_files.len()),
                description: "高风险文件需要重点关注和重构".to_string(),
                suggestion: "按风险等级排序，优先处理熵值最高的文件".to_string(),
                affected_files: entropy.high_risk_files.clone(),
                effort: EffortEstimate::Days,
                impact: ImpactLevel::Medium,
            });
        }

        advices
    }

    fn generate_summary(&self, advices: &[Advice]) -> AdviceSummary {
        let critical_count = advices.iter().filter(|a| a.severity == AdviceSeverity::Critical).count();
        let high_count = advices.iter().filter(|a| a.severity == AdviceSeverity::High).count();
        let medium_count = advices.iter().filter(|a| a.severity == AdviceSeverity::Medium).count();
        let low_count = advices.iter().filter(|a| a.severity == AdviceSeverity::Low).count();

        let mut recommended_actions = Vec::new();
        
        if critical_count > 0 {
            recommended_actions.push("立即修复所有Critical级别的架构违规".to_string());
        }
        if high_count > 0 {
            recommended_actions.push("优先处理高风险文件和复杂度问题".to_string());
        }
        if medium_count > 0 {
            recommended_actions.push("制定计划降低模块耦合度".to_string());
        }
        if advices.iter().any(|a| a.category == AdviceCategory::TestCoverage) {
            recommended_actions.push("增加测试覆盖率".to_string());
        }

        AdviceSummary {
            critical_count,
            high_count,
            medium_count,
            low_count,
            total_issues: advices.len(),
            recommended_actions,
        }
    }

    pub fn format_result(&self, result: &AdviceResult) -> String {
        let mut output = String::new();

        output.push_str("\n💡 架构改进建议\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        output.push_str(&format!("总建议数: {}\n", result.summary.total_issues));
        output.push_str(&format!("🔴 Critical: {}\n", result.summary.critical_count));
        output.push_str(&format!("🟠 High: {}\n", result.summary.high_count));
        output.push_str(&format!("🟡 Medium: {}\n", result.summary.medium_count));
        output.push_str(&format!("🟢 Low: {}\n", result.summary.low_count));

        if !result.summary.recommended_actions.is_empty() {
            output.push_str("\n🎯 推荐行动:\n");
            for (i, action) in result.summary.recommended_actions.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, action));
            }
        }

        if !result.advices.is_empty() {
            output.push_str("\n📋 详细建议:\n");
            output.push_str("──────────────────────────────────────────────────────────────\n\n");

            for advice in &result.advices {
                let severity_icon = match advice.severity {
                    AdviceSeverity::Critical => "🔴",
                    AdviceSeverity::High => "🟠",
                    AdviceSeverity::Medium => "🟡",
                    AdviceSeverity::Low => "🟢",
                };

                let effort_label = match advice.effort {
                    EffortEstimate::Minutes => "⏱️ 几分钟",
                    EffortEstimate::Hours => "⏳ 几小时",
                    EffortEstimate::Days => "📅 几天",
                    EffortEstimate::Weeks => "📆 几周",
                };

                let impact_icon = match advice.impact {
                    ImpactLevel::Transformational => "🚀",
                    ImpactLevel::High => "⭐",
                    ImpactLevel::Medium => "📈",
                    ImpactLevel::Low => "📉",
                };

                output.push_str(&format!("{} {} - {}\n", severity_icon, advice.title, impact_icon));
                output.push_str(&format!("   {}\n", advice.description));
                output.push_str(&format!("   💡 {}\n", advice.suggestion));
                output.push_str(&format!("   {} · {}\n", effort_label, impact_icon));

                if !advice.affected_files.is_empty() {
                    output.push_str("   📎 影响文件:\n");
                    for f in advice.affected_files.iter().take(5) {
                        output.push_str(&format!("      • {}\n", f));
                    }
                    if advice.affected_files.len() > 5 {
                        output.push_str(&format!("      • ... 及其他 {} 个文件\n", advice.affected_files.len() - 5));
                    }
                }

                output.push_str("\n");
            }
        }

        output.push_str("💡 提示: 根据建议的优先级和工作量，制定合理的重构计划。\n");

        output
    }
}

impl Default for ArchitectureAdvisor {
    fn default() -> Self {
        Self::new()
    }
}
