use crate::application::ports::decision_store::DecisionStorePort;
use crate::application::ports::handoff_exporter::HandoffExporterPort;
use crate::application::ports::progress_store::ProgressStorePort;
use crate::domain::errors::CellResult;
use crate::domain::handoff::*;
use chrono::Utc;
use std::path::Path;
use walkdir::WalkDir;

pub struct HandoffService<E: HandoffExporterPort, P: ProgressStorePort, D: DecisionStorePort> {
    exporter: E,
    progress_store: P,
    decision_store: D,
}

impl<E: HandoffExporterPort, P: ProgressStorePort, D: DecisionStorePort>
    HandoffService<E, P, D>
{
    pub fn new(exporter: E, progress_store: P, decision_store: D) -> Self {
        Self { exporter, progress_store, decision_store }
    }

    pub fn generate(&self, project_path: &str, project_name: &str, author: Option<&str>) -> CellResult<HandoffPackage> {
        let mut pkg = HandoffPackage::new(project_name);
        pkg.generated_by = author.map(|s| s.to_string());

        pkg.project_overview = Self::build_project_overview(project_path, project_name)?;

        if let Ok(Some(progress)) = self.progress_store.load_current(project_path) {
            pkg.current_task = TaskContext {
                name: progress.task_name.clone(),
                description: progress.description.clone(),
                status: format!("{:?}", progress.status),
                started_at: Some(progress.started_at),
                last_activity_at: Some(progress.updated_at),
            };
            pkg.next_actions = progress
                .next_steps
                .iter()
                .filter(|s| !s.done)
                .map(|s| NextActionItem {
                    id: s.id,
                    description: s.description.clone(),
                    priority: s.priority,
                    estimated_minutes: s.estimated_minutes,
                    prerequisites: Vec::new(),
                    acceptance_criteria: Vec::new(),
                })
                .collect();
            pkg.progress = Some(progress);
        }

        if let Ok(decisions) = self.decision_store.load_all(project_path) {
            pkg.decisions = decisions;
        }

        pkg.recent_files = Self::build_recent_files(project_path);
        pkg.development_rules = Self::build_development_rules(project_path);
        pkg.quick_start = Self::build_quick_start();
        pkg.related_files = Self::build_related_files(project_path);

        pkg.environment_info = Self::build_environment_info();
        pkg.architecture_snapshot = Self::build_architecture_snapshot(project_path);
        pkg.entropy_snapshot = Self::build_entropy_snapshot(project_path);

        pkg.open_questions = Self::collect_open_questions(project_path);

        pkg.validate();
        Ok(pkg)
    }

    pub fn export_json(&self, package: &HandoffPackage, output_path: &str) -> CellResult<String> {
        self.exporter.export_json(package, output_path)
    }

    pub fn export_markdown(&self, package: &HandoffPackage, output_path: &str) -> CellResult<String> {
        self.exporter.export_markdown(package, output_path)
    }

    pub fn import(&self, path: &str) -> CellResult<HandoffPackage> {
        self.exporter.import_json(path)
    }

    pub fn validate_package(package: &mut HandoffPackage) -> &HandoffValidation {
        package.validate()
    }

    fn build_project_overview(project_path: &str, project_name: &str) -> CellResult<ProjectOverview> {
        let mut overview = ProjectOverview {
            name: project_name.to_string(),
            description: String::new(),
            architecture_style: "Cell Architecture (Hexagonal + DDD)".to_string(),
            tech_stack: vec!["Rust".to_string()],
            key_directories: Vec::new(),
        };

        let src_path = Path::new(project_path).join("src");
        if src_path.exists() {
            let layers = ["domain", "application", "adapters", "interfaces"];
            for layer in &layers {
                let layer_path = src_path.join(layer);
                if layer_path.exists() {
                    let file_count = count_files(&layer_path);
                    overview.key_directories.push(DirectoryInfo {
                        path: format!("src/{}", layer),
                        purpose: Self::layer_purpose(layer),
                        file_count,
                    });
                }
            }
        }

        Ok(overview)
    }

    fn layer_purpose(layer: &str) -> String {
        match layer {
            "domain" => "领域内核：核心业务模型、实体、值对象、领域服务".to_string(),
            "application" => "应用层：用例编排、Port接口、应用服务".to_string(),
            "adapters" => "适配器层：技术实现、外部系统对接".to_string(),
            "interfaces" => "接口层：CLI/API/UI等外部接入点".to_string(),
            _ => format!("{} layer", layer),
        }
    }

    fn build_environment_info() -> EnvironmentInfo {
        let rust_version = std::env::var("RUSTC_VERSION")
            .ok()
            .or_else(|| {
                std::process::Command::new("rustc")
                    .arg("--version")
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            });

        EnvironmentInfo {
            rust_version,
            dependencies: Vec::new(),
            build_status: BuildStatus::Unknown,
            test_status: TestStatus::Unknown,
        }
    }

    fn build_recent_files(project_path: &str) -> Vec<RecentFileInfo> {
        let mut files = Vec::new();
        let root = Path::new(project_path);

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let (Ok(metadata), Ok(rel_path)) =
                    (path.metadata(), path.strip_prefix(root))
                {
                    let rel_str = rel_path.to_string_lossy().to_string();
                    if rel_str.starts_with("src/")
                        || rel_str.starts_with("src\\")
                        || rel_str == "Cargo.toml"
                        || rel_str == "README.md"
                    {
                        let modified = metadata
                            .modified()
                            .ok()
                            .map(|t| {
                                let std_time: std::time::SystemTime = t;
                                chrono::DateTime::<Utc>::from(std_time)
                            })
                            .unwrap_or_else(Utc::now);
                        let loc = std::fs::read_to_string(path)
                            .map(|c| c.lines().count())
                            .unwrap_or(0);
                        files.push(RecentFileInfo {
                            path: rel_str,
                            last_modified: modified,
                            lines_of_code: loc,
                            change_type: "modified".to_string(),
                        });
                    }
                }
            }
        }

        files.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        files.truncate(20);
        files
    }

    fn build_development_rules(project_path: &str) -> Vec<String> {
        let mut rules = vec![
            "严格遵循四层架构（domain/application/adapters/interfaces），依赖只能向内".to_string(),
            "domain 层不能依赖任何外部框架或库（除了基础标准库）".to_string(),
            "使用 Cell 架构的 Port/Adapter 模式进行外部依赖隔离".to_string(),
            "所有公共 API 必须有单元测试覆盖".to_string(),
            "代码提交前必须通过 cargo clippy 和 cargo fmt 检查".to_string(),
            "架构约束通过编译期测试强制执行（architecture_tests）".to_string(),
            "熵值阈值不超过 60，超过必须先优化再继续开发".to_string(),
        ];

        let cell_dir = Path::new(project_path).join(".cell");
        if cell_dir.join("decisions").exists() {
            rules.push("重要技术决策必须记录到决策日志（cell decision new）".to_string());
        }

        rules
    }

    fn build_quick_start() -> Vec<String> {
        vec![
            "第一步：阅读本交接包，了解项目现状和当前任务".to_string(),
            "第二步：运行 `cell tools status` 查看可用工具".to_string(),
            "第三步：运行 `cell dashboard` 打开可视化仪表盘了解全局状态".to_string(),
            "第四步：运行 `cargo test` 确认所有测试通过".to_string(),
            "第五步：查看决策记录（cell decision list）了解技术选型背景".to_string(),
            "第六步：从 next_actions 中选择最高优先级任务开始开发".to_string(),
            "开发过程中随时使用 `cell progress log` 记录进度".to_string(),
            "遇到阻塞使用 `cell progress block` 记录，解除后用 `cell progress unblock`".to_string(),
        ]
    }

    fn build_related_files(project_path: &str) -> Vec<FileSummary> {
        let mut files = Vec::new();
        let root = Path::new(project_path);
        let src_path = root.join("src");

        if src_path.exists() {
            for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let (Ok(rel_path), Ok(content)) =
                        (path.strip_prefix(root), std::fs::read_to_string(path))
                    {
                        let rel_str = rel_path.to_string_lossy().to_string();
                        let role = Self::detect_file_role(&rel_str);
                        files.push(FileSummary {
                            path: rel_str,
                            role,
                            description: String::new(),
                            lines_of_code: content.lines().count(),
                            modification_count: 0,
                            last_modified: None,
                        });
                    }
                }
            }
        }

        files
    }

    fn detect_file_role(path: &str) -> String {
        let lower = path.to_lowercase();
        if lower.contains("domain") {
            "领域模型".to_string()
        } else if lower.contains("application") && lower.contains("service") {
            "应用服务".to_string()
        } else if lower.contains("application") && lower.contains("port") {
            "端口接口".to_string()
        } else if lower.contains("adapter") {
            "适配器".to_string()
        } else if lower.contains("interface") || lower.contains("cli") {
            "接口层".to_string()
        } else if lower.contains("test") {
            "测试".to_string()
        } else {
            "其他".to_string()
        }
    }

    fn build_architecture_snapshot(project_path: &str) -> ArchitectureSnapshot {
        let src_path = Path::new(project_path).join("src");
        let mut layers = Vec::new();
        let layer_names = ["domain", "application", "adapters", "interfaces"];

        for name in &layer_names {
            let layer_path = src_path.join(name);
            if layer_path.exists() {
                let file_count = count_files(&layer_path);
                layers.push(LayerInfo {
                    name: name.to_string(),
                    file_count,
                    internal_deps: 0,
                    external_deps: 0,
                });
            }
        }

        ArchitectureSnapshot {
            layers,
            total_violations: 0,
            domain_external_deps: 0,
        }
    }

    fn build_entropy_snapshot(project_path: &str) -> EntropySnapshot {
        use crate::application::entropy_service::run_entropy_check;
        let mut dimensions = std::collections::HashMap::new();

        if let Ok(report) = run_entropy_check(project_path) {
            dimensions.insert("structural".to_string(), report.dimensions.structural);
            dimensions.insert("complexity".to_string(), report.dimensions.complexity);
            dimensions.insert("coupling".to_string(), report.dimensions.coupling);
            dimensions.insert("naming".to_string(), report.dimensions.naming);
            dimensions.insert("test".to_string(), report.dimensions.test);

            EntropySnapshot {
                overall_score: report.overall_score,
                threshold: 60.0,
                dimensions,
                file_count: report.file_count,
                total_lines: report.total_lines,
            }
        } else {
            EntropySnapshot {
                overall_score: 0.0,
                threshold: 60.0,
                dimensions,
                file_count: 0,
                total_lines: 0,
            }
        }
    }

    fn collect_open_questions(project_path: &str) -> Vec<String> {
        let mut questions = Vec::new();
        let cell_dir = Path::new(project_path).join(".cell");

        let todo_path = cell_dir.join("TODO.md");
        if todo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&todo_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
                        questions.push(trimmed.trim_start_matches("- [ ] ").trim_start_matches("* [ ] ").to_string());
                    }
                }
            }
        }

        questions
    }
}

fn count_files(path: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    count += 1;
                } else if ft.is_dir() {
                    count += count_files(&entry.path());
                }
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::progress_store::ProgressStorePort;
    use crate::application::ports::decision_store::DecisionStorePort;
    use crate::domain::progress::ProgressLog;
    use crate::domain::decision::DecisionRecord;
    use std::cell::RefCell;

    struct MockExporter;

    impl HandoffExporterPort for MockExporter {
        fn export_json(&self, _package: &HandoffPackage, _output_path: &str) -> CellResult<String> {
            Ok("output.json".to_string())
        }

        fn export_markdown(&self, _package: &HandoffPackage, _output_path: &str) -> CellResult<String> {
            Ok("output.md".to_string())
        }

        fn import_json(&self, _path: &str) -> CellResult<HandoffPackage> {
            Ok(HandoffPackage::new("test"))
        }
    }

    struct MockProgressStore {
        current: RefCell<Option<ProgressLog>>,
    }

    impl MockProgressStore {
        fn new() -> Self {
            Self { current: RefCell::new(None) }
        }
    }

    impl ProgressStorePort for MockProgressStore {
        fn load_current(&self, _project_path: &str) -> CellResult<Option<ProgressLog>> {
            Ok(self.current.borrow().clone())
        }

        fn save_current(&self, _project_path: &str, log: &ProgressLog) -> CellResult<()> {
            *self.current.borrow_mut() = Some(log.clone());
            Ok(())
        }

        fn list_history(&self, _project_path: &str) -> CellResult<Vec<ProgressLog>> {
            Ok(Vec::new())
        }

        fn archive(&self, _project_path: &str, _log: &ProgressLog) -> CellResult<()> {
            Ok(())
        }
    }

    struct MockDecisionStore;

    impl DecisionStorePort for MockDecisionStore {
        fn load_all(&self, _project_path: &str) -> CellResult<Vec<DecisionRecord>> {
            Ok(Vec::new())
        }

        fn load_by_id(&self, _project_path: &str, _id: &str) -> CellResult<Option<DecisionRecord>> {
            Ok(None)
        }

        fn save(&self, _project_path: &str, _decision: &DecisionRecord) -> CellResult<()> {
            Ok(())
        }

        fn delete(&self, _project_path: &str, _id: &str) -> CellResult<()> {
            Ok(())
        }

        fn get_metrics(&self, _project_path: &str) -> CellResult<crate::domain::decision::DecisionMetrics> {
            use std::collections::HashMap;
            Ok(crate::domain::decision::DecisionMetrics {
                total_decisions: 0,
                accepted_count: 0,
                rejected_count: 0,
                superseded_count: 0,
                by_category: HashMap::new(),
                last_7_days: 0,
                last_30_days: 0,
            })
        }
    }

    #[test]
    fn test_generate_package() {
        let exporter = MockExporter;
        let store = MockProgressStore::new();
        let decision_store = MockDecisionStore;
        let service = HandoffService::new(exporter, store, decision_store);
        let pkg = service.generate(".", "test-project", Some("agent-1")).unwrap();

        assert_eq!(pkg.project_name, "test-project");
        assert_eq!(pkg.generated_by, Some("agent-1".to_string()));
        assert_eq!(pkg.project_overview.architecture_style, "Cell Architecture (Hexagonal + DDD)");
    }

    #[test]
    fn test_validate_empty_package() {
        let mut pkg = HandoffPackage::new("test");
        let validation = pkg.validate();
        assert!(!validation.is_complete);
        assert!(validation.missing_fields.contains(&"current_task.name".to_string()));
    }

    #[test]
    fn test_to_markdown() {
        let pkg = HandoffPackage::new("test-project");
        let md = pkg.to_markdown();
        assert!(md.contains("# 交接包: test-project"));
        assert!(md.contains("## 1. 项目概览"));
    }
}
