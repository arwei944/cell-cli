use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_id: String,
    pub agent_name: String,
    pub role: String,
    pub overall_score: f64,
    pub quality_score: f64,
    pub speed_score: f64,
    pub reliability_score: f64,
    pub entropy_impact: f64,
    pub skill_tags: Vec<SkillTag>,
    pub stats: AgentStats,
    pub registered_at: String,
    pub last_active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTag {
    pub name: String,
    pub confidence: f64,
    pub task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub on_time_rate: f64,
    pub first_pass_rate: f64,
    pub avg_task_duration_minutes: f64,
    pub bugs_introduced: usize,
    pub reviews_requested: usize,
    pub review_approval_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRanking {
    pub rank: usize,
    pub agent_id: String,
    pub agent_name: String,
    pub score: f64,
    pub role: String,
    pub trend: RankingTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RankingTrend {
    Up,
    Down,
    Stable,
    New,
}

pub struct AgentProfileService;

impl AgentProfileService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_profile(&self, project_path: &str, agent_id: &str) -> CellResult<AgentProfile> {
        let profile_path = Path::new(project_path)
            .join(".cell")
            .join("agent_profiles")
            .join(format!("{agent_id}.json"));

        if profile_path.exists() {
            let content = std::fs::read_to_string(&profile_path)?;
            let profile: AgentProfile = serde_json::from_str(&content)?;
            Ok(profile)
        } else {
            let profile = self.create_default_profile(agent_id);
            self.save_profile(project_path, &profile)?;
            Ok(profile)
        }
    }

    fn create_default_profile(&self, agent_id: &str) -> AgentProfile {
        AgentProfile {
            agent_id: agent_id.to_string(),
            agent_name: format!("Agent-{}", &agent_id[..8.min(agent_id.len())]),
            role: "Developer".to_string(),
            overall_score: 75.0,
            quality_score: 75.0,
            speed_score: 70.0,
            reliability_score: 80.0,
            entropy_impact: 0.0,
            skill_tags: vec![
                SkillTag {
                    name: "Rust".to_string(),
                    confidence: 0.7,
                    task_count: 0,
                },
                SkillTag {
                    name: "架构设计".to_string(),
                    confidence: 0.6,
                    task_count: 0,
                },
            ],
            stats: AgentStats {
                total_tasks: 0,
                completed_tasks: 0,
                on_time_rate: 0.0,
                first_pass_rate: 0.0,
                avg_task_duration_minutes: 0.0,
                bugs_introduced: 0,
                reviews_requested: 0,
                review_approval_rate: 0.0,
            },
            registered_at: chrono::Utc::now().to_rfc3339(),
            last_active: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn save_profile(&self, project_path: &str, profile: &AgentProfile) -> CellResult<()> {
        let profiles_dir = Path::new(project_path)
            .join(".cell")
            .join("agent_profiles");
        std::fs::create_dir_all(&profiles_dir)?;

        let file_path = profiles_dir.join(format!("{}.json", profile.agent_id));
        std::fs::write(&file_path, serde_json::to_string_pretty(profile)?)?;

        Ok(())
    }

    pub fn list_profiles(&self, project_path: &str) -> CellResult<Vec<AgentProfile>> {
        let profiles_dir = Path::new(project_path)
            .join(".cell")
            .join("agent_profiles");

        let mut profiles = Vec::new();

        if profiles_dir.exists() {
            for entry in std::fs::read_dir(&profiles_dir)? {
                let entry = entry?;
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    let content = std::fs::read_to_string(entry.path())?;
                    if let Ok(profile) = serde_json::from_str::<AgentProfile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }

        profiles.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(profiles)
    }

    pub fn get_ranking(&self, project_path: &str) -> CellResult<Vec<AgentRanking>> {
        let profiles = self.list_profiles(project_path)?;

        let rankings: Vec<AgentRanking> = profiles
            .iter()
            .enumerate()
            .map(|(i, p)| AgentRanking {
                rank: i + 1,
                agent_id: p.agent_id.clone(),
                agent_name: p.agent_name.clone(),
                score: p.overall_score,
                role: p.role.clone(),
                trend: if i == 0 {
                    RankingTrend::Stable
                } else {
                    RankingTrend::New
                },
            })
            .collect();

        Ok(rankings)
    }

    pub fn update_task_completion(
        &self,
        project_path: &str,
        agent_id: &str,
        success: bool,
        on_time: bool,
        _duration_minutes: u64,
    ) -> CellResult<()> {
        let mut profile = self.get_profile(project_path, agent_id)?;

        profile.stats.total_tasks += 1;
        if success {
            profile.stats.completed_tasks += 1;
        }

        if profile.stats.total_tasks > 0 {
            profile.stats.on_time_rate = if on_time {
                profile.stats.on_time_rate.mul_add(profile.stats.total_tasks as f64 - 1.0, 1.0)
                    / profile.stats.total_tasks as f64
            } else {
                (profile.stats.on_time_rate * (profile.stats.total_tasks as f64 - 1.0))
                    / profile.stats.total_tasks as f64
            };
        }

        profile.overall_score =
            (profile.quality_score + profile.speed_score + profile.reliability_score) / 3.0;
        profile.last_active = chrono::Utc::now().to_rfc3339();

        self.save_profile(project_path, &profile)?;
        Ok(())
    }

    pub fn format_profile(&self, profile: &AgentProfile) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n🤖 Agent 画像: {}\n\n", profile.agent_name));
        output.push_str(&format!("  ID: {}\n", profile.agent_id));
        output.push_str(&format!("  角色: {}\n", profile.role));
        output.push_str(&format!("  注册时间: {}\n", profile.registered_at));
        output.push_str(&format!("  最后活跃: {}\n", profile.last_active));

        output.push_str(&format!(
            "\n📊 综合评分: {:.1} / 100\n\n",
            profile.overall_score
        ));
        output.push_str(&format!("  质量分: {:.1}\n", profile.quality_score));
        output.push_str(&format!("  速度分: {:.1}\n", profile.speed_score));
        output.push_str(&format!("  可靠度: {:.1}\n", profile.reliability_score));
        output.push_str(&format!("  熵值影响: {:+.1}\n", profile.entropy_impact));

        output.push_str("\n📈 统计数据:\n");
        output.push_str(&format!(
            "  总任务数: {} (完成: {})\n",
            profile.stats.total_tasks, profile.stats.completed_tasks
        ));
        output.push_str(&format!(
            "  按时完成率: {:.1}%\n",
            profile.stats.on_time_rate * 100.0
        ));
        output.push_str(&format!(
            "  一次通过率: {:.1}%\n",
            profile.stats.first_pass_rate * 100.0
        ));
        output.push_str(&format!(
            "  平均任务耗时: {:.1} 分钟\n",
            profile.stats.avg_task_duration_minutes
        ));
        output.push_str(&format!("  引入 Bug 数: {}\n", profile.stats.bugs_introduced));
        output.push_str(&format!(
            "  评审通过率: {:.1}%\n",
            profile.stats.review_approval_rate * 100.0
        ));

        output.push_str("\n🏷️  技能标签:\n");
        for tag in &profile.skill_tags {
            let bar_len = (tag.confidence * 20.0) as usize;
            let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
            output.push_str(&format!(
                "  {} |{}| {:.0}% ({} 任务)\n",
                tag.name, bar, tag.confidence * 100.0, tag.task_count
            ));
        }

        output
    }

    pub fn format_ranking(&self, rankings: &[AgentRanking]) -> String {
        let mut output = String::new();

        output.push_str("\n🏆 Agent 排行榜\n\n");
        output.push_str(&format!(
            "  {:>3}  {:<20}  {:<10}  {:>6}  {}\n",
            "排名", "名称", "角色", "评分", "趋势"
        ));
        output.push_str(&format!("  {}\n", "─".repeat(60)));

        for r in rankings {
            let trend_icon = match r.trend {
                RankingTrend::Up => "⬆️",
                RankingTrend::Down => "⬇️",
                RankingTrend::Stable => "➡️",
                RankingTrend::New => "🆕",
            };
            output.push_str(&format!(
                "  {:>3}  {:<20}  {:<10}  {:>5.1}  {}\n",
                r.rank, r.agent_name, r.role, r.score, trend_icon
            ));
        }

        output
    }
}

impl Default for AgentProfileService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_service_new() {
        let service = AgentProfileService::new();
        let _ = service;
    }

    #[test]
    fn test_get_profile() {
        let service = AgentProfileService::new();
        let profile = service.get_profile(".", "test-agent-001").unwrap();
        assert_eq!(profile.agent_id, "test-agent-001");
        assert!(profile.overall_score > 0.0);
    }

    #[test]
    fn test_list_profiles() {
        let service = AgentProfileService::new();
        let profiles = service.list_profiles(".").unwrap();
        assert!(!profiles.is_empty());
    }

    #[test]
    fn test_get_ranking() {
        let service = AgentProfileService::new();
        let _ = service.get_profile(".", "test-agent-ranking-1").unwrap();
        let _ = service.get_profile(".", "test-agent-ranking-2").unwrap();
        let rankings = service.get_ranking(".").unwrap();
        assert!(!rankings.is_empty());
    }

    #[test]
    fn test_update_task_completion() {
        let service = AgentProfileService::new();
        let before = service.get_profile(".", "test-agent-update-task").unwrap();
        let before_completed = before.stats.completed_tasks;
        service
            .update_task_completion(".", "test-agent-update-task", true, true, 30)
            .unwrap();
        let profile = service.get_profile(".", "test-agent-update-task").unwrap();
        assert_eq!(profile.stats.completed_tasks, before_completed + 1);
    }
}
