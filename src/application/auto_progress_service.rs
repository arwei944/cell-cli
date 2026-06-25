use crate::domain::errors::CellResult;
use crate::domain::progress::{EventType, ProgressLog, ProgressStatus, NextStep};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

pub struct AutoProgressTracker {
    last_commit_hash: Option<String>,
    last_entropy_score: Option<f64>,
    last_file_states: HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitState {
    pub branch: String,
    pub commit_hash: String,
    pub commit_message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub changed_files: Vec<ChangedFile>,
    pub uncommitted_changes: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub change_type: FileChangeType,
    pub lines_added: i32,
    pub lines_removed: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutoProgressSnapshot {
    pub timestamp: DateTime<Utc>,
    pub entropy_score: f64,
    pub entropy_change: f64,
    pub files_modified: usize,
    pub lines_changed: i32,
    pub test_status: TestStatus,
    pub build_status: BuildStatus,
    pub active_blockers: usize,
    pub completed_steps: usize,
    pub pending_steps: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TestStatus {
    Unknown,
    AllPassing,
    SomeFailing,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BuildStatus {
    Unknown,
    Passing,
    Failing,
}

impl AutoProgressTracker {
    pub fn new() -> Self {
        Self {
            last_commit_hash: None,
            last_entropy_score: None,
            last_file_states: HashMap::new(),
        }
    }

    pub fn analyze_current_state(&mut self, project_path: &str) -> CellResult<(ProgressLog, AutoProgressSnapshot)> {
        let git_state = self.get_git_state(project_path)?;
        let file_states = self.scan_file_states(project_path)?;

        let (status, blockers, next_steps) = self.infer_progress_state(project_path, &git_state, &file_states)?;

        let mut log = ProgressLog::new(
            &self.infer_task_name(&git_state),
            &self.infer_task_description(&git_state),
        );
        log.status = status.clone();
        log.start(Some("auto-tracker"));

        for blocker in &blockers {
            log.add_blocker(&blocker.description);
        }
        for step in &next_steps {
            log.add_next_step(&step.description, step.priority, step.estimated_minutes);
        }

        if git_state.uncommitted_changes {
            log.add_event(EventType::FileModified, "检测到未提交的更改", None);
        }

        for file in git_state.changed_files.iter().take(5) {
            log.add_related_file(&file.path);
        }

        let snapshot = AutoProgressSnapshot {
            timestamp: Utc::now(),
            entropy_score: self.last_entropy_score.unwrap_or(0.0),
            entropy_change: 0.0,
            files_modified: git_state.changed_files.len(),
            lines_changed: git_state.changed_files.iter()
                .map(|f| f.lines_added + f.lines_removed)
                .sum(),
            test_status: self.check_test_status(project_path),
            build_status: self.check_build_status(project_path),
            active_blockers: blockers.len(),
            completed_steps: next_steps.iter().filter(|s| s.done).count(),
            pending_steps: next_steps.iter().filter(|s| !s.done).count(),
        };

        self.last_file_states = file_states;
        self.last_commit_hash = Some(git_state.commit_hash.clone());

        Ok((log, snapshot))
    }

    fn get_git_state(&self, project_path: &str) -> CellResult<GitState> {
        let git_dir = Path::new(project_path).join(".git");
        if !git_dir.exists() {
            return Err(crate::domain::errors::CellError::NotFound(
                "Not a git repository".to_string(),
            ));
        }

        let branch = self.run_git_command(project_path, &["branch", "--show-current"])
            .unwrap_or_else(|_| "unknown".to_string());

        let commit_hash = self.run_git_command(project_path, &["rev-parse", "HEAD"])
            .map(|s| s.chars().take(8).collect())
            .unwrap_or_else(|_| "unknown".to_string());

        let commit_message = self.run_git_command(project_path, &["log", "-1", "--format=%s"])
            .unwrap_or_else(|_| "No commits".to_string());

        let author = self.run_git_command(project_path, &["log", "-1", "--format=%an"])
            .unwrap_or_else(|_| "unknown".to_string());

        let timestamp_str = self.run_git_command(project_path, &["log", "-1", "--format=%aI"])
            .unwrap_or_else(|_| Utc::now().to_rfc3339());
        let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let diff_output = self.run_git_command(project_path, &["diff", "--stat", "HEAD"])
            .unwrap_or_default();
        let uncommitted_changes = !diff_output.trim().is_empty() && diff_output.contains("file changed");

        let status_output = self.run_git_command(project_path, &["status", "--porcelain"])
            .unwrap_or_default();

        let mut changed_files = Vec::new();
        for line in status_output.lines() {
            if line.len() >= 3 {
                let status_chars: String = line.chars().take(2).collect();
                let path = line[3..].trim().to_string();

                let change_type = match status_chars.as_str() {
                    "A " => FileChangeType::Added,
                    "M " | " M" => FileChangeType::Modified,
                    "D " => FileChangeType::Deleted,
                    "R " => FileChangeType::Renamed,
                    "??" => FileChangeType::Added,
                    _ => FileChangeType::Modified,
                };

                changed_files.push(ChangedFile {
                    path,
                    change_type,
                    lines_added: 0,
                    lines_removed: 0,
                });
            }
        }

        Ok(GitState {
            branch,
            commit_hash,
            commit_message,
            author,
            timestamp,
            changed_files,
            uncommitted_changes,
        })
    }

    fn run_git_command(&self, project_path: &str, args: &[&str]) -> CellResult<String> {
        use std::process::Command;
        let output = Command::new("git")
            .args(args)
            .current_dir(project_path)
            .output()
            .map_err(|e| crate::domain::errors::CellError::Other(format!("Git command failed: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(crate::domain::errors::CellError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    fn scan_file_states(&self, project_path: &str) -> CellResult<HashMap<String, u64>> {
        let mut states = HashMap::new();
        let src_dir = Path::new(project_path).join("src");

        if src_dir.exists() {
            self.scan_directory(&src_dir, &mut states)?;
        }

        Ok(states)
    }

    fn scan_directory(&self, dir: &Path, states: &mut HashMap<String, u64>) -> CellResult<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory(&path, states)?;
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            let duration = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            states.insert(path.to_string_lossy().to_string(), duration);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn infer_progress_state(
        &self,
        project_path: &str,
        git_state: &GitState,
        _file_states: &HashMap<String, u64>,
    ) -> CellResult<(ProgressStatus, Vec<crate::domain::progress::Blocker>, Vec<NextStep>)> {
        let mut status = ProgressStatus::InProgress;
        let blockers = Vec::new();
        let mut next_steps = Vec::new();

        if !git_state.uncommitted_changes && !git_state.changed_files.is_empty() {
            next_steps.push(NextStep {
                id: Uuid::new_v4(),
                description: "提交当前更改".to_string(),
                priority: 1,
                estimated_minutes: Some(2),
                done: false,
            });
        }

        let test_status = self.check_test_status(project_path);
        match test_status {
            TestStatus::SomeFailing => {
                status = ProgressStatus::Blocked;
                next_steps.push(NextStep {
                    id: Uuid::new_v4(),
                    description: "修复失败的测试".to_string(),
                    priority: 1,
                    estimated_minutes: Some(30),
                    done: false,
                });
            }
            TestStatus::AllPassing => {
                next_steps.push(NextStep {
                    id: Uuid::new_v4(),
                    description: "运行测试 - 全部通过".to_string(),
                    priority: 2,
                    estimated_minutes: Some(0),
                    done: true,
                });
            }
            TestStatus::Unknown => {}
        }

        let build_status = self.check_build_status(project_path);
        match build_status {
            BuildStatus::Failing => {
                status = ProgressStatus::Blocked;
                next_steps.push(NextStep {
                    id: Uuid::new_v4(),
                    description: "修复编译错误".to_string(),
                    priority: 1,
                    estimated_minutes: Some(15),
                    done: false,
                });
            }
            BuildStatus::Passing => {
                next_steps.push(NextStep {
                    id: Uuid::new_v4(),
                    description: "编译检查 - 通过".to_string(),
                    priority: 2,
                    estimated_minutes: Some(0),
                    done: true,
                });
            }
            BuildStatus::Unknown => {}
        }

        if git_state.branch != "main" && git_state.branch != "master" {
            next_steps.push(NextStep {
                id: Uuid::new_v4(),
                description: format!("合并分支到 main (当前: {})", git_state.branch),
                priority: 3,
                estimated_minutes: Some(10),
                done: false,
            });
        }

        let has_new_files: bool = git_state.changed_files.iter()
            .any(|f| matches!(f.change_type, FileChangeType::Added));
        if has_new_files {
            next_steps.push(NextStep {
                id: Uuid::new_v4(),
                description: "添加新功能的测试".to_string(),
                priority: 2,
                estimated_minutes: Some(20),
                done: false,
            });
        }

        Ok((status, blockers, next_steps))
    }

    fn infer_task_name(&self, git_state: &GitState) -> String {
        let msg = &git_state.commit_message;

        if msg.contains("feat:") {
            msg.split("feat:").nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "新功能开发".to_string())
        } else if msg.contains("fix:") {
            msg.split("fix:").nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "修复问题".to_string())
        } else if msg.contains("refactor:") {
            msg.split("refactor:").nth(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "代码重构".to_string())
        } else if msg == "No commits" || msg.is_empty() {
            "初始开发".to_string()
        } else {
            msg.chars().take(50).collect::<String>()
        }
    }

    fn infer_task_description(&self, git_state: &GitState) -> String {
        let mut desc = format!(
            "分支: {}, 作者: {}, 文件变更: {} 个",
            git_state.branch,
            git_state.author,
            git_state.changed_files.len()
        );

        if git_state.uncommitted_changes {
            desc.push_str(", 有未提交更改");
        }

        if !git_state.changed_files.is_empty() {
            let rust_files: Vec<_> = git_state.changed_files.iter()
                .filter(|f| f.path.ends_with(".rs"))
                .take(3)
                .collect();

            if !rust_files.is_empty() {
                desc.push_str("\n最近修改: ");
                for f in rust_files {
                    desc.push_str(&format!("{} ", f.path));
                }
            }
        }

        desc
    }

    fn check_test_status(&self, project_path: &str) -> TestStatus {
        use std::process::Command;

        let output = Command::new("cargo")
            .args(["test", "--lib", "--", "--test-threads=1", "--quiet"])
            .current_dir(project_path)
            .output();

        match output {
            Ok(o) if o.status.success() => TestStatus::AllPassing,
            Ok(_) => TestStatus::SomeFailing,
            Err(_) => TestStatus::Unknown,
        }
    }

    fn check_build_status(&self, project_path: &str) -> BuildStatus {
        use std::process::Command;

        let output = Command::new("cargo")
            .args(["check", "--quiet"])
            .current_dir(project_path)
            .output();

        match output {
            Ok(o) if o.status.success() => BuildStatus::Passing,
            Ok(_) => BuildStatus::Failing,
            Err(_) => BuildStatus::Unknown,
        }
    }
}

impl Default for AutoProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}
