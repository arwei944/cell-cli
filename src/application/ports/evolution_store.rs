use crate::domain::errors::CellResult;
use crate::domain::evolution::EvolutionLog;

pub trait EvolutionStorePort {
    fn load_current_cycle(&self, project_path: &str) -> CellResult<Option<EvolutionLog>>;
    fn save_current_cycle(&self, project_path: &str, log: &EvolutionLog) -> CellResult<()>;
    fn list_history(&self, project_path: &str) -> CellResult<Vec<EvolutionLog>>;
    fn archive_cycle(&self, project_path: &str, log: &EvolutionLog) -> CellResult<()>;
    fn get_next_cycle_number(&self, project_path: &str) -> CellResult<u32>;
}
