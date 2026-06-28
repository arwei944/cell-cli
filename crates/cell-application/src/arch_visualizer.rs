use crate::arch_service::{ArchitectureRules, ValidationResult};
use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationOptions {
    pub output_format: OutputFormat,
    pub include_violations: bool,
    pub include_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Mermaid,
    PlantUml,
    Ascii,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Mermaid,
            include_violations: true,
            include_stats: true,
        }
    }
}

pub struct ArchitectureVisualizer;

impl ArchitectureVisualizer {
    pub fn new() -> Self {
        Self
    }

    pub fn visualize(&self, project_path: &str, options: VisualizationOptions) -> CellResult<String> {
        let root = Path::new(project_path);
        let rules = ArchitectureRules::default();
        let validation = super::arch_service::validate_architecture(root, &rules);

        match options.output_format {
            OutputFormat::Mermaid => Ok(self.generate_mermaid(&validation, &rules, options)),
            OutputFormat::PlantUml => Ok(self.generate_plantuml(&validation, &rules, options)),
            OutputFormat::Ascii => Ok(self.generate_ascii(&validation, &rules, options)),
        }
    }

    fn generate_mermaid(&self, validation: &ValidationResult, _rules: &ArchitectureRules, options: VisualizationOptions) -> String {
        let mut output = String::new();
        
        output.push_str("```mermaid\n");
        output.push_str("flowchart TD\n");
        output.push_str("    subgraph Architecture Layers\n");
        
        let layer_order = ["domain", "application", "adapters", "interfaces"];
        for (i, layer) in layer_order.iter().enumerate() {
            let stats = validation.layer_stats.get(*layer);
            let violations = stats.map_or(0, |s| s.violations);
            let file_count = stats.map_or(0, |s| s.file_count);
            
            let color = if violations > 0 { "style=fill:#ffcccc" } else { "style=fill:#ccffcc" };
            let label = if options.include_stats {
                format!("{}[{}<br/>{} files{}]", 
                    layer, layer, file_count,
                    if violations > 0 { format!("<br/>{violations} violations") } else { String::new() }
                )
            } else {
                format!("{layer}[{layer}]")
            };
            
            output.push_str(&format!("        {label} {color}\n"));
            
            if i < layer_order.len() - 1 {
                output.push_str(&format!("        {} --> {}[{}]\n", layer, layer_order[i + 1], layer_order[i + 1]));
            }
        }
        
        output.push_str("    end\n");
        
        if options.include_violations && !validation.violations.is_empty() {
            output.push_str("\n    subgraph Violations\n");
            for (i, v) in validation.violations.iter().enumerate() {
                output.push_str(&format!("        V{}[\"{:?}: {} → {}\"]\n", i, v.severity, v.from_module, v.to_module));
            }
            output.push_str("    end\n");
            
            for v in &validation.violations {
                let from_layer = v.from_module.split("::").next().unwrap_or("");
                let to_layer = v.to_module.split("::").next().unwrap_or("");
                output.push_str(&format!("        {from_layer} -.->|❌| {to_layer}\n"));
            }
        }
        
        output.push_str("```\n");
        output
    }

    fn generate_plantuml(&self, validation: &ValidationResult, _rules: &ArchitectureRules, options: VisualizationOptions) -> String {
        let mut output = String::new();
        
        output.push_str("@startuml\n");
        output.push_str("title Architecture Layers\n");
        output.push_str("skinparam backgroundColor #FEFEFE\n");
        
        let layer_order = ["domain", "application", "adapters", "interfaces"];
        for (i, layer) in layer_order.iter().enumerate() {
            let stats = validation.layer_stats.get(*layer);
            let violations = stats.map_or(0, |s| s.violations);
            
            let color = if violations > 0 { "LightRed" } else { "LightGreen" };
            let label = if options.include_stats {
                let file_count = stats.map_or(0, |s| s.file_count);
                format!("{layer} ({file_count})")
            } else {
                layer.to_string()
            };
            
            output.push_str(&format!("rectangle \"{}\" as {} #{} {}\n", label, layer, color, if i > 0 { format!("below {}", layer_order[i - 1]) } else { String::new() }));
            
            if i > 0 {
                output.push_str(&format!("{} --> {}\n", layer_order[i - 1], layer));
            }
        }
        
        if options.include_violations && !validation.violations.is_empty() {
            output.push_str("\n' Violations\n");
            for v in &validation.violations {
                let from_layer = v.from_module.split("::").next().unwrap_or("");
                let to_layer = v.to_module.split("::").next().unwrap_or("");
                output.push_str(&format!("{} -[#red,dashed]-> {} : {}\n", from_layer, to_layer, v.message));
            }
        }
        
        output.push_str("@enduml\n");
        output
    }

    fn generate_ascii(&self, validation: &ValidationResult, _rules: &ArchitectureRules, options: VisualizationOptions) -> String {
        let mut output = String::new();
        
        output.push_str("\n┌──────────────────────────────────────────────────────────────┐\n");
        output.push_str("│                    Architecture Layers                      │\n");
        output.push_str("├──────────────────────────────────────────────────────────────┤\n");
        
        let layer_order = ["domain", "application", "adapters", "interfaces"];
        for (i, layer) in layer_order.iter().enumerate() {
            let stats = validation.layer_stats.get(*layer);
            let violations = stats.map_or(0, |s| s.violations);
            let file_count = stats.map_or(0, |s| s.file_count);
            let internal_deps = stats.map_or(0, |s| s.internal_deps);
            let external_deps = stats.map_or(0, |s| s.external_deps);
            
            let icon = if violations > 0 { "❌" } else { "✅" };
            let label = match *layer {
                "domain" => "Domain",
                "application" => "Application",
                "adapters" => "Adapters",
                "interfaces" => "Interfaces",
                _ => *layer,
            };
            
            output.push_str(&format!("│  {icon} {label} │ "));
            if i > 0 {
                output.push_str("↑ ");
            } else {
                output.push_str("  ");
            }
            
            if options.include_stats {
                output.push_str(&format!("{file_count} files, {internal_deps} internal, {external_deps} external, {violations} violations"));
            }
            
            output.push('\n');
        }
        
        output.push_str("└──────────────────────────────────────────────────────────────┘\n");
        
        if options.include_violations && !validation.violations.is_empty() {
            output.push_str("\n⚠️  Violations:\n");
            output.push_str("──────────────────────────────────────────────────────────────\n");
            
            for v in &validation.violations {
                output.push_str(&format!("  {} → {}: {}\n", v.from_module, v.to_module, v.message));
            }
        }
        
        output
    }
}

impl Default for ArchitectureVisualizer {
    fn default() -> Self {
        Self::new()
    }
}
