use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevEnvStatus {
    pub agent_id: String,
    pub agent_role: String,
    pub current_phase: String,
    pub architecture_healthy: bool,
    pub entropy_score: f64,
    pub entropy_level: String,
    pub entropy_trend: String,
    pub pending_tasks: usize,
    pub active_features: Vec<String>,
    pub pending_decisions: usize,
    pub git_branch: String,
    pub git_dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextStepSuggestion {
    pub priority: u8,
    pub title: String,
    pub command: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub project_name: String,
    pub architecture_summary: String,
    pub key_decisions: Vec<String>,
    pub current_tasks: Vec<String>,
    pub active_features: Vec<String>,
    pub known_issues: Vec<String>,
    pub next_steps: Vec<String>,
    pub generated_at: String,
}

pub struct DevEnvService;

impl DevEnvService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_status(&self, project_path: &str) -> CellResult<DevEnvStatus> {
        let cell_dir = Path::new(project_path).join(".cell");

        let agent_id = std::fs::read_to_string(cell_dir.join("current_agent"))
            .unwrap_or_else(|_| "unregistered".to_string());

        let baseline_file = cell_dir.join("entropy_baseline.json");
        let (entropy_score, entropy_level) = if baseline_file.exists() {
            let content = std::fs::read_to_string(&baseline_file)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            (
                json["overall_score"].as_f64().unwrap_or(0.0),
                json["overall_level"]
                    .as_str()
                    .unwrap_or("Unknown")
                    .to_string(),
            )
        } else {
            (0.0, "Unknown".to_string())
        };

        let pending_tasks = if cell_dir.join("tasks").exists() {
            cell_dir.join("tasks").read_dir()?.count()
        } else {
            0
        };

        let pending_decisions = if cell_dir.join("decisions").exists() {
            cell_dir.join("decisions").read_dir()?.count()
        } else {
            0
        };

        let (git_branch, git_dirty) = self.get_git_status(project_path);
        let current_phase = self.get_current_phase(project_path);

        let active_features = Vec::new();
        let architecture_healthy = entropy_score < 50.0;
        let entropy_trend = "stable".to_string();

        Ok(DevEnvStatus {
            agent_id,
            agent_role: "Developer".to_string(),
            current_phase,
            architecture_healthy,
            entropy_score,
            entropy_level,
            entropy_trend,
            pending_tasks,
            active_features,
            pending_decisions,
            git_branch,
            git_dirty,
        })
    }

    fn get_current_phase(&self, project_path: &str) -> String {
        let progress_file = Path::new(project_path)
            .join(".cell")
            .join("progress")
            .join("current.json");

        if progress_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&progress_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(phase) = json["current_phase"].as_str() {
                        return phase.to_string();
                    }
                }
            }
        }
        "Idle".to_string()
    }

    fn get_git_status(&self, project_path: &str) -> (String, bool) {
        let branch = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(project_path)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let dirty = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(project_path)
            .output()
            .ok()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        (branch, dirty)
    }

    pub fn get_next_suggestions(&self, project_path: &str) -> CellResult<Vec<NextStepSuggestion>> {
        let status = self.get_status(project_path)?;
        let mut suggestions = Vec::new();

        if status.agent_id == "unregistered" {
            suggestions.push(NextStepSuggestion {
                priority: 1,
                title: "注册 Agent 身份".to_string(),
                command: "cell dev start".to_string(),
                reason: "尚未注册 Agent 身份，需要先初始化开发环境".to_string(),
            });
            return Ok(suggestions);
        }

        if status.current_phase == "Idle" {
            suggestions.push(NextStepSuggestion {
                priority: 1,
                title: "查看待处理任务".to_string(),
                command: "cell task list".to_string(),
                reason: "当前没有进行中的任务，先看看有什么活可以干".to_string(),
            });
            suggestions.push(NextStepSuggestion {
                priority: 2,
                title: "了解项目架构".to_string(),
                command: "cell arch status".to_string(),
                reason: "熟悉一下项目的架构现状和健康度".to_string(),
            });
        } else {
            match status.current_phase.as_str() {
                "Designing" => {
                    suggestions.push(NextStepSuggestion {
                        priority: 1,
                        title: "继续设计阶段".to_string(),
                        command: "cell dev design".to_string(),
                        reason: "当前处于设计阶段，完成架构设计后进入编码".to_string(),
                    });
                }
                "Coding" => {
                    suggestions.push(NextStepSuggestion {
                        priority: 1,
                        title: "代码检查点".to_string(),
                        command: "cell dev checkpoint".to_string(),
                        reason: "编码一段时间了，跑一下检查看看质量".to_string(),
                    });
                    if status.git_dirty {
                        suggestions.push(NextStepSuggestion {
                            priority: 2,
                            title: "提交当前进度".to_string(),
                            command: "git add . && git commit".to_string(),
                            reason: "工作区有未提交的改动，建议先提交保存进度".to_string(),
                        });
                    }
                }
                "Verifying" => {
                    suggestions.push(NextStepSuggestion {
                        priority: 1,
                        title: "运行完整验证".to_string(),
                        command: "cell dev verify --deep".to_string(),
                        reason: "当前处于验证阶段，运行深度验证确保质量".to_string(),
                    });
                }
                _ => {
                    suggestions.push(NextStepSuggestion {
                        priority: 1,
                        title: "查看工作流状态".to_string(),
                        command: "cell dev status".to_string(),
                        reason: "看看当前进展到哪一步了".to_string(),
                    });
                }
            }
        }

        if !status.architecture_healthy {
            suggestions.push(NextStepSuggestion {
                priority: 10,
                title: "修复架构问题".to_string(),
                command: "cell arch lint --fix".to_string(),
                reason: "架构健康度不达标，需要先修复违规".to_string(),
            });
        }

        suggestions.sort_by_key(|s| s.priority);
        Ok(suggestions)
    }

    pub fn generate_context_snapshot(&self, project_path: &str) -> CellResult<ContextSnapshot> {
        let status = self.get_status(project_path)?;
        let cell_dir = Path::new(project_path).join(".cell");

        let project_name = Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut key_decisions = Vec::new();
        let decisions_dir = cell_dir.join("decisions");
        if decisions_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&decisions_dir) {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(title) = json["title"].as_str() {
                                key_decisions.push(title.to_string());
                            }
                        }
                    }
                    if key_decisions.len() >= 10 {
                        break;
                    }
                }
            }
        }

        let mut current_tasks = Vec::new();
        let tasks_dir = cell_dir.join("tasks");
        if tasks_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(title) = json["title"].as_str() {
                                current_tasks.push(title.to_string());
                            }
                        }
                    }
                    if current_tasks.len() >= 10 {
                        break;
                    }
                }
            }
        }

        let architecture_summary = format!(
            "熵值 {:.1} ({}), 架构健康: {}",
            status.entropy_score,
            status.entropy_level,
            if status.architecture_healthy { "是" } else { "否" }
        );

        let known_issues = if !status.architecture_healthy {
            vec!["架构健康度不达标，需要修复".to_string()]
        } else {
            Vec::new()
        };

        let next_steps = self
            .get_next_suggestions(project_path)?
            .iter()
            .map(|s| format!("{} - {}", s.title, s.command))
            .collect();

        Ok(ContextSnapshot {
            project_name,
            architecture_summary,
            key_decisions,
            current_tasks,
            active_features: status.active_features,
            known_issues,
            next_steps,
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn reset_environment(&self, project_path: &str, scope: &str) -> CellResult<()> {
        let cell_dir = Path::new(project_path).join(".cell");

        match scope {
            "all" => {
                if cell_dir.exists() {
                    std::fs::remove_dir_all(&cell_dir)?;
                }
                println!("🗑️  已清除所有运行时数据");
            }
            "agent" => {
                let agent_dir = cell_dir.join("agents");
                if agent_dir.exists() {
                    std::fs::remove_dir_all(&agent_dir)?;
                }
                let current_file = cell_dir.join("current_agent");
                if current_file.exists() {
                    std::fs::remove_file(&current_file)?;
                }
                println!("🗑️  已清除 Agent 注册信息");
            }
            "progress" => {
                let progress_dir = cell_dir.join("progress");
                if progress_dir.exists() {
                    std::fs::remove_dir_all(&progress_dir)?;
                }
                println!("🗑️  已清除进度数据");
            }
            _ => {
                println!("⚠️  未知的重置范围: {}", scope);
            }
        }

        Ok(())
    }

    pub fn format_status(&self, status: &DevEnvStatus) -> String {
        let mut output = String::new();

        output.push_str("\n📊 开发环境状态\n\n");

        output.push_str(&format!("  🤖 Agent: {} ({})\n", status.agent_id, status.agent_role));
        output.push_str(&format!("  📋 当前阶段: {}\n", status.current_phase));

        let health_icon = if status.architecture_healthy { "✅" } else { "❌" };
        output.push_str(&format!(
            "  🏗️  架构状态: {} 健康\n",
            health_icon
        ));

        output.push_str(&format!(
            "  📊 熵值: {:.1} ({}) [{}]\n",
            status.entropy_score, status.entropy_level, status.entropy_trend
        ));

        output.push_str(&format!("  📝 待处理任务: {}\n", status.pending_tasks));
        output.push_str(&format!("  🤔 待决策: {}\n", status.pending_decisions));
        output.push_str(&format!(
            "  🌿 Git: {}{}\n",
            status.git_branch,
            if status.git_dirty { " (有未提交改动)" } else { "" }
        ));

        output
    }

    pub fn format_suggestions(&self, suggestions: &[NextStepSuggestion]) -> String {
        let mut output = String::new();

        output.push_str("\n💡 下一步建议\n\n");

        for (i, s) in suggestions.iter().enumerate() {
            output.push_str(&format!(
                "  {}. {} [优先级: {}]\n",
                i + 1,
                s.title,
                s.priority
            ));
            output.push_str(&format!("     命令: {}\n", s.command));
            output.push_str(&format!("     原因: {}\n\n", s.reason));
        }

        output
    }

    pub fn format_context_snapshot(&self, snapshot: &ContextSnapshot) -> String {
        let mut output = String::new();

        output.push_str(&format!("# 项目上下文快照: {}\n\n", snapshot.project_name));
        output.push_str(&format!(
            "> 生成时间: {}\n\n",
            snapshot.generated_at
        ));

        output.push_str("## 架构概览\n\n");
        output.push_str(&format!("- {}\n\n", snapshot.architecture_summary));

        output.push_str("## 关键决策\n\n");
        if snapshot.key_decisions.is_empty() {
            output.push_str("- 暂无记录\n");
        } else {
            for d in &snapshot.key_decisions {
                output.push_str(&format!("- {}\n", d));
            }
        }
        output.push('\n');

        output.push_str("## 当前任务\n\n");
        if snapshot.current_tasks.is_empty() {
            output.push_str("- 暂无任务\n");
        } else {
            for t in &snapshot.current_tasks {
                output.push_str(&format!("- {}\n", t));
            }
        }
        output.push('\n');

        if !snapshot.known_issues.is_empty() {
            output.push_str("## 已知问题\n\n");
            for issue in &snapshot.known_issues {
                output.push_str(&format!("- ⚠️  {}\n", issue));
            }
            output.push('\n');
        }

        output.push_str("## 下一步建议\n\n");
        for (i, s) in snapshot.next_steps.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, s));
        }

        output
    }
}

impl Default for DevEnvService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_status() {
        let service = DevEnvService::new();
        let status = service.get_status(".").unwrap();
        assert!(!status.agent_id.is_empty());
    }

    #[test]
    fn test_next_suggestions() {
        let service = DevEnvService::new();
        let suggestions = service.get_next_suggestions(".").unwrap();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_context_snapshot() {
        let service = DevEnvService::new();
        let snapshot = service.generate_context_snapshot(".").unwrap();
        assert!(!snapshot.project_name.is_empty());
    }
}
