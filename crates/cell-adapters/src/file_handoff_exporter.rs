use cell_application::ports::handoff_exporter::HandoffExporterPort;
use cell_domain::errors::{CellError, CellResult};
use cell_domain::handoff::HandoffPackage;
use std::fs;
use std::path::Path;

pub struct FileHandoffExporter;

impl FileHandoffExporter {
    pub fn new() -> Self {
        Self
    }

    fn ensure_parent_dir(path: &str) -> CellResult<()> {
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty() && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

impl Default for FileHandoffExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl HandoffExporterPort for FileHandoffExporter {
    fn export_json(&self, package: &HandoffPackage, output_path: &str) -> CellResult<String> {
        Self::ensure_parent_dir(output_path)?;
        let content = serde_json::to_string_pretty(package)?;
        fs::write(output_path, &content)?;
        Ok(output_path.to_string())
    }

    fn export_markdown(&self, package: &HandoffPackage, output_path: &str) -> CellResult<String> {
        Self::ensure_parent_dir(output_path)?;
        let content = package.to_markdown();
        fs::write(output_path, &content)?;
        Ok(output_path.to_string())
    }

    fn import_json(&self, path: &str) -> CellResult<HandoffPackage> {
        if !Path::new(path).exists() {
            return Err(CellError::Config(format!("Handoff file not found: {path}")));
        }
        let content = fs::read_to_string(path)?;
        let package: HandoffPackage = serde_json::from_str(&content)?;
        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_and_import_json() {
        let dir = tempdir().unwrap();
        let exporter = FileHandoffExporter::new();
        let pkg = HandoffPackage::new("test-project");

        let out_path = dir.path().join("handoff.json");
        let result = exporter.export_json(&pkg, out_path.to_str().unwrap()).unwrap();
        assert_eq!(result, out_path.to_str().unwrap());
        assert!(out_path.exists());

        let imported = exporter.import_json(out_path.to_str().unwrap()).unwrap();
        assert_eq!(imported.project_name, "test-project");
    }

    #[test]
    fn test_export_markdown() {
        let dir = tempdir().unwrap();
        let exporter = FileHandoffExporter::new();
        let pkg = HandoffPackage::new("test-project");

        let out_path = dir.path().join("handoff.md");
        let result = exporter.export_markdown(&pkg, out_path.to_str().unwrap()).unwrap();
        assert_eq!(result, out_path.to_str().unwrap());

        let content = fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("# 交接包: test-project"));
    }

    #[test]
    fn test_import_nonexistent_fails() {
        let exporter = FileHandoffExporter::new();
        let result = exporter.import_json("/nonexistent/path.json");
        assert!(result.is_err());
    }
}
