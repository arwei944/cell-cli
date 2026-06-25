use crate::adapters::file_evolution_store::FileEvolutionStore;
use crate::application::evolution_service::EvolutionService;
use crate::application::progress_bar::StepProgress;
use crate::domain::errors::{CellError, CellResult};
use crate::domain::evolution::{
    EffortEstimate, ImpactLevel, ImprovementCategory, IssueCategory, IssueSeverity,
};
use crate::interfaces::cli::*;

pub fn cmd_evolve(args: EvolveArgs) -> CellResult<()> {
    let store = FileEvolutionStore::new();
    let service = EvolutionService::new(store);

    match args.sub {
        EvolveSub::Cycle { action } => handle_cycle(&service, action),
        EvolveSub::Issue { title, description, category, severity } => {
            let cat = parse_issue_category(category.as_deref().unwrap_or("other"))?;
            let sev = parse_issue_severity(severity.as_deref().unwrap_or("medium"))?;
            let desc = description.unwrap_or_default();
            let log = service.report_issue(".", &title, &desc, cat, sev)?;
            let issue = log.issues.last()
                .ok_or_else(|| CellError::Config("Failed to add issue".to_string()))?;
            println!("🐛 问题已记录: {}", title);
            println!("   ID: {}", issue.id);
            println!("   分类: {:?}", issue.category);
            println!("   严重度: {:?}", issue.severity);
            Ok(())
        }
        EvolveSub::Add { title, description, category, impact, effort } => {
            let cat = parse_improvement_category(category.as_deref().unwrap_or("new-feature"))?;
            let imp = parse_impact_level(impact.as_deref().unwrap_or("medium"))?;
            let eff = parse_effort_estimate(effort.as_deref().unwrap_or("hours"))?;
            let desc = description.unwrap_or_default();
            let log = service.add_improvement(".", &title, &desc, cat, imp, eff)?;
            let improvement = log.improvements.last()
                .ok_or_else(|| CellError::Config("Failed to add improvement".to_string()))?;
            println!("💡 改进建议已添加: {}", title);
            println!("   ID: {}", improvement.id);
            println!("   分类: {:?}", improvement.category);
            println!("   影响: {:?}", improvement.expected_impact);
            println!("   工作量: {:?}", improvement.implementation_effort);
            Ok(())
        }
        EvolveSub::Suggest { apply } => handle_suggest(&service, apply),
        EvolveSub::Apply { id, by } => {
            let log = service.apply_improvement(".", &id, by.as_deref())?;
            let imp = log.improvements.iter().find(|i| i.id.to_string() == id)
                .ok_or_else(|| CellError::Config("Improvement not found".to_string()))?;
            println!("✅ 改进已应用: {}", imp.title);
            if let Some(by_name) = &imp.applied_by {
                println!("   应用者: {}", by_name);
            }
            Ok(())
        }
        EvolveSub::Status {} => show_evolve_status(&service),
        EvolveSub::History {} => show_evolve_history(&service),
        EvolveSub::Stats {} => show_evolve_stats(&service),
        EvolveSub::Scan {} => handle_scan(&service),
    }
}

fn handle_cycle(service: &EvolutionService<FileEvolutionStore>, action: CycleAction) -> CellResult<()> {
    match action {
        CycleAction::Start => {
            let log = service.start_cycle(".")?;
            println!("🔄 进化周期 #{} 已开始", log.cycle_number);
            println!("   ID: {}", log.log_id);
            println!("   阶段: CollectingIssues");
        }
        CycleAction::Complete => {
            let log = service.complete_cycle(".")?;
            println!("✅ 进化周期 #{} 已完成", log.cycle_number);
            println!("   问题数: {}", log.issues.len());
            println!("   改进数: {}", log.improvements.len());
        }
    }
    Ok(())
}

fn handle_suggest(service: &EvolutionService<FileEvolutionStore>, apply: bool) -> CellResult<()> {
    let suggestions = service.generate_suggestions(".")?;
    if suggestions.is_empty() {
        println!("ℹ️  当前没有足够的数据生成改进建议。");
        println!("   请先使用 'cell evolve issue' 记录遇到的问题。");
    } else {
        println!("🤖 智能改进建议 ({} 条):\n", suggestions.len());
        for (i, s) in suggestions.iter().enumerate() {
            let impact_icon = match s.expected_impact {
                ImpactLevel::Transformational => "🚀",
                ImpactLevel::High => "⭐",
                ImpactLevel::Medium => "📈",
                ImpactLevel::Low => "📉",
                ImpactLevel::Minimal => "🔹",
            };
            println!("{}. {} {} [{:?} | {:?}]",
                i + 1, impact_icon, s.title, s.expected_impact, s.implementation_effort
            );
            println!("   {}", s.description);
            if !s.related_issue_ids.is_empty() {
                println!("   关联问题: {} 个", s.related_issue_ids.len());
            }
            println!();
        }
        if apply {
            println!("⚠️  自动应用改进建议功能需要人工确认。");
            println!("   请使用 'cell evolve apply <id>' 手动应用。");
        }
    }
    Ok(())
}

fn show_evolve_status(service: &EvolutionService<FileEvolutionStore>) -> CellResult<()> {
    match service.get_current_cycle(".")? {
        Some(log) => {
            println!("🔄 当前进化周期: #{}", log.cycle_number);
            println!("   阶段: {:?}", log.phase);
            println!("   开始时间: {}", log.started_at.format("%Y-%m-%d %H:%M:%S"));
            println!("   问题数: {} (严重: {})", log.issues.len(), log.critical_issues().len());
            println!("   改进建议: {} (待处理: {})", log.improvements.len(), log.pending_improvements().len());

            let top_cats = log.top_issue_categories(5);
            if !top_cats.is_empty() {
                println!("\n📊 问题分类统计:");
                for (cat, count) in &top_cats {
                    println!("   {:?}: {}", cat, count);
                }
            }
            print_evolve_issues(&log.issues);
            print_evolve_improvements(&log.improvements);
        }
        None => {
            println!("ℹ️  没有进行中的进化周期。");
            println!("   使用 'cell evolve cycle start' 开始一个新周期。");
        }
    }
    Ok(())
}

fn print_evolve_issues(issues: &[crate::domain::evolution::Issue]) {
    if issues.is_empty() { return; }
    println!("\n🐛 问题列表:");
    for issue in issues {
        let sev_icon = match issue.severity {
            IssueSeverity::Critical => "🔴",
            IssueSeverity::High => "🟠",
            IssueSeverity::Medium => "🟡",
            IssueSeverity::Low => "🟢",
            IssueSeverity::Trivial => "⚪",
        };
        println!("  {} [{}] {} ({:?})", sev_icon, issue.id, issue.title, issue.category);
    }
}

fn print_evolve_improvements(improvements: &[crate::domain::evolution::Improvement]) {
    if improvements.is_empty() { return; }
    println!("\n💡 改进建议:");
    for imp in improvements {
        let status_icon = match imp.status {
            crate::domain::evolution::ImprovementStatus::Proposed => "📝",
            crate::domain::evolution::ImprovementStatus::Planned => "📋",
            crate::domain::evolution::ImprovementStatus::InProgress => "🔧",
            crate::domain::evolution::ImprovementStatus::Applied => "✅",
            crate::domain::evolution::ImprovementStatus::Verified => "✅",
            crate::domain::evolution::ImprovementStatus::Rejected => "❌",
            crate::domain::evolution::ImprovementStatus::RolledBack => "↩️",
        };
        println!("  {} [{}] {} (影响: {:?})", status_icon, imp.id, imp.title, imp.expected_impact);
    }
}

fn show_evolve_history(service: &EvolutionService<FileEvolutionStore>) -> CellResult<()> {
    let history = service.list_history(".")?;
    if history.is_empty() {
        println!("ℹ️  没有进化历史记录。");
    } else {
        println!("📜 进化历史 (共 {} 个周期):\n", history.len());
        for log in &history {
            println!("  周期 #{} [{:?}] - 问题: {}, 改进: {}",
                log.cycle_number, log.phase, log.issues.len(), log.improvements.len()
            );
        }
    }
    Ok(())
}

fn show_evolve_stats(service: &EvolutionService<FileEvolutionStore>) -> CellResult<()> {
    let summary = service.get_evolution_summary(".")?;
    println!("📊 自进化统计:");
    println!("   已完成周期: {}", summary.cycles_completed);
    println!("   当前周期活跃: {}", if summary.current_cycle_active { "是" } else { "否" });
    println!("   总问题数: {}", summary.total_issues_reported);
    println!("   总改进建议: {}", summary.total_improvements_proposed);
    println!("   已应用改进: {}", summary.total_improvements_applied);
    if !summary.top_categories.is_empty() {
        println!("\n   高频问题分类:");
        let mut cats: Vec<_> = summary.top_categories.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in &cats {
            println!("      {:?}: {}", cat, count);
        }
    }
    Ok(())
}

fn handle_scan(service: &EvolutionService<FileEvolutionStore>) -> CellResult<()> {
    if service.get_current_cycle(".")?.is_none() {
        service.start_cycle(".")?;
    }
    let steps = vec![
        ("项目结构分析", "扫描项目文件和目录结构"),
        ("代码质量检查", "分析代码复杂度和规范"),
        ("测试覆盖率检测", "检查测试文件和覆盖率"),
        ("文档完整性检查", "检查README和文档"),
        ("性能瓶颈检测", "检测潜在性能问题"),
        ("流程效率分析", "分析开发流程效率"),
        ("生成问题报告", "汇总分析结果"),
    ];
    let mut progress = StepProgress::new(steps);
    for _ in 0..6 { progress.start_next(); progress.complete_current(); }
    progress.start_next();
    let log = service.auto_diagnose(".")?;
    progress.complete_current();
    progress.render_summary();

    println!();
    if log.issues.is_empty() {
        println!("✅ 未发现明显问题！");
    } else {
        println!("🐛 发现 {} 个潜在问题：\n", log.issues.len());
        for issue in &log.issues {
            let sev_icon = match issue.severity {
                IssueSeverity::Critical => "🔴",
                IssueSeverity::High => "🟠",
                IssueSeverity::Medium => "🟡",
                IssueSeverity::Low => "🟢",
                IssueSeverity::Trivial => "⚪",
            };
            println!("  {} [{:?}] {}", sev_icon, issue.category, issue.title);
            println!("     {}", issue.description);
            println!();
        }
        println!("💡 运行 'cell evolve suggest' 查看改进建议");
    }
    Ok(())
}

fn parse_issue_category(s: &str) -> CellResult<IssueCategory> {
    match s.to_lowercase().as_str() {
        "process" | "process-efficiency" => Ok(IssueCategory::ProcessEfficiency),
        "tool" | "tool-intelligence" => Ok(IssueCategory::ToolIntelligence),
        "quality" | "quality-gate" => Ok(IssueCategory::QualityGate),
        "handoff" | "handoff-completeness" => Ok(IssueCategory::HandoffCompleteness),
        "codegen" | "code-generation" => Ok(IssueCategory::CodeGeneration),
        "docs" | "documentation" => Ok(IssueCategory::Documentation),
        "arch" | "architecture" | "architecture-drift" => Ok(IssueCategory::ArchitectureDrift),
        "entropy" | "entropy-growth" => Ok(IssueCategory::EntropyGrowth),
        "test" | "testing" => Ok(IssueCategory::Testing),
        "perf" | "performance" => Ok(IssueCategory::Performance),
        "other" => Ok(IssueCategory::Other),
        _ => Err(CellError::Config(format!(
            "Unknown issue category: {}. Valid: process, tool, quality, handoff, codegen, docs, arch, entropy, test, other",
            s
        ))),
    }
}

fn parse_issue_severity(s: &str) -> CellResult<IssueSeverity> {
    match s.to_lowercase().as_str() {
        "critical" => Ok(IssueSeverity::Critical),
        "high" => Ok(IssueSeverity::High),
        "medium" => Ok(IssueSeverity::Medium),
        "low" => Ok(IssueSeverity::Low),
        "trivial" => Ok(IssueSeverity::Trivial),
        _ => Err(CellError::Config(format!(
            "Unknown severity: {}. Valid: critical, high, medium, low, trivial", s
        ))),
    }
}

fn parse_improvement_category(s: &str) -> CellResult<ImprovementCategory> {
    match s.to_lowercase().as_str() {
        "process" | "process-optimization" => Ok(ImprovementCategory::ProcessOptimization),
        "new-feature" | "feature" => Ok(ImprovementCategory::NewFeature),
        "quality-gate" | "constraint" => Ok(ImprovementCategory::ConstraintAdjustment),
        "template" | "template-improvement" => Ok(ImprovementCategory::TemplateImprovement),
        "docs" | "documentation" => Ok(ImprovementCategory::DocumentationUpdate),
        "automation" => Ok(ImprovementCategory::Automation),
        "refactoring" | "refactor" => Ok(ImprovementCategory::Refactoring),
        _ => Err(CellError::Config(format!(
            "Unknown improvement category: {}. Valid: process, new-feature, quality-gate, template, docs, automation, refactoring",
            s
        ))),
    }
}

fn parse_impact_level(s: &str) -> CellResult<ImpactLevel> {
    match s.to_lowercase().as_str() {
        "transformational" => Ok(ImpactLevel::Transformational),
        "high" => Ok(ImpactLevel::High),
        "medium" => Ok(ImpactLevel::Medium),
        "low" => Ok(ImpactLevel::Low),
        "minimal" => Ok(ImpactLevel::Minimal),
        _ => Err(CellError::Config(format!(
            "Unknown impact level: {}. Valid: transformational, high, medium, low, minimal", s
        ))),
    }
}

fn parse_effort_estimate(s: &str) -> CellResult<EffortEstimate> {
    match s.to_lowercase().as_str() {
        "minutes" => Ok(EffortEstimate::Minutes),
        "hours" => Ok(EffortEstimate::Hours),
        "days" => Ok(EffortEstimate::Days),
        "weeks" => Ok(EffortEstimate::Weeks),
        _ => Err(CellError::Config(format!(
            "Unknown effort estimate: {}. Valid: minutes, hours, days, weeks", s
        ))),
    }
}
