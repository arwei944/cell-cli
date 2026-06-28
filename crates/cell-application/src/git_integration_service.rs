use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub is_clean: bool,
    pub staged_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub deleted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub last_commit: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStat {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub files: Vec<FileDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file: String,
    pub insertions: usize,
    pub deletions: usize,
    pub status: DiffStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Unknown,
}

impl DiffStatus {
    pub fn from_char(c: char) -> Self {
        match c {
            'A' => Self::Added,
            'M' => Self::Modified,
            'D' => Self::Deleted,
            'R' => Self::Renamed,
            _ => Self::Unknown,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Added => "新增",
            Self::Modified => "修改",
            Self::Deleted => "删除",
            Self::Renamed => "重命名",
            Self::Unknown => "未知",
        }
    }
}

pub struct GitIntegrationService;

impl GitIntegrationService {
    pub fn new() -> Self {
        Self
    }

    pub fn is_git_repo(&self, project_path: &str) -> bool {
        self.run_git(project_path, &["rev-parse", "--git-dir"]).is_ok()
    }

    pub fn get_status(&self, project_path: &str) -> CellResult<GitStatus> {
        let branch = self.get_current_branch(project_path)?;
        
        let porcelain = self.run_git(project_path, &["status", "--porcelain"])?;
        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();
        let mut deleted = Vec::new();

        for line in porcelain.lines() {
            if line.len() < 2 {
                continue;
            }
            let file = line[2..].trim().to_string();
            let status_char = line.chars().next().unwrap_or(' ');
            let worktree_char = line.chars().nth(1).unwrap_or(' ');

            match status_char {
                'A' | 'M' | 'D' | 'R' => staged.push(file.clone()),
                _ => {}
            }
            match worktree_char {
                'M' => modified.push(file.clone()),
                'D' => deleted.push(file.clone()),
                _ => {}
            }
            if status_char == '?' && worktree_char == '?' {
                untracked.push(file);
            }
        }

        let is_clean = staged.is_empty() && modified.is_empty() && untracked.is_empty() && deleted.is_empty();

        Ok(GitStatus {
            branch,
            is_clean,
            staged_files: staged,
            modified_files: modified,
            untracked_files: untracked,
            deleted_files: deleted,
        })
    }

    pub fn get_current_branch(&self, project_path: &str) -> CellResult<String> {
        let output = self.run_git(project_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(output.trim().to_string())
    }

    pub fn list_branches(&self, project_path: &str) -> CellResult<Vec<BranchInfo>> {
        let output = self.run_git(project_path, &["branch", "-vv"])?;
        let mut branches = Vec::new();

        for line in output.lines() {
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }

            let is_current = line.starts_with('*');
            let name_start = if is_current { 2 } else { 0 };
            let rest = &line[name_start..];
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            let name = parts[0].trim().to_string();
            
            let last_commit = if parts.len() > 1 {
                let commit_part = parts[1].trim();
                let commit_parts: Vec<&str> = commit_part.splitn(2, ' ').collect();
                Some(commit_parts[0].to_string())
            } else {
                None
            };

            let mut ahead = 0;
            let mut behind = 0;
            if let Ok(sync) = self.run_git(project_path, &["rev-list", "--left-right", "--count", format!("{name}...@{{u}}").as_str()]) {
                let counts: Vec<&str> = sync.trim().split('\t').collect();
                if counts.len() == 2 {
                    ahead = counts[0].parse().unwrap_or(0);
                    behind = counts[1].parse().unwrap_or(0);
                }
            }

            branches.push(BranchInfo {
                name,
                is_current,
                last_commit,
                ahead,
                behind,
            });
        }

        Ok(branches)
    }

    pub fn get_recent_commits(&self, project_path: &str, limit: usize) -> CellResult<Vec<GitCommit>> {
        let format = "%H%n%h%n%s%n%an%n%aI%n---";
        let output = self.run_git(
            project_path,
            &["log", &format!("-{limit}"), &format!("--pretty=format:{format}")],
        )?;

        let mut commits = Vec::new();
        let mut blocks = output.split("---");

        for block in blocks.by_ref() {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 5 {
                continue;
            }

            let mut files_changed = Vec::new();
            if let Ok(name_only) = self.run_git(
                project_path,
                &["show", "--name-only", "--pretty=format:", lines[0]],
            ) {
                files_changed = name_only
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(std::string::ToString::to_string)
                    .collect();
            }

            commits.push(GitCommit {
                hash: lines[0].to_string(),
                short_hash: lines[1].to_string(),
                message: lines[2].to_string(),
                author: lines[3].to_string(),
                date: lines[4].to_string(),
                files_changed,
            });
        }

        Ok(commits)
    }

    pub fn get_diff_stats(&self, project_path: &str, target: Option<&str>) -> CellResult<DiffStat> {
        let args = if let Some(t) = target {
            vec!["diff", "--stat", t]
        } else {
            vec!["diff", "--stat"]
        };

        let stat_output = self.run_git(project_path, &args)?;
        let name_status_args = if let Some(t) = target {
            vec!["diff", "--name-status", t]
        } else {
            vec!["diff", "--name-status"]
        };
        let name_status = self.run_git(project_path, &name_status_args)?;

        let mut files = Vec::new();
        let mut total_insertions = 0;
        let mut total_deletions = 0;

        for line in name_status.lines() {
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() < 2 {
                continue;
            }
            let status_char = parts[0].chars().next().unwrap_or(' ');
            let file = parts[1].to_string();
            
            files.push(FileDiff {
                file,
                insertions: 0,
                deletions: 0,
                status: DiffStatus::from_char(status_char),
            });
        }

        let last_line = stat_output.lines().last().unwrap_or("");
        let parts: Vec<&str> = last_line.split(',').collect();
        let files_changed = if let Some(p) = parts.first() {
            p.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0)
        } else {
            0
        };

        for part in &parts[1..] {
            let part = part.trim();
            if part.contains('+') {
                total_insertions = part
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if part.contains('-') {
                total_deletions = part
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }

        Ok(DiffStat {
            files_changed: if files_changed > 0 { files_changed } else { files.len() },
            insertions: total_insertions,
            deletions: total_deletions,
            files,
        })
    }

    pub fn install_hooks(&self, project_path: &str) -> CellResult<Vec<String>> {
        let hooks_dir = Path::new(project_path).join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir)?;

        let mut installed = Vec::new();

        let pre_commit = r#"#!/bin/sh
# Cell Architecture pre-commit hook
# 运行快速验证和架构检查

cell verify --fast 2>/dev/null || {
    echo "❌ 快速验证失败，请修复后再提交"
    exit 1
}

cell arch validate 2>/dev/null || {
    echo "⚠️  架构验证存在违规，但不阻止提交"
}

exit 0
"#;

        let pre_commit_path = hooks_dir.join("pre-commit");
        std::fs::write(&pre_commit_path, pre_commit)?;
        self.make_executable(&pre_commit_path)?;
        installed.push("pre-commit".to_string());

        let commit_msg = r#"#!/bin/sh
# Cell Architecture commit-msg hook
# 检查提交信息格式

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(head -1 "$COMMIT_MSG_FILE")

# 简单检查: 非空且有一定长度
if [ ${#COMMIT_MSG} -lt 3 ]; then
    echo "❌ 提交信息太短，至少3个字符"
    exit 1
fi

exit 0
"#;

        let commit_msg_path = hooks_dir.join("commit-msg");
        std::fs::write(&commit_msg_path, commit_msg)?;
        self.make_executable(&commit_msg_path)?;
        installed.push("commit-msg".to_string());

        Ok(installed)
    }

    fn make_executable(&self, path: &Path) -> CellResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms)?;
        }
        #[cfg(not(unix))]
        {
            let _ = path;
        }
        Ok(())
    }

    fn run_git(&self, project_path: &str, args: &[&str]) -> CellResult<String> {
        let output = Command::new("git")
            .current_dir(project_path)
            .args(args)
            .output()
            .map_err(|e| cell_domain::errors::CellError::Config(format!("git command failed: {e}")))?;

        if !output.status.success() {
            return Err(cell_domain::errors::CellError::Config(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for GitIntegrationService {
    fn default() -> Self {
        Self::new()
    }
}
