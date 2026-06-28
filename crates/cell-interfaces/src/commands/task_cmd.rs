use cell_application::task_discovery_service::{TaskDiscoveryService, TaskPriority, TaskStatus};
use cell_domain::errors::CellResult;
use crate::cli::{TaskArgs, TaskSub};

pub fn cmd_task(args: TaskArgs) -> CellResult<()> {
    let service = TaskDiscoveryService::new();
    let project_path = ".";

    let priority_filter = match args.priority.as_deref() {
        Some("p0") => Some(TaskPriority::P0),
        Some("p1") => Some(TaskPriority::P1),
        Some("p2") => Some(TaskPriority::P2),
        Some("p3") => Some(TaskPriority::P3),
        _ => None,
    };

    let status_filter = match args.status.as_deref() {
        Some("pending") => Some(TaskStatus::Pending),
        Some("in_progress") => Some(TaskStatus::InProgress),
        Some("done") => Some(TaskStatus::Done),
        Some("blocked") => Some(TaskStatus::Blocked),
        _ => None,
    };

    match args.sub {
        TaskSub::List {} => {
            let tasks = service.list_tasks(project_path, priority_filter.as_ref(), status_filter.as_ref())?;
            println!("{}", service.format_task_list(&tasks));
        }
        TaskSub::Discover {} => {
            let report = service.discover_all(project_path)?;
            println!("{}", service.format_report(&report));
        }
        TaskSub::Next {} => {
            match service.get_next_task(project_path)? {
                Some(task) => {
                    println!("\n🎯 下一个推荐任务\n");
                    let priority_icon = match task.priority {
                        TaskPriority::P0 => "🔴",
                        TaskPriority::P1 => "🟠",
                        TaskPriority::P2 => "🟡",
                        TaskPriority::P3 => "🟢",
                    };
                    println!("  {} {}\n", priority_icon, task.title);
                    println!("  ID: {}", task.id);
                    println!("  优先级: {:?}", task.priority);
                    println!("  来源: {:?}", task.source);
                    if let Some(desc) = &task.description {
                        println!("  描述: {desc}");
                    }
                    println!("\n💡 认领任务: cell task claim {}", task.id);
                }
                None => {
                    println!("\n✅ 没有待处理的任务，干得好！");
                }
            }
        }
        TaskSub::Show { id } => {
            let report = service.discover_all(project_path)?;
            match report.tasks.iter().find(|t| t.id == id) {
                Some(task) => {
                    println!("\n📋 任务详情\n");
                    println!("  标题: {}", task.title);
                    println!("  ID: {}", task.id);
                    println!("  优先级: {:?}", task.priority);
                    println!("  状态: {:?}", task.status);
                    println!("  来源: {:?}", task.source);
                    println!("  创建时间: {}", task.created_at);
                    if let Some(desc) = &task.description {
                        println!("  描述: {desc}");
                    }
                    if !task.dependencies.is_empty() {
                        println!("  依赖: {:?}", task.dependencies);
                    }
                    if let Some(effort) = &task.estimated_effort {
                        println!("  预估工作量: {effort}");
                    }
                    if let Some(assignee) = &task.assignee {
                        println!("  负责人: {assignee}");
                    }
                }
                None => {
                    println!("\n❌ 未找到任务: {id}");
                }
            }
        }
        TaskSub::Claim { id } => {
            println!("\n✅ 已认领任务: {id}");
            println!("  开始工作吧！运行 `cell dev start <任务名>` 启动工作流");
        }
        TaskSub::Done { id } => {
            println!("\n🎉 任务完成: {id}");
            println!("  干得好！运行 `cell task next` 查看下一个任务");
        }
    }

    Ok(())
}
