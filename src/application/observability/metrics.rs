//! 可观测性服务：性能指标跟踪
//! Observability service: performance metrics tracking

use crate::domain::observability::{CommandExecution, CommandStat, PerformanceMetrics};
use std::sync::Mutex;

fn default_command_stat() -> CommandStat {
    CommandStat {
        count: 0,
        avg_time_ms: 0.0,
        min_time_ms: f64::MAX,
        max_time_ms: 0.0,
        success_count: 0,
        failure_count: 0,
    }
}

pub struct ObservabilityService {
    performance_data: Mutex<PerformanceMetrics>,
    command_history: Mutex<Vec<CommandExecution>>,
}

impl ObservabilityService {
    pub fn new() -> Self {
        Self {
            performance_data: Mutex::new(PerformanceMetrics::default()),
            command_history: Mutex::new(Vec::new()),
        }
    }

    pub fn record_command_start(&self, command: &str, args: Vec<String>) -> usize {
        let exec = CommandExecution::new(command, args);
        let mut history = self.command_history.lock().expect("command_history mutex poisoned");
        history.push(exec);
        history.len() - 1
    }

    pub fn record_command_end(&self, index: usize, success: bool, error: Option<&str>) {
        let mut history = self.command_history.lock().expect("command_history mutex poisoned");
        if let Some(exec) = history.get_mut(index) {
            exec.complete(success, error);
            self.update_performance_metrics(exec);
        }
    }

    fn update_performance_metrics(&self, exec: &CommandExecution) {
        let mut metrics = self.performance_data.lock().expect("performance_data mutex poisoned");
        metrics.command_count += 1;

        let duration = exec.duration_ms.unwrap_or(0.0);
        metrics.avg_command_time_ms = running_average(
            metrics.avg_command_time_ms,
            duration,
            metrics.command_count as usize,
        );

        let cmd_stat = metrics
            .command_stats
            .entry(exec.command.clone())
            .or_insert_with(default_command_stat);
        update_command_stat(cmd_stat, duration, exec.success.unwrap_or(false));

        metrics.success_rate = compute_success_rate(&metrics);
        metrics.last_updated = chrono::Utc::now();
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_data
            .lock()
            .expect("performance_data mutex poisoned")
            .clone()
    }
}

impl Default for ObservabilityService {
    fn default() -> Self { Self::new() }
}

fn running_average(previous: f64, current: f64, count: usize) -> f64 {
    if count <= 1 {
        current
    } else {
        (previous * (count - 1) as f64 + current) / count as f64
    }
}

fn update_command_stat(stat: &mut CommandStat, duration: f64, success: bool) {
    stat.count += 1;
    stat.avg_time_ms = running_average(stat.avg_time_ms, duration, stat.count as usize);
    stat.min_time_ms = stat.min_time_ms.min(duration);
    stat.max_time_ms = stat.max_time_ms.max(duration);
    if success {
        stat.success_count += 1;
    } else {
        stat.failure_count += 1;
    }
}

fn compute_success_rate(metrics: &PerformanceMetrics) -> f64 {
    if metrics.command_count == 0 {
        return 100.0;
    }
    let total_success: u64 = metrics.command_stats.values().map(|s| s.success_count).sum();
    total_success as f64 / metrics.command_count as f64 * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tracking() {
        let service = ObservabilityService::new();
        let idx = service.record_command_start("test", vec!["arg1".to_string()]);
        service.record_command_end(idx, true, None);

        let metrics = service.get_performance_metrics();
        assert_eq!(metrics.command_count, 1);
        assert_eq!(metrics.success_rate, 100.0);
    }
}
