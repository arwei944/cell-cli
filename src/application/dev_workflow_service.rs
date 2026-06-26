use crate::application::arch_service::{ArchitectureRules, validate_architecture};
use crate::application::entropy_service;
use crate::application::entropy_trend_service::EntropyTrendService;
use crate::application::fast_verify_service::FastVerifyService;
use crate::application::ports::decision_store::DecisionStorePort;
use crate::application::ports::evolution_store::EvolutionStorePort;
use crate::application::ports::handoff_exporter::HandoffExporterPort;
use crate::application::ports::progress_store::ProgressStorePort;
use crate::domain::decision::DecisionCategory;
use crate::domain::errors::CellResult;
use crate::domain::evolution::{IssueCategory, IssueSeverity};
use crate::domain::progress::EventType;
use crate::application::progress_service::ProgressService;
use crate::application::handoff_service::HandoffService;
use crate::application::evolution_service::EvolutionService;
use crate::application::decision_service::DecisionService;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevPhase {
    Starting,
    Designing,
    Coding,
    Verifying,
    HandingOff,
    Complete,
}

impl DevPhase {
    pub fn label(&self) -> &str {
        match self {
            DevPhase::Starting => "🚀 启动阶段",
            DevPhase::Designing => "🎨 设计阶段",
            DevPhase::Coding => "💻 开发阶段",
            DevPhase::Verifying => "✅ 验证阶段",
            DevPhase::HandingOff => "📦 交接阶段",
            DevPhase::Complete => "🎉 完成",
        }
    }

    pub fn next(&self) -> Option<DevPhase> {
        match self {
            DevPhase::Starting => Some(DevPhase::Designing),
            DevPhase::Designing => Some(DevPhase::Coding),
            DevPhase::Coding => Some(DevPhase::Verifying),
            DevPhase::Verifying => Some(DevPhase::HandingOff),
            DevPhase::HandingOff => Some(DevPhase::Complete),
            DevPhase::Complete => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevWorkflowState {
    pub task_name: String,
    pub current_phase: DevPhase,
    pub baseline_entropy: Option<f64>,
    pub baseline_violations: usize,
    pub decisions: Vec<String>,
    pub started_at: String,
    pub last_updated: String,
}

#[derive(Debug, Clone)]
pub struct PhaseResult {
    pub phase: DevPhase,
    pub success: bool,
    pub duration_ms: u64,
    pub checks: Vec<PhaseCheck>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PhaseCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub struct DevWorkflowService<P: ProgressStorePort + Clone, E: EvolutionStorePort, H: HandoffExporterPort, D: DecisionStorePort> {
    progress_service: ProgressService<P>,
    handoff_service: HandoffService<H, P, D>,
    evolution_service: EvolutionService<E>,
    _decision_marker: std::marker::PhantomData<D>,
}

impl<P: ProgressStorePort + Clone, E: EvolutionStorePort, H: HandoffExporterPort, D: DecisionStorePort>
    DevWorkflowService<P, E, H, D>
{
    pub fn new(progress_store: P, evolution_store: E, handoff_exporter: H, decision_store: D) -> Self {
        Self {
            progress_service: ProgressService::new(progress_store.clone()),
            handoff_service: HandoffService::new(handoff_exporter, progress_store, decision_store),
            evolution_service: EvolutionService::new(evolution_store),
            _decision_marker: std::marker::PhantomData,
        }
    }

    pub fn start_task(&self, project_path: &str, task_name: &str, description: Option<&str>) -> CellResult<PhaseResult> {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();
        let mut warnings = Vec::new();

        self.progress_service.start_task(
            project_path,
            task_name,
            description.unwrap_or(""),
            Some("cell-architect"),
        )?;
        checks.push(PhaseCheck {
            name: "进度追踪启动".to_string(),
            passed: true,
            detail: "任务进度已开始追踪".to_string(),
        });

        self.progress_service.log_event(
            project_path,
            EventType::Start,
            "开发工作流启动",
            Some("使用 Cell 架构工具链进行开发"),
        )?;

        let baseline_entropy = match entropy_service::run_entropy_check(project_path) {
            Ok(report) => {
                checks.push(PhaseCheck {
                    name: "熵值基线".to_string(),
                    passed: true,
                    detail: format!("基线熵值: {:.2}", report.overall_score),
                });
                Some(report.overall_score)
            }
            Err(e) => {
                warnings.push(format!("熵值基线获取失败: {}", e));
                None
            }
        };

        let rules = ArchitectureRules::default();
        let arch_result = validate_architecture(Path::new(project_path), &rules);
        checks.push(PhaseCheck {
            name: "架构基线".to_string(),
            passed: arch_result.passed,
            detail: if arch_result.passed {
                "架构合规".to_string()
            } else {
                format!("{} 个违规", arch_result.violations.len())
            },
        });

        if !arch_result.passed {
            warnings.push(format!("架构存在 {} 个违规，请在开发中注意", arch_result.violations.len()));
        }

        let state = DevWorkflowState {
            task_name: task_name.to_string(),
            current_phase: DevPhase::Starting,
            baseline_entropy,
            baseline_violations: arch_result.violations.len(),
            decisions: Vec::new(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        };
        self.save_state(project_path, &state)?;

        Ok(PhaseResult {
            phase: DevPhase::Starting,
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            warnings,
        })
    }

    pub fn design_phase(&self, project_path: &str) -> CellResult<PhaseResult> {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();
        let mut warnings = Vec::new();

        self.progress_service.log_event(
            project_path,
            EventType::Update,
            "进入设计阶段",
            Some("进行架构设计和影响分析"),
        )?;

        let trend_service = EntropyTrendService::new();
        match trend_service.analyze(project_path) {
            Ok(trend) => {
                checks.push(PhaseCheck {
                    name: "熵值趋势分析".to_string(),
                    passed: true,
                    detail: format!("趋势: {}, 变化率: {:.2}%", trend.trend.label(), trend.change_rate),
                });
                if trend.trend == crate::application::entropy_trend_service::TrendDirection::Increasing {
                    warnings.push("熵值呈上升趋势，注意控制复杂度".to_string());
                }
            }
            Err(e) => {
                warnings.push(format!("熵值趋势分析失败: {}", e));
            }
        }

        let rules = ArchitectureRules::default();
        let arch_result = validate_architecture(Path::new(project_path), &rules);
        checks.push(PhaseCheck {
            name: "架构现状分析".to_string(),
            passed: arch_result.passed,
            detail: format!("{} 层, {} 个违规", 
                arch_result.layer_stats.len(), 
                arch_result.violations.len()
            ),
        });

        self.progress_service.log_event(
            project_path,
            EventType::Update,
            "设计阶段完成",
            Some("架构设计和影响分析完成"),
        )?;

        let mut state = self.load_state(project_path)?;
        state.current_phase = DevPhase::Designing;
        state.last_updated = chrono::Utc::now().to_rfc3339();
        self.save_state(project_path, &state)?;

        Ok(PhaseResult {
            phase: DevPhase::Designing,
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            warnings,
        })
    }

    pub fn code_checkpoint(&self, project_path: &str, message: &str) -> CellResult<PhaseResult> {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();
        let mut warnings = Vec::new();

        self.progress_service.log_event(
            project_path,
            EventType::Update,
            message,
            None,
        )?;

        let rules = ArchitectureRules::default();
        let result = validate_architecture(Path::new(project_path), &rules);
        checks.push(PhaseCheck {
            name: "架构合规检查".to_string(),
            passed: result.passed,
            detail: format!("{} 个违规", result.violations.len()),
        });
        if !result.violations.is_empty() {
            warnings.push(format!("发现 {} 个架构违规", result.violations.len()));
        }

        let state = self.load_state(project_path)?;
        if let Some(baseline) = state.baseline_entropy {
            match entropy_service::run_entropy_check(project_path) {
                Ok(current) => {
                    let diff = current.overall_score - baseline;
                    checks.push(PhaseCheck {
                        name: "熵值变化监控".to_string(),
                        passed: diff < 5.0,
                        detail: format!("基线: {:.2}, 当前: {:.2}, 变化: {:+.2}", 
                            baseline, current.overall_score, diff),
                    });
                    if diff >= 5.0 {
                        warnings.push(format!("熵值上升 {:.2}，注意控制复杂度", diff));
                    }
                }
                Err(e) => {
                    warnings.push(format!("熵值检查失败: {}", e));
                }
            }
        }

        Ok(PhaseResult {
            phase: DevPhase::Coding,
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            warnings,
        })
    }

    pub fn verify_phase(&self, project_path: &str, deep: bool) -> CellResult<PhaseResult> {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();
        let mut warnings = Vec::new();

        self.progress_service.log_event(
            project_path,
            EventType::Update,
            "进入验证阶段",
            Some("运行完整验证流程"),
        )?;

        let verifier = FastVerifyService::new();
        let verify_result = if deep {
            verifier.deep_check(project_path)
        } else {
            verifier.quick_check(project_path)
        };

        match verify_result {
            Ok(result) => {
                for check in &result.checks {
                    checks.push(PhaseCheck {
                        name: check.name.clone(),
                        passed: check.passed,
                        detail: check.details.clone().unwrap_or_default(),
                    });
                }
                if !result.passed {
                    warnings.push("验证未全部通过，请检查失败项".to_string());
                }
            }
            Err(e) => {
                checks.push(PhaseCheck {
                    name: "验证执行".to_string(),
                    passed: false,
                    detail: format!("失败: {}", e),
                });
            }
        }

        let rules = ArchitectureRules::default();
        let arch_result = validate_architecture(Path::new(project_path), &rules);
        let state = self.load_state(project_path)?;
        let new_violations = arch_result.violations.len() as isize - state.baseline_violations as isize;
        checks.push(PhaseCheck {
            name: "架构违规变化".to_string(),
            passed: new_violations <= 0,
            detail: format!("基线: {}, 当前: {}, 变化: {:+}", 
                state.baseline_violations, arch_result.violations.len(), new_violations),
        });

        let mut state = self.load_state(project_path)?;
        state.current_phase = DevPhase::Verifying;
        state.last_updated = chrono::Utc::now().to_rfc3339();
        self.save_state(project_path, &state)?;

        let success = checks.iter().all(|c| c.passed);
        Ok(PhaseResult {
            phase: DevPhase::Verifying,
            success,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            warnings,
        })
    }

    pub fn handoff_phase(&self, project_path: &str, message: Option<&str>) -> CellResult<PhaseResult> {
        let start = std::time::Instant::now();
        let mut checks = Vec::new();
        let mut warnings = Vec::new();

        self.progress_service.log_event(
            project_path,
            EventType::Update,
            "进入交接阶段",
            Some("生成交接包并准备提交"),
        )?;

        let state = self.load_state(project_path)?;
        let handoff_result = self.handoff_service.generate(
            project_path,
            &state.task_name,
            Some("cell-architect"),
        );

        match handoff_result {
            Ok(pkg) => {
                checks.push(PhaseCheck {
                    name: "交接包生成".to_string(),
                    passed: true,
                    detail: format!("已生成: {}", pkg.package_id),
                });
            }
            Err(e) => {
                checks.push(PhaseCheck {
                    name: "交接包生成".to_string(),
                    passed: false,
                    detail: format!("失败: {}", e),
                });
                warnings.push(format!("交接包生成失败: {}", e));
            }
        }

        let current_entropy = match entropy_service::run_entropy_check(project_path) {
            Ok(report) => {
                checks.push(PhaseCheck {
                    name: "最终熵值".to_string(),
                    passed: report.overall_score < 60.0,
                    detail: format!("最终熵值: {:.2}", report.overall_score),
                });
                Some(report.overall_score)
            }
            Err(e) => {
                warnings.push(format!("熵值计算失败: {}", e));
                None
            }
        };

        if let (Some(baseline), Some(current)) = (state.baseline_entropy, current_entropy) {
            let diff = current - baseline;
            if diff > 0.0 {
                match self.evolution_service.report_issue(
                    project_path,
                    "熵值上升",
                    &format!("本次开发熵值上升 {:.2}", diff),
                    IssueCategory::EntropyGrowth,
                    IssueSeverity::Medium,
                ) {
                    Ok(_) => {
                        checks.push(PhaseCheck {
                            name: "自进化记录".to_string(),
                            passed: true,
                            detail: "熵值上升问题已记录".to_string(),
                        });
                    }
                    Err(e) => {
                        warnings.push(format!("自进化记录失败: {}", e));
                    }
                }
            }
        }

        self.progress_service.complete_task(project_path)?;
        checks.push(PhaseCheck {
            name: "任务完成".to_string(),
            passed: true,
            detail: "进度追踪已结束".to_string(),
        });

        let mut state = self.load_state(project_path)?;
        state.current_phase = DevPhase::HandingOff;
        state.last_updated = chrono::Utc::now().to_rfc3339();
        self.save_state(project_path, &state)?;

        let _ = message;
        Ok(PhaseResult {
            phase: DevPhase::HandingOff,
            success: true,
            duration_ms: start.elapsed().as_millis() as u64,
            checks,
            warnings,
        })
    }

    pub fn current_status(&self, project_path: &str) -> CellResult<DevWorkflowState> {
        self.load_state(project_path)
    }

    pub fn format_phase_result(&self, result: &PhaseResult) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n{} {}\n", result.phase.label(), "═".repeat(50)));
        output.push_str(&format!("  耗时: {}ms\n\n", result.duration_ms));

        output.push_str("  检查结果:\n");
        for check in &result.checks {
            let icon = if check.passed { "✅" } else { "❌" };
            output.push_str(&format!("    {} {:<20} - {}\n", icon, check.name, check.detail));
        }

        if !result.warnings.is_empty() {
            output.push_str("\n  ⚠️  警告:\n");
            for w in &result.warnings {
                output.push_str(&format!("    • {}\n", w));
            }
        }

        let status = if result.success { "✅ 通过" } else { "❌ 失败" };
        output.push_str(&format!("\n  阶段结果: {}\n", status));

        output
    }

    fn save_state(&self, project_path: &str, state: &DevWorkflowState) -> CellResult<()> {
        let state_dir = Path::new(project_path).join(".cell/workflow");
        std::fs::create_dir_all(&state_dir)?;
        let content = serde_json::to_string_pretty(state)?;
        std::fs::write(state_dir.join("state.json"), content)?;
        Ok(())
    }

    fn load_state(&self, project_path: &str) -> CellResult<DevWorkflowState> {
        let state_file = Path::new(project_path).join(".cell/workflow/state.json");
        if !state_file.exists() {
            return Err(crate::domain::errors::CellError::Config(
                "工作流未启动，请先运行 cell dev start".to_string()
            ));
        }
        let content = std::fs::read_to_string(state_file)?;
        let state: DevWorkflowState = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn record_decision(&self, project_path: &str, store: D, title: &str, context: &str, decision: &str) -> CellResult<()> {
        let service = DecisionService::new(store);
        
        let record = service.record_decision(
            project_path,
            title,
            context,
            decision,
            "",
            DecisionCategory::Architecture,
            Some("workflow"),
        )?;

        let mut state = self.load_state(project_path)?;
        state.decisions.push(record.id.simple().to_string());
        state.last_updated = chrono::Utc::now().to_rfc3339();
        self.save_state(project_path, &state)?;

        Ok(())
    }
}
