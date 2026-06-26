use crate::adapters::file_decision_store::FileDecisionStore;
use crate::adapters::file_handoff_exporter::FileHandoffExporter;
use crate::adapters::file_progress_store::FileProgressStore;
use crate::application::entropy_service::run_entropy_check;
use crate::application::fast_verify_service::FastVerifyService;
use crate::application::handoff_service::HandoffService;
use crate::application::progress_bar::StepProgress;
use crate::application::progress_service::ProgressService;
use crate::domain::errors::{CellError, CellResult};
use crate::domain::progress::EventType;
use crate::interfaces::cli::*;
use std::path::Path;
use std::process::Command;

fn run_git_command(args: &[&str]) -> CellResult<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| CellError::Config(format!("Git command failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CellError::Config(format!("Git failed: {}", stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 使用 GitHub CLI 推送代码（避免凭证交互）
fn push_with_gh(branch: &str) -> CellResult<()> {
    // 使用 git push 配合 gh 的凭证助手
    let output = Command::new("git")
        .args(["push", "origin", branch])
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| CellError::Config(format!("Git push failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CellError::Config(format!("Git push failed: {}", stderr)));
    }
    Ok(())
}

/// 快速Git提交（只添加handoff目录）
fn fast_git_commit(message: &str) -> CellResult<()> {
    // 只添加 .handoff 目录
    let add_output = Command::new("git")
        .args(["add", ".handoff/"])
        .output()
        .map_err(|e| CellError::Config(format!("Git add failed: {}", e)))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr).trim().to_string();
        return Err(CellError::Config(format!("Git add failed: {}", stderr)));
    }

    // 检查是否有变更需要提交
    let status_output = Command::new("git")
        .args(["status", "--porcelain", ".handoff/"])
        .output()
        .map_err(|e| CellError::Config(format!("Git status failed: {}", e)))?;

    let status = String::from_utf8_lossy(&status_output.stdout).trim().to_string();

    if status.is_empty() {
        // 没有变更，跳过提交
        return Ok(());
    }

    // 快速提交
    let commit_output = Command::new("git")
        .args(["commit", "-m", message, "--no-gpg-sign", "--no-verify"])
        .output()
        .map_err(|e| CellError::Config(format!("Git commit failed: {}", e)))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr).trim().to_string();
        return Err(CellError::Config(format!("Git commit failed: {}", stderr)));
    }

    Ok(())
}

pub fn cmd_handoff(args: HandoffArgs) -> CellResult<()> {
    let exporter = FileHandoffExporter::new();
    let progress_store = FileProgressStore::new();
    let decision_store = FileDecisionStore::new();
    let handoff_service = HandoffService::new(exporter, progress_store.clone(), decision_store);

    match args.sub {
        HandoffSub::Generate { name, output, author, no_md, no_json } => {
            handle_handoff_generate(&handoff_service, name, output, author.as_deref(), no_md, no_json)?;
        }
        HandoffSub::Show { path } => {
            let pkg_path = path.unwrap_or_else(|| ".handoff/latest.json".to_string());
            let pkg = handoff_service.import(&pkg_path)?;
            println!("{}", pkg.to_markdown());
        }
        HandoffSub::Validate { path } => {
            let mut pkg = handoff_service.import(&path)?;
            let validation = pkg.validate();
            if validation.is_complete {
                println!("✅ 交接包验证通过 - 可以直接接手");
            } else {
                println!("❌ 交接包不完整 - 缺少以下字段:");
                for f in &validation.missing_fields {
                    println!("  - {}", f);
                }
            }
            if !validation.warnings.is_empty() {
                println!("\n⚠️  警告:");
                for w in &validation.warnings {
                    println!("  - {}", w);
                }
            }
        }
        HandoffSub::Commit { message, author, quick, no_push } => {
            handle_handoff_commit(&handoff_service, message, author.as_deref(), quick, no_push)?;
        }
    }
    Ok(())
}

fn handle_handoff_generate(
    handoff_service: &HandoffService<FileHandoffExporter, FileProgressStore, FileDecisionStore>,
    name: Option<String>,
    output: Option<String>,
    author: Option<&str>,
    no_md: bool,
    no_json: bool,
) -> CellResult<()> {
    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir().ok()
            .and_then(|p| p.file_name().and_then(|n| n.to_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "cell-project".to_string())
    });
    let out_dir = output.unwrap_or_else(|| ".handoff".to_string());
    let base_name = format!("handoff_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));

    let steps = vec![
        ("收集项目概览", "项目名称、描述、技术栈"),
        ("生成架构快照", "分析架构分层和模块"),
        ("收集决策记录", "读取ADR和技术决策"),
        ("收集进度信息", "当前任务和进度"),
        ("生成熵值快照", "计算架构熵值"),
        ("收集最近文件", "扫描最近修改的文件"),
        ("生成开发规范", "提取架构规则和约束"),
        ("生成快速指南", "8步快速上手指南"),
        ("导出交接包", "生成JSON和Markdown"),
    ];
    let mut progress = StepProgress::new(steps);
    for _ in 0..8 { progress.start_next(); progress.complete_current(); }
    progress.start_next();

    let pkg = handoff_service.generate(".", &project_name, author)?;
    if !no_json { export_handoff(handoff_service, &pkg, &out_dir, &base_name, "json")?; }
    if !no_md { export_handoff(handoff_service, &pkg, &out_dir, &base_name, "md")?; }

    progress.complete_current();
    progress.render_summary();
    print_handoff_result(&pkg, &out_dir, &base_name, no_md, no_json);
    Ok(())
}

fn export_handoff(
    handoff_service: &HandoffService<FileHandoffExporter, FileProgressStore, FileDecisionStore>,
    pkg: &crate::domain::handoff::HandoffPackage,
    out_dir: &str,
    base_name: &str,
    ext: &str,
) -> CellResult<()> {
    let path = Path::new(out_dir).join(format!("{}.{}", base_name, ext));
    let path_str = path.to_str().ok_or_else(|| CellError::Config(format!("Invalid {} path", ext)))?;
    if ext == "json" {
        handoff_service.export_json(pkg, path_str)?;
    } else {
        handoff_service.export_markdown(pkg, path_str)?;
    }
    Ok(())
}

fn print_handoff_result(
    pkg: &crate::domain::handoff::HandoffPackage,
    out_dir: &str,
    base_name: &str,
    no_md: bool,
    no_json: bool,
) {
    println!();
    if !no_json { println!("📄 JSON 交接包: {}/{}.json", out_dir, base_name); }
    if !no_md { println!("📝 Markdown 交接包: {}/{}.md", out_dir, base_name); }
    if !pkg.validation.is_complete {
        println!("\n⚠️  警告: 交接包不完整！");
        for w in &pkg.validation.warnings { println!("  - {}", w); }
    } else if pkg.validation.warnings.is_empty() {
        println!("\n✅ 交接包完整，可以直接接手！");
    } else {
        println!("\nℹ️  交接包有 {} 条警告:", pkg.validation.warnings.len());
        for w in &pkg.validation.warnings { println!("  - {}", w); }
    }
}

fn handle_handoff_commit(
    handoff_service: &HandoffService<FileHandoffExporter, FileProgressStore, FileDecisionStore>,
    message: Option<String>,
    author: Option<&str>,
    quick: bool,
    no_push: bool,
) -> CellResult<()> {
    let steps = vec![
        ("验证代码质量", "运行 cargo check / tests"),
        ("检查架构熵值", "计算架构熵值基线"),
        ("生成交接包", "创建 handoff JSON/Md"),
        ("Git 提交", "提交代码和交接包"),
        ("Git 推送", "推送到远程仓库"),
    ];
    let mut progress = StepProgress::new(steps);

    progress.start_next();
    let verify_service = FastVerifyService::new();
    if quick {
        let result = verify_service.quick_check(".")?;
        if !result.passed {
            progress.fail_current(&format!("快速验证失败"));
            std::process::exit(1);
        }
    } else {
        let result = verify_service.deep_check(".")?;
        if !result.passed {
            progress.fail_current(&format!("深度验证失败"));
            std::process::exit(1);
        }
    }
    progress.complete_current();

    progress.start_next();
    let report = run_entropy_check(".")?;
    println!("   📊 熵值得分: {:.1} ({})", report.overall_score, report.grade.label());
    if report.overall_score > 50.0 {
        println!("   ⚠️  警告: 熵值高于阈值 (50.0)");
    }
    progress.complete_current();

    progress.start_next();
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().and_then(|n| n.to_str().map(|s| s.to_string())))
        .unwrap_or_else(|| "cell-project".to_string());
    let pkg = handoff_service.generate(".", &project_name, author)?;
    handoff_service.export_json(&pkg, ".handoff/latest.json")?;
    handoff_service.export_markdown(&pkg, ".handoff/latest.md")?;
    println!("   📄 交接包: .handoff/latest.json");
    println!("   📝 交接文档: .handoff/latest.md");
    progress.complete_current();

    progress.start_next();
    let commit_msg = message.unwrap_or_else(|| format!(
        "chore: handoff commit - entropy={:.1}",
        report.overall_score
    ));

    fast_git_commit(&commit_msg)?;
    println!("   📦 提交成功: {}", commit_msg);
    progress.complete_current();

    if !no_push {
        progress.start_next();
        push_with_gh("master")?;
        println!("   🚀 已推送到远程仓库");
        progress.complete_current();
    } else {
        progress.skip_current();
    }

    progress.render_summary();
    println!();
    println!("🎉 无漂移交接完成！");
    println!("   下一智能体可通过以下命令接手:");
    println!("   cell handoff show .handoff/latest.json");
    println!("   cell handoff validate .handoff/latest.json");
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn cmd_progress(args: ProgressArgs) -> CellResult<()> {
    let store = FileProgressStore::new();
    let service = ProgressService::new(store);

    match args.sub {
        ProgressSub::Start { name, description, assignee } => {
            let desc = description.unwrap_or_default();
            let log = service.start_task(".", &name, &desc, assignee.as_deref())?;
            println!("✅ 任务已开始: {}", log.task_name);
            println!("   ID: {}", log.task_id);
        }
        ProgressSub::Status { timeline } => {
            show_progress_status(&service, timeline)?;
        }
        ProgressSub::Log { message, kind, details } => {
            let event_type = parse_event_type(kind.as_deref().unwrap_or("note"))?;
            let log = service.log_event(".", event_type.clone(), &message, details.as_deref())?;
            println!("✅ 已记录: {:?} - {}", event_type, message);
            let _ = log;
        }
        ProgressSub::Block { description } => {
            let log = service.add_blocker(".", &description)?;
            let blocker_id = log.blockers.last().map(|b| b.id)
                .ok_or_else(|| CellError::Config("Failed to add blocker".to_string()))?;
            println!("🚫 阻塞已记录: {}", description);
            println!("   ID: {}", blocker_id);
        }
        ProgressSub::Unblock { id, resolution } => {
            service.resolve_blocker(".", &id, &resolution)?;
            println!("✅ 阻塞已解决: {}", id);
        }
        ProgressSub::Next { description, priority, minutes } => {
            let log = service.add_next_step(".", &description, priority, minutes)?;
            let step_id = log.next_steps.last().map(|s| s.id)
                .ok_or_else(|| CellError::Config("Failed to add next step".to_string()))?;
            println!("📝 已添加下一步: {} (优先级: {})", description, priority);
            println!("   ID: {}", step_id);
        }
        ProgressSub::Done { id } => {
            service.complete_next_step(".", &id)?;
            println!("✅ 步骤已完成: {}", id);
        }
        ProgressSub::Complete {} => {
            let log = service.complete_task(".")?;
            println!("🎉 任务已完成: {}", log.task_name);
            println!("   总事件数: {}", log.timeline.len());
        }
        ProgressSub::History {} => {
            show_progress_history(&service)?;
        }
        ProgressSub::File { path } => {
            service.add_related_file(".", &path)?;
            println!("📎 已关联文件: {}", path);
        }
    }
    Ok(())
}

fn show_progress_status(service: &ProgressService<FileProgressStore>, timeline: bool) -> CellResult<()> {
    match service.get_current(".")? {
        Some(log) => {
            println!("📋 当前任务: {}", log.task_name);
            println!("   状态: {:?}", log.status);
            println!("   开始时间: {}", log.started_at.format("%Y-%m-%d %H:%M:%S"));
            if !log.description.is_empty() {
                println!("   描述: {}", log.description);
            }
            if let Some(a) = &log.assignee {
                println!("   负责人: {}", a);
            }
            println!("   活跃阻塞: {}", log.active_blockers_count());
            println!("   待办步骤: {}", log.pending_next_steps_count());
            println!("   相关文件: {}", log.related_files.len());

            if timeline {
                println!("\n📅 时间线:");
                for event in &log.timeline {
                    println!("  [{}] {:?}: {}",
                        event.timestamp.format("%H:%M:%S"),
                        event.event_type,
                        event.message
                    );
                }
            }
            if !log.blockers.is_empty() {
                println!("\n🚫 阻塞问题:");
                for b in &log.blockers {
                    let icon = match b.status {
                        crate::domain::progress::BlockerStatus::Active => "🔴",
                        crate::domain::progress::BlockerStatus::Resolved => "🟢",
                        crate::domain::progress::BlockerStatus::Bypassed => "🟡",
                    };
                    println!("  {} {} ({})", icon, b.description, b.id);
                }
            }
            if !log.next_steps.is_empty() {
                println!("\n📝 下一步计划:");
                for s in &log.next_steps {
                    let check = if s.done { "✅" } else { "⬜" };
                    println!("  {} [P{}] {}", check, s.priority, s.description);
                }
            }
        }
        None => println!("ℹ️  没有进行中的任务。使用 'cell progress start <name>' 开始一个新任务。"),
    }
    Ok(())
}

fn show_progress_history(service: &ProgressService<FileProgressStore>) -> CellResult<()> {
    let history = service.list_history(".")?;
    if history.is_empty() {
        println!("ℹ️  没有历史任务记录。");
    } else {
        println!("📜 历史任务 (共 {} 个):", history.len());
        for log in &history {
            println!("  [{:?}] {} - {}",
                log.status, log.task_name,
                log.started_at.format("%Y-%m-%d")
            );
        }
    }
    Ok(())
}

fn parse_event_type(s: &str) -> CellResult<EventType> {
    match s.to_lowercase().as_str() {
        "start" => Ok(EventType::Start),
        "update" => Ok(EventType::Update),
        "decision" => Ok(EventType::Decision),
        "blocker" => Ok(EventType::Blocker),
        "file" => Ok(EventType::FileModified),
        "test-pass" => Ok(EventType::TestPass),
        "test-fail" => Ok(EventType::TestFail),
        "complete" => Ok(EventType::Complete),
        "cancel" => Ok(EventType::Cancel),
        "note" => Ok(EventType::Note),
        _ => Err(CellError::Config(format!(
            "Unknown event type: {}. Valid: start, update, decision, blocker, note, complete",
            s
        ))),
    }
}
