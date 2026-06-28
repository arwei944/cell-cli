use cell_domain::errors::CellResult;
use cell_domain::handoff::HandoffPackage;

pub trait HandoffExporterPort {
    fn export_json(&self, package: &HandoffPackage, output_path: &str) -> CellResult<String>;
    fn export_markdown(&self, package: &HandoffPackage, output_path: &str) -> CellResult<String>;
    fn import_json(&self, path: &str) -> CellResult<HandoffPackage>;
}
