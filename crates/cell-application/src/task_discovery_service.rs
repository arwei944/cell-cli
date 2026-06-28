use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    P0,
    P1,
    P2,
    P3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub source: TaskSource,
    pub estimated_effort: Option<String>,
    pub dependencies: Vec<String>,
    pub created_at: String,
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskSource {
    Roadmap,
    TodoComment,
    Issue,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDiscoveryReport {
    pub total_tasks: usize,
    pub by_priority: std::collections::HashMap<String, usize>,
    pub by_source: std::collections::HashMap<String, usize>,
    pub by_status: std::collections::HashMap<String, usize>,
    pub tasks: Vec<TaskItem>,
}

pub struct TaskDiscoveryService;

impl TaskDiscoveryService {
    pub fn new() -> Self {
        Self
    }

    pub fn discover_all(&self, project_path: &str) -> CellResult<TaskDiscoveryReport> {
        let mut tasks = Vec::new();

        let roadmap_tasks = self.parse_roadmap(project_path)?;
        tasks.extend(roadmap_tasks);

        let todo_tasks = self.scan_todo_comments(project_path)?;
        tasks.extend(todo_tasks);

        Self::sort_tasks_by_priority(&mut tasks);

        let mut by_priority = std::collections::HashMap::new();
        let mut by_source = std::collections::HashMap::new();
        let mut by_status = std::collections::HashMap::new();

        for task in &tasks {
            *by_priority
                .entry(format!("{:?}", task.priority))
                .or_insert(0) += 1;
            *by_source
                .entry(format!("{:?}", task.source))
                .or_insert(0) += 1;
            *by_status
                .entry(format!("{:?}", task.status))
                .or_insert(0) += 1;
        }

        Ok(TaskDiscoveryReport {
            total_tasks: tasks.len(),
            by_priority,
            by_source,
            by_status,
            tasks,
        })
    }

    pub fn parse_roadmap(&self, project_path: &str) -> CellResult<Vec<TaskItem>> {
        let roadmap_path = Path::new(project_path).join("ROADMAP.md");
        if !roadmap_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&roadmap_path)?;
        let tasks = self.extract_tasks_from_markdown(&content);
        Ok(tasks)
    }

    fn extract_tasks_from_markdown(&self, content: &str) -> Vec<TaskItem> {
        let mut tasks = Vec::new();
        let mut current_priority = TaskPriority::P2;
        let lines = content.lines();

        for line in lines {
            let line = line.trim();

            if line.starts_with("# P0") || line.contains("P0") && line.contains("核心") {
                current_priority = TaskPriority::P0;
                continue;
            }
            if line.starts_with("# P1") || line.contains("P1") && line.contains("增强") {
                current_priority = TaskPriority::P1;
                continue;
            }
            if line.starts_with("# P2") || line.contains("P2") && line.contains("体验") {
                current_priority = TaskPriority::P2;
                continue;
            }
            if line.starts_with("# P3") || line.contains("P3") && line.contains("愿景") {
                current_priority = TaskPriority::P3;
                continue;
            }

            if line.starts_with("- [ ]") || line.starts_with("* [ ]") {
                let title = line.trim_start_matches("- [ ]").trim_start_matches("* [ ]").trim();
                if !title.is_empty() {
                    let id = format!("roadmap-{}", tasks.len());
                    tasks.push(TaskItem {
                        id,
                        title: title.to_string(),
                        description: None,
                        priority: current_priority.clone(),
                        status: TaskStatus::Pending,
                        source: TaskSource::Roadmap,
                        estimated_effort: None,
                        dependencies: Vec::new(),
                        created_at: chrono::Utc::now().to_rfc3339(),
                        assignee: None,
                    });
                }
            }

            if (line.starts_with("- ✅") || line.contains("✅")) && !line.contains("待开发") {
                continue;
            }
        }

        tasks
    }

    pub fn scan_todo_comments(&self, project_path: &str) -> CellResult<Vec<TaskItem>> {
        let mut tasks = Vec::new();
        let src_path = Path::new(project_path).join("src");

        if !src_path.exists() {
            return Ok(tasks);
        }

        self.scan_directory(&src_path, &mut tasks, 0)?;
        Ok(tasks)
    }

    fn scan_directory(
        &self,
        dir: &Path,
        tasks: &mut Vec<TaskItem>,
        depth: usize,
    ) -> CellResult<()> {
        if depth > 10 {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_directory(&path, tasks, depth + 1)?;
            } else if path.is_file()
                && let Some(ext) = path.extension()
                    && (ext == "rs" || ext == "ts" || ext == "js" || ext == "py") {
                        self.scan_file_for_todos(&path, tasks)?;
                    }
        }
        Ok(())
    }

    fn scan_file_for_todos(&self, file_path: &Path, tasks: &mut Vec<TaskItem>) -> CellResult<()> {
        let content = std::fs::read_to_string(file_path)?;
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            let todo_patterns = ["TODO:", "TODO ", "FIXME:", "FIXME ", "HACK:", "HACK "];
            for pattern in &todo_patterns {
                if let Some(pos) = line.find(pattern) {
                    let text = line[pos + pattern.len()..].trim().to_string();
                    if !text.is_empty() {
                        let id = format!("todo-{file_name}-{line_num}");
                        let is_fixme = pattern.contains("FIXME");
                        let is_hack = pattern.contains("HACK");

                        let priority = if is_fixme {
                            TaskPriority::P1
                        } else if is_hack {
                            TaskPriority::P2
                        } else {
                            TaskPriority::P2
                        };

                        tasks.push(TaskItem {
                            id,
                            title: text.clone(),
                            description: Some(format!(
                                "在 {} 第 {} 行",
                                file_path.display(),
                                line_num + 1
                            )),
                            priority,
                            status: TaskStatus::Pending,
                            source: TaskSource::TodoComment,
                            estimated_effort: None,
                            dependencies: Vec::new(),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            assignee: None,
                        });
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn get_next_task(&self, project_path: &str) -> CellResult<Option<TaskItem>> {
        let report = self.discover_all(project_path)?;

        for task in &report.tasks {
            if task.status == TaskStatus::Pending && task.dependencies.is_empty() {
                return Ok(Some(task.clone()));
            }
        }

        Ok(None)
    }

    pub fn list_tasks(
        &self,
        project_path: &str,
        priority: Option<&TaskPriority>,
        status: Option<&TaskStatus>,
    ) -> CellResult<Vec<TaskItem>> {
        let report = self.discover_all(project_path)?;
        let mut filtered: Vec<TaskItem> = report.tasks;

        if let Some(p) = priority {
            filtered.retain(|t| t.priority == *p);
        }

        if let Some(s) = status {
            filtered.retain(|t| t.status == *s);
        }

        Self::sort_tasks_by_priority(&mut filtered);
        Ok(filtered)
    }

    fn sort_tasks_by_priority(tasks: &mut Vec<TaskItem>) {
        tasks.sort_by(|a, b| {
            let pa = match a.priority {
                TaskPriority::P0 => 0,
                TaskPriority::P1 => 1,
                TaskPriority::P2 => 2,
                TaskPriority::P3 => 3,
            };
            let pb = match b.priority {
                TaskPriority::P0 => 0,
                TaskPriority::P1 => 1,
                TaskPriority::P2 => 2,
                TaskPriority::P3 => 3,
            };
            pa.cmp(&pb)
        });
    }

    pub fn format_report(&self, report: &TaskDiscoveryReport) -> String {
        let mut output = String::new();

        output.push_str("\n📋 任务发现报告\n\n");
        output.push_str(&format!("  总计: {} 个任务\n", report.total_tasks));

        output.push_str("\n  按优先级:\n");
        for key in ["P0", "P1", "P2", "P3"] {
            let count = report.by_priority.get(key).copied().unwrap_or(0);
            output.push_str(&format!("    {key}: {count}\n"));
        }

        output.push_str("\n  按来源:\n");
        for (source, count) in &report.by_source {
            output.push_str(&format!("    {source}: {count}\n"));
        }

        output.push_str("\n  任务列表 (按优先级排序):\n\n");

        for (i, task) in report.tasks.iter().enumerate() {
            if i >= 20 {
                output.push_str(&format!(
                    "\n  ... 还有 {} 个任务，使用 `cell task list` 查看全部\n",
                    report.tasks.len() - 20
                ));
                break;
            }

            let priority_icon = match task.priority {
                TaskPriority::P0 => "🔴",
                TaskPriority::P1 => "🟠",
                TaskPriority::P2 => "🟡",
                TaskPriority::P3 => "🟢",
            };

            let status_icon = match task.status {
                TaskStatus::Pending => "⬜",
                TaskStatus::InProgress => "🔄",
                TaskStatus::Done => "✅",
                TaskStatus::Blocked => "🚫",
            };

            output.push_str(&format!(
                "  {} {} {} {}\n",
                priority_icon, status_icon, task.id, task.title
            ));
        }

        output
    }

    pub fn format_task_list(&self, tasks: &[TaskItem]) -> String {
        let mut output = String::new();

        if tasks.is_empty() {
            output.push_str("\n✅ 没有找到匹配的任务\n");
            return output;
        }

        output.push_str(&format!("\n📋 任务列表 (共 {} 个)\n\n", tasks.len()));

        for (i, task) in tasks.iter().enumerate() {
            let priority_icon = match task.priority {
                TaskPriority::P0 => "🔴",
                TaskPriority::P1 => "🟠",
                TaskPriority::P2 => "🟡",
                TaskPriority::P3 => "🟢",
            };

            let status_icon = match task.status {
                TaskStatus::Pending => "⬜",
                TaskStatus::InProgress => "🔄",
                TaskStatus::Done => "✅",
                TaskStatus::Blocked => "🚫",
            };

            output.push_str(&format!(
                "  {}. {} {} [{}] {}\n",
                i + 1,
                priority_icon,
                status_icon,
                task.id,
                task.title
            ));

            if let Some(desc) = &task.description {
                output.push_str(&format!("     {desc}\n"));
            }
        }

        output
    }
}

impl Default for TaskDiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_discover_all() {
        let service = TaskDiscoveryService::new();
        let report = service.discover_all(".").unwrap();
        assert!(report.total_tasks > 0);
    }

    #[test]
    fn test_parse_roadmap() {
        let dir = tempdir().unwrap();
        let roadmap_path = dir.path().join("ROADMAP.md");
        let content = r"
# Roadmap

# P0 核心功能
- [ ] 实现用户认证系统
- [ ] 实现数据持久化

# P1 增强功能
- [ ] 添加搜索功能
";
        fs::write(&roadmap_path, content).unwrap();

        let service = TaskDiscoveryService::new();
        let tasks = service.parse_roadmap(dir.path().to_str().unwrap()).unwrap();
        assert!(!tasks.is_empty());
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].priority, TaskPriority::P0);
        assert_eq!(tasks[2].priority, TaskPriority::P1);
    }

    #[test]
    fn test_scan_todos() {
        let service = TaskDiscoveryService::new();
        let tasks = service.scan_todo_comments(".").unwrap();
        assert!(!tasks.is_empty());
    }

    #[test]
    fn test_get_next_task() {
        let service = TaskDiscoveryService::new();
        let next = service.get_next_task(".").unwrap();
        assert!(next.is_some());
    }
}
