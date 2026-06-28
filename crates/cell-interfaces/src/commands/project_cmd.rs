use cell_application::multi_project_service::MultiProjectService;
use cell_domain::errors::CellResult;
use crate::cli::{ProjectArgs, ProjectSub};

pub fn cmd_project(args: ProjectArgs) -> CellResult<()> {
    let service = MultiProjectService::new();
    let root_path = ".";

    match args.sub {
        ProjectSub::List {} => {
            let projects = service.list_projects(root_path)?;
            let current = service.get_current_project(root_path)?;
            
            println!("\n📁 项目列表\n");
            if projects.is_empty() {
                println!("  暂无项目");
            } else {
                for p in &projects {
                    let current_marker = if current.as_ref().is_some_and(|c| c.name == p.name) {
                        "▶"
                    } else {
                        " "
                    };
                    let desc = p.description.as_deref().unwrap_or("-");
                    println!("  {} {:<20} {}", current_marker, p.name, desc);
                    println!("      路径: {}", p.path);
                }
            }
            println!();
        }
        ProjectSub::Current {} => {
            match service.get_current_project(root_path)? {
                Some(project) => {
                    println!("\n📌 当前项目\n");
                    println!("  名称: {}", project.name);
                    println!("  路径: {}", project.path);
                    if let Some(desc) = &project.description {
                        println!("  描述: {desc}");
                    }
                    println!();
                }
                None => {
                    println!("\n⚠️  未设置当前项目\n");
                }
            }
        }
        ProjectSub::Switch { name } => {
            service.switch_project(root_path, &name)?;
            println!("\n✅ 已切换到项目: {name}\n");
        }
        ProjectSub::Add { name, path, description } => {
            let project = service.add_project(root_path, &name, &path, description.as_deref())?;
            println!("\n✅ 项目已添加\n");
            println!("  名称: {}", project.name);
            println!("  路径: {}", project.path);
            println!();
        }
        ProjectSub::Remove { name } => {
            service.remove_project(root_path, &name)?;
            println!("\n✅ 项目已移除: {name}\n");
        }
    }

    Ok(())
}
