use crate::application::ports::code_generator::{CodeGeneratorPort, GeneratedFile};
use crate::domain::cell_spec::{CellSpec, PortSpec, AdapterSpec};

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        TemplateEngine
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGeneratorPort for TemplateEngine {
    fn render_cell_structure(&self, spec: &CellSpec) -> Vec<GeneratedFile> {
        let mut files = Vec::new();

        files.push(GeneratedFile {
            path: format!("{}/Cargo.toml", spec.name),
            content: render_cargo_toml(spec),
        });

        files.push(GeneratedFile {
            path: format!("{}/README.md", spec.name),
            content: render_readme(spec),
        });

        files.push(GeneratedFile {
            path: format!("{}/src/lib.rs", spec.name),
            content: render_lib_rs(spec),
        });

        files.push(GeneratedFile {
            path: format!("{}/src/domain/mod.rs", spec.name),
            content: render_domain_mod(spec),
        });

        files.push(GeneratedFile {
            path: format!("{}/src/application/mod.rs", spec.name),
            content: render_application_mod(spec),
        });

        files.push(GeneratedFile {
            path: format!("{}/src/adapters/mod.rs", spec.name),
            content: render_adapters_mod(spec),
        });

        files.push(GeneratedFile {
            path: format!("{}/src/interfaces/mod.rs", spec.name),
            content: "// Interfaces layer - external entry points\n".to_string(),
        });

        files.push(GeneratedFile {
            path: format!("{}/spec.yaml", spec.name),
            content: render_spec_yaml(spec),
        });

        for port in &spec.ports {
            files.push(GeneratedFile {
                path: format!("{}/src/application/ports/{}.rs", spec.name, to_snake_case(&port.name)),
                content: render_port(port),
            });
        }

        for adapter in &spec.adapters {
            files.push(GeneratedFile {
                path: format!("{}/src/adapters/{}.rs", spec.name, to_snake_case(&adapter.name)),
                content: render_adapter(adapter),
            });
        }

        files
    }
}

fn render_cargo_toml(spec: &CellSpec) -> String {
    let deps: Vec<String> = spec
        .dependencies
        .iter()
        .map(|d| format!("{} = \"1\"", d))
        .collect();

    format!(
        r#"[package]
name = "{}"
version = "{}"
edition = "2024"
description = "{}"

[dependencies]
serde = {{ version = "1", features = ["derive"] }}
thiserror = "1"
{}
"#,
        spec.name,
        spec.version,
        spec.description,
        deps.join("\n")
    )
}

fn render_readme(spec: &CellSpec) -> String {
    format!(
        r#"# {}

{}

## 架构

```
src/
├── domain/       # 领域层（纯业务逻辑）
├── application/  # 应用层（用例编排 + Port 定义）
├── adapters/     # 适配器层（Port 实现）
└── interfaces/   # 接口层（外部入口）
```

## Ports

{}

## Adapters

{}
"#,
        spec.name,
        spec.description,
        spec.ports
            .iter()
            .map(|p| format!("- `{}` ({})", p.name, p.kind.as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        spec.adapters
            .iter()
            .map(|a| format!("- `{}` (port: {})", a.name, a.port))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn render_lib_rs(spec: &CellSpec) -> String {
    let mut content = String::from(
        r#"pub mod domain;
pub mod application;
pub mod adapters;
pub mod interfaces;
"#,
    );

    if !spec.ports.is_empty() {
        content.push_str("\npub use application::ports::*;\n");
    }

    content
}

fn render_domain_mod(spec: &CellSpec) -> String {
    format!(
        r#"// Domain layer for {}
// Pure business logic, no external dependencies
"#,
        spec.name
    )
}

fn render_application_mod(spec: &CellSpec) -> String {
    let mut content = String::from("// Application layer\n");
    if !spec.ports.is_empty() {
        content.push_str("pub mod ports;\n");
    }
    content
}

fn render_adapters_mod(spec: &CellSpec) -> String {
    let mut content = String::from("// Adapters layer - implement ports\n");
    for adapter in &spec.adapters {
        content.push_str(&format!("pub mod {};\n", to_snake_case(&adapter.name)));
    }
    content
}

fn render_spec_yaml(spec: &CellSpec) -> String {
    serde_yaml::to_string(spec).unwrap_or_default()
}

fn render_port(port: &PortSpec) -> String {
    let input_type = port.input.as_deref().unwrap_or("()");
    let output_type = port.output.as_deref().unwrap_or("()");
    let async_kw = if port.is_async { "async " } else { "" };

    format!(
        r#"use crate::domain::errors::CellResult;

pub trait {} {{
    fn {}execute(&self, input: {}) -> CellResult<{}>;
}}
"#,
        to_pascal_case(&port.name),
        async_kw,
        input_type,
        output_type
    )
}

fn render_adapter(adapter: &AdapterSpec) -> String {
    format!(
        r#"// Adapter: {} (kind: {:?})
// Implements port: {}

pub struct {}Adapter;

impl {}Adapter {{
    pub fn new() -> Self {{
        Self
    }}
}}
"#,
        adapter.name,
        adapter.kind,
        adapter.port,
        to_pascal_case(&adapter.name),
        to_pascal_case(&adapter.name),
    )
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut upper = true;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            result.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cell_spec::PortKind;
    use crate::domain::cell_spec::AdapterKind;

    fn sample_spec() -> CellSpec {
        CellSpec {
            name: "user-cell".to_string(),
            description: "User management cell".to_string(),
            version: "0.1.0".to_string(),
            ports: vec![PortSpec {
                name: "CreateUser".to_string(),
                kind: PortKind::UseCase,
                description: "Create a new user".to_string(),
                input: Some("CreateUserInput".to_string()),
                output: Some("User".to_string()),
                is_async: true,
            }],
            adapters: vec![AdapterSpec {
                name: "InMemoryUserRepo".to_string(),
                kind: AdapterKind::InMemory,
                port: "UserRepository".to_string(),
                description: "In memory user repository".to_string(),
            }],
            dependencies: vec![],
            tags: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn test_render_creates_files() {
        let engine = TemplateEngine::new();
        let spec = sample_spec();
        let files = engine.render_cell_structure(&spec);
        assert!(files.len() > 5);
    }

    #[test]
    fn test_cargo_toml_contains_name() {
        let engine = TemplateEngine::new();
        let spec = sample_spec();
        let files = engine.render_cell_structure(&spec);
        let cargo = files.iter().find(|f| f.path.ends_with("Cargo.toml")).unwrap();
        assert!(cargo.content.contains("user-cell"));
    }

    #[test]
    fn test_port_generated() {
        let engine = TemplateEngine::new();
        let spec = sample_spec();
        let files = engine.render_cell_structure(&spec);
        let port_file = files.iter().find(|f| f.path.contains("ports/"));
        assert!(port_file.is_some());
        assert!(port_file.unwrap().content.contains("trait"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("CreateUser"), "create_user");
        assert_eq!(to_snake_case("user"), "user");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("create_user"), "CreateUser");
        assert_eq!(to_pascal_case("User"), "User");
    }

    #[test]
    fn test_implements_code_generator_port() {
        let engine = TemplateEngine::new();
        let _: &dyn CodeGeneratorPort = &engine;
    }
}
