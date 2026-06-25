use crate::application::auto_progress_service::{AutoProgressTracker, AutoProgressSnapshot};
use crate::adapters::file_decision_store::FileDecisionStore;
use crate::adapters::file_evolution_store::FileEvolutionStore;
use crate::adapters::file_progress_store::FileProgressStore;
use crate::application::decision_service::DecisionService;
use crate::application::entropy_service::run_entropy_check;
use crate::application::evolution_service::EvolutionService;
use crate::application::observability::{build_dashboard, ObservabilityService};
use crate::application::progress_service::ProgressService;
use crate::domain::entropy::EntropyReport;
use crate::domain::errors::CellResult;
use crate::domain::observability::DashboardData;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, Json, IntoResponse};
use axum::routing::get;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

/// 将可序列化值包装为 JSON 响应，统一处理序列化错误
fn json_ok<T: Serialize>(value: T) -> Result<Json<serde_json::Value>, StatusCode> {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

struct AppState {
    project_path: String,
    observability: ObservabilityService,
    entropy_cache: Mutex<Option<CachedEntropy>>,
    #[allow(dead_code)]
    auto_tracker: AutoProgressTracker,
    auto_progress_cache: Mutex<Option<AutoProgressSnapshot>>,
}

struct CachedEntropy {
    report: EntropyReport,
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
}

impl AppState {
    fn new(project_path: String) -> Self {
        Self {
            project_path,
            observability: ObservabilityService::new(),
            entropy_cache: Mutex::new(None),
            auto_tracker: AutoProgressTracker::new(),
            auto_progress_cache: Mutex::new(None),
        }
    }

    fn get_cached_entropy(&self) -> Option<EntropyReport> {
        self.entropy_cache
            .lock()
            .ok()
            .and_then(|cache| cache.as_ref().map(|c| c.report.clone()))
    }

    fn set_cached_entropy(&self, report: EntropyReport) {
        if let Ok(mut cache) = self.entropy_cache.lock() {
            *cache = Some(CachedEntropy {
                report,
                timestamp: Utc::now(),
            });
        }
    }
}

pub async fn start_dashboard_server(project_path: &str, port: u16) -> CellResult<()> {
    let shared_state = Arc::new(AppState::new(project_path.to_string()));

    let state_clone = shared_state.clone();
    let path = project_path.to_string();
    tokio::spawn(async move {
        println!("⏳ 正在初始化熵值计算缓存（首次可能需要几分钟）...");
        if let Ok(report) = run_entropy_check(&path) {
            state_clone.set_cached_entropy(report);
            println!("✅ 熵值缓存初始化完成");
        }
    });

    let app = Router::new()
        .route("/", get(dashboard_page))
        .route("/api/dashboard", get(get_dashboard_data))
        .route("/api/entropy", get(get_entropy_data))
        .route("/api/entropy/refresh", get(refresh_entropy_data))
        .route("/api/progress", get(get_progress_data))
        .route("/api/progress/auto", get(get_auto_progress))
        .route("/api/progress/auto/refresh", get(refresh_auto_progress))
        .route("/api/decisions", get(get_decisions_data))
        .route("/api/decisions/{id}", get(get_decision_detail))
        .route("/api/evolution", get(get_evolution_data))
        .route("/api/metrics", get(get_performance_metrics))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 仪表盘已启动: http://localhost:{}", port);
    println!("   按 Ctrl+C 停止");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::domain::errors::CellError::Config(format!("绑定端口失败: {e}")))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::domain::errors::CellError::Config(format!("服务运行失败: {e}")))?;

    Ok(())
}

async fn dashboard_page() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

async fn get_dashboard_data(State(state): State<Arc<AppState>>) -> Result<Json<DashboardData>, StatusCode> {
    let entropy_report = state.get_cached_entropy().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let progress_store = FileProgressStore::new();
    let progress_service = ProgressService::new(progress_store);
    let current_progress = progress_service.get_current(&state.project_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let decision_store = FileDecisionStore::new();
    let decision_service = DecisionService::new(decision_store);
    let decisions = decision_service.list_decisions(&state.project_path, None, None).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let evolution_store = FileEvolutionStore::new();
    let evolution_service = EvolutionService::new(evolution_store);
    let evolution = evolution_service.get_current_cycle(&state.project_path).ok().flatten();

    let dashboard = build_dashboard(
        &state.observability,
        &state.project_path,
        &entropy_report,
        current_progress.as_ref(),
        &decisions,
        evolution.as_ref(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(dashboard))
}

async fn get_entropy_data(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let report = state.get_cached_entropy().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    json_ok(report)
}

async fn refresh_entropy_data(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let state_clone = state.clone();
    let path = state.project_path.clone();
    tokio::spawn(async move {
        if let Ok(report) = run_entropy_check(&path) {
            state_clone.set_cached_entropy(report);
        }
    });
    Ok(Json(serde_json::json!({ "status": "refreshing", "message": "熵值计算已在后台启动，请稍后刷新查看结果" })))
}

async fn get_progress_data(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = FileProgressStore::new();
    let service = ProgressService::new(store);
    let current = service.get_current(&state.project_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    json_ok(current)
}

async fn get_auto_progress(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Ok(cache) = state.auto_progress_cache.lock()
        && let Some(snapshot) = cache.as_ref()
    {
        return json_ok(snapshot);
    }

    Err(StatusCode::NOT_FOUND)
}

async fn refresh_auto_progress(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let state_clone = state.clone();
    let path = state.project_path.clone();

    tokio::spawn(async move {
        let mut tracker = AutoProgressTracker::new();
        if let Ok((_log, snapshot)) = tracker.analyze_current_state(&path)
            && let Ok(mut cache) = state_clone.auto_progress_cache.lock()
        {
            *cache = Some(snapshot);
        }
    });

    Ok(Json(serde_json::json!({
        "status": "refreshing",
        "message": "自动进度分析已在后台启动，请稍后刷新"
    })))
}

async fn get_decisions_data(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = FileDecisionStore::new();
    let service = DecisionService::new(store);
    let decisions = service.list_decisions(&state.project_path, None, None).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let metrics = service.get_metrics(&state.project_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "decisions": decisions,
        "metrics": metrics
    })))
}

async fn get_decision_detail(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = FileDecisionStore::new();
    let service = DecisionService::new(store);
    let decision = service.get_decision(&state.project_path, &id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match decision {
        Some(d) => json_ok(d),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_evolution_data(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = FileEvolutionStore::new();
    let service = EvolutionService::new(store);
    let current = service.get_current_cycle(&state.project_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let history = service.list_history(&state.project_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stats = service.get_evolution_summary(&state.project_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "current_cycle": current,
        "history": history,
        "stats": stats
    })))
}

async fn get_performance_metrics(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = state.observability.get_performance_metrics();
    json_ok(metrics)
}

const DASHBOARD_HTML: &str = include_str!("dashboard.html");