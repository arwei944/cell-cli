use crate::domain::errors::CellResult;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub modules: HashMap<String, ModuleInfo>,
    pub dependencies: Vec<DependencyEdge>,
    pub total_modules: usize,
    pub total_dependencies: usize,
    pub has_circular_deps: bool,
    pub circular_deps: Vec<Vec<String>>,
    pub layer_violations: Vec<LayerViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub layer: String,
    pub file_count: usize,
    pub lines_of_code: usize,
    pub incoming_deps: usize,
    pub outgoing_deps: usize,
    pub depends_on: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub file: String,
    pub line: usize,
    pub is_cross_layer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerViolation {
    pub from_layer: String,
    pub to_layer: String,
    pub from_module: String,
    pub to_module: String,
    pub file: String,
    pub description: String,
}

pub struct DependencyAnalyzer {
    root: String,
}

impl DependencyAnalyzer {
    pub fn new(root: &str) -> Self {
        Self {
            root: root.to_string(),
        }
    }

    pub fn analyze(&self) -> CellResult<DependencyGraph> {
        let mut modules: HashMap<String, ModuleInfo> = HashMap::new();
        let mut dependencies: Vec<DependencyEdge> = Vec::new();
        let mut module_files: HashMap<String, Vec<String>> = HashMap::new();

        let root_path = Path::new(&self.root);
        let src_dir = root_path.join("src");

        if !src_dir.exists() {
            return Err(crate::domain::errors::CellError::NotFound(
                "src directory not found".to_string(),
            ));
        }

        for entry in WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = path
                .strip_prefix(root_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let module_name = self.get_module_name(path, &src_dir);
            let layer = self.detect_layer_from_name(&module_name);
            let line_count = self.count_lines(path);

            module_files
                .entry(module_name.clone())
                .or_insert_with(Vec::new)
                .push(rel_path.clone());

            let module = modules.entry(module_name.clone()).or_insert(ModuleInfo {
                name: module_name.clone(),
                path: module_name.clone(),
                layer: layer.clone(),
                file_count: 0,
                lines_of_code: 0,
                incoming_deps: 0,
                outgoing_deps: 0,
                depends_on: Vec::new(),
                dependents: Vec::new(),
            });

            module.file_count += 1;
            module.lines_of_code += line_count;

            if let Ok(content) = std::fs::read_to_string(path) {
                let deps = self.extract_dependencies(&content, &module_name);
                for (dep_module, line) in deps {
                    let dep_layer = self.detect_layer_from_name(&dep_module);
                    let is_cross_layer = layer != dep_layer && !dep_layer.is_empty();

                    dependencies.push(DependencyEdge {
                        from: module_name.clone(),
                        to: dep_module.clone(),
                        file: rel_path.clone(),
                        line,
                        is_cross_layer,
                    });
                }
            }
        }

        for dep in &dependencies {
            if let Some(from_module) = modules.get_mut(&dep.from) {
                if !from_module.depends_on.contains(&dep.to) {
                    from_module.depends_on.push(dep.to.clone());
                    from_module.outgoing_deps += 1;
                }
            }
            if let Some(to_module) = modules.get_mut(&dep.to) {
                if !to_module.dependents.contains(&dep.from) {
                    to_module.dependents.push(dep.from.clone());
                    to_module.incoming_deps += 1;
                }
            }
        }

        let total_modules = modules.len();
        let total_dependencies = dependencies.len();

        let circular_deps = self.detect_circular_deps(&modules);
        let has_circular_deps = !circular_deps.is_empty();

        let layer_violations = self.detect_layer_violations(&dependencies, &modules);

        Ok(DependencyGraph {
            modules,
            dependencies,
            total_modules,
            total_dependencies,
            has_circular_deps,
            circular_deps,
            layer_violations,
        })
    }

    fn get_module_name(&self, path: &Path, src_dir: &Path) -> String {
        let rel = path.strip_prefix(src_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let rel = rel.replace('\\', "/");
        let parts: Vec<&str> = rel.split('/').collect();

        if parts.len() <= 1 {
            return "root".to_string();
        }

        if parts.len() == 2 {
            if let Some(first) = parts.first() {
                if *first == "main.rs" || *first == "lib.rs" {
                    return "root".to_string();
                }
                return first.to_string();
            }
        }

        if parts.len() >= 2 {
            let top_layer = parts[0];
            let second = parts.get(1).copied().unwrap_or("");

            if second.ends_with(".rs") {
                return format!("{}::{}", top_layer, second.trim_end_matches(".rs"));
            } else {
                return format!("{}::{}", top_layer, second);
            }
        }

        "root".to_string()
    }

    #[allow(dead_code)]
    fn detect_layer(&self, path: &Path, src_dir: &Path) -> String {
        let rel = path.strip_prefix(src_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let rel = rel.replace('\\', "/");

        if rel.starts_with("domain/") || rel == "domain.rs" {
            "domain".to_string()
        } else if rel.starts_with("application/") || rel == "application.rs" {
            "application".to_string()
        } else if rel.starts_with("adapters/") || rel == "adapters.rs" {
            "adapters".to_string()
        } else if rel.starts_with("interfaces/") || rel == "interfaces.rs" {
            "interfaces".to_string()
        } else {
            "other".to_string()
        }
    }

    fn detect_layer_from_name(&self, module_name: &str) -> String {
        let lower = module_name.to_lowercase();
        if lower.starts_with("crate::domain") || lower.starts_with("domain") {
            "domain".to_string()
        } else if lower.starts_with("crate::application") || lower.starts_with("application") {
            "application".to_string()
        } else if lower.starts_with("crate::adapters") || lower.starts_with("adapters") {
            "adapters".to_string()
        } else if lower.starts_with("crate::interfaces") || lower.starts_with("interfaces") {
            "interfaces".to_string()
        } else {
            String::new()
        }
    }

    fn count_lines(&self, path: &Path) -> usize {
        std::fs::read_to_string(path)
            .map(|c| c.lines().count())
            .unwrap_or(0)
    }

    fn extract_dependencies(&self, content: &str, _current_module: &str) -> Vec<(String, usize)> {
        let mut deps = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("use crate::") {
                let module_path = trimmed.trim_start_matches("use crate::")
                    .split(';')
                    .next()
                    .unwrap_or("");

                let parts: Vec<&str> = module_path.split("::").collect();
                if parts.len() >= 2 {
                    let clean_path = format!("{}::{}", parts[0], parts[1]);
                    deps.push((clean_path, line_num + 1));
                } else if parts.len() == 1 && !parts[0].is_empty() {
                    deps.push((parts[0].to_string(), line_num + 1));
                }
            }
        }

        deps
    }

    fn detect_circular_deps(&self, modules: &HashMap<String, ModuleInfo>) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = Vec::new();

        for module_name in modules.keys() {
            self.dfs_cycle_detection(module_name, modules, &mut visited, &mut rec_stack, &mut cycles);
        }

        cycles
    }

    fn dfs_cycle_detection(
        &self,
        node: &str,
        modules: &HashMap<String, ModuleInfo>,
        visited: &mut HashSet<String>,
        rec_stack: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.push(node.to_string());

        if let Some(module) = modules.get(node) {
            for dep in &module.depends_on {
                if !visited.contains(dep) {
                    self.dfs_cycle_detection(dep, modules, visited, rec_stack, cycles);
                } else if rec_stack.contains(dep) {
                    let start_idx = rec_stack.iter().position(|x| x == dep).unwrap_or(0);
                    let cycle: Vec<String> = rec_stack[start_idx..].to_vec();
                    if cycle.len() > 1 && !cycles.iter().any(|c| c.len() == cycle.len() && c.iter().all(|x| cycle.contains(x))) {
                        cycles.push(cycle);
                    }
                }
            }
        }

        rec_stack.pop();
    }

    fn detect_layer_violations(
        &self,
        dependencies: &[DependencyEdge],
        modules: &HashMap<String, ModuleInfo>,
    ) -> Vec<LayerViolation> {
        let mut violations = Vec::new();

        fn layer_rank(layer: &str) -> usize {
            match layer {
                "domain" => 0,
                "application" => 1,
                "adapters" => 2,
                "interfaces" => 3,
                _ => 99,
            }
        }

        for dep in dependencies {
            let from_module = modules.get(&dep.from);
            let to_module = modules.get(&dep.to);

            if let (Some(from), Some(to)) = (from_module, to_module) {
                let from_rank = layer_rank(&from.layer);
                let to_rank = layer_rank(&to.layer);

                if from_rank < to_rank && from_rank < 4 && to_rank < 4 {
                    let description = format!(
                        "{}层 ({}) 依赖了 {}层 ({}), 违反依赖方向",
                        from.layer, dep.from, to.layer, dep.to
                    );

                    violations.push(LayerViolation {
                        from_layer: from.layer.clone(),
                        to_layer: to.layer.clone(),
                        from_module: dep.from.clone(),
                        to_module: dep.to.clone(),
                        file: dep.file.clone(),
                        description,
                    });
                }
            }
        }

        violations
    }

    pub fn format_summary(&self, graph: &DependencyGraph) -> String {
        let mut output = String::new();

        output.push_str("\n📊 依赖分析报告\n\n");

        output.push_str("## 概览\n\n");
        output.push_str(&format!("| 指标 | 值 |\n"));
        output.push_str("|------|-----|\n");
        output.push_str(&format!("| 模块数 | {} |\n", graph.total_modules));
        output.push_str(&format!("| 依赖关系数 | {} |\n", graph.total_dependencies));

        let mut layers: HashMap<String, usize> = HashMap::new();
        for module in graph.modules.values() {
            *layers.entry(module.layer.clone()).or_insert(0) += 1;
        }

        output.push_str("\n## 各层模块分布\n\n");
        let mut sorted_layers: Vec<_> = layers.iter().collect();
        sorted_layers.sort_by(|a, b| b.1.cmp(a.1));
        for (layer, count) in &sorted_layers {
            output.push_str(&format!("- **{}**: {} 个模块\n", layer, count));
        }

        if !graph.layer_violations.is_empty() {
            output.push_str(&format!(
                "\n## ⚠️  {} 个层依赖违规\n\n",
                graph.layer_violations.len()
            ));
            for v in &graph.layer_violations {
                output.push_str(&format!(
                    "- {} → {}: {}\n",
                    v.from_layer, v.to_layer, v.description
                ));
            }
        }

        if graph.has_circular_deps {
            output.push_str(&format!(
                "\n## 🔴 {} 个循环依赖\n\n",
                graph.circular_deps.len()
            ));
            for cycle in &graph.circular_deps {
                output.push_str(&format!("  - {}\n", cycle.join(" → ")));
            }
        }

        output.push_str("\n## 模块依赖统计\n\n");
        output.push_str("| 模块 | 层级 | 入度 | 出度 | 代码行数 |\n");
        output.push_str("|------|------|------|------|----------|\n");

        let mut sorted_modules: Vec<&ModuleInfo> = graph.modules.values().collect();
        sorted_modules.sort_by(|a, b| b.lines_of_code.cmp(&a.lines_of_code));

        for module in sorted_modules.iter().take(15) {
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                module.name,
                module.layer,
                module.incoming_deps,
                module.outgoing_deps,
                module.lines_of_code
            ));
        }

        output
    }

    pub fn generate_mermaid(&self, graph: &DependencyGraph) -> String {
        let mut output = String::new();

        output.push_str("```mermaid\ngraph TD\n\n");

        output.push_str("  subgraph 领域层\n");
        for module in graph.modules.values().filter(|m| m.layer == "domain") {
            output.push_str(&format!("    {0}[\"{0}\"]\n", module.name));
        }
        output.push_str("  end\n\n");

        output.push_str("  subgraph 应用层\n");
        for module in graph.modules.values().filter(|m| m.layer == "application") {
            output.push_str(&format!("    {0}[\"{0}\"]\n", module.name));
        }
        output.push_str("  end\n\n");

        output.push_str("  subgraph 适配器层\n");
        for module in graph.modules.values().filter(|m| m.layer == "adapters") {
            output.push_str(&format!("    {0}[\"{0}\"]\n", module.name));
        }
        output.push_str("  end\n\n");

        output.push_str("  subgraph 接口层\n");
        for module in graph.modules.values().filter(|m| m.layer == "interfaces") {
            output.push_str(&format!("    {0}[\"{0}\"]\n", module.name));
        }
        output.push_str("  end\n\n");

        for dep in &graph.dependencies {
            let style = if dep.is_cross_layer { " -.-> " } else { " --> " };
            output.push_str(&format!("  {}{}{}\n", dep.from, style, dep.to));
        }

        output.push_str("```\n");

        output
    }
}
