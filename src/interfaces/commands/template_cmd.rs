use crate::application::template_service::{TemplateCategory, TemplateService};
use crate::domain::errors::CellResult;
use crate::interfaces::cli::*;
use std::collections::HashMap;

pub fn cmd_template(args: TemplateArgs) -> CellResult<()> {
    let service = TemplateService::new();

    match args.sub {
        TemplateSub::List { category } => {
            let cat = category.as_ref().map(|c| parse_category(c)).transpose()?;
            let templates = service.list_templates(cat.as_ref());
            
            println!("\n📦 模板市场\n");
            println!("  共 {} 个模板\n", templates.len());
            
            let mut current_category: Option<TemplateCategory> = None;
            
            for template in &templates {
                if current_category.as_ref() != Some(&template.category) {
                    current_category = Some(template.category.clone());
                    println!("  {} [{}]", template.category.label(), template.category.to_string());
                    println!("  {}", "─".repeat(50));
                }
                println!("    • {} ({})", template.name, template.id);
                println!("      {}", template.description);
                println!("      标签: {}", template.tags.join(", "));
                println!("      作者: {} | 版本: {}", template.author, template.version);
                println!();
            }
        }
        TemplateSub::Show { id } => {
            match service.get_template(&id) {
                Some(template) => {
                    println!("\n📋 模板详情\n");
                    println!("  名称: {}", template.name);
                    println!("  ID: {}", template.id);
                    println!("  分类: {}", template.category.label());
                    println!("  版本: {}", template.version);
                    println!("  作者: {}", template.author);
                    println!("  架构模式: {}", template.architecture_pattern);
                    println!();
                    println!("  描述:");
                    println!("    {}", template.description);
                    println!();
                    println!("  标签: {}", template.tags.join(", "));
                    println!();
                    println!("  变量 ({} 个):", template.variables.len());
                    for var in &template.variables {
                        let req = if var.required { "必填" } else { "可选" };
                        let default = var.default_value.as_deref().unwrap_or("无");
                        println!("    • {} ({}) - {}", var.name, req, var.description);
                        println!("      默认值: {}", default);
                    }
                    println!();
                    println!("  文件 ({} 个):", template.files.len());
                    for file in &template.files {
                        println!("    • {}", file.path);
                    }
                    println!();
                }
                None => {
                    println!("\n❌ 模板 '{}' 不存在\n", id);
                    println!("  使用 `cell template list` 查看可用模板\n");
                }
            }
        }
        TemplateSub::Apply { id, path, var, force } => {
            let project_path = path.as_deref().unwrap_or(".");
            
            let mut variables = HashMap::new();
            for v in &var {
                if let Some((key, value)) = v.split_once('=') {
                    variables.insert(key.to_string(), value.to_string());
                }
            }

            let result = service.apply_template(project_path, &id, variables, force)?;

            if result.success {
                println!("\n✅ 模板应用成功\n");
                println!("  模板: {}", result.template_id);
                println!("  创建文件: {} 个", result.files_created.len());
                
                if !result.files_skipped.is_empty() {
                    println!("  跳过文件: {} 个（已存在，使用 --force 覆盖）", result.files_skipped.len());
                }
                
                println!();
                for f in &result.files_created {
                    println!("    + {}", f);
                }
                if !result.files_skipped.is_empty() {
                    for f in &result.files_skipped {
                        println!("    ~ {}", f);
                    }
                }
                println!();
            } else {
                println!("\n❌ 模板应用失败\n");
                for e in &result.errors {
                    println!("  • {}", e);
                }
                println!();
            }
        }
        TemplateSub::Categories {} => {
            let categories = service.list_categories();
            println!("\n📁 模板分类\n");
            for cat in &categories {
                let count = service.list_templates(Some(cat)).len();
                println!("  {} - {} 个模板", cat.label(), count);
            }
            println!();
        }
    }

    Ok(())
}

fn parse_category(s: &str) -> CellResult<TemplateCategory> {
    match s.to_lowercase().as_str() {
        "basic" => Ok(TemplateCategory::Basic),
        "crud" | "crud_service" | "crud-service" => Ok(TemplateCategory::CrudService),
        "microservice" | "micro" => Ok(TemplateCategory::Microservice),
        "event" | "event_driven" | "event-driven" => Ok(TemplateCategory::EventDriven),
        "cli" | "cli_tool" | "cli-tool" => Ok(TemplateCategory::CliTool),
        "library" | "lib" => Ok(TemplateCategory::Library),
        "fullstack" | "full_stack" | "full-stack" => Ok(TemplateCategory::FullStack),
        "custom" => Ok(TemplateCategory::Custom),
        _ => Err(crate::domain::errors::CellError::Config(format!(
            "Unknown category: {}. Valid: basic, crud, microservice, event, cli, library, fullstack, custom",
            s
        ))),
    }
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TemplateCategory::Basic => "basic",
            TemplateCategory::CrudService => "crud_service",
            TemplateCategory::Microservice => "microservice",
            TemplateCategory::EventDriven => "event_driven",
            TemplateCategory::CliTool => "cli_tool",
            TemplateCategory::Library => "library",
            TemplateCategory::FullStack => "full_stack",
            TemplateCategory::Custom => "custom",
        };
        write!(f, "{}", s)
    }
}
