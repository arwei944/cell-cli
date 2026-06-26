use crate::domain::event_schema::EventSchema;

/// 事件 Schema 服务
pub struct EventSchemaService;

impl EventSchemaService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_schemas(&self) -> Vec<EventSchema> {
        Vec::new()
    }
}

impl Default for EventSchemaService {
    fn default() -> Self {
        Self::new()
    }
}
