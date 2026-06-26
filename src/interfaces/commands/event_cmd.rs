use crate::domain::errors::{CellResult, CellError};
use crate::domain::event_schema::{EventSchema, EventSchemaRegistry, EventField, EventFieldType};
use crate::interfaces::cli::EventSub;
use std::fs;
use std::path::Path;

pub fn cmd_event(args: crate::interfaces::cli::EventArgs) -> CellResult<()> {
    match args.sub {
        EventSub::List { name } => list_schemas(name),
        EventSub::Show { name, version } => show_schema(&name, version.as_deref()),
        EventSub::Check { name, old, new } => check_compatibility(&name, &old, &new),
        EventSub::Proto { input, output } => generate_proto(&input, output.as_deref()),
        EventSub::JsonSchema { input, output } => generate_json_schema(&input, output.as_deref()),
    }
}

fn list_schemas(name_filter: Option<String>) -> CellResult<()> {
    let registry = load_schemas_from_dir("./events")?;
    let list = registry.list_schemas();

    println!("\n📋 Event Schema Registry\n");
    println!("{:<30} {:<20}", "Event Name", "Versions");
    println!("{}", "-".repeat(55));

    let filtered: Vec<_> = list.iter()
        .filter(|(name, _)| {
            name_filter.as_ref()
                .map(|f| name.to_lowercase().contains(&f.to_lowercase()))
                .unwrap_or(true)
        })
        .collect();

    for (name, versions) in &filtered {
        println!("{:<30} {:<20}", name, versions.join(", "));
    }

    println!("\n共 {} 个事件 Schema", filtered.len());

    Ok(())
}

fn show_schema(name: &str, version: Option<&str>) -> CellResult<()> {
    let registry = load_schemas_from_dir("./events")?;

    let schema = match version {
        Some(v) => registry.get_version(name, v)
            .ok_or_else(|| CellError::NotFound(format!(
                "Schema '{}' version '{}' not found", name, v
            )))?,
        None => registry.get_latest(name)
            .ok_or_else(|| CellError::NotFound(format!(
                "Schema '{}' not found", name
            )))?,
    };

    println!("\n📦 Event Schema: {}", schema.name);
    println!("   Version: {}", schema.version);
    println!("   Source: {}", schema.source);
    println!("   Description: {}", schema.description);
    println!();
    println!("Fields:");
    println!("  {:<5} {:<25} {:<20} {:<8} {}", "Tag", "Name", "Type", "Required", "Description");
    println!("  {}", "-".repeat(85));

    for field in &schema.fields {
        let req = if field.required { "Yes" } else { "No" };
        let dep = if field.deprecated { " [deprecated]" } else { "" };
        println!(
            "  {:<5} {:<25} {:<20} {:<8} {}{}",
            field.position, field.name, field.field_type.description(), req, field.description, dep
        );
    }

    if !schema.nested_types.is_empty() {
        println!("\nNested Types:");
        for (nested_name, fields) in &schema.nested_types {
            println!("  {} ({} fields)", nested_name, fields.len());
        }
    }

    Ok(())
}

fn check_compatibility(_name: &str, old_path: &str, new_path: &str) -> CellResult<()> {
    let old = load_schema_from_file(old_path)?;
    let new = load_schema_from_file(new_path)?;

    let result = new.check_compatibility(&old);

    println!("\n🔍 Compatibility Check");
    println!("   Old: {} v{}", old.name, old.version);
    println!("   New: {} v{}", new.name, new.version);
    println!();

    if result.compatible && result.errors.is_empty() {
        println!("✅ Result: COMPATIBLE");
    } else {
        println!("❌ Result: BREAKING CHANGE");
    }
    println!();

    if !result.errors.is_empty() {
        println!("🚨 Errors (breaking changes):");
        for err in &result.errors {
            println!("   • {}", err);
        }
        println!();
    }

    if !result.warnings.is_empty() {
        println!("⚠️  Warnings:");
        for warn in &result.warnings {
            println!("   • {}", warn);
        }
        println!();
    }

    if !result.info.is_empty() {
        println!("ℹ️  Info:");
        for info in &result.info {
            println!("   • {}", info);
        }
        println!();
    }

    Ok(())
}

fn generate_proto(input: &str, output: Option<&str>) -> CellResult<()> {
    let schema = load_schema_from_file(input)?;
    let proto = schema.to_proto();

    match output {
        Some(path) => {
            fs::write(path, &proto)?;
            println!("\n✅ Proto file generated: {}", path);
        }
        None => {
            println!("\n📝 Proto Definition:");
            println!("{}", "=".repeat(60));
            println!("{}", proto);
            println!("{}", "=".repeat(60));
        }
    }

    Ok(())
}

fn generate_json_schema(input: &str, output: Option<&str>) -> CellResult<()> {
    let schema = load_schema_from_file(input)?;
    let json = schema.to_json_schema();
    let json_str = serde_json::to_string_pretty(&json)
        .map_err(|e| CellError::Config(e.to_string()))?;

    match output {
        Some(path) => {
            fs::write(path, &json_str)?;
            println!("\n✅ JSON Schema generated: {}", path);
        }
        None => {
            println!("\n📝 JSON Schema:");
            println!("{}", "=".repeat(60));
            println!("{}", json_str);
            println!("{}", "=".repeat(60));
        }
    }

    Ok(())
}

fn load_schema_from_file(path: &str) -> CellResult<EventSchema> {
    let content = fs::read_to_string(path)
        .map_err(|e| CellError::NotFound(format!(
            "Cannot read file '{}': {}", path, e
        )))?;

    let schema: EventSchema = if path.ends_with(".json") {
        serde_json::from_str(&content)
            .map_err(|e| CellError::Config(format!(
                "JSON parse error: {}", e
            )))?
    } else {
        serde_yaml::from_str(&content)
            .map_err(|e| CellError::Config(format!(
                "YAML parse error: {}", e
            )))?
    };

    Ok(schema)
}

fn load_schemas_from_dir(dir: &str) -> CellResult<EventSchemaRegistry> {
    let mut registry = EventSchemaRegistry::new();

    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Ok(registry);
    }

    for entry in fs::read_dir(dir_path)
        .map_err(|e| CellError::NotFound(e.to_string()))?
    {
        let entry = entry.map_err(|e| CellError::Config(e.to_string()))?;
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str());
            if matches!(ext, Some("yaml") | Some("yml") | Some("json")) {
                if let Ok(schema) = load_schema_from_file(path.to_str().unwrap()) {
                    let _ = registry.register(schema);
                }
            }
        }
    }

    Ok(registry)
}

#[allow(dead_code)]
fn sample_schema() -> EventSchema {
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
    schema
}
