use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMessage {
    pub event_type: DashboardEventType,
    pub data: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardEventType {
    EntropyUpdated,
    ProgressUpdated,
    ArchitectureChanged,
    ViolationDetected,
    DecisionCreated,
    FeatureMounted,
    TestCompleted,
    BuildCompleted,
    GitCommit,
    AgentHeartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyUpdate {
    pub score: f64,
    pub grade: String,
    pub trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub phase: String,
    pub percent: f64,
    pub tasks_completed: usize,
    pub tasks_total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureUpdate {
    pub violations: usize,
    pub layer_stats: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub entropy: Option<EntropyUpdate>,
    pub progress: Option<ProgressUpdate>,
    pub architecture: Option<ArchitectureUpdate>,
    pub last_update: String,
    pub active_agents: Vec<String>,
}

pub struct WebSocketDashboardService {
    broadcaster: broadcast::Sender<DashboardMessage>,
    state: Arc<tokio::sync::RwLock<DashboardState>>,
}

impl WebSocketDashboardService {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(100);
        let state = Arc::new(tokio::sync::RwLock::new(DashboardState {
            entropy: None,
            progress: None,
            architecture: None,
            last_update: chrono::Utc::now().to_rfc3339(),
            active_agents: Vec::new(),
        }));
        Self { broadcaster, state }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DashboardMessage> {
        self.broadcaster.subscribe()
    }

    pub async fn broadcast(&self, message: DashboardMessage) -> CellResult<()> {
        let _ = self.broadcaster.send(message);
        Ok(())
    }

    pub async fn update_entropy(&self, score: f64, grade: &str, trend: &str) -> CellResult<()> {
        let mut state = self.state.write().await;
        state.entropy = Some(EntropyUpdate {
            score,
            grade: grade.to_string(),
            trend: trend.to_string(),
        });
        state.last_update = chrono::Utc::now().to_rfc3339();
        
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::EntropyUpdated,
            data: serde_json::to_value(&state.entropy).unwrap_or(serde_json::Value::Null),
            timestamp: state.last_update.clone(),
        }).await?;
        Ok(())
    }

    pub async fn update_progress(&self, phase: &str, percent: f64, completed: usize, total: usize) -> CellResult<()> {
        let mut state = self.state.write().await;
        state.progress = Some(ProgressUpdate {
            phase: phase.to_string(),
            percent,
            tasks_completed: completed,
            tasks_total: total,
        });
        state.last_update = chrono::Utc::now().to_rfc3339();

        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::ProgressUpdated,
            data: serde_json::to_value(&state.progress).unwrap_or(serde_json::Value::Null),
            timestamp: state.last_update.clone(),
        }).await?;
        Ok(())
    }

    pub async fn update_architecture(&self, violations: usize, layer_stats: std::collections::HashMap<String, usize>) -> CellResult<()> {
        let mut state = self.state.write().await;
        state.architecture = Some(ArchitectureUpdate {
            violations,
            layer_stats,
        });
        state.last_update = chrono::Utc::now().to_rfc3339();

        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::ArchitectureChanged,
            data: serde_json::to_value(&state.architecture).unwrap_or(serde_json::Value::Null),
            timestamp: state.last_update.clone(),
        }).await?;
        Ok(())
    }

    pub async fn notify_violation(&self, violation: crate::arch_service::Violation) -> CellResult<()> {
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::ViolationDetected,
            data: serde_json::to_value(&violation).unwrap_or(serde_json::Value::Null),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await?;
        Ok(())
    }

    pub async fn notify_decision_created(&self, decision_id: &str, title: &str) -> CellResult<()> {
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::DecisionCreated,
            data: serde_json::json!({
                "decision_id": decision_id,
                "title": title,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await?;
        Ok(())
    }

    pub async fn notify_feature_mounted(&self, feature_name: &str) -> CellResult<()> {
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::FeatureMounted,
            data: serde_json::json!({
                "feature_name": feature_name,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await?;
        Ok(())
    }

    pub async fn notify_test_completed(&self, passed: usize, failed: usize) -> CellResult<()> {
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::TestCompleted,
            data: serde_json::json!({
                "passed": passed,
                "failed": failed,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await?;
        Ok(())
    }

    pub async fn notify_build_completed(&self, success: bool, duration_ms: u64) -> CellResult<()> {
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::BuildCompleted,
            data: serde_json::json!({
                "success": success,
                "duration_ms": duration_ms,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await?;
        Ok(())
    }

    pub async fn notify_git_commit(&self, hash: &str, message: &str) -> CellResult<()> {
        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::GitCommit,
            data: serde_json::json!({
                "hash": hash,
                "message": message,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await?;
        Ok(())
    }

    pub async fn agent_heartbeat(&self, agent_name: &str) -> CellResult<()> {
        let mut state = self.state.write().await;
        if !state.active_agents.contains(&agent_name.to_string()) {
            state.active_agents.push(agent_name.to_string());
        }
        state.last_update = chrono::Utc::now().to_rfc3339();

        self.broadcast(DashboardMessage {
            event_type: DashboardEventType::AgentHeartbeat,
            data: serde_json::json!({
                "agent_name": agent_name,
                "active_agents": state.active_agents.clone(),
            }),
            timestamp: state.last_update.clone(),
        }).await?;
        Ok(())
    }

    pub async fn get_state(&self) -> DashboardState {
        self.state.read().await.clone()
    }

    pub fn generate_ws_html(&self) -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Cell Dashboard - WebSocket</title>
    <style>
        body { font-family: system-ui; background: #1a1a2e; color: #eee; margin: 0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .card { background: #16213e; border-radius: 8px; padding: 20px; margin-bottom: 20px; }
        .card h2 { margin: 0 0 15px; color: #4ecca3; }
        .metric { display: inline-block; margin-right: 30px; }
        .metric-value { font-size: 2em; font-weight: bold; color: #4ecca3; }
        .metric-label { color: #888; font-size: 0.8em; }
        #log { height: 300px; overflow-y: scroll; background: #0f0f23; padding: 10px; border-radius: 4px; font-size: 12px; }
        .log-entry { padding: 4px 0; border-bottom: 1px solid #333; }
        .log-time { color: #666; }
        .log-type { color: #4ecca3; font-weight: bold; }
        .status { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 12px; }
        .status.connected { background: #2ecc71; color: #fff; }
        .status.disconnected { background: #e74c3c; color: #fff; }
    </style>
</head>
<body>
    <div class="container">
        <div class="card">
            <h2>📊 Cell Architecture Dashboard <span id="ws-status" class="status disconnected">Disconnected</span></h2>
            <div class="metric">
                <div class="metric-value" id="entropy-score">--</div>
                <div class="metric-label">Entropy Score</div>
            </div>
            <div class="metric">
                <div class="metric-value" id="entropy-grade">--</div>
                <div class="metric-label">Grade</div>
            </div>
            <div class="metric">
                <div class="metric-value" id="violations">--</div>
                <div class="metric-label">Violations</div>
            </div>
            <div class="metric">
                <div class="metric-value" id="progress">--%</div>
                <div class="metric-label">Progress</div>
            </div>
        </div>
        <div class="card">
            <h2>🤖 Active Agents</h2>
            <div id="agents">None</div>
        </div>
        <div class="card">
            <h2>📜 Event Log</h2>
            <div id="log"></div>
        </div>
    </div>
    <script>
        const ws = new WebSocket('ws://localhost:3000/ws/dashboard');
        ws.onopen = () => document.getElementById('ws-status').className = 'status connected';
        ws.onclose = () => document.getElementById('ws-status').className = 'status disconnected';
        ws.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            addLog(msg.event_type, msg.timestamp);
            updateMetrics(msg);
        };
        function addLog(type, time) {
            const log = document.getElementById('log');
            const entry = document.createElement('div');
            entry.className = 'log-entry';
            entry.innerHTML = `<span class="log-time">${time}</span> <span class="log-type">${type}</span>`;
            log.appendChild(entry);
            log.scrollTop = log.scrollHeight;
        }
        function updateMetrics(msg) {
            if (msg.event_type === 'entropy_updated') {
                document.getElementById('entropy-score').textContent = msg.data.score.toFixed(2);
                document.getElementById('entropy-grade').textContent = msg.data.grade;
            }
            if (msg.event_type === 'progress_updated') {
                document.getElementById('progress').textContent = msg.data.percent.toFixed(1) + '%';
            }
            if (msg.event_type === 'architecture_changed') {
                document.getElementById('violations').textContent = msg.data.violations;
            }
            if (msg.event_type === 'agent_heartbeat') {
                document.getElementById('agents').textContent = msg.data.active_agents.join(', ') || 'None';
            }
        }
    </script>
</body>
</html>"#.to_string()
    }
}

impl Default for WebSocketDashboardService {
    fn default() -> Self {
        Self::new()
    }
}