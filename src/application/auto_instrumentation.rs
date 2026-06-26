/// 自动埋点配置
#[derive(Debug, Clone)]
pub struct InstrumentationConfig {
    pub enable_trace: bool,
    pub enable_metrics: bool,
    pub enable_logging: bool,
}

impl Default for InstrumentationConfig {
    fn default() -> Self {
        Self {
            enable_trace: true,
            enable_metrics: true,
            enable_logging: true,
        }
    }
}

/// 自动埋点服务
/// 在代码生成时为 UseCase、Repository、HTTP Handler 自动注入 Trace/Metrics/Log 埋点
pub struct AutoInstrumentationService {
    config: InstrumentationConfig,
}

impl AutoInstrumentationService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: InstrumentationConfig) -> Self {
        Self { config }
    }
}

impl Default for AutoInstrumentationService {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoInstrumentationService {
    /// 生成 UseCase 的 Trace 埋点代码片段
    pub fn usecase_trace_snippet(&self, usecase_name: &str) -> String {
        if !self.config.enable_trace {
            return String::new();
        }
        format!(
            "    // Auto-instrumented: Trace span\n    let _span = tracing::info_span!(\"usecase\", usecase = %\"{}\").entered();\n",
            usecase_name
        )
    }

    /// 生成 Repository 的埋点代码片段
    pub fn repository_snippet(&self, _repo_name: &str) -> &'static str {
        let mut lines = Vec::new();
        if self.config.enable_trace {
            lines.push("    // Auto-instrumented: Repository trace");
        }
        if self.config.enable_metrics {
            lines.push("    // Auto-instrumented: Repository metrics");
        }
        if self.config.enable_logging {
            lines.push("    // Auto-instrumented: Repository logging");
        }
        if lines.is_empty() {
            ""
        } else {
            // 返回第一行作为代表，完整实现需要模板支持多行
            lines[0]
        }
    }

    /// 生成 HTTP Handler 的埋点代码片段
    pub fn http_handler_snippet(&self, handler_name: &str) -> String {
        if !self.config.enable_trace {
            return String::new();
        }
        format!(
            "    // Auto-instrumented: HTTP handler trace span\n    let _span = tracing::info_span!(\"http_handler\", handler = %\"{}\").entered();\n",
            handler_name
        )
    }

    /// 生成消息发布/消费的埋点代码片段
    pub fn messaging_snippet(&self, topic: &str) -> String {
        if !self.config.enable_trace {
            return String::new();
        }
        format!(
            "    // Auto-instrumented: Messaging trace\n    let _span = tracing::info_span!(\"messaging\", topic = %\"{}\").entered();\n",
            topic
        )
    }

    /// 检查是否启用某种埋点
    pub fn is_enabled(&self, kind: InstrumentationKind) -> bool {
        match kind {
            InstrumentationKind::Trace => self.config.enable_trace,
            InstrumentationKind::Metrics => self.config.enable_metrics,
            InstrumentationKind::Logging => self.config.enable_logging,
        }
    }

    /// 生成 Cargo.toml 依赖项（tracing, metrics 等）
    pub fn cargo_dependencies(&self) -> Vec<(&'static str, &'static str)> {
        let mut deps = Vec::new();
        if self.config.enable_trace {
            deps.push(("tracing", "0.1"));
            deps.push(("tracing-subscriber", "0.3"));
        }
        if self.config.enable_metrics {
            deps.push(("metrics", "0.21"));
            deps.push(("metrics-exporter-prometheus", "0.13"));
        }
        deps
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InstrumentationKind {
    Trace,
    Metrics,
    Logging,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_enables_all() {
        let service = AutoInstrumentationService::new();
        assert!(service.is_enabled(InstrumentationKind::Trace));
        assert!(service.is_enabled(InstrumentationKind::Metrics));
        assert!(service.is_enabled(InstrumentationKind::Logging));
    }

    #[test]
    fn test_disabled_trace() {
        let config = InstrumentationConfig {
            enable_trace: false,
            ..Default::default()
        };
        let service = AutoInstrumentationService::with_config(config);
        assert!(!service.is_enabled(InstrumentationKind::Trace));
        assert!(service.is_enabled(InstrumentationKind::Metrics));
        assert!(service.usecase_trace_snippet("test").is_empty());
    }

    #[test]
    fn test_cargo_dependencies() {
        let service = AutoInstrumentationService::new();
        let deps = service.cargo_dependencies();
        assert!(deps.iter().any(|(name, _)| *name == "tracing"));
        assert!(deps.iter().any(|(name, _)| *name == "metrics"));
    }
}
