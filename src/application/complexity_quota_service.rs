use crate::domain::errors::{CellError, CellResult};
use crate::domain::complexity_quota::ComplexityQuota;

/// 复杂度配额服务
pub struct ComplexityQuotaService;

impl ComplexityQuotaService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_quota(&self, name: &str) -> CellResult<ComplexityQuota> {
        Err(CellError::NotFound(format!("复杂度配额 '{}' 不存在", name)))
    }

    pub fn check_quota(&self, name: &str, _required: f64) -> CellResult<bool> {
        match self.get_quota(name) {
            Ok(quota) => Ok(quota.current_cyclomatic < quota.cyclomatic_complexity_limit),
            Err(_) => Ok(false),
        }
    }

    pub fn format_quota(&self, quota: &ComplexityQuota) -> String {
        format!(
            "📊 复杂度配额: {}\n  圈复杂度: {:.1}/{:.1}\n  状态: {}",
            quota.name,
            quota.current_cyclomatic,
            quota.cyclomatic_complexity_limit,
            if quota.current_cyclomatic < quota.cyclomatic_complexity_limit { "✅ 可用" } else { "❌ 超出" }
        )
    }
}

impl Default for ComplexityQuotaService {
    fn default() -> Self {
        Self::new()
    }
}
