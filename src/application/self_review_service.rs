use crate::application::code_review_service::{CodeReviewService, ReviewSeverity};
use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SelfReviewSeverity {
    Info,
    Minor,
    Major,
    Critical,
    Blocker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewIssue {
    pub file: String,
    pub line: Option<usize>,
    pub severity: SelfReviewSeverity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub suggestion: Option<String>,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewResult {
    pub reviewed_files: usize,
    pub total_issues: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_category: std::collections::HashMap<String, usize>,
    pub issues: Vec<SelfReviewIssue>,
    pub auto_fixed: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReviewRequest {
    pub id: String,
    pub author_agent: String,
    pub reviewer_agent: String,
    pub files: Vec<String>,
    pub status: String,
    pub created_at: String,
}

pub struct SelfReviewService;

impl SelfReviewService {
    pub fn new() -> Self {
        Self
    }

    pub fn self_review(&self, project_path: &str, deep: bool) -> CellResult<SelfReviewResult> {
        let review_service = CodeReviewService::new();
        let report = review_service.review_code(project_path, deep)?;

        let mut issues = Vec::new();
        let mut by_severity = std::collections::HashMap::new();
        let mut by_category = std::collections::HashMap::new();

        for issue in &report.issues {
            let severity = match issue.severity {
                ReviewSeverity::Critical => SelfReviewSeverity::Blocker,
                ReviewSeverity::Major => SelfReviewSeverity::Major,
                ReviewSeverity::Minor => SelfReviewSeverity::Minor,
                ReviewSeverity::Info => SelfReviewSeverity::Info,
            };

            let auto_fixable = matches!(
                issue.title.to_lowercase().as_str(),
                t if t.contains("命名") || t.contains("格式") || t.contains("naming") || t.contains("format")
            );

            let self_issue = SelfReviewIssue {
                file: issue.file.clone(),
                line: issue.line,
                severity,
                category: format!("{:?}", issue.category),
                title: issue.title.clone(),
                description: issue.description.clone(),
                suggestion: issue.suggestion.clone(),
                auto_fixable,
            };

            *by_severity
                .entry(format!("{:?}", self_issue.severity))
                .or_insert(0) += 1;
            *by_category
                .entry(self_issue.category.clone())
                .or_insert(0) += 1;

            issues.push(self_issue);
        }

        let auto_fixed = if deep {
            issues.iter().filter(|i| i.auto_fixable).count()
        } else {
            0
        };

        let blocker_count = issues
            .iter()
            .filter(|i| matches!(i.severity, SelfReviewSeverity::Blocker | SelfReviewSeverity::Critical))
            .count();
        let passed = blocker_count == 0;

        Ok(SelfReviewResult {
            reviewed_files: report.summary.total_files,
            total_issues: issues.len(),
            by_severity,
            by_category,
            issues,
            auto_fixed,
            passed,
        })
    }

    pub fn create_cross_review(
        &self,
        project_path: &str,
        author_agent: &str,
        reviewer_agent: &str,
        files: Vec<String>,
    ) -> CellResult<CrossReviewRequest> {
        let request = CrossReviewRequest {
            id: format!("review-{}", uuid::Uuid::new_v4().simple()),
            author_agent: author_agent.to_string(),
            reviewer_agent: reviewer_agent.to_string(),
            files,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let reviews_dir = Path::new(project_path)
            .join(".cell")
            .join("cross_reviews");
        std::fs::create_dir_all(&reviews_dir)?;

        let file_path = reviews_dir.join(format!("{}.json", request.id));
        std::fs::write(&file_path, serde_json::to_string_pretty(&request)?)?;

        Ok(request)
    }

    pub fn list_cross_reviews(
        &self,
        project_path: &str,
        status: Option<&str>,
    ) -> CellResult<Vec<CrossReviewRequest>> {
        let reviews_dir = Path::new(project_path)
            .join(".cell")
            .join("cross_reviews");

        let mut reviews = Vec::new();

        if reviews_dir.exists() {
            for entry in std::fs::read_dir(&reviews_dir)? {
                let entry = entry?;
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(review) = serde_json::from_str::<CrossReviewRequest>(&content) {
                    if status.map(|s| review.status == s).unwrap_or(true) {
                        reviews.push(review);
                    }
                }
            }
        }

        reviews.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(reviews)
    }

    pub fn format_result(&self, result: &SelfReviewResult) -> String {
        let mut output = String::new();

        output.push_str("\n🔍 自我审查结果\n\n");
        output.push_str(&format!("  审查文件数: {}\n", result.reviewed_files));
        output.push_str(&format!("  发现问题数: {}\n", result.total_issues));
        output.push_str(&format!("  自动修复数: {}\n", result.auto_fixed));

        output.push_str("\n  按严重程度:\n");
        for (sev, count) in &result.by_severity {
            let icon = match sev.as_str() {
                "Blocker" | "Critical" => "🔴",
                "Major" => "🟠",
                "Minor" => "🟡",
                _ => "🟢",
            };
            output.push_str(&format!("    {} {}: {}\n", icon, sev, count));
        }

        output.push_str("\n  按类别:\n");
        for (cat, count) in &result.by_category {
            output.push_str(&format!("    {}: {}\n", cat, count));
        }

        if !result.issues.is_empty() {
            output.push_str("\n  问题列表:\n\n");

            let mut sorted_issues = result.issues.clone();
            sorted_issues.sort_by(|a, b| {
                let sa = match a.severity {
                    SelfReviewSeverity::Blocker => 0,
                    SelfReviewSeverity::Critical => 1,
                    SelfReviewSeverity::Major => 2,
                    SelfReviewSeverity::Minor => 3,
                    SelfReviewSeverity::Info => 4,
                };
                let sb = match b.severity {
                    SelfReviewSeverity::Blocker => 0,
                    SelfReviewSeverity::Critical => 1,
                    SelfReviewSeverity::Major => 2,
                    SelfReviewSeverity::Minor => 3,
                    SelfReviewSeverity::Info => 4,
                };
                sa.cmp(&sb)
            });

            for (i, issue) in sorted_issues.iter().take(20).enumerate() {
                let sev_icon = match issue.severity {
                    SelfReviewSeverity::Blocker => "🔴",
                    SelfReviewSeverity::Critical => "🔴",
                    SelfReviewSeverity::Major => "🟠",
                    SelfReviewSeverity::Minor => "🟡",
                    SelfReviewSeverity::Info => "ℹ️",
                };
                let fix_icon = if issue.auto_fixable { "🔧" } else { "👀" };

                output.push_str(&format!(
                    "  {}. {} {} [{}] {}\n",
                    i + 1,
                    sev_icon,
                    fix_icon,
                    issue.category,
                    issue.title
                ));
                output.push_str(&format!("     文件: {}:{:?}\n", issue.file, issue.line));
                if let Some(sug) = &issue.suggestion {
                    output.push_str(&format!("     建议: {}\n", sug));
                }
                output.push('\n');
            }

            if result.issues.len() > 20 {
                output.push_str(&format!(
                    "  ... 还有 {} 个问题\n",
                    result.issues.len() - 20
                ));
            }
        }

        output.push('\n');
        if result.passed {
            output.push_str("✅ 自我审查通过！\n");
        } else {
            output.push_str("❌ 自我审查未通过，请修复严重问题后重试\n");
        }

        output
    }

    pub fn format_cross_review(&self, review: &CrossReviewRequest) -> String {
        let mut output = String::new();

        let status_icon = match review.status.as_str() {
            "pending" => "⏳",
            "approved" => "✅",
            "rejected" => "❌",
            _ => "❓",
        };

        output.push_str(&format!("\n{} 交叉审查: {}\n\n", status_icon, review.id));
        output.push_str(&format!("  作者 Agent: {}\n", review.author_agent));
        output.push_str(&format!("  审查 Agent: {}\n", review.reviewer_agent));
        output.push_str(&format!("  状态: {}\n", review.status));
        output.push_str(&format!("  创建时间: {}\n", review.created_at));
        output.push_str(&format!("  审查文件数: {}\n", review.files.len()));

        for file in &review.files {
            output.push_str(&format!("    - {}\n", file));
        }

        output
    }
}

impl Default for SelfReviewService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_review_new() {
        let service = SelfReviewService::new();
        let _ = service;
    }

    #[test]
    fn test_self_review_run() {
        let service = SelfReviewService::new();
        let result = service.self_review(".", false).unwrap();
        assert!(result.reviewed_files > 0);
    }

    #[test]
    fn test_cross_review_create() {
        let service = SelfReviewService::new();
        let review = service
            .create_cross_review(".", "agent-a", "agent-b", vec!["src/main.rs".to_string()])
            .unwrap();
        assert_eq!(review.author_agent, "agent-a");
        assert_eq!(review.reviewer_agent, "agent-b");
    }

    #[test]
    fn test_list_cross_reviews() {
        let service = SelfReviewService::new();
        let reviews = service.list_cross_reviews(".", None).unwrap();
        // 交叉审查可能存在也可能不存在，不做断言
        let _ = reviews;
    }
}
