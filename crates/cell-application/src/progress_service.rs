use crate::ports::progress_store::ProgressStorePort;
use cell_domain::errors::{CellError, CellResult};
use cell_domain::progress::{EventType, ProgressLog, ProgressStatus};
use std::path::Path;

pub struct ProgressService<T: ProgressStorePort> {
    store: T,
}

impl<T: ProgressStorePort> ProgressService<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }

    pub fn start_task(&self, project_path: &str, name: &str, description: &str, assignee: Option<&str>) -> CellResult<ProgressLog> {
        if let Some(current) = self.store.load_current(project_path)? {
            if current.status == ProgressStatus::InProgress || current.status == ProgressStatus::Blocked {
                return Err(CellError::Config(format!(
                    "There is already an active task '{}'. Complete or cancel it first.",
                    current.task_name
                )));
            }
            self.store.archive(project_path, &current)?;
        }

        let mut log = ProgressLog::new(name, description);
        log.start(assignee);
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn get_current(&self, project_path: &str) -> CellResult<Option<ProgressLog>> {
        self.store.load_current(project_path)
    }

    pub fn log_event(&self, project_path: &str, event_type: EventType, message: &str, details: Option<&str>) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        log.add_event(event_type, message, details);
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn add_blocker(&self, project_path: &str, description: &str) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        log.add_blocker(description);
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn resolve_blocker(&self, project_path: &str, blocker_id: &str, resolution: &str) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        let id = uuid::Uuid::parse_str(blocker_id).map_err(|e| CellError::Config(format!("Invalid blocker UUID: {e}")))?;
        if !log.resolve_blocker(id, resolution) {
            return Err(CellError::Config(format!("Blocker with id {blocker_id} not found")));
        }
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn add_next_step(&self, project_path: &str, description: &str, priority: u8, estimated_minutes: Option<u32>) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        log.add_next_step(description, priority, estimated_minutes);
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn complete_next_step(&self, project_path: &str, step_id: &str) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        let id = uuid::Uuid::parse_str(step_id).map_err(|e| CellError::Config(format!("Invalid step UUID: {e}")))?;
        if !log.complete_next_step(id) {
            return Err(CellError::Config(format!("Next step with id {step_id} not found")));
        }
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn add_related_file(&self, project_path: &str, file_path: &str) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        log.add_related_file(file_path);
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn update_status(&self, project_path: &str, status: ProgressStatus) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        log.status = status.clone();
        log.updated_at = chrono::Utc::now();
        log.add_event(EventType::Update, &format!("Status changed to {status:?}"), None);
        if status == ProgressStatus::Completed {
            log.complete();
        }
        self.store.save_current(project_path, &log)?;
        Ok(log)
    }

    pub fn complete_task(&self, project_path: &str) -> CellResult<ProgressLog> {
        let mut log = self.require_current(project_path)?;
        log.complete();
        self.store.archive(project_path, &log)?;
        Ok(log)
    }

    pub fn list_history(&self, project_path: &str) -> CellResult<Vec<ProgressLog>> {
        self.store.list_history(project_path)
    }

    fn require_current(&self, project_path: &str) -> CellResult<ProgressLog> {
        self.store.load_current(project_path)?.ok_or_else(|| {
            CellError::Config("No active task. Start one with 'progress start' first.".to_string())
        })
    }
}

pub fn default_progress_dir(project_path: &str) -> String {
    Path::new(project_path).join(".cell").join("progress").to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockStore {
        current: RefCell<Option<ProgressLog>>,
        history: RefCell<Vec<ProgressLog>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                current: RefCell::new(None),
                history: RefCell::new(Vec::new()),
            }
        }
    }

    impl ProgressStorePort for MockStore {
        fn load_current(&self, _project_path: &str) -> CellResult<Option<ProgressLog>> {
            Ok(self.current.borrow().clone())
        }

        fn save_current(&self, _project_path: &str, log: &ProgressLog) -> CellResult<()> {
            *self.current.borrow_mut() = Some(log.clone());
            Ok(())
        }

        fn list_history(&self, _project_path: &str) -> CellResult<Vec<ProgressLog>> {
            Ok(self.history.borrow().clone())
        }

        fn archive(&self, _project_path: &str, log: &ProgressLog) -> CellResult<()> {
            self.history.borrow_mut().push(log.clone());
            *self.current.borrow_mut() = None;
            Ok(())
        }
    }

    #[test]
    fn test_start_task() {
        let store = MockStore::new();
        let service = ProgressService::new(store);
        let log = service.start_task(".", "Test task", "A test task", Some("agent-1")).unwrap();
        assert_eq!(log.task_name, "Test task");
        assert_eq!(log.status, ProgressStatus::InProgress);
        assert_eq!(log.assignee, Some("agent-1".to_string()));
        assert_eq!(log.timeline.len(), 1);
    }

    #[test]
    fn test_add_blocker_and_resolve() {
        let store = MockStore::new();
        let service = ProgressService::new(store);
        service.start_task(".", "Test", "Desc", None).unwrap();

        let log = service.add_blocker(".", "Can't compile").unwrap();
        assert_eq!(log.status, ProgressStatus::Blocked);
        assert_eq!(log.active_blockers_count(), 1);

        let blocker_id = log.blockers[0].id.to_string();
        let log = service.resolve_blocker(".", &blocker_id, "Fixed typo").unwrap();
        assert_eq!(log.status, ProgressStatus::InProgress);
        assert_eq!(log.active_blockers_count(), 0);
    }

    #[test]
    fn test_next_steps() {
        let store = MockStore::new();
        let service = ProgressService::new(store);
        service.start_task(".", "Test", "Desc", None).unwrap();

        let log = service.add_next_step(".", "Write tests", 1, Some(30)).unwrap();
        assert_eq!(log.pending_next_steps_count(), 1);

        let step_id = log.next_steps[0].id.to_string();
        let log = service.complete_next_step(".", &step_id).unwrap();
        assert_eq!(log.pending_next_steps_count(), 0);
    }

    #[test]
    fn test_no_active_task_fails() {
        let store = MockStore::new();
        let service = ProgressService::new(store);
        let result = service.log_event(".", EventType::Note, "test", None);
        assert!(result.is_err());
    }
}
