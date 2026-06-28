use crate::decision::DecisionRecord;
use crate::progress::ProgressLog;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandoffPackage {
    pub package_id: Uuid,
    pub project_name: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<String>,

    pub project_overview: ProjectOverview,
    pub current_task: TaskContext,
    pub architecture_snapshot: ArchitectureSnapshot,
    pub entropy_snapshot: EntropySnapshot,
    pub progress: Option<ProgressLog>,
    pub related_files: Vec<FileSummary>,
    pub recent_files: Vec<RecentFileInfo>,
    pub open_questions: Vec<String>,
    pub decisions: Vec<DecisionRecord>,
    pub next_actions: Vec<NextActionItem>,
    pub environment_info: EnvironmentInfo,
    pub development_rules: Vec<String>,
    pub quick_start: Vec<String>,
    pub validation: HandoffValidation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectOverview {
    pub name: String,
    pub description: String,
    pub architecture_style: String,
    pub tech_stack: Vec<String>,
    pub key_directories: Vec<DirectoryInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryInfo {
    pub path: String,
    pub purpose: String,
    pub file_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskContext {
    pub name: String,
    pub description: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchitectureSnapshot {
    pub layers: Vec<LayerInfo>,
    pub total_violations: usize,
    pub domain_external_deps: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayerInfo {
    pub name: String,
    pub file_count: usize,
    pub internal_deps: usize,
    pub external_deps: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropySnapshot {
    pub overall_score: f64,
    pub threshold: f64,
    pub dimensions: std::collections::HashMap<String, f64>,
    pub file_count: usize,
    pub total_lines: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileSummary {
    pub path: String,
    pub role: String,
    pub description: String,
    pub lines_of_code: usize,
    pub modification_count: u32,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentFileInfo {
    pub path: String,
    pub last_modified: DateTime<Utc>,
    pub lines_of_code: usize,
    pub change_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NextActionItem {
    pub id: Uuid,
    pub description: String,
    pub priority: u8,
    pub estimated_minutes: Option<u32>,
    pub prerequisites: Vec<String>,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvironmentInfo {
    pub rust_version: Option<String>,
    pub dependencies: Vec<DependencyInfo>,
    pub build_status: BuildStatus,
    pub test_status: TestStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    Passing,
    Failing,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TestStatus {
    AllPassing,
    SomeFailing,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandoffValidation {
    pub is_complete: bool,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
}

impl HandoffPackage {
    pub fn new(project_name: &str) -> Self {
        let now = Utc::now();
        Self {
            package_id: Uuid::new_v4(),
            project_name: project_name.to_string(),
            generated_at: now,
            generated_by: None,
            project_overview: ProjectOverview {
                name: project_name.to_string(),
                description: String::new(),
                architecture_style: "Cell Architecture (Hexagonal + DDD)".to_string(),
                tech_stack: Vec::new(),
                key_directories: Vec::new(),
            },
            current_task: TaskContext {
                name: String::new(),
                description: String::new(),
                status: "unknown".to_string(),
                started_at: None,
                last_activity_at: None,
            },
            architecture_snapshot: ArchitectureSnapshot {
                layers: Vec::new(),
                total_violations: 0,
                domain_external_deps: 0,
            },
            entropy_snapshot: EntropySnapshot {
                overall_score: 0.0,
                threshold: 5.0,
                dimensions: std::collections::HashMap::new(),
                file_count: 0,
                total_lines: 0,
            },
            progress: None,
            related_files: Vec::new(),
            recent_files: Vec::new(),
            open_questions: Vec::new(),
            decisions: Vec::new(),
            next_actions: Vec::new(),
            environment_info: EnvironmentInfo {
                rust_version: None,
                dependencies: Vec::new(),
                build_status: BuildStatus::Unknown,
                test_status: TestStatus::Unknown,
            },
            development_rules: Vec::new(),
            quick_start: Vec::new(),
            validation: HandoffValidation {
                is_complete: false,
                missing_fields: Vec::new(),
                warnings: Vec::new(),
            },
        }
    }

    pub fn validate(&mut self) -> &HandoffValidation {
        let mut missing = Vec::new();
        let mut warnings = Vec::new();

        if self.current_task.name.is_empty() {
            missing.push("current_task.name".to_string());
        }
        if self.next_actions.is_empty() {
            warnings.push("No next actions defined - handoff recipient may not know what to do next".to_string());
        }
        if self.related_files.is_empty() {
            warnings.push("No related files listed - recipient won't know which files to look at".to_string());
        }
        if self.architecture_snapshot.total_violations > 0 {
            warnings.push(format!(
                "Architecture has {} violations - should be fixed before handoff",
                self.architecture_snapshot.total_violations
            ));
        }
        if self.entropy_snapshot.overall_score > self.entropy_snapshot.threshold {
            warnings.push(format!(
                "Entropy score {:.2} exceeds threshold {:.2} - system complexity is too high",
                self.entropy_snapshot.overall_score, self.entropy_snapshot.threshold
            ));
        }

        self.validation = HandoffValidation {
            is_complete: missing.is_empty(),
            missing_fields: missing,
            warnings,
        };
        &self.validation
    }

    #[allow(clippy::too_many_lines)]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# 交接包: {}\n\n", self.project_name));
        md.push_str(&format!("生成时间: {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        if let Some(by) = &self.generated_by {
            md.push_str(&format!("生成者: {by}\n\n"));
        }

        md.push_str("## 1. 项目概览\n\n");
        md.push_str(&format!("- **架构风格**: {}\n", self.project_overview.architecture_style));
        md.push_str(&format!("- **技术栈**: {}\n", self.project_overview.tech_stack.join(", ")));
        md.push_str("\n### 关键目录\n\n");
        for dir in &self.project_overview.key_directories {
            md.push_str(&format!("- `{}`: {} ({} files)\n", dir.path, dir.purpose, dir.file_count));
        }

        md.push_str("\n## 2. 当前任务\n\n");
        md.push_str(&format!("- **名称**: {}\n", self.current_task.name));
        md.push_str(&format!("- **状态**: {}\n", self.current_task.status));
        md.push_str(&format!("- **描述**: {}\n\n", self.current_task.description));

        if let Some(progress) = &self.progress {
            md.push_str("## 3. 进度时间线\n\n");
            for event in &progress.timeline {
                md.push_str(&format!(
                    "- [{}] **{:?}**: {}\n",
                    event.timestamp.format("%H:%M:%S"),
                    event.event_type,
                    event.message
                ));
            }
            md.push('\n');

            if !progress.blockers.is_empty() {
                md.push_str("### 阻塞问题\n\n");
                for blocker in &progress.blockers {
                    let status = match blocker.status {
                        crate::progress::BlockerStatus::Active => "🔴 活跃",
                        crate::progress::BlockerStatus::Resolved => "🟢 已解决",
                        crate::progress::BlockerStatus::Bypassed => "🟡 已绕过",
                    };
                    md.push_str(&format!("- {}: {} ({})\n", status, blocker.description, blocker.id));
                }
                md.push('\n');
            }
        }

        md.push_str("## 4. 架构快照\n\n");
        md.push_str(&format!(
            "- **违规数量**: {}\n",
            self.architecture_snapshot.total_violations
        ));
        md.push_str(&format!(
            "- **Domain 层外部依赖**: {}\n\n",
            self.architecture_snapshot.domain_external_deps
        ));
        for layer in &self.architecture_snapshot.layers {
            md.push_str(&format!(
                "- **{}**: {} files, {} internal deps, {} external deps\n",
                layer.name, layer.file_count, layer.internal_deps, layer.external_deps
            ));
        }

        md.push_str("\n## 5. 熵值快照\n\n");
        md.push_str(&format!(
            "- **总分**: {:.2} (阈值: {:.2})\n",
            self.entropy_snapshot.overall_score, self.entropy_snapshot.threshold
        ));
        md.push_str(&format!("- **文件数**: {}\n", self.entropy_snapshot.file_count));
        md.push_str(&format!("- **总行数**: {}\n\n", self.entropy_snapshot.total_lines));

        md.push_str("## 6. 相关文件\n\n");
        for f in &self.related_files {
            md.push_str(&format!(
                "- `{}` [{}]: {} ({} LOC)\n",
                f.path, f.role, f.description, f.lines_of_code
            ));
        }

        md.push_str("\n## 7. 下一步行动\n\n");
        for (i, action) in self.next_actions.iter().enumerate() {
            md.push_str(&format!("{}. **{}** (优先级: {})\n", i + 1, action.description, action.priority));
            if let Some(est) = action.estimated_minutes {
                md.push_str(&format!("   - 预估时间: {est} 分钟\n"));
            }
            if !action.acceptance_criteria.is_empty() {
                md.push_str("   - 验收标准:\n");
                for criteria in &action.acceptance_criteria {
                    md.push_str(&format!("     - [ ] {criteria}\n"));
                }
            }
        }

        if !self.open_questions.is_empty() {
            md.push_str("\n## 8. 待解决问题\n\n");
            for q in &self.open_questions {
                md.push_str(&format!("- [ ] {q}\n"));
            }
        }

        if !self.decisions.is_empty() {
            md.push_str("\n## 9. 决策记录 (ADR)\n\n");
            for adr in &self.decisions {
                use crate::decision::DecisionStatus;
                let status_str = match adr.status {
                    DecisionStatus::Proposed => "提议中",
                    DecisionStatus::Accepted => "已接受",
                    DecisionStatus::Rejected => "已拒绝",
                    DecisionStatus::Deprecated => "已废弃",
                    DecisionStatus::Superseded => "已替代",
                };
                md.push_str(&format!("- **{}** [{}]: {}\n", adr.id.simple(), status_str, adr.title));
                md.push_str(&format!("  - 分类: {}\n", adr.category.label()));
                md.push_str(&format!("  - 理由: {}\n", adr.rationale));
                if !adr.alternatives.is_empty() {
                    md.push_str("  - 备选方案:\n");
                    for alt in &adr.alternatives {
                        md.push_str(&format!("    - {}\n", alt.name));
                    }
                }
            }
        }

        md.push_str("\n## 10. 环境信息\n\n");
        if let Some(rust_ver) = &self.environment_info.rust_version {
            md.push_str(&format!("- **Rust 版本**: {rust_ver}\n"));
        }
        let build_str = match self.environment_info.build_status {
            BuildStatus::Passing => "✅ 通过",
            BuildStatus::Failing => "❌ 失败",
            BuildStatus::Unknown => "❓ 未知",
        };
        let test_str = match self.environment_info.test_status {
            TestStatus::AllPassing => "✅ 全部通过",
            TestStatus::SomeFailing => "❌ 部分失败",
            TestStatus::Unknown => "❓ 未知",
        };
        md.push_str(&format!("- **构建状态**: {build_str}\n"));
        md.push_str(&format!("- **测试状态**: {test_str}\n"));

        if !self.development_rules.is_empty() {
            md.push_str("\n## 11. 开发规范与约束\n\n");
            for (i, rule) in self.development_rules.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, rule));
            }
        }

        if !self.quick_start.is_empty() {
            md.push_str("\n## 12. 快速上手指南\n\n");
            for step in &self.quick_start {
                md.push_str(&format!("- {step}\n"));
            }
        }

        if !self.recent_files.is_empty() {
            md.push_str("\n## 13. 最近修改文件\n\n");
            md.push_str("| 文件 | 修改时间 | 行数 | 类型 |\n");
            md.push_str("|------|----------|------|------|\n");
            for f in &self.recent_files {
                md.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    f.path,
                    f.last_modified.format("%Y-%m-%d %H:%M"),
                    f.lines_of_code,
                    f.change_type
                ));
            }
        }

        md.push_str("\n## 14. 交接包验证\n\n");
        if self.validation.is_complete {
            md.push_str("✅ 交接包完整，可以开始接手\n\n");
        } else {
            md.push_str("⚠️  交接包不完整，缺少以下字段:\n\n");
            for field in &self.validation.missing_fields {
                md.push_str(&format!("- ❌ {field}\n"));
            }
            md.push('\n');
        }
        if !self.validation.warnings.is_empty() {
            md.push_str("### 警告\n\n");
            for w in &self.validation.warnings {
                md.push_str(&format!("- ⚠️  {w}\n"));
            }
        }

        md
    }
}
