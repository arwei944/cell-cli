use crate::domain::errors::CellResult;
use crate::domain::progress::ProgressLog;

pub trait ProgressStorePort {
    fn load_current(&self, project_path: &str) -> CellResult<Option<ProgressLog>>;
    fn save_current(&self, project_path: &str, log: &ProgressLog) -> CellResult<()>;
    fn list_history(&self, project_path: &str) -> CellResult<Vec<ProgressLog>>;
    fn archive(&self, project_path: &str, log: &ProgressLog) -> CellResult<()>;
}
