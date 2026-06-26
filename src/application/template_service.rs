use crate::domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub version: String,
    pub tags: Vec<String>,
    pub author: String,
    pub files: Vec<TemplateFile>,
    pub variables: Vec<TemplateVariable>,
    pub architecture_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Basic,
    CrudService,
    Microservice,
    EventDriven,
    CliTool,
    Library,
    FullStack,
    Custom,
}

impl TemplateCategory {
    pub fn label(&self) -> &str {
        match self {
            TemplateCategory::Basic => "基础项目",
            TemplateCategory::CrudService => "CRUD服务",
            TemplateCategory::Microservice => "微服务",
            TemplateCategory::EventDriven => "事件驱动",
            TemplateCategory::CliTool => "CLI工具",
            TemplateCategory::Library => "类库",
            TemplateCategory::FullStack => "全栈应用",
            TemplateCategory::Custom => "自定义",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub template_id: String,
    pub success: bool,
    pub files_created: Vec<String>,
    pub files_skipped: Vec<String>,
    pub errors: Vec<String>,
    pub variables_used: HashMap<String, String>,
}

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_templates(&self, category: Option<&TemplateCategory>) -> Vec<Template> {
        let mut templates = self.builtin_templates();
        if let Some(cat) = category {
            templates.retain(|t| t.category == *cat);
        }
        templates
    }

    pub fn get_template(&self, id: &str) -> Option<Template> {
        self.builtin_templates().into_iter().find(|t| t.id == id)
    }

    pub fn apply_template(&self, project_path: &str, template_id: &str, variables: HashMap<String, String>, force: bool) -> CellResult<ApplyResult> {
        let template = self.get_template(template_id)
            .ok_or_else(|| crate::domain::errors::CellError::Config(format!("Template '{}' not found", template_id)))?;

        let mut files_created = Vec::new();
        let mut files_skipped = Vec::new();
        let mut errors = Vec::new();
        let mut vars_used = HashMap::new();

        for var in &template.variables {
            let value = variables.get(&var.name)
                .cloned()
                .or_else(|| var.default_value.clone());
            
            if let Some(v) = value {
                vars_used.insert(var.name.clone(), v);
            } else if var.required {
                errors.push(format!("Required variable '{}' is missing: {}", var.name, var.description));
            }
        }

        if !errors.is_empty() {
            return Ok(ApplyResult {
                template_id: template_id.to_string(),
                success: false,
                files_created,
                files_skipped,
                errors,
                variables_used: vars_used,
            });
        }

        let project_root = Path::new(project_path);

        for file in &template.files {
            let rendered_path = self.render_template(&file.path, &vars_used);
            let target_path = project_root.join(&rendered_path);

            if target_path.exists() && !force {
                files_skipped.push(rendered_path);
                continue;
            }

            if let Some(parent) = target_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    errors.push(format!("Failed to create dir {:?}: {}", parent, e));
                    continue;
                }
            }

            let rendered_content = self.render_template(&file.content, &vars_used);
            
            match std::fs::write(&target_path, &rendered_content) {
                Ok(_) => files_created.push(rendered_path),
                Err(e) => errors.push(format!("Failed to write {}: {}", rendered_path, e)),
            }
        }

        let success = errors.is_empty();
        Ok(ApplyResult {
            template_id: template_id.to_string(),
            success,
            files_created,
            files_skipped,
            errors,
            variables_used: vars_used,
        })
    }

    pub fn list_categories(&self) -> Vec<TemplateCategory> {
        vec![
            TemplateCategory::Basic,
            TemplateCategory::CrudService,
            TemplateCategory::Microservice,
            TemplateCategory::EventDriven,
            TemplateCategory::CliTool,
            TemplateCategory::Library,
            TemplateCategory::FullStack,
            TemplateCategory::Custom,
        ]
    }

    fn render_template(&self, content: &str, variables: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    fn builtin_templates(&self) -> Vec<Template> {
        vec![
            self.basic_template(),
            self.crud_service_template(),
            self.microservice_template(),
            self.cli_tool_template(),
            self.library_template(),
        ]
    }

    fn basic_template(&self) -> Template {
        Template {
            id: "basic".to_string(),
            name: "基础四层架构项目".to_string(),
            description: "标准的 Cell Architecture 四层架构基础项目模板".to_string(),
            category: TemplateCategory::Basic,
            version: "1.0.0".to_string(),
            tags: vec!["基础".to_string(), "四层架构".to_string()],
            author: "Cell Architecture".to_string(),
            files: vec![
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content: r#"[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/domain/mod.rs".to_string(),
                    content: "pub mod errors;\npub mod value_objects;\n".to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/domain/errors.rs".to_string(),
                    content: r#"use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/application/mod.rs".to_string(),
                    content: "pub mod ports;\n".to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/adapters/mod.rs".to_string(),
                    content: "// Adapters implement ports\n".to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/interfaces/mod.rs".to_string(),
                    content: "// Interfaces handle I/O\n".to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/lib.rs".to_string(),
                    content: "pub mod domain;\npub mod application;\npub mod adapters;\npub mod interfaces;\n".to_string(),
                    is_binary: false,
                },
            ],
            variables: vec![
                TemplateVariable {
                    name: "project_name".to_string(),
                    description: "项目名称".to_string(),
                    default_value: Some("my-project".to_string()),
                    required: true,
                    choices: None,
                },
            ],
            architecture_pattern: "四层架构 (Domain, Application, Adapters, Interfaces)".to_string(),
        }
    }

    fn crud_service_template(&self) -> Template {
        Template {
            id: "crud-service".to_string(),
            name: "CRUD 服务模板".to_string(),
            description: "完整的增删改查服务，包含领域模型、服务层、存储层".to_string(),
            category: TemplateCategory::CrudService,
            version: "1.0.0".to_string(),
            tags: vec!["CRUD".to_string(), "服务".to_string(), "RESTful".to_string()],
            author: "Cell Architecture".to_string(),
            files: vec![
                TemplateFile {
                    path: "src/domain/entities/{{entity}}.rs".to_string(),
                    content: r#"use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{Entity}} {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl {{Entity}} {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = chrono::Utc::now();
    }
}
"#.to_string(),
                    is_binary: false,
                },
            ],
            variables: vec![
                TemplateVariable {
                    name: "entity".to_string(),
                    description: "实体名称 (小写蛇形)".to_string(),
                    default_value: Some("user".to_string()),
                    required: true,
                    choices: None,
                },
                TemplateVariable {
                    name: "Entity".to_string(),
                    description: "实体名称 (大驼峰)".to_string(),
                    default_value: Some("User".to_string()),
                    required: true,
                    choices: None,
                },
            ],
            architecture_pattern: "领域驱动设计 - CRUD 服务".to_string(),
        }
    }

    fn microservice_template(&self) -> Template {
        Template {
            id: "microservice".to_string(),
            name: "微服务模板".to_string(),
            description: "面向微服务架构的服务模板，包含 API、事件、健康检查".to_string(),
            category: TemplateCategory::Microservice,
            version: "1.0.0".to_string(),
            tags: vec!["微服务".to_string(), "API".to_string(), "Docker".to_string()],
            author: "Cell Architecture".to_string(),
            files: vec![
                TemplateFile {
                    path: "Dockerfile".to_string(),
                    content: r#"FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.19
COPY --from=builder /app/target/release/{{service_name}} /usr/local/bin/
EXPOSE 8080
CMD ["{{service_name}}"]
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "README.md".to_string(),
                    content: r#"# {{service_name}}

## 架构说明

- **Domain Layer**: 领域模型和业务规则
- **Application Layer**: 用例和服务编排
- **Adapters Layer**: 外部系统适配（数据库、消息队列等）
- **Interfaces Layer**: API 接口和入口

## 快速开始

\`\`\`bash
cargo run
\`\`\`
"#.to_string(),
                    is_binary: false,
                },
            ],
            variables: vec![
                TemplateVariable {
                    name: "service_name".to_string(),
                    description: "服务名称".to_string(),
                    default_value: Some("my-service".to_string()),
                    required: true,
                    choices: None,
                },
            ],
            architecture_pattern: "微服务 + 四层架构".to_string(),
        }
    }

    fn cli_tool_template(&self) -> Template {
        Template {
            id: "cli-tool".to_string(),
            name: "CLI 工具模板".to_string(),
            description: "命令行工具模板，使用 Clap 框架".to_string(),
            category: TemplateCategory::CliTool,
            version: "1.0.0".to_string(),
            tags: vec!["CLI".to_string(), "命令行".to_string()],
            author: "Cell Architecture".to_string(),
            files: vec![
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content: r#"[package]
name = "{{cli_name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: r#"use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "{{cli_name}}")]
#[command(about = "{{cli_description}}")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Run,
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => println!("Initializing..."),
        Commands::Run => println!("Running..."),
        Commands::Status => println!("Status: OK"),
    }
}
"#.to_string(),
                    is_binary: false,
                },
            ],
            variables: vec![
                TemplateVariable {
                    name: "cli_name".to_string(),
                    description: "CLI 工具名称".to_string(),
                    default_value: Some("my-cli".to_string()),
                    required: true,
                    choices: None,
                },
                TemplateVariable {
                    name: "cli_description".to_string(),
                    description: "CLI 工具描述".to_string(),
                    default_value: Some("A command line tool".to_string()),
                    required: false,
                    choices: None,
                },
            ],
            architecture_pattern: "CLI 应用 + Clap".to_string(),
        }
    }

    fn library_template(&self) -> Template {
        Template {
            id: "library".to_string(),
            name: "Rust 库模板".to_string(),
            description: "标准 Rust 库项目模板，包含示例和测试".to_string(),
            category: TemplateCategory::Library,
            version: "1.0.0".to_string(),
            tags: vec!["库".to_string(), "Library".to_string()],
            author: "Cell Architecture".to_string(),
            files: vec![
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content: r#"[package]
name = "{{lib_name}}"
version = "0.1.0"
edition = "2021"
description = "{{lib_description}}"

[dependencies]
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/lib.rs".to_string(),
                    content: r#"//! {{lib_name}}
//!
//! {{lib_description}}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add(2, 2), 4);
    }
}
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "examples/basic.rs".to_string(),
                    content: r#"use {{lib_name}}::add;

fn main() {
    println!("2 + 2 = {}", add(2, 2));
}
"#.to_string(),
                    is_binary: false,
                },
            ],
            variables: vec![
                TemplateVariable {
                    name: "lib_name".to_string(),
                    description: "库名称".to_string(),
                    default_value: Some("my-lib".to_string()),
                    required: true,
                    choices: None,
                },
                TemplateVariable {
                    name: "lib_description".to_string(),
                    description: "库描述".to_string(),
                    default_value: Some("A Rust library".to_string()),
                    required: false,
                    choices: None,
                },
            ],
            architecture_pattern: "标准 Rust Library".to_string(),
        }
    }
}

impl Default for TemplateService {
    fn default() -> Self {
        Self::new()
    }
}
