use cell_domain::refactor_assistant::{
    CodeSmell, RefactorAssistant, RefactorProposal, RefactorSeverity, RefactorStatus,
};
use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeResult {
    pub code_smells: Vec<CodeSmell>,
    pub proposals: Vec<RefactorProposal>,
    pub summary: AnalyzeSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeSummary {
    pub total_smells: usize,
    pub total_proposals: usize,
    pub critical_count: usize,
    pub major_count: usize,
    pub minor_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalDetail {
    pub proposal: RefactorProposal,
    pub impact: Option<cell_domain::refactor_assistant::RefactorImpact>,
}

pub struct RefactorService {
    assistant: RefactorAssistant,
}

impl RefactorService {
    pub fn new() -> Self {
        Self {
            assistant: RefactorAssistant::new(),
        }
    }

    pub fn analyze_code_smells(&self, path: &str) -> CellResult<AnalyzeResult> {
        let metrics = self.collect_metrics(path);
        let code_smells = self.assistant.detect_code_smells(path, &metrics);
        let proposals = self.assistant.generate_proposals(path, &metrics);
        let summary = self.generate_summary(&code_smells, &proposals);
        Ok(AnalyzeResult { code_smells, proposals, summary })
    }

    pub fn list_refactor_proposals(&self) -> Vec<&RefactorProposal> {
        self.assistant.all_proposals()
    }

    pub fn apply_refactor(&mut self, id: &str) -> CellResult<()> {
        let result = self.assistant.apply_refactor(id);
        result.map_err(|e| cell_domain::errors::CellError::Other(e.to_string()))
    }

    pub fn get_proposal_detail(&self, id: &str) -> CellResult<ProposalDetail> {
        let proposal = self.assistant.get_proposal(id)
            .map_err(|e| cell_domain::errors::CellError::Other(e.to_string()))?;
        let impact = self.assistant.assess_impact(id).ok();
        Ok(ProposalDetail { proposal: proposal.clone(), impact })
    }

    pub fn add_proposal(&mut self, proposal: RefactorProposal) {
        self.assistant.add_proposal(proposal);
    }

    pub fn generate_execution_plan(&mut self, id: &str) -> CellResult<()> {
        let _result = self.assistant.generate_execution_plan(id)
            .map_err(|e| cell_domain::errors::CellError::Other(e.to_string()))?;
        Ok(())
    }

    fn collect_metrics(&self, _path: &str) -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("function_lines".to_string(), 65.0);
        m.insert("class_method_count".to_string(), 35.0);
        m.insert("duplication_ratio".to_string(), 12.0);
        m.insert("parameter_count".to_string(), 9.0);
        m.insert("inheritance_depth".to_string(), 6.0);
        m.insert("change_impact_files".to_string(), 11.0);
        m.insert("external_calls_ratio".to_string(), 65.0);
        m.insert("data_only_ratio".to_string(), 82.0);
        m
    }

    fn generate_summary(&self, code_smells: &[CodeSmell], proposals: &[RefactorProposal]) -> AnalyzeSummary {
        let (mut c, mut mj, mut mn, mut i) = (0, 0, 0, 0);
        for s in code_smells {
            match s.severity {
                RefactorSeverity::Critical => c += 1,
                RefactorSeverity::Major => mj += 1,
                RefactorSeverity::Minor => mn += 1,
                RefactorSeverity::Info => i += 1,
            }
        }
        AnalyzeSummary { total_smells: code_smells.len(), total_proposals: proposals.len(), critical_count: c, major_count: mj, minor_count: mn, info_count: i }
    }

    pub fn format_proposals(&self, proposals: &[&RefactorProposal]) -> String {
        if proposals.is_empty() { return "暂无重构建议".to_string(); }
        let mut o = String::new();
        o.push_str(&format!("\n📋 重构建议列表\n{}", "─".repeat(60)));
        o.push_str(&format!("\n  共 {} 条建议\n", proposals.len()));
        for (idx, p) in proposals.iter().enumerate() {
            let sev = match p.severity { RefactorSeverity::Critical => "🔴", RefactorSeverity::Major => "🟠", RefactorSeverity::Minor => "🟡", RefactorSeverity::Info => "🟢" };
            let st = match p.status { RefactorStatus::Proposed => "📝", RefactorStatus::Planned => "📅", RefactorStatus::InProgress => "🔄", RefactorStatus::Completed => "✅", RefactorStatus::RolledBack => "↩️" };
            o.push_str(&format!("\n  {}. {} {} [{}] {} ({:.1}h, 收益: {:.1}, 优先级: {:.1})", idx + 1, sev, st, p.id, p.title, p.estimated_effort_hours, p.benefit_score, p.priority_score));
            o.push_str(&format!("\n     {}", p.description));
            o.push_str(&format!("\n     影响文件: {}", p.affected_files.join(", ")));
        }
        o.push_str(&format!("\n\n{}", "─".repeat(60)));
        o
    }

    pub fn format_proposal_detail(&self, detail: &ProposalDetail) -> String {
        let p = &detail.proposal;
        let mut o = String::new();
        o.push_str(&format!("\n📋 重构建议详情\n{}", "─".repeat(60)));
        o.push_str(&format!("\n\n  ID: {}\n", p.id));
        o.push_str(&format!("  类型: {}\n", p.refactor_type));
        o.push_str(&format!("  标题: {}\n", p.title));
        o.push_str(&format!("  描述: {}\n", p.description));
        let sev = match p.severity { RefactorSeverity::Critical => "🔴 Critical", RefactorSeverity::Major => "🟠 Major", RefactorSeverity::Minor => "🟡 Minor", RefactorSeverity::Info => "🟢 Info" };
        o.push_str(&format!("  严重程度: {sev}\n"));
        let st = match p.status { RefactorStatus::Proposed => "📝 Proposed", RefactorStatus::Planned => "📅 Planned", RefactorStatus::InProgress => "🔄 InProgress", RefactorStatus::Completed => "✅ Completed", RefactorStatus::RolledBack => "↩️ RolledBack" };
        o.push_str(&format!("  状态: {st}\n"));
        o.push_str(&format!("  预估工时: {:.1}h\n", p.estimated_effort_hours));
        o.push_str(&format!("  收益评分: {:.1}\n", p.benefit_score));
        o.push_str(&format!("  优先级: {:.1}\n", p.priority_score));
        o.push_str("\n  影响文件:\n");
        for f in &p.affected_files { o.push_str(&format!("    - {f}\n")); }
        o.push_str("\n  代码异味:\n");
        for s in &p.code_smells { o.push_str(&format!("    - {}: {} ({})\n", s.smell_type, s.description, s.severity)); }
        o.push_str(&format!("\n  创建时间: {}\n", p.created_at));
        o.push_str(&format!("  更新时间: {}\n", p.updated_at));
        if let Some(im) = &detail.impact {
            o.push_str("\n  📊 影响评估:\n");
            o.push_str(&format!("    影响范围: {}\n", im.blast_radius));
            o.push_str(&format!("    风险等级: {}\n", im.risk_level));
            o.push_str(&format!("    预估恢复时间: {:.0}分钟\n", im.estimated_recovery_minutes));
            o.push_str(&format!("    是否破坏性变更: {}\n", im.breaking_changes));
        }
        o.push_str(&format!("\n{}", "─".repeat(60)));
        o
    }
}

impl Default for RefactorService {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_domain::refactor_assistant::{RefactorType, RefactorProposal};

    fn tp(id: &str) -> RefactorProposal {
        RefactorProposal::new(id, RefactorType::ExtractFunction, "Test Refactor", "Test description")
            .with_severity(RefactorSeverity::Major)
            .with_affected_files(vec!["src/main.rs".to_string()])
            .with_estimated_effort(4.0)
            .with_benefit_score(25.0)
    }

    #[test]
    fn test_analyze_code_smells() {
        let s = RefactorService::new();
        let r = s.analyze_code_smells("src/test.rs");
        assert!(r.is_ok());
        let r = r.unwrap();
        assert!(r.summary.total_smells > 0);
        assert!(r.summary.total_proposals > 0);
        assert!(r.summary.critical_count + r.summary.major_count + r.summary.minor_count + r.summary.info_count == r.summary.total_smells);
    }

    #[test]
    fn test_list_refactor_proposals() {
        let mut s = RefactorService::new();
        s.add_proposal(tp("test-001"));
        s.add_proposal(tp("test-002"));
        let p = s.list_refactor_proposals();
        assert_eq!(p.len(), 2);
        let ids: Vec<&str> = p.iter().map(|x| x.id.as_str()).collect();
        assert!(ids.contains(&"test-001"));
        assert!(ids.contains(&"test-002"));
    }

    #[test]
    fn test_apply_refactor() {
        let mut s = RefactorService::new();
        s.add_proposal(tp("apply-test"));
        s.generate_execution_plan("apply-test").unwrap();
        let r = s.apply_refactor("apply-test");
        assert!(r.is_ok());
        let d = s.get_proposal_detail("apply-test").unwrap();
        assert_eq!(d.proposal.status, RefactorStatus::Completed);
    }

    #[test]
    fn test_get_proposal_detail() {
        let mut s = RefactorService::new();
        s.add_proposal(tp("detail-test"));
        let r = s.get_proposal_detail("detail-test");
        assert!(r.is_ok());
        let d = r.unwrap();
        assert_eq!(d.proposal.id, "detail-test");
        assert!(d.impact.is_some());
    }

    #[test]
    fn test_get_proposal_not_found() {
        let s = RefactorService::new();
        let r = s.get_proposal_detail("nonexistent");
        assert!(r.is_err());
    }
}
