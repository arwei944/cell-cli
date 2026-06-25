use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressLog {
    pub task_id: Uuid,
    pub task_name: String,
    pub description: String,
    pub status: ProgressStatus,
    pub timeline: Vec<TimelineEvent>,
    pub related_files: Vec<String>,
    pub blockers: Vec<Blocker>,
    pub next_steps: Vec<NextStep>,
    pub assignee: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Blocked,
    Review,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimelineEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub message: String,
    pub details: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EventType {
    Start,
    Update,
    Decision,
    Blocker,
    FileModified,
    TestPass,
    TestFail,
    Complete,
    Cancel,
    Note,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blocker {
    pub id: Uuid,
    pub description: String,
    pub status: BlockerStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum BlockerStatus {
    Active,
    Resolved,
    Bypassed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NextStep {
    pub id: Uuid,
    pub description: String,
    pub priority: u8,
    pub estimated_minutes: Option<u32>,
    pub done: bool,
}

impl ProgressLog {
    pub fn new(task_name: &str, description: &str) -> Self {
        let now = Utc::now();
        Self {
            task_id: Uuid::new_v4(),
            task_name: task_name.to_string(),
            description: description.to_string(),
            status: ProgressStatus::NotStarted,
            timeline: Vec::new(),
            related_files: Vec::new(),
            blockers: Vec::new(),
            next_steps: Vec::new(),
            assignee: None,
            started_at: now,
            updated_at: now,
            completed_at: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn start(&mut self, assignee: Option<&str>) {
        self.status = ProgressStatus::InProgress;
        self.started_at = Utc::now();
        self.updated_at = self.started_at;
        if let Some(a) = assignee {
            self.assignee = Some(a.to_string());
        }
        self.add_event(EventType::Start, "Task started", None);
    }

    pub fn add_event(&mut self, event_type: EventType, message: &str, details: Option<&str>) {
        let event = TimelineEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
            author: self.assignee.clone(),
        };
        self.updated_at = event.timestamp;
        self.timeline.push(event);
    }

    pub fn add_blocker(&mut self, description: &str) -> Uuid {
        let blocker = Blocker {
            id: Uuid::new_v4(),
            description: description.to_string(),
            status: BlockerStatus::Active,
            created_at: Utc::now(),
            resolved_at: None,
            resolution: None,
        };
        let id = blocker.id;
        self.blockers.push(blocker);
        self.status = ProgressStatus::Blocked;
        self.updated_at = Utc::now();
        self.add_event(EventType::Blocker, &format!("Blocker added: {}", description), None);
        id
    }

    pub fn resolve_blocker(&mut self, blocker_id: Uuid, resolution: &str) -> bool {
        let idx = self.blockers.iter().position(|b| b.id == blocker_id);
        if let Some(i) = idx {
            let description = self.blockers[i].description.clone();
            self.blockers[i].status = BlockerStatus::Resolved;
            self.blockers[i].resolved_at = Some(Utc::now());
            self.blockers[i].resolution = Some(resolution.to_string());
            self.updated_at = Utc::now();
            self.add_event(
                EventType::Note,
                &format!("Blocker resolved: {}", description),
                Some(resolution),
            );
            if self.blockers.iter().all(|b| b.status != BlockerStatus::Active)
                && self.status == ProgressStatus::Blocked
            {
                self.status = ProgressStatus::InProgress;
            }
            return true;
        }
        false
    }

    pub fn add_next_step(&mut self, description: &str, priority: u8, estimated_minutes: Option<u32>) -> Uuid {
        let step = NextStep {
            id: Uuid::new_v4(),
            description: description.to_string(),
            priority,
            estimated_minutes,
            done: false,
        };
        let id = step.id;
        self.next_steps.push(step);
        self.updated_at = Utc::now();
        id
    }

    pub fn complete_next_step(&mut self, step_id: Uuid) -> bool {
        let idx = self.next_steps.iter().position(|s| s.id == step_id);
        if let Some(i) = idx {
            let description = self.next_steps[i].description.clone();
            self.next_steps[i].done = true;
            self.updated_at = Utc::now();
            self.add_event(EventType::Update, &format!("Completed: {}", description), None);
            return true;
        }
        false
    }

    pub fn add_related_file(&mut self, path: &str) {
        if !self.related_files.iter().any(|f| f == path) {
            self.related_files.push(path.to_string());
            self.updated_at = Utc::now();
        }
    }

    pub fn complete(&mut self) {
        self.status = ProgressStatus::Completed;
        let now = Utc::now();
        self.completed_at = Some(now);
        self.updated_at = now;
        self.add_event(EventType::Complete, "Task completed", None);
    }

    pub fn active_blockers_count(&self) -> usize {
        self.blockers.iter().filter(|b| b.status == BlockerStatus::Active).count()
    }

    pub fn pending_next_steps_count(&self) -> usize {
        self.next_steps.iter().filter(|s| !s.done).count()
    }
}
