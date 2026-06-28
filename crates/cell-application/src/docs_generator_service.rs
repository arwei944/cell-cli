use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    pub output_dir: String,
    pub format: DocsFormat,
    pub include_private: bool,
    pub include_tests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DocsFormat {
    Markdown,
    Html,
    Pdf,
    OpenApi,
    All,
}

impl DocsFormat {
    pub fn label(&self) -> &str {
        match self {
            Self::Markdown => "Markdown",
            Self::Html => "HTML",
            Self::Pdf => "PDF",
            Self::OpenApi => "OpenAPI",
            Self::All => "All",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedDoc {
    pub name: String,
    pub format: DocsFormat,
    pub path: String,
    pub content_preview: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDoc {
    pub title: String,
    pub overview: String,
    pub layers: Vec<LayerDoc>,
    pub dependencies: Vec<DependencyDoc>,
    pub decisions: Vec<DecisionDoc>,
    pub metrics: MetricsDoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDoc {
    pub name: String,
    pub description: String,
    pub modules: Vec<String>,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyDoc {
    pub from: String,
    pub to: String,
    pub type_: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionDoc {
    pub id: String,
    pub title: String,
    pub status: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsDoc {
    pub entropy_score: f64,
    pub entropy_grade: String,
    pub violations: usize,
    pub test_coverage: f64,
}

pub struct DocsGeneratorService;

impl DocsGeneratorService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_architecture_docs(&self, project_path: &str, config: &DocsConfig) -> CellResult<Vec<GeneratedDoc>> {
        let mut docs = Vec::new();
        let output_dir = Path::new(project_path).join(&config.output_dir);
        std::fs::create_dir_all(&output_dir)?;

        let arch_doc = self.collect_architecture_info(project_path)?;

        if config.format == DocsFormat::Markdown || config.format == DocsFormat::All {
            let md_content = self.generate_markdown(&arch_doc);
            let md_path = output_dir.join("ARCHITECTURE.md");
            std::fs::write(&md_path, &md_content)?;
            docs.push(GeneratedDoc {
                name: "架构文档".to_string(),
                format: DocsFormat::Markdown,
                path: md_path.to_string_lossy().to_string(),
                content_preview: md_content.lines().take(10).collect::<Vec<_>>().join("\n"),
                generated_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        if config.format == DocsFormat::Html || config.format == DocsFormat::All {
            let html_content = self.generate_html(&arch_doc);
            let html_path = output_dir.join("architecture.html");
            std::fs::write(&html_path, &html_content)?;
            docs.push(GeneratedDoc {
                name: "架构文档".to_string(),
                format: DocsFormat::Html,
                path: html_path.to_string_lossy().to_string(),
                content_preview: "...".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        Ok(docs)
    }

    pub fn generate_api_docs(&self, project_path: &str, config: &DocsConfig) -> CellResult<Vec<GeneratedDoc>> {
        let mut docs = Vec::new();
        let output_dir = Path::new(project_path).join(&config.output_dir);
        std::fs::create_dir_all(&output_dir)?;

        let api_spec = self.collect_api_info(project_path)?;

        if config.format == DocsFormat::OpenApi || config.format == DocsFormat::All {
            let openapi_content = self.generate_openapi(&api_spec);
            let openapi_path = output_dir.join("openapi.json");
            std::fs::write(&openapi_path, &openapi_content)?;
            docs.push(GeneratedDoc {
                name: "API文档".to_string(),
                format: DocsFormat::OpenApi,
                path: openapi_path.to_string_lossy().to_string(),
                content_preview: "...".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            });
        }

        Ok(docs)
    }

    pub fn generate_decision_docs(&self, project_path: &str, config: &DocsConfig) -> CellResult<Vec<GeneratedDoc>> {
        let mut docs = Vec::new();
        let output_dir = Path::new(project_path).join(&config.output_dir);
        std::fs::create_dir_all(&output_dir)?;

        let decisions = self.collect_decisions(project_path)?;
        let md_content = self.generate_adr_markdown(&decisions);
        let md_path = output_dir.join("DECISIONS.md");
        std::fs::write(&md_path, &md_content)?;

        docs.push(GeneratedDoc {
            name: "决策记录文档".to_string(),
            format: DocsFormat::Markdown,
            path: md_path.to_string_lossy().to_string(),
            content_preview: md_content.lines().take(10).collect::<Vec<_>>().join("\n"),
            generated_at: chrono::Utc::now().to_rfc3339(),
        });

        Ok(docs)
    }

    pub fn generate_all(&self, project_path: &str, config: &DocsConfig) -> CellResult<Vec<GeneratedDoc>> {
        let mut docs = Vec::new();
        docs.extend(self.generate_architecture_docs(project_path, config)?);
        docs.extend(self.generate_api_docs(project_path, config)?);
        docs.extend(self.generate_decision_docs(project_path, config)?);
        Ok(docs)
    }

    fn collect_architecture_info(&self, project_path: &str) -> CellResult<ArchitectureDoc> {
        use crate::arch_service::{ArchitectureRules, validate_architecture};
        use crate::entropy_service::run_entropy_check;

        let rules = ArchitectureRules::default();
        let validation = validate_architecture(Path::new(project_path), &rules);
        let entropy_report = run_entropy_check(project_path)?;

        let layers = vec![
            LayerDoc {
                name: "Domain".to_string(),
                description: "领域层 - 核心业务逻辑，零外部依赖".to_string(),
                modules: vec!["entropy.rs".to_string(), "workflow.rs".to_string(), "errors.rs".to_string()],
                rules: vec!["禁止依赖外部库".to_string(), "禁止依赖其他层".to_string()],
            },
            LayerDoc {
                name: "Application".to_string(),
                description: "应用层 - 服务编排，调用领域层".to_string(),
                modules: vec!["entropy_service.rs".to_string(), "arch_service.rs".to_string()],
                rules: vec!["只能调用领域层".to_string(), "通过端口调用适配器".to_string()],
            },
            LayerDoc {
                name: "Adapters".to_string(),
                description: "适配器层 - 实现端口接口".to_string(),
                modules: vec!["file_adapter.rs".to_string(), "git_adapter.rs".to_string()],
                rules: vec!["实现端口接口".to_string(), "可依赖外部库".to_string()],
            },
            LayerDoc {
                name: "Interfaces".to_string(),
                description: "接口层 - CLI/API入口".to_string(),
                modules: vec!["cli.rs".to_string(), "commands/".to_string()],
                rules: vec!["调用应用层".to_string(), "不直接访问领域层".to_string()],
            },
        ];

        let dependencies = validation.violations.iter().map(|v| DependencyDoc {
            from: v.from_module.clone(),
            to: v.to_module.clone(),
            type_: "违规依赖".to_string(),
            description: v.message.clone(),
        }).collect();

        let decisions = self.collect_decisions(project_path)?;

        Ok(ArchitectureDoc {
            title: "Cell Architecture".to_string(),
            overview: "低熵架构，面向AI原生开发".to_string(),
            layers,
            dependencies,
            decisions,
            metrics: MetricsDoc {
                entropy_score: entropy_report.overall_score,
                entropy_grade: format!("{}", entropy_report.grade),
                violations: validation.violations.len(),
                test_coverage: 0.0,
            },
        })
    }

    fn collect_api_info(&self, _project_path: &str) -> CellResult<ApiSpec> {
        Ok(ApiSpec {
            title: "Cell Architecture API".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec![
                Endpoint {
                    path: "/api/entropy".to_string(),
                    method: "GET".to_string(),
                    description: "获取熵值报告".to_string(),
                },
                Endpoint {
                    path: "/api/architecture".to_string(),
                    method: "GET".to_string(),
                    description: "获取架构信息".to_string(),
                },
                Endpoint {
                    path: "/api/progress".to_string(),
                    method: "GET".to_string(),
                    description: "获取进度状态".to_string(),
                },
            ],
        })
    }

    fn collect_decisions(&self, _project_path: &str) -> CellResult<Vec<DecisionDoc>> {
        // 注意: 实际的决策记录获取应该在接口层完成，然后传入
        // 这里返回空列表作为默认值，避免违反架构规则
        Ok(Vec::new())
    }

    pub fn generate_with_decisions(&self, project_path: &str, decisions: Vec<DecisionDoc>, config: &DocsConfig) -> CellResult<Vec<GeneratedDoc>> {
        let mut docs = Vec::new();
        docs.extend(self.generate_architecture_docs(project_path, config)?);
        docs.extend(self.generate_api_docs(project_path, config)?);
        
        // 使用传入的决策记录生成文档
        let output_dir = std::path::Path::new(project_path).join(&config.output_dir);
        std::fs::create_dir_all(&output_dir)?;
        
        let md_content = self.generate_adr_markdown(&decisions);
        let md_path = output_dir.join("DECISIONS.md");
        std::fs::write(&md_path, &md_content)?;
        
        docs.push(GeneratedDoc {
            name: "决策记录文档".to_string(),
            format: DocsFormat::Markdown,
            path: md_path.to_string_lossy().to_string(),
            content_preview: md_content.lines().take(10).collect::<Vec<_>>().join("\n"),
            generated_at: chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(docs)
    }

    fn generate_markdown(&self, doc: &ArchitectureDoc) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("# {}\n\n", doc.title));
        md.push_str(&format!("{}\n\n", doc.overview));
        
        md.push_str("## 架构分层\n\n");
        for layer in &doc.layers {
            md.push_str(&format!("### {}\n\n", layer.name));
            md.push_str(&format!("{}\n\n", layer.description));
            md.push_str("**模块:**\n");
            for m in &layer.modules {
                md.push_str(&format!("- {m}\n"));
            }
            md.push_str("\n**规则:**\n");
            for r in &layer.rules {
                md.push_str(&format!("- {r}\n"));
            }
            md.push('\n');
        }

        md.push_str("## 指标\n\n");
        md.push_str(&format!("- 熵值分数: {:.2}\n", doc.metrics.entropy_score));
        md.push_str(&format!("- 熵值等级: {}\n", doc.metrics.entropy_grade));
        md.push_str(&format!("- 架构违规: {} 个\n", doc.metrics.violations));
        md.push('\n');

        if !doc.dependencies.is_empty() {
            md.push_str("## 依赖违规\n\n");
            for dep in &doc.dependencies {
                md.push_str(&format!("- {} → {}: {}\n", dep.from, dep.to, dep.description));
            }
            md.push('\n');
        }

        md
    }

    fn generate_html(&self, doc: &ArchitectureDoc) -> String {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str(&format!("<title>{}</title>\n", doc.title));
        html.push_str("<style>\n");
        html.push_str("body { font-family: system-ui; margin: 40px; }\n");
        html.push_str(".layer { background: #f5f5f5; padding: 20px; margin: 10px 0; border-radius: 8px; }\n");
        html.push_str(".metric { display: inline-block; margin-right: 30px; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&format!("<h1>{}</h1>\n", doc.title));
        html.push_str(&format!("<p>{}</p>\n", doc.overview));
        
        html.push_str("<h2>架构分层</h2>\n");
        for layer in &doc.layers {
            html.push_str("<div class=\"layer\">\n");
            html.push_str(&format!("<h3>{}</h3>\n", layer.name));
            html.push_str(&format!("<p>{}</p>\n", layer.description));
            html.push_str("</div>\n");
        }

        html.push_str("<h2>指标</h2>\n");
        html.push_str(&format!("<div class=\"metric\">熵值: {:.2}</div>\n", doc.metrics.entropy_score));
        html.push_str(&format!("<div class=\"metric\">等级: {}</div>\n", doc.metrics.entropy_grade));
        html.push_str(&format!("<div class=\"metric\">违规: {}</div>\n", doc.metrics.violations));
        
        html.push_str("</body>\n</html>\n");
        html
    }

    fn generate_openapi(&self, spec: &ApiSpec) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": spec.title,
                "version": spec.version,
            },
            "paths": spec.endpoints.iter().map(|e| {
                (e.path.clone(), serde_json::json!({
                    e.method.to_lowercase(): {
                        "summary": e.description,
                        "responses": {
                            "200": {
                                "description": "Success",
                            }
                        }
                    }
                }))
            }).collect::<std::collections::HashMap<String, serde_json::Value>>()
        })).unwrap_or_default()
    }

    fn generate_adr_markdown(&self, decisions: &[DecisionDoc]) -> String {
        let mut md = String::new();
        
        md.push_str("# Architecture Decision Records (ADR)\n\n");
        md.push_str("| ID | Title | Status | Date |\n");
        md.push_str("|---|---|---|---|\n");
        
        for d in decisions {
            md.push_str(&format!("| {} | {} | {} | {} |\n", d.id, d.title, d.status, d.date));
        }
        
        md
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiSpec {
    title: String,
    version: String,
    endpoints: Vec<Endpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Endpoint {
    path: String,
    method: String,
    description: String,
}

impl Default for DocsGeneratorService {
    fn default() -> Self {
        Self::new()
    }
}