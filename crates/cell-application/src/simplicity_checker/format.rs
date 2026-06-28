//! 代码质量检测器：报告格式化输出
//! Code quality checker: report formatting

use super::types::{FileReport, Grade, Issue, SimplicityReport, Severity};
use std::collections::HashMap;

/// 格式化代码质量报告为可读字符串
pub fn format_report(report: &SimplicityReport) -> String {
    let mut out = String::new();

    let grade_icon = match report.grade {
        Grade::S => "🏆", Grade::A => "🥇", Grade::B => "🥈",
        Grade::C => "🥉", Grade::D => "⚠️", Grade::F => "❌",
    };

    out.push_str("\n🧹 Code Quality Report\n\n");
    out.push_str(&format!("  Score: {:.1}/100  {}{:?}\n\n", report.score, grade_icon, report.grade));

    out.push_str("  📊 Dimension Scores\n\n");
    let mut dims: Vec<_> = report.dimension_scores.iter().collect();
    dims.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (name, score) in dims {
        let bar_len = (score / 10.0) as usize;
        let bar = format!("{}{}", "█".repeat(bar_len.min(10)), "░".repeat(10 - bar_len));
        out.push_str(&format!("    {bar} {score:>5.1}  {name}\n"));
    }
    out.push('\n');

    render_overview(&mut out, report);
    render_worst_files(&mut out, report);
    render_issue_categories(&mut out, report);
    render_top_issues(&mut out, report);

    out
}

fn render_overview(out: &mut String, report: &SimplicityReport) {
    out.push_str("  ┌─────────────────────────────────────────────────────┐\n");
    out.push_str("  │                   Overview                          │\n");
    out.push_str("  ├─────────────────────────────────────────────────────┤\n");
    out.push_str(&format!("  │  Files:        {:<37}│\n", report.total_files));
    out.push_str(&format!("  │  Total lines:  {:<36}│\n", report.total_lines));
    out.push_str(&format!("  │  Functions:    {:<37}│\n", report.total_functions));
    if report.total_functions > 0 {
        out.push_str(&format!("  │  Avg fn lines: {:<34.1}│\n",
            report.total_lines as f64 / report.total_functions as f64));
    }
    out.push_str("  ├─────────────────────────────────────────────────────┤\n");
    out.push_str(&format!("  │  Long files:   {:<36}│\n", report.summary.long_files));
    out.push_str(&format!("  │  Long fns:     {:<36}│\n", report.summary.long_functions));
    out.push_str(&format!("  │  Large structs:{:<36}│\n", report.summary.large_structs));
    out.push_str(&format!("  │  High complex: {:<36}│\n", report.summary.complex_functions));
    out.push_str(&format!("  │  Deep nesting: {:<36}│\n", report.summary.deep_nesting));
    out.push_str(&format!("  │  Many args:    {:<36}│\n", report.summary.many_args));
    out.push_str(&format!("  │  Magic nums:   {:<36}│\n", report.summary.magic_numbers));
    out.push_str(&format!("  │  unwrap uses:  {:<36}│\n", report.summary.unwrap_usage));
    out.push_str(&format!("  │  clone abuse:  {:<36}│\n", report.summary.clone_overuse));
    out.push_str(&format!("  │  Low comments: {:<36}│\n", report.summary.low_comment_files));
    out.push_str(&format!("  │  TODOs:        {:<36}│\n", report.summary.todo_markers));
    out.push_str(&format!("  │  unsafe:       {:<36}│\n", report.summary.unsafe_usage));
    out.push_str("  ├─────────────────────────────────────────────────────┤\n");
    out.push_str(&format!("  │  Total issues: {:<36}│\n", report.issues.len()));
    out.push_str("  └─────────────────────────────────────────────────────┘\n\n");
}

fn render_worst_files(out: &mut String, report: &SimplicityReport) {
    let needs_improve: Vec<&FileReport> = report.files.iter()
        .filter(|f| f.score < 85.0).take(10).collect();

    if !needs_improve.is_empty() {
        out.push_str("  📁 Worst scoring files\n\n");
        for f in needs_improve {
            let bar_len = (f.score / 10.0) as usize;
            let bar = format!("{}{}", "█".repeat(bar_len.min(10)), "░".repeat(10 - bar_len));
            out.push_str(&format!(
                "    {} {:>5.1}  {:<42} {} lines {} fns\n",
                bar, f.score, f.path, f.lines, f.fn_count
            ));
        }
        out.push('\n');
    }
}

fn render_issue_categories(out: &mut String, report: &SimplicityReport) {
    let warnings: Vec<&Issue> = report.issues.iter()
        .filter(|i| i.severity == Severity::Warning).collect();

    if warnings.is_empty() { return; }

    out.push_str("  ⚠️  Issue categories\n\n");
    let mut by_cat: HashMap<&super::types::Category, usize> = HashMap::new();
    for issue in &warnings {
        *by_cat.entry(&issue.category).or_insert(0) += 1;
    }
    let mut cats: Vec<_> = by_cat.into_iter().collect();
    cats.sort_by_key(|c| std::cmp::Reverse(c.1));
    for (cat, count) in cats {
        out.push_str(&format!("    {}: {} items\n", cat.display_name(), count));
    }
    out.push('\n');
}

fn render_top_issues(out: &mut String, report: &SimplicityReport) {
    let top: Vec<&Issue> = report.issues.iter()
        .filter(|i| i.severity == Severity::Warning).take(8).collect();

    if top.is_empty() { return; }

    out.push_str("  💡 Top issues (first 8)\n\n");
    for issue in top {
        let ln = issue.line.map(|l| format!(":{l}")).unwrap_or_default();
        out.push_str(&format!("    - {}{}\n", issue.path, ln));
        out.push_str(&format!("      [{}] {}\n", issue.category.display_name(), issue.message));
        out.push_str(&format!("      Suggestion: {}\n\n", issue.suggestion));
    }
}
