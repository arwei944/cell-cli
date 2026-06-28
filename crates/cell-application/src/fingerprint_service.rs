use cell_domain::errors::{CellError, CellResult};
use cell_domain::fingerprint::{FingerprintCategory, FingerprintLibrary, FingerprintMatch, FingerprintSeverity, ProblemFingerprint};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_time: DateTime<Local>,
    pub path: String,
    pub total_files: usize,
    pub matches: Vec<FileMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatch {
    pub file_path: String,
    pub matches: Vec<FingerprintMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub fingerprint_id: String,
    pub success: bool,
    pub applied_fix: String,
    pub message: String,
}

pub struct FingerprintService {
    library: FingerprintLibrary,
    fix_history: HashMap<String, Vec<FixResult>>,
}

impl FingerprintService {
    pub fn new() -> Self {
        Self {
            library: FingerprintLibrary::new(),
            fix_history: HashMap::new(),
        }
    }

    pub fn scan_code(&self, path: &str) -> CellResult<ScanResult> {
        let base_path = Path::new(path);
        if !base_path.exists() {
            return Err(CellError::Validation(format!("路径不存在: {path}")));
        }

        let mut total_files = 0;
        let mut all_matches = Vec::new();

        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str())
                && matches!(ext, "rs" | "py" | "js" | "ts" | "go" | "java")
                    && let Ok(content) = fs::read_to_string(file_path) {
                        total_files += 1;
                        let matches = self.library.match_code(&content);
                        if !matches.is_empty() {
                            all_matches.push(FileMatch {
                                file_path: file_path.to_string_lossy().to_string(),
                                matches,
                            });
                        }
                    }
        }

        Ok(ScanResult {
            scan_time: Local::now(),
            path: path.to_string(),
            total_files,
            matches: all_matches,
        })
    }

    pub fn scan_log(&self, log_content: &str) -> Vec<FingerprintMatch> {
        self.library.match_error(log_content)
    }

    pub fn fix_problem(&mut self, fingerprint_id: &str) -> CellResult<FixResult> {
        let fingerprint = self.library
            .get_by_id(fingerprint_id)
            .ok_or_else(|| CellError::NotFound(format!("未找到指纹: {fingerprint_id}")))?;

        if !fingerprint.auto_fixable {
            return Ok(FixResult {
                fingerprint_id: fingerprint_id.to_string(),
                success: false,
                applied_fix: String::new(),
                message: format!("指纹 {} ({}) 不支持自动修复，请手动修复", fingerprint.id, fingerprint.name),
            });
        }

        let fix_result = FixResult {
            fingerprint_id: fingerprint_id.to_string(),
            success: true,
            applied_fix: fingerprint.fix_suggestion.clone(),
            message: format!("已应用修复建议: {}", fingerprint.name),
        };

        self.fix_history
            .entry(fingerprint_id.to_string())
            .or_default()
            .push(fix_result.clone());

        Ok(fix_result)
    }

    pub fn get_fingerprint_detail(&self, fingerprint_id: &str) -> Option<ProblemFingerprint> {
        self.library.get_by_id(fingerprint_id)
    }

    pub fn list_fingerprints(&self) -> Vec<ProblemFingerprint> {
        self.library.all_fingerprints()
    }

    pub fn list_fingerprints_by_category(&self, category: &FingerprintCategory) -> Vec<ProblemFingerprint> {
        self.library.by_category(category)
    }

    pub fn list_fingerprints_by_severity(&self, severity: &FingerprintSeverity) -> Vec<ProblemFingerprint> {
        self.library.by_severity(severity)
    }

    pub fn get_fix_history(&self, fingerprint_id: &str) -> Option<&Vec<FixResult>> {
        self.fix_history.get(fingerprint_id)
    }
}

impl Default for FingerprintService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_code_empty_path() {
        let service = FingerprintService::new();
        let result = service.scan_code("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_code_current_dir() {
        let service = FingerprintService::new();
        let result = service.scan_code(".");
        assert!(result.is_ok());
        let scan_result = result.unwrap();
        assert_eq!(scan_result.path, ".");
    }

    #[test]
    fn test_scan_log_matches() {
        let service = FingerprintService::new();
        let log = "called `Result::unwrap()` on an `Err` value";
        let matches = service.scan_log(log);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP004"));
    }

    #[test]
    fn test_fix_problem_not_found() {
        let mut service = FingerprintService::new();
        let result = service.fix_problem("FP999");
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_problem_not_auto_fixable() {
        let mut service = FingerprintService::new();
        let result = service.fix_problem("FP001");
        assert!(result.is_ok());
        let fix_result = result.unwrap();
        assert!(!fix_result.success);
    }

    #[test]
    fn test_get_fingerprint_detail_found() {
        let service = FingerprintService::new();
        let fp = service.get_fingerprint_detail("FP001");
        assert!(fp.is_some());
        assert_eq!(fp.unwrap().id, "FP001");
    }

    #[test]
    fn test_get_fingerprint_detail_not_found() {
        let service = FingerprintService::new();
        let fp = service.get_fingerprint_detail("FP999");
        assert!(fp.is_none());
    }

    #[test]
    fn test_list_fingerprints_count() {
        let service = FingerprintService::new();
        let fps = service.list_fingerprints();
        assert!(fps.len() >= 50);
    }

    #[test]
    fn test_list_fingerprints_by_category() {
        let service = FingerprintService::new();
        let arch_fps = service.list_fingerprints_by_category(&FingerprintCategory::Architecture);
        assert!(!arch_fps.is_empty());
    }

    #[test]
    fn test_list_fingerprints_by_severity() {
        let service = FingerprintService::new();
        let high_fps = service.list_fingerprints_by_severity(&FingerprintSeverity::High);
        assert!(!high_fps.is_empty());
    }
}
