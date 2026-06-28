use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventFieldType {
    String,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Bytes,
    Timestamp,
    Nested(String),
    Array(Box<Self>),
    Optional(Box<Self>),
}

impl EventFieldType {
    pub fn description(&self) -> String {
        match self {
            Self::String => "string".to_string(),
            Self::Int32 => "int32".to_string(),
            Self::Int64 => "int64".to_string(),
            Self::Float32 => "float".to_string(),
            Self::Float64 => "double".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Bytes => "bytes".to_string(),
            Self::Timestamp => "timestamp".to_string(),
            Self::Nested(name) => format!("message({name})"),
            Self::Array(inner) => format!("repeated {}", inner.description()),
            Self::Optional(inner) => format!("optional {}", inner.description()),
        }
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String, Self::String)
            | (Self::Int32, Self::Int32 | Self::Int64)
            | (Self::Int64, Self::Int64)
            | (Self::Float32, Self::Float32 | Self::Float64)
            | (Self::Float64, Self::Float64)
            | (Self::Bool, Self::Bool)
            | (Self::Bytes, Self::Bytes)
            | (Self::Timestamp, Self::Timestamp) => true,
            (Self::Nested(a), Self::Nested(b)) => a == b,
            (Self::Array(a), Self::Array(b)) | (Self::Optional(a), Self::Optional(b)) => {
                a.is_compatible_with(b)
            }
            (_, Self::Optional(b)) => self.is_compatible_with(b),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventField {
    pub name: String,
    pub field_type: EventFieldType,
    pub description: String,
    pub required: bool,
    pub position: i32,
    pub deprecated: bool,
    pub since_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchema {
    pub name: String,
    pub version: String,
    pub description: String,
    pub fields: Vec<EventField>,
    pub nested_types: HashMap<String, Vec<EventField>>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

impl EventSchema {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            fields: Vec::new(),
            nested_types: HashMap::new(),
            source: String::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn add_field(&mut self, field: EventField) -> &mut Self {
        self.fields.push(field);
        self
    }

    pub fn check_compatibility(&self, old: &Self) -> CompatibilityResult {
        let mut result = CompatibilityResult::new();

        for old_field in &old.fields {
            let found = self.fields.iter().find(|f| f.position == old_field.position);
            
            match found {
                None => {
                    if old_field.required {
                        result.errors.push(format!(
                            "Field '{}' (tag {}) was removed but was required",
                            old_field.name, old_field.position
                        ));
                    } else {
                        result.warnings.push(format!(
                            "Field '{}' (tag {}) was removed",
                            old_field.name, old_field.position
                        ));
                    }
                }
                Some(new_field) => {
                    if new_field.name != old_field.name {
                        result.warnings.push(format!(
                            "Field name changed: '{}' -> '{}' (tag {})",
                            old_field.name, new_field.name, old_field.position
                        ));
                    }

                    if !new_field.field_type.is_compatible_with(&old_field.field_type) {
                        result.errors.push(format!(
                            "Field '{}' type changed incompatibly: {} -> {}",
                            new_field.name,
                            old_field.field_type.description(),
                            new_field.field_type.description()
                        ));
                    }

                    if old_field.required && !new_field.required {
                        result.warnings.push(format!(
                            "Field '{}' changed from required to optional",
                            new_field.name
                        ));
                    }

                    if !old_field.required && new_field.required && old.version != self.version {
                        result.errors.push(format!(
                            "Field '{}' changed from optional to required (breaking)",
                            new_field.name
                        ));
                    }

                    if old_field.deprecated && !new_field.deprecated {
                        result.info.push(format!(
                            "Field '{}' was un-deprecated",
                            new_field.name
                        ));
                    }
                }
            }
        }

        for new_field in &self.fields {
            let found = old.fields.iter().find(|f| f.position == new_field.position);
            if found.is_none() {
                if new_field.required {
                    result.warnings.push(format!(
                        "New required field '{}' (tag {}) added - may break consumers",
                        new_field.name, new_field.position
                    ));
                } else {
                    result.info.push(format!(
                        "New optional field '{}' (tag {}) added",
                        new_field.name, new_field.position
                    ));
                }
            }
        }

        result
    }

    pub fn to_proto(&self) -> String {
        let mut lines = Vec::new();
        lines.push("syntax = \"proto3\";".to_string());
        lines.push(String::new());
        lines.push(format!("package {};", self.name.to_lowercase().replace(' ', "_")));
        lines.push(String::new());
        lines.push("/**".to_string());
        lines.push(format!(" * {}", self.description));
        lines.push(format!(" * Version: {}", self.version));
        lines.push(format!(" * Source: {}", self.source));
        lines.push(" */".to_string());
        lines.push(String::new());

        for (nested_name, nested_fields) in &self.nested_types {
            lines.push(format!("message {nested_name} {{"));
            for field in nested_fields {
                lines.push(format!(
                    "  {} {} = {}; // {}",
                    field.field_type.description(),
                    field.name,
                    field.position,
                    field.description
                ));
            }
            lines.push("}".to_string());
            lines.push(String::new());
        }

        lines.push(format!("message {}Event {{", self.name));
        for field in &self.fields {
            let dep_str = if field.deprecated { " [deprecated = true]" } else { "" };
            lines.push(format!(
                "  {} {} = {}{}; // {}",
                field.field_type.description(),
                field.name,
                field.position,
                dep_str,
                field.description
            ));
        }
        lines.push("}".to_string());

        lines.join("\n")
    }

    pub fn to_json_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for field in &self.fields {
            let prop = match &field.field_type {
                EventFieldType::String => {
                    serde_json::json!({ "type": "string", "description": field.description })
                }
                EventFieldType::Int32 | EventFieldType::Int64 => {
                    serde_json::json!({ "type": "integer", "description": field.description })
                }
                EventFieldType::Float32 | EventFieldType::Float64 => {
                    serde_json::json!({ "type": "number", "description": field.description })
                }
                EventFieldType::Bool => {
                    serde_json::json!({ "type": "boolean", "description": field.description })
                }
                _ => serde_json::json!({ "type": "object", "description": field.description }),
            };
            properties.insert(field.name.clone(), prop);
            if field.required {
                required.push(field.name.clone());
            }
        }

        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": self.name,
            "description": self.description,
            "version": self.version,
            "type": "object",
            "properties": properties,
            "required": required
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    pub compatible: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl CompatibilityResult {
    pub fn new() -> Self {
        Self {
            compatible: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    pub fn is_breaking(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for CompatibilityResult {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventSchemaRegistry {
    schemas: HashMap<String, Vec<EventSchema>>,
}

impl EventSchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register(&mut self, schema: EventSchema) -> Result<&EventSchema, String> {
        let name = schema.name.clone();
        let versions = self.schemas.entry(name).or_default();

        if let Some(latest) = versions.last() {
            let compat = schema.check_compatibility(latest);
            if compat.is_breaking() {
                return Err(format!(
                    "Breaking change detected: {:?}",
                    compat.errors
                ));
            }
        }

        versions.push(schema);
        Ok(versions.last().unwrap())
    }

    pub fn get_latest(&self, name: &str) -> Option<&EventSchema> {
        self.schemas.get(name).and_then(|v| v.last())
    }

    pub fn get_version(&self, name: &str, version: &str) -> Option<&EventSchema> {
        self.schemas
            .get(name)
            .and_then(|v| v.iter().find(|s| s.version == version))
    }

    pub fn list_schemas(&self) -> Vec<(String, Vec<String>)> {
        self.schemas
            .iter()
            .map(|(name, versions)| {
                (name.clone(), versions.iter().map(|s| s.version.clone()).collect())
            })
            .collect()
    }

    pub fn all_schemas(&self) -> Vec<&EventSchema> {
        let mut result = Vec::new();
        for versions in self.schemas.values() {
            result.extend(versions.iter());
        }
        result
    }
}

impl Default for EventSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema_v1() -> EventSchema {
        let mut schema = EventSchema::new("UserCreated", "1.0.0");
        schema.description = "User created event".to_string();
        schema.source = "user-service".to_string();
        schema.add_field(EventField {
            name: "user_id".to_string(),
            field_type: EventFieldType::String,
            description: "Unique user identifier".to_string(),
            required: true,
            position: 1,
            deprecated: false,
            since_version: "1.0.0".to_string(),
        });
        schema.add_field(EventField {
            name: "email".to_string(),
            field_type: EventFieldType::String,
            description: "User email address".to_string(),
            required: true,
            position: 2,
            deprecated: false,
            since_version: "1.0.0".to_string(),
        });
        schema.add_field(EventField {
            name: "display_name".to_string(),
            field_type: EventFieldType::String,
            description: "Display name".to_string(),
            required: false,
            position: 3,
            deprecated: false,
            since_version: "1.0.0".to_string(),
        });
        schema
    }

    #[test]
    fn test_schema_creation() {
        let schema = create_test_schema_v1();
        assert_eq!(schema.name, "UserCreated");
        assert_eq!(schema.version, "1.0.0");
        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_compatible_change_add_optional_field() {
        let v1 = create_test_schema_v1();
        let mut v2 = create_test_schema_v1();
        v2.version = "1.1.0".to_string();
        v2.add_field(EventField {
            name: "avatar_url".to_string(),
            field_type: EventFieldType::String,
            description: "User avatar URL".to_string(),
            required: false,
            position: 4,
            deprecated: false,
            since_version: "1.1.0".to_string(),
        });

        let result = v2.check_compatibility(&v1);
        assert!(!result.is_breaking());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_breaking_change_remove_required_field() {
        let v1 = create_test_schema_v1();
        let mut v2 = create_test_schema_v1();
        v2.version = "2.0.0".to_string();
        v2.fields.retain(|f| f.name != "email");

        let result = v2.check_compatibility(&v1);
        assert!(result.is_breaking());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_breaking_change_type_mismatch() {
        let v1 = create_test_schema_v1();
        let mut v2 = create_test_schema_v1();
        v2.version = "2.0.0".to_string();
        v2.fields[1].field_type = EventFieldType::Int32;

        let result = v2.check_compatibility(&v1);
        assert!(result.is_breaking());
    }

    #[test]
    fn test_to_proto_generation() {
        let schema = create_test_schema_v1();
        let proto = schema.to_proto();
        assert!(proto.contains("syntax = \"proto3\""));
        assert!(proto.contains("message UserCreatedEvent"));
        assert!(proto.contains("user_id"));
        assert!(proto.contains("email"));
    }

    #[test]
    fn test_to_json_schema() {
        let schema = create_test_schema_v1();
        let json = schema.to_json_schema();
        assert_eq!(json["title"], "UserCreated");
        assert_eq!(json["version"], "1.0.0");
        assert!(json["properties"]["user_id"].is_object());
        assert!(json["required"].as_array().unwrap().contains(&serde_json::json!("user_id")));
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = EventSchemaRegistry::new();
        let schema = create_test_schema_v1();
        registry.register(schema).unwrap();

        let latest = registry.get_latest("UserCreated");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, "1.0.0");
    }

    #[test]
    fn test_registry_rejects_breaking_change() {
        let mut registry = EventSchemaRegistry::new();
        let v1 = create_test_schema_v1();
        registry.register(v1).unwrap();

        let mut v2 = create_test_schema_v1();
        v2.version = "2.0.0".to_string();
        v2.fields.retain(|f| f.name != "email");

        let result = registry.register(v2);
        assert!(result.is_err());
    }

    #[test]
    fn test_field_type_compatibility() {
        assert!(EventFieldType::String.is_compatible_with(&EventFieldType::String));
        assert!(EventFieldType::Int32.is_compatible_with(&EventFieldType::Int64));
        assert!(!EventFieldType::String.is_compatible_with(&EventFieldType::Int32));
        assert!(EventFieldType::Int32.is_compatible_with(&EventFieldType::Optional(Box::new(EventFieldType::Int64))));
    }

    #[test]
    fn test_list_schemas() {
        let mut registry = EventSchemaRegistry::new();
        registry.register(create_test_schema_v1()).unwrap();

        let list = registry.list_schemas();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "UserCreated");
        assert_eq!(list[0].1, vec!["1.0.0"]);
    }
}
