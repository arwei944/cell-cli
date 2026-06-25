use crate::domain::decision::{DecisionRecord, DecisionMetrics};
use crate::domain::errors::CellResult;

pub trait DecisionStorePort {
    fn save(&self, path: &str, decision: &DecisionRecord) -> CellResult<()>;
    fn load_all(&self, path: &str) -> CellResult<Vec<DecisionRecord>>;
    fn load_by_id(&self, path: &str, id: &str) -> CellResult<Option<DecisionRecord>>;
    fn delete(&self, path: &str, id: &str) -> CellResult<()>;
    fn get_metrics(&self, path: &str) -> CellResult<DecisionMetrics>;
}
