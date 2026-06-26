use crate::application::git_integration_service::GitIntegrationService;
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;

pub fn cmd_git(args: GitArgs) -> CellResult<()> {
    let service = GitIntegrationService::new();
    let project_path = ".";

    match args.sub {
        GitSub::Status {} => {
            let status = service.get_status(project_path)?;
            
            println!("\n📊 Git 状态\n");
            println!("  分支: {}", status.branch);
            println!("  工作区: {}", if status.is_clean { "✅ 干净" } else { "⚠️  有变更" });
            println!();

            if !status.staged_files.is_empty() {
                println!("  暂存文件 ({}):", status.staged_files.len());
                for f in &status.staged_files {
                    println!("    📌 {}", f);
                }
                println!();
            }
            if !status.modified_files.is_empty() {
                println!("  修改文件 ({}):", status.modified_files.len());
                for f in &status.modified_files {
                    println!("    ✏️  {}", f);
                }
                println!();
            }
            if !status.deleted_files.is_empty() {
                println!("  删除文件 ({}):", status.deleted_files.len());
                for f in &status.deleted_files {
                    println!("    🗑️  {}", f);
                }
                println!();
            }
            if !status.untracked_files.is_empty() {
                println!("  未追踪文件 ({}):", status.untracked_files.len());
                for f in &status.untracked_files {
                    println!("    ❓ {}", f);
                }
                println!();
            }
        }
        GitSub::Branches {} => {
            let branches = service.list_branches(project_path)?;
            
            println!("\n🌿 分支列表\n");
            for b in &branches {
                let current = if b.is_current { "*" } else { " " };
                let commit = b.last_commit.as_deref().unwrap_or("-");
                let sync = if b.ahead > 0 || b.behind > 0 {
                    format!(" ↑{} ↓{}", b.ahead, b.behind)
                } else {
                    String::new()
                };
                println!("  {} {:<30} {}{}", current, b.name, commit, sync);
            }
            println!();
        }
        GitSub::Log { count } => {
            let commits = service.get_recent_commits(project_path, count.unwrap_or(10))?;
            
            println!("\n📜 提交历史\n");
            for commit in &commits {
                println!("  commit {}", commit.hash);
                println!("  Author: {}", commit.author);
                println!("  Date:   {}", commit.date);
                println!();
                println!("      {}", commit.message);
                if !commit.files_changed.is_empty() {
                    println!();
                    println!("      变更文件: {}", commit.files_changed.len());
                }
                println!();
            }
        }
        GitSub::Diff { target } => {
            let diff = service.get_diff_stats(project_path, target.as_deref())?;
            
            println!("\n📈 变更统计\n");
            println!("  变更文件: {} 个", diff.files_changed);
            println!("  新增行数: +{}", diff.insertions);
            println!("  删除行数: -{}", diff.deletions);
            println!();

            if !diff.files.is_empty() {
                println!("  文件列表:");
                for f in &diff.files {
                    println!("    {} {}", f.status.label(), f.file);
                }
                println!();
            }
        }
        GitSub::Hooks { install } => {
            if install {
                match service.install_hooks(project_path) {
                    Ok(hooks) => {
                        println!("\n✅ Git Hook 安装成功\n");
                        for h in &hooks {
                            println!("    • {}", h);
                        }
                        println!();
                    }
                    Err(e) => {
                        println!("\n❌ 安装失败: {}\n", e);
                    }
                }
            } else {
                println!("\n🔧 Git Hook 管理\n");
                println!("  使用 --install 安装 Git Hook");
                println!();
                println!("  包含的 Hook:");
                println!("    • pre-commit  - 提交前运行快速验证");
                println!("    • commit-msg  - 提交信息格式检查");
                println!();
            }
        }
    }

    Ok(())
}
