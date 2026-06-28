use crate::ports::code_generator::{CodeGeneratorPort, GeneratedFile};
use cell_domain::cell_spec::{CellSpec, PortKind, AdapterKind};
use cell_domain::errors::{CellError, CellResult};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub telemetry: bool,
    pub cell_name: Option<String>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            telemetry: true,
            cell_name: None,
        }
    }
}

pub struct GenerateService<T: CodeGeneratorPort> {
    code_generator: T,
    options: GenerateOptions,
}

impl<T: CodeGeneratorPort> GenerateService<T> {
    pub fn new(code_generator: T) -> Self {
        Self { 
            code_generator,
            options: GenerateOptions::default(),
        }
    }

    pub fn with_options(mut self, options: GenerateOptions) -> Self {
        self.options = options;
        self
    }

    pub fn generate_cell_from_spec(&self, spec_path: &str, output: &str, force: bool) -> CellResult<Vec<String>> {
        let spec = Self::load_spec(spec_path)?;
        let output_path = Path::new(output);
        if output_path.exists() && !force {
            return Err(CellError::AlreadyExists(format!(
                "Output directory {output} already exists (use --force to overwrite)"
            )));
        }
        let files = self.code_generator.render_cell_structure(&spec);
        Self::write_files(&files, output_path)?;
        Ok(files.iter().map(|f| f.path.clone()).collect())
    }

    pub fn generate_cell_from_name(
        &self,
        name: &str,
        output: &str,
        force: bool,
    ) -> CellResult<Vec<String>> {
        let spec = CellSpec {
            name: name.to_string(),
            ..Default::default()
        };
        let output_path = Path::new(output);
        if output_path.exists() && !force {
            return Err(CellError::AlreadyExists(format!(
                "Output directory {output} already exists (use --force to overwrite)"
            )));
        }
        let files = self.code_generator.render_cell_structure(&spec);
        Self::write_files(&files, output_path)?;
        Ok(files.iter().map(|f| f.path.clone()).collect())
    }

    pub fn generate_port(
        &self,
        name: &str,
        kind: PortKind,
        output: &str,
    ) -> CellResult<String> {
        use cell_domain::cell_spec::PortSpec;

        let port = PortSpec {
            name: name.to_string(),
            kind,
            description: String::new(),
            input: None,
            output: None,
            is_async: true,
        };

        let port_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/application/ports/{port_name}.rs"));
        let content = render_port_trait(&port);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_adapter(
        &self,
        name: &str,
        kind: AdapterKind,
        port: &str,
        output: &str,
    ) -> CellResult<String> {
        let adapter_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/adapters/{adapter_name}.rs"));
        let content = render_adapter_struct(name, kind, port, self.options.telemetry);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    fn load_spec(path: &str) -> CellResult<CellSpec> {
        let content = fs::read_to_string(path)?;
        let spec: CellSpec = if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)?
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)?
        } else if path.ends_with(".toml") {
            toml::from_str(&content)?
        } else {
            return Err(CellError::Config(format!(
                "Unsupported spec format: {path}. Use yaml, json, or toml."
            )));
        };
        Ok(spec)
    }

    fn write_files(files: &[GeneratedFile], base: &Path) -> CellResult<()> {
        for file in files {
            let file_path = base.join(&file.path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, &file.content)?;
        }
        Ok(())
    }

    pub fn generate_entity(
        &self,
        name: &str,
        fields: Option<&str>,
        output: &str,
    ) -> CellResult<String> {
        let file_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/domain/entities/{file_name}.rs"));
        let content = render_entity(name, fields);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_value_object(
        &self,
        name: &str,
        fields: Option<&str>,
        output: &str,
    ) -> CellResult<String> {
        let file_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/domain/value_objects/{file_name}.rs"));
        let content = render_value_object(name, fields);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_aggregate(
        &self,
        name: &str,
        output: &str,
    ) -> CellResult<String> {
        let file_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/domain/aggregates/{file_name}.rs"));
        let content = render_aggregate(name);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_domain_event(
        &self,
        name: &str,
        fields: Option<&str>,
        output: &str,
    ) -> CellResult<String> {
        let file_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/domain/events/{file_name}.rs"));
        let content = render_domain_event(name, fields);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_domain_service(
        &self,
        name: &str,
        output: &str,
    ) -> CellResult<String> {
        let file_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/domain/services/{file_name}.rs"));
        let content = render_domain_service(name);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_repository(
        &self,
        name: &str,
        entity: &str,
        output: &str,
    ) -> CellResult<String> {
        let file_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/application/ports/repositories/{file_name}.rs"));
        let content = render_repository(name, entity);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_usecase(
        &self,
        name: &str,
        input: Option<&str>,
        output: Option<&str>,
        generate_impl: bool,
        output_dir: &str,
    ) -> CellResult<Vec<String>> {
        let mut files = Vec::new();
        let snake_name = to_snake_case(name);
        let pascal_name = to_pascal_case(name);

        let port_path = Path::new(output_dir).join(format!("src/application/ports/{snake_name}.rs"));
        let port_content = render_usecase_port(&pascal_name, input, output);
        if let Some(parent) = port_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&port_path, &port_content)?;
        files.push(port_path.to_string_lossy().to_string());

        let input_path = Path::new(output_dir).join(format!("src/application/dto/{snake_name}_input.rs"));
        let input_content = render_dto(&format!("{pascal_name}Input"), input);
        if let Some(parent) = input_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&input_path, &input_content)?;
        files.push(input_path.to_string_lossy().to_string());

        let output_path = Path::new(output_dir).join(format!("src/application/dto/{snake_name}_output.rs"));
        let output_content = render_dto(&format!("{pascal_name}Output"), output);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, &output_content)?;
        files.push(output_path.to_string_lossy().to_string());

        if generate_impl {
            let impl_path = Path::new(output_dir).join(format!("src/application/usecases/{snake_name}.rs"));
            let impl_content = render_usecase_impl(&pascal_name, self.options.telemetry);
            if let Some(parent) = impl_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&impl_path, &impl_content)?;
            files.push(impl_path.to_string_lossy().to_string());
        }

        Ok(files)
    }
}

fn render_port_trait(port: &cell_domain::cell_spec::PortSpec) -> String {
    let input_type = port.input.as_deref().unwrap_or("()");
    let output_type = port.output.as_deref().unwrap_or("()");
    let async_kw = if port.is_async { "async " } else { "" };

    format!(
        r"use cell_domain::errors::CellResult;

pub trait {} {{
    fn {}execute(&self, input: {}) -> CellResult<{}>;
}}
",
        to_pascal_case(&port.name),
        async_kw,
        input_type,
        output_type
    )
}

fn render_adapter_struct(name: &str, kind: AdapterKind, port: &str, telemetry: bool) -> String {
    let pascal = to_pascal_case(name);
    let port_pascal = to_pascal_case(port);
    let snake_port = to_snake_case(port);
    let kind_str = format!("{kind:?}").to_lowercase();
    
    let tracing_import = if telemetry { "use tracing::{info_span, debug_span, Instrument};\n" } else { "" };
    let metrics_import = if telemetry { "use metrics::counter;\n" } else { "" };
    
    match kind {
        AdapterKind::Http => {
            let handler_impl = if telemetry {
                format!(r#"
    // HTTP Handler with Metrics + Trace + Log
    async fn handle_{snake_port}(
        State(usecase): State<Box<dyn {port_pascal} + Send + Sync>>,
        Json(input): Json<()>,
    ) -> CellResult<Json<()>> {{
        let span = info_span!("http_handler", 
            handler = "{snake_port}",
            method = "POST",
            layer = "adapter"
        );
        async move {{
            tracing::info!("Processing HTTP request: {snake_port}");
            counter!("http_requests_total", 
                "handler" => "{snake_port}",
                "method" => "POST"
            ).increment(1);
            
            // TODO: 调用 usecase.execute(input).await
            Ok(Json(()))
        }}
        .instrument(span)
        .await
    }}"#
                )
            } else {
                format!(r"
    async fn handle_{snake_port}(
        State(usecase): State<Box<dyn {port_pascal} + Send + Sync>>,
        Json(input): Json<()>,
    ) -> CellResult<Json<()>> {{
        // TODO: 调用 usecase.execute(input).await
        Ok(Json(()))
    }}"
                )
            };

            let new_body = if telemetry {
                format!(r#"        tracing::info!("Initializing HTTP adapter: {pascal}");"#)
            } else {
                String::new()
            };

            format!(
r#"use crate::ports::{snake_port}::{port_pascal};
use cell_domain::errors::CellResult;
use axum::{{Json, extract::State}};
{tracing_import}{metrics_import}
pub struct {pascal}HttpAdapter {{
    usecase: Box<dyn {port_pascal} + Send + Sync>,
}}

impl {pascal}HttpAdapter {{
    pub fn new(usecase: Box<dyn {port_pascal} + Send + Sync>) -> Self {{
{new_body}
        Self {{ usecase }}
    }}

    pub fn routes(&self) -> axum::Router {{
        axum::Router::new()
            // TODO: 添加路由
            // .route("/api/{snake_port}", axum::routing::post(Self::handle_{snake_port}))
            // .with_state(self.usecase.clone())
    }}
{handler_impl}
}}
"#,
            )
        }
        AdapterKind::Postgres | AdapterKind::InMemory | AdapterKind::Redis | AdapterKind::File => {
            let metric_fn = if telemetry {
                format!(r#"
    fn record_metric(operation: &str) {{
        counter!("repository_operations_total", 
            "adapter" => "{pascal}", 
            "operation" => operation,
            "kind" => "{kind_str}"
        ).increment(1);
    }}"#)
            } else {
                String::new()
            };

            let new_body = if telemetry {
                format!(r#"        tracing::info!("Initializing {kind_str} repository adapter: {pascal}");"#)
            } else {
                String::new()
            };

            let method_body = |op: &str| -> String {
                if telemetry {
                    match op {
                        "find_by_id" => format!(r#"
    async fn find_by_id(&self, id: &str) -> CellResult<Option<()>> {{
        let span = debug_span!("repo_find_by_id", id = %id, adapter = "{pascal}");
        let _enter = span.enter();
        Self::record_metric("find_by_id");
        
        // TODO: 实现 find_by_id
        Ok(None)
    }}"#),
                        "save" => format!(r#"
    async fn save(&self, entity: &()) -> CellResult<()> {{
        let span = debug_span!("repo_save", adapter = "{pascal}");
        let _enter = span.enter();
        Self::record_metric("save");
        
        // TODO: 实现 save
        Ok(())
    }}"#),
                        "delete" => format!(r#"
    async fn delete(&self, id: &str) -> CellResult<()> {{
        let span = debug_span!("repo_delete", id = %id, adapter = "{pascal}");
        let _enter = span.enter();
        Self::record_metric("delete");
        
        // TODO: 实现 delete
        Ok(())
    }}"#),
                        "list" => format!(r#"
    async fn list(&self, page: u32, page_size: u32) -> CellResult<Vec<()>> {{
        let span = debug_span!("repo_list", page = page, page_size = page_size, adapter = "{pascal}");
        let _enter = span.enter();
        Self::record_metric("list");
        
        // TODO: 实现 list
        Ok(vec![])
    }}"#),
                        _ => String::new(),
                    }
                } else {
                    match op {
                        "find_by_id" => r"
    async fn find_by_id(&self, id: &str) -> CellResult<Option<()>> {
        // TODO: 实现 find_by_id
        Ok(None)
    }".to_string(),
                        "save" => r"
    async fn save(&self, entity: &()) -> CellResult<()> {
        // TODO: 实现 save
        Ok(())
    }".to_string(),
                        "delete" => r"
    async fn delete(&self, id: &str) -> CellResult<()> {
        // TODO: 实现 delete
        Ok(())
    }".to_string(),
                        "list" => r"
    async fn list(&self, page: u32, page_size: u32) -> CellResult<Vec<()>> {
        // TODO: 实现 list
        Ok(vec![])
    }".to_string(),
                        _ => String::new(),
                    }
                }
            };

            format!(
r"use crate::ports::repositories::{snake_port}::{port_pascal};
use cell_domain::errors::CellResult;
{tracing_import}{metrics_import}
pub struct {pascal}Repository {{
    // TODO: 添加 {kind_str} 特定的依赖
}}

impl {pascal}Repository {{
    pub fn new() -> Self {{
{new_body}
        Self {{}}
    }}{metric_fn}
}}

#[async_trait::async_trait]
impl {port_pascal} for {pascal}Repository {{
{find_by_id}{save}{delete}{list}
}}
",
                pascal = pascal,
                port_pascal = port_pascal,
                snake_port = snake_port,
                kind_str = kind_str,
                tracing_import = tracing_import,
                metrics_import = metrics_import,
                new_body = new_body,
                metric_fn = metric_fn,
                find_by_id = method_body("find_by_id"),
                save = method_body("save"),
                delete = method_body("delete"),
                list = method_body("list"),
            )
        }
        AdapterKind::Kafka | AdapterKind::Nats => {
            let metric_fn = if telemetry {
                format!(r#"
    fn record_metric(operation: &str) {{
        counter!("message_operations_total", 
            "adapter" => "{pascal}", 
            "operation" => operation,
            "broker" => "{kind_str}"
        ).increment(1);
    }}"#)
            } else {
                String::new()
            };

            let new_body = if telemetry {
                format!(r#"        tracing::info!("Initializing {kind_str} message adapter: {pascal}");"#)
            } else {
                String::new()
            };

            let publish_fn = if telemetry {
                format!(r#"
    async fn publish(&self, topic: &str, payload: &[u8]) -> CellResult<()> {{
        let span = info_span!("message_publish", 
            topic = topic, 
            adapter = "{pascal}",
            broker = "{kind_str}"
        );
        let _enter = span.enter();
        Self::record_metric("publish");
        tracing::info!(topic = topic, "Publishing message");
        
        // TODO: 实现 publish
        Ok(())
    }}

    async fn consume(&self, topic: &str) -> CellResult<Vec<u8>> {{
        let span = info_span!("message_consume", 
            topic = topic, 
            adapter = "{pascal}",
            broker = "{kind_str}"
        );
        let _enter = span.enter();
        Self::record_metric("consume");
        tracing::info!(topic = topic, "Consuming message");
        
        // TODO: 实现 consume
        Ok(vec![])
    }}"#
                )
            } else {
                r"
    async fn publish(&self, topic: &str, payload: &[u8]) -> CellResult<()> {
        // TODO: 实现 publish
        Ok(())
    }

    async fn consume(&self, topic: &str) -> CellResult<Vec<u8>> {
        // TODO: 实现 consume
        Ok(vec![])
    }".to_string()
            };

            format!(
r"use crate::ports::{snake_port}::{port_pascal};
use cell_domain::errors::CellResult;
{tracing_import}{metrics_import}
pub struct {pascal}MessageAdapter {{
    // TODO: 添加 {kind_str} producer/consumer
}}

impl {pascal}MessageAdapter {{
    pub fn new() -> Self {{
{new_body}
        Self {{}}
    }}{metric_fn}{publish_fn}
}}
",
            )
        }
        AdapterKind::Grpc => {
            let new_body = if telemetry {
                format!(r#"        tracing::info!("Initializing gRPC adapter: {pascal}");"#)
            } else {
                String::new()
            };

            format!(
r"use crate::ports::{snake_port}::{port_pascal};
use cell_domain::errors::CellResult;
{tracing_import}
pub struct {pascal}GrpcAdapter {{
    usecase: Box<dyn {port_pascal} + Send + Sync>,
}}

impl {pascal}GrpcAdapter {{
    pub fn new(usecase: Box<dyn {port_pascal} + Send + Sync>) -> Self {{
{new_body}
        Self {{ usecase }}
    }}

    // gRPC service implementation
    // TODO: 实现 gRPC service
}}
",
            )
        }
        AdapterKind::Mock => {
            let new_body = if telemetry {
                format!(r#"        tracing::debug!("Initializing mock adapter: {pascal}");"#)
            } else {
                String::new()
            };

            format!(
r"use crate::ports::{snake_port}::{port_pascal};
use cell_domain::errors::CellResult;
{tracing_import}
pub struct {pascal}MockAdapter {{
    // TODO: 添加 mock 数据存储
}}

impl {pascal}MockAdapter {{
    pub fn new() -> Self {{
{new_body}
        Self {{}}
    }}
}}

#[cfg(test)]
impl {pascal}MockAdapter {{
    // TODO: 添加测试辅助方法
}}
",
            )
        }
    }
}

fn parse_fields(fields: Option<&str>) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let Some(fields_str) = fields {
        for field in fields_str.split(',') {
            let field = field.trim();
            if field.is_empty() {
                continue;
            }
            let parts: Vec<&str> = field.split(':').collect();
            if parts.len() >= 2 {
                result.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
            } else {
                result.push((parts[0].trim().to_string(), "String".to_string()));
            }
        }
    }
    result
}

fn format_struct_fields(fields: &[(String, String)]) -> String {
    if fields.is_empty() {
        String::new()
    } else {
        fields.iter()
            .map(|(name, ty)| format!("    pub {name}: {ty}"))
            .collect::<Vec<_>>()
            .join(",\n")
    }
}

fn render_entity(name: &str, fields: Option<&str>) -> String {
    let pascal = to_pascal_case(name);
    let fields_vec = parse_fields(fields);
    let fields_str = format_struct_fields(&fields_vec);
    
    let id_field = if fields_vec.iter().any(|(n, _)| n == "id") {
        String::new()
    } else {
        "    pub id: String,".to_string()
    };

    let all_fields = if id_field.is_empty() {
        fields_str
    } else if fields_str.is_empty() {
        id_field
    } else {
        format!("{id_field}\n{fields_str}")
    };

    format!(
        r"use cell_domain::errors::CellResult;

#[derive(Debug, Clone)]
pub struct {pascal} {{
{all_fields}
}}

impl {pascal} {{
    pub fn new(id: String) -> Self {{
        Self {{
            id,
        }}
    }}

    pub fn id(&self) -> &str {{
        &self.id
    }}
}}
",
    )
}

fn render_value_object(name: &str, fields: Option<&str>) -> String {
    let pascal = to_pascal_case(name);
    let fields_vec = parse_fields(fields);
    let fields_str = format_struct_fields(&fields_vec);

    format!(
        r"#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct {} {{
{}
}}

impl {} {{
    pub fn new({}) -> CellResult<Self> {{
        Ok(Self {{
            {}
        }})
    }}
}}
",
        pascal,
        fields_str,
        pascal,
        fields_vec.iter().map(|(n, t)| format!("{n}: {t}")).collect::<Vec<_>>().join(", "),
        fields_vec.iter().map(|(n, _)| format!("{n},")).collect::<Vec<_>>().join("\n            "),
    )
}

fn render_aggregate(name: &str) -> String {
    let pascal = to_pascal_case(name);

    format!(
        r"use cell_domain::events::DomainEvent;

#[derive(Debug, Clone)]
pub struct {pascal}Aggregate {{
    id: String,
    events: Vec<Box<dyn DomainEvent>>,
}}

impl {pascal}Aggregate {{
    pub fn new(id: String) -> Self {{
        Self {{
            id,
            events: Vec::new(),
        }}
    }}

    pub fn id(&self) -> &str {{
        &self.id
    }}

    fn record_event(&mut self, event: Box<dyn DomainEvent>) {{
        self.events.push(event);
    }}

    pub fn events(&self) -> &[Box<dyn DomainEvent>] {{
        &self.events
    }}

    pub fn clear_events(&mut self) {{
        self.events.clear();
    }}
}}
",
    )
}

fn render_domain_event(name: &str, fields: Option<&str>) -> String {
    let pascal = to_pascal_case(name);
    let fields_vec = parse_fields(fields);
    let fields_str = format_struct_fields(&fields_vec);

    let occurred_at = if fields_vec.iter().any(|(n, _)| n == "occurred_at") {
        String::new()
    } else {
        "    pub occurred_at: chrono::DateTime<chrono::Utc>,".to_string()
    };

    let all_fields = if occurred_at.is_empty() {
        fields_str
    } else if fields_str.is_empty() {
        occurred_at
    } else {
        format!("{fields_str}\n{occurred_at}")
    };

    format!(
        r#"use cell_domain::events::DomainEvent;

#[derive(Debug, Clone)]
pub struct {}Event {{
{}
}}

impl {}Event {{
    pub fn new({}) -> Self {{
        Self {{
            {}
            occurred_at: chrono::Utc::now(),
        }}
    }}
}}

impl DomainEvent for {}Event {{
    fn event_type(&self) -> &str {{
        "{}"
    }}

    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {{
        self.occurred_at
    }}
}}
"#,
        pascal,
        all_fields,
        pascal,
        fields_vec.iter().map(|(n, t)| format!("{n}: {t}")).collect::<Vec<_>>().join(", "),
        fields_vec.iter().map(|(n, _)| format!("{n},")).collect::<Vec<_>>().join("\n            "),
        pascal,
        snake_case_to_words(&to_snake_case(&pascal)),
    )
}

fn snake_case_to_words(s: &str) -> String {
    s.replace('_', " ")
}

fn render_domain_service(name: &str) -> String {
    let pascal = to_pascal_case(name);

    format!(
        r"pub struct {pascal};

impl {pascal} {{
    pub fn new() -> Self {{
        Self
    }}
}}
",
    )
}

fn render_repository(name: &str, entity: &str) -> String {
    let pascal = to_pascal_case(name);
    let entity_pascal = to_pascal_case(entity);

    format!(
        r"use cell_domain::errors::CellResult;
use cell_domain::entities::{entity_pascal};

#[async_trait::async_trait]
pub trait {pascal} {{
    async fn find_by_id(&self, id: &str) -> CellResult<Option<{entity_pascal}>>;

    async fn save(&self, entity: &{entity_pascal}) -> CellResult<()>;

    async fn delete(&self, id: &str) -> CellResult<()>;

    async fn list(&self, page: u32, page_size: u32) -> CellResult<Vec<{entity_pascal}>>;
}}
",
    )
}

fn render_usecase_port(name: &str, input: Option<&str>, output: Option<&str>) -> String {
    let pascal = to_pascal_case(name);
    let input_type = if input.is_some() {
        format!("{pascal}Input")
    } else {
        "()".to_string()
    };
    let output_type = if output.is_some() {
        format!("{pascal}Output")
    } else {
        "()".to_string()
    };

    format!(
        r"use crate::dto::{pascal}Input;
use crate::dto::{pascal}Output;
use cell_domain::errors::CellResult;

#[async_trait::async_trait]
pub trait {pascal}UseCase {{
    async fn execute(&self, input: {input_type}) -> CellResult<{output_type}>;
}}
",
    )
}

fn render_dto(name: &str, fields: Option<&str>) -> String {
    let pascal = to_pascal_case(name);
    let fields_vec = parse_fields(fields);
    let fields_str = format_struct_fields(&fields_vec);

    format!(
        r"#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct {pascal} {{
{fields_str}
}}
",
    )
}

fn render_usecase_impl(name: &str, telemetry: bool) -> String {
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);

    let telemetry_imports = if telemetry {
        "use tracing::{info_span, Instrument};\n"
    } else {
        ""
    };

    let execute_body = if telemetry {
        format!(
            r#"    async fn execute(&self, input: {pascal}Input) -> CellResult<{pascal}Output> {{
        let span = info_span!("usecase_execute", 
            usecase = "{snake}",
            layer = "application"
        );
        async move {{
            todo!("Implement {snake} use case")
        }}
        .instrument(span)
        .await
    }}"#
        )
    } else {
        format!(
            r#"    async fn execute(&self, input: {pascal}Input) -> CellResult<{pascal}Output> {{
        todo!("Implement {snake} use case")
    }}"#
        )
    };

    format!(
        r"use crate::dto::{pascal}Input;
use crate::dto::{pascal}Output;
use crate::ports::{pascal}UseCase;
use cell_domain::errors::CellResult;
{telemetry_imports}
pub struct {pascal}UseCaseImpl {{
}}

impl {pascal}UseCaseImpl {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}

#[async_trait::async_trait]
impl {pascal}UseCase for {pascal}UseCaseImpl {{
{execute_body}
}}
",
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

    struct MockGenerator;

    impl CodeGeneratorPort for MockGenerator {
        fn render_cell_structure(&self, spec: &CellSpec) -> Vec<GeneratedFile> {
            vec![
                GeneratedFile {
                    path: format!("{}/Cargo.toml", spec.name),
                    content: format!("[package]\nname = \"{}\"\n", spec.name),
                },
                GeneratedFile {
                    path: format!("{}/src/lib.rs", spec.name),
                    content: "pub mod domain;\n".to_string(),
                },
            ]
        }
    }

    #[test]
    fn test_generate_from_name() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("output");
        let files = service
            .generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), false)
            .unwrap();
        assert!(files.len() > 1);
        assert!(output_dir.join("test-cell/Cargo.toml").exists());
        assert!(output_dir.join("test-cell/src/lib.rs").exists());
    }

    #[test]
    fn test_generate_port() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/application/ports")).unwrap();

        let result = service.generate_port("CreateUser", PortKind::UseCase, dir.path().to_str().unwrap());
        assert!(result.is_ok());

        let file_path = dir.path().join("src/application/ports/create_user.rs");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("trait CreateUser"));
    }

    #[test]
    fn test_generate_adapter() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/adapters")).unwrap();

        let result = service.generate_adapter(
            "InMemoryUserRepo",
            AdapterKind::InMemory,
            "UserRepository",
            dir.path().to_str().unwrap(),
        );
        assert!(result.is_ok());

        let file_path = dir.path().join("src/adapters/in_memory_user_repo.rs");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("InMemoryUserRepoRepository"));
        assert!(content.contains("impl UserRepository for InMemoryUserRepoRepository"));
        assert!(content.contains("find_by_id"));
        assert!(content.contains("save"));
        assert!(content.contains("delete"));
        assert!(content.contains("list"));
    }

    #[test]
    fn test_generate_force_flag() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("output");

        service
            .generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), false)
            .unwrap();

        let result =
            service.generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), false);
        assert!(result.is_err());

        let result2 =
            service.generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), true);
        assert!(result2.is_ok());
    }
}
