use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyReport {
    pub overall_score: f64,
    pub grade: EntropyGrade,
    pub dimensions: EntropyDimensions,
    pub dimension_weights: DimensionWeights,
    pub file_count: usize,
    pub total_lines: usize,
    pub breakdown: Vec<FileEntropy>,
    pub high_risk_files: Vec<String>,
}

impl EntropyReport {
    pub fn to_pretty_text(&self) -> String {
        let mut lines = Vec::new();

        lines.push("╔══════════════════════════════════════════════════════════════╗".to_string());
        lines.push("║           📊 Cell 架构熵值分析报告                          ║".to_string());
        lines.push("╚══════════════════════════════════════════════════════════════╝".to_string());
        lines.push(String::new());

        lines.push(format!("  总体熵值: {:.2} / 100", self.overall_score));
        lines.push(format!("  等级评估: {} {}", self.grade.emoji(), self.grade.label()));
        lines.push(format!("  分析文件: {} 个", self.file_count));
        lines.push(format!("  代码行数: {} 行", self.total_lines));
        lines.push(String::new());

        lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
        lines.push("  五维熵值明细".to_string());
        lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());

        let dims = &self.dimensions;
        let _weights = &self.dimension_weights;

        lines.push(format_bar("🏗️  结构熵 (25%)", dims.structural, 30));
        lines.push(format_bar("🔄 复杂度熵 (25%)", dims.complexity, 30));
        lines.push(format_bar("🔗 耦合熵   (20%)", dims.coupling, 30));
        lines.push(format_bar("📝 命名熵   (15%)", dims.naming, 30));
        lines.push(format_bar("🧪 测试熵   (15%)", dims.test, 30));

        lines.push(String::new());

        if !self.high_risk_files.is_empty() {
            lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
            lines.push(format!("  ⚠️  高风险文件 ({} 个)", self.high_risk_files.len()));
            lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
            for f in &self.high_risk_files {
                lines.push(format!("    🔴 {}", f));
            }
            lines.push(String::new());
        }

        let mut sorted_breakdown = self.breakdown.clone();
        sorted_breakdown.sort_by(|a, b| {
            let score_a = a.complexity_score.max(a.structural_score).max(a.naming_score);
            let score_b = b.complexity_score.max(b.structural_score).max(b.naming_score);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        let top_n = sorted_breakdown.len().min(10);
        if top_n > 0 {
            lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
            lines.push(format!("  📁 Top {} 最高熵值文件", top_n));
            lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
            lines.push(format!("    {:<40} {:>6} {:>6} {:>6} {:>6}", "文件", "结构", "复杂度", "命名", "行数"));
            lines.push("    " .to_string() + &"─".repeat(62));
            for f in sorted_breakdown.iter().take(top_n) {
                let display_path = if f.path.len() > 38 {
                    format!("...{}", &f.path[f.path.len().saturating_sub(35)..])
                } else {
                    f.path.clone()
                };
                lines.push(format!(
                    "    {:<40} {:>5.1}% {:>5.1}% {:>5.1}% {:>6}",
                    display_path, f.structural_score, f.complexity_score, f.naming_score, f.lines
                ));
            }
        }

        lines.push(String::new());
        lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
        lines.push("  💡 说明: 熵值越低表示系统越有序、越健康".to_string());
        lines.push("     健康 < 20 | 注意 < 40 | 警告 < 60 | 危险 < 80 | 灾难 ≥ 80".to_string());
        lines.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());

        lines.join("\n")
    }
}

fn format_bar(label: &str, value: f64, bar_width: usize) -> String {
    let filled = ((value / 100.0) * bar_width as f64).round() as usize;
    let filled = filled.min(bar_width);
    let empty = bar_width - filled;

    let bar_char = if value < 20.0 {
        "█"
    } else if value < 40.0 {
        "█"
    } else if value < 60.0 {
        "█"
    } else if value < 80.0 {
        "█"
    } else {
        "█"
    };

    format!(
        "  {:<20} [{}{}] {:>5.1}%",
        label,
        bar_char.repeat(filled),
        "░".repeat(empty),
        value
    )
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntropyDimensions {
    pub structural: f64,
    pub complexity: f64,
    pub coupling: f64,
    pub naming: f64,
    pub test: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DimensionWeights {
    pub structural: f64,
    pub complexity: f64,
    pub coupling: f64,
    pub naming: f64,
    pub test: f64,
}

impl Default for DimensionWeights {
    fn default() -> Self {
        Self {
            structural: 0.25,
            complexity: 0.25,
            coupling: 0.20,
            naming: 0.15,
            test: 0.15,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EntropyGrade {
    Healthy,
    Notice,
    Warning,
    Danger,
    Critical,
}

impl EntropyGrade {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 20.0 => EntropyGrade::Healthy,
            s if s < 40.0 => EntropyGrade::Notice,
            s if s < 60.0 => EntropyGrade::Warning,
            s if s < 80.0 => EntropyGrade::Danger,
            _ => EntropyGrade::Critical,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            EntropyGrade::Healthy => "健康",
            EntropyGrade::Notice => "注意",
            EntropyGrade::Warning => "警告",
            EntropyGrade::Danger => "危险",
            EntropyGrade::Critical => "灾难",
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            EntropyGrade::Healthy => "🟢",
            EntropyGrade::Notice => "🔵",
            EntropyGrade::Warning => "🟡",
            EntropyGrade::Danger => "🟠",
            EntropyGrade::Critical => "🔴",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntropy {
    pub path: String,
    pub lines: usize,
    pub complexity_score: f64,
    pub structural_score: f64,
    pub naming_score: f64,
    pub function_count: usize,
    pub max_nesting_depth: usize,
}

pub fn shannon_entropy(probabilities: &[f64]) -> f64 {
    let total: f64 = probabilities.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let mut entropy = 0.0;
    for &p in probabilities {
        if p > 0.0 {
            let prob = p / total;
            entropy -= prob * prob.log2();
        }
    }
    entropy
}

pub fn normalize_entropy(entropy: f64, max_possible: f64) -> f64 {
    if max_possible <= 0.0 {
        return 0.0;
    }
    (entropy / max_possible * 100.0).clamp(0.0, 100.0)
}

pub fn calculate_structural_for_file(content: &str, file_path: &str, total_files: usize) -> FileStructuralInfo {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    let mut functions = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits_vec = Vec::new();
    let mut mods_vec = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
            functions.push(trimmed.to_string());
        }
        if trimmed.starts_with("struct ") || trimmed.contains(" struct ") {
            structs.push(trimmed.to_string());
        }
        if trimmed.starts_with("enum ") || trimmed.contains(" enum ") {
            enums.push(trimmed.to_string());
        }
        if trimmed.starts_with("trait ") || trimmed.contains(" trait ") {
            traits_vec.push(trimmed.to_string());
        }
        if trimmed.starts_with("mod ") || trimmed.contains(" pub mod ") {
            mods_vec.push(trimmed.to_string());
        }
    }

    let total_items = functions.len() + structs.len() + enums.len() + traits_vec.len() + mods_vec.len();

    let size_entropy = if line_count > 0 {
        let mut size_buckets = [0.0; 5];
        for line in &lines {
            let len = line.len();
            let bucket = match len {
                0..=20 => 0,
                21..=40 => 1,
                41..=60 => 2,
                61..=80 => 3,
                _ => 4,
            };
            size_buckets[bucket] += 1.0;
        }
        shannon_entropy(&size_buckets)
    } else {
        0.0
    };
    let max_size_entropy = (5.0_f64).log2();
    let size_score = normalize_entropy(size_entropy, max_size_entropy);

    let item_distribution = [
        functions.len() as f64,
        structs.len() as f64,
        enums.len() as f64,
        traits_vec.len() as f64,
        mods_vec.len() as f64,
    ];
    let item_entropy = shannon_entropy(&item_distribution);
    let max_item_entropy = if total_items > 0 { (5.0_f64).log2() } else { 1.0 };
    let item_score = normalize_entropy(item_entropy, max_item_entropy);

    let line_penalty = if line_count > 300 {
        ((line_count - 300) as f64 / 100.0).min(30.0)
    } else {
        0.0
    };

    let density_penalty = if total_items > 0 && line_count > 0 {
        let density = total_items as f64 / line_count as f64;
        if density > 0.3 {
            (density - 0.3) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    let overall_structural = (size_score * 0.3 + item_score * 0.3 + line_penalty * 2.0 + density_penalty * 0.4)
        .clamp(0.0, 100.0);

    let _ = file_path;
    let _ = total_files;

    FileStructuralInfo {
        score: overall_structural,
        function_count: functions.len(),
        struct_count: structs.len(),
        enum_count: enums.len(),
        trait_count: traits_vec.len(),
        line_count,
    }
}

pub struct FileStructuralInfo {
    pub score: f64,
    pub function_count: usize,
    pub struct_count: usize,
    pub enum_count: usize,
    pub trait_count: usize,
    pub line_count: usize,
}

pub fn calculate_complexity_for_file(content: &str) -> FileComplexityInfo {
    let lines: Vec<&str> = content.lines().collect();

    let mut max_depth = 0;
    let mut current_depth = 0;
    let mut branch_count = 0;
    let mut loop_count = 0;
    let mut match_count = 0;
    let mut function_count = 0;
    let mut closure_count = 0;

    let mut depth_distribution = [0.0; 8];

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
            function_count += 1;
        }

        if trimmed.contains("||") || trimmed.contains("move |") {
            closure_count += 1;
        }

        if trimmed.starts_with("if ")
            || trimmed.starts_with("} else if ")
            || trimmed.contains(" else if ")
            || trimmed.starts_with("if let ")
            || trimmed.contains(" && ")
            || trimmed.contains(" || ")
        {
            branch_count += 1;
        }

        if trimmed.starts_with("for ")
            || trimmed.starts_with("loop ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("iter()")
            || trimmed.contains(".map(")
            || trimmed.contains(".filter(")
        {
            loop_count += 1;
        }

        if trimmed.starts_with("match ") {
            match_count += 1;
        }

        let opens = line.chars().filter(|&c| c == '{').count();
        let closes = line.chars().filter(|&c| c == '}').count();
        current_depth += opens as i32 - closes as i32;
        if current_depth > max_depth {
            max_depth = current_depth;
        }

        let depth_idx = (current_depth.max(0) as usize).min(7);
        depth_distribution[depth_idx] += 1.0;
    }

    let branch_entropy = shannon_entropy(&[
        branch_count as f64,
        loop_count as f64,
        match_count as f64,
        function_count as f64,
        closure_count as f64,
    ]);
    let max_branch_entropy = (5.0_f64).log2();
    let branch_score = normalize_entropy(branch_entropy, max_branch_entropy);

    let depth_entropy = shannon_entropy(&depth_distribution);
    let max_depth_entropy = (8.0_f64).log2();
    let depth_score = normalize_entropy(depth_entropy, max_depth_entropy);

    let nesting_penalty = if max_depth > 4 {
        ((max_depth - 4) as f64 * 10.0).min(25.0)
    } else {
        0.0
    };

    let cyclomatic_complexity = branch_count + loop_count + match_count + function_count + 1;
    let cc_score = if cyclomatic_complexity <= 10 {
        cyclomatic_complexity as f64
    } else if cyclomatic_complexity <= 20 {
        10.0 + (cyclomatic_complexity - 10) as f64 * 2.0
    } else {
        30.0 + (cyclomatic_complexity - 20) as f64 * 1.5
    }.min(60.0);

    let overall_complexity = (cc_score * 0.4 + depth_score * 0.3 + branch_score * 0.15 + nesting_penalty * 0.15)
        .clamp(0.0, 100.0);

    FileComplexityInfo {
        score: overall_complexity,
        cyclomatic_complexity,
        max_nesting_depth: max_depth.max(0) as usize,
        function_count,
        branch_count,
        loop_count,
        match_count,
    }
}

pub struct FileComplexityInfo {
    pub score: f64,
    pub cyclomatic_complexity: usize,
    pub max_nesting_depth: usize,
    pub function_count: usize,
    pub branch_count: usize,
    pub loop_count: usize,
    pub match_count: usize,
}

pub fn calculate_naming_for_file(content: &str) -> f64 {
    let lines: Vec<&str> = content.lines().collect();
    let mut identifiers = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
            if let Some(name) = extract_identifier(trimmed, "fn") {
                identifiers.push(name);
            }
        }
        if trimmed.starts_with("struct ") || trimmed.contains(" struct ") {
            if let Some(name) = extract_identifier(trimmed, "struct") {
                identifiers.push(name);
            }
        }
        if trimmed.starts_with("enum ") || trimmed.contains(" enum ") {
            if let Some(name) = extract_identifier(trimmed, "enum") {
                identifiers.push(name);
            }
        }
        if trimmed.starts_with("trait ") || trimmed.contains(" trait ") {
            if let Some(name) = extract_identifier(trimmed, "trait") {
                identifiers.push(name);
            }
        }
        if trimmed.starts_with("let ") || trimmed.starts_with("let mut ") {
            if let Some(name) = extract_let_name(trimmed) {
                if name.len() > 1 {
                    identifiers.push(name);
                }
            }
        }
    }

    if identifiers.is_empty() {
        return 0.0;
    }

    let mut length_distribution = [0.0; 6];
    let mut abbreviation_count = 0;
    let mut single_char_count = 0;
    let mut inconsistent_count = 0;

    for id in &identifiers {
        let len = id.len();
        let bucket = match len {
            1..=2 => 0,
            3..=5 => 1,
            6..=10 => 2,
            11..=15 => 3,
            16..=20 => 4,
            _ => 5,
        };
        length_distribution[bucket] += 1.0;

        if len == 1 {
            single_char_count += 1;
        }

        if is_abbreviation(id) {
            abbreviation_count += 1;
        }

        if has_inconsistent_casing(id) {
            inconsistent_count += 1;
        }
    }

    let length_entropy = shannon_entropy(&length_distribution);
    let max_length_entropy = (6.0_f64).log2();
    let length_score = normalize_entropy(length_entropy, max_length_entropy);

    let abbr_ratio = abbreviation_count as f64 / identifiers.len() as f64;
    let abbr_score = (abbr_ratio * 100.0).clamp(0.0, 50.0);

    let single_char_ratio = single_char_count as f64 / identifiers.len() as f64;
    let single_char_score = (single_char_ratio * 100.0).clamp(0.0, 30.0);

    let inconsistent_ratio = inconsistent_count as f64 / identifiers.len() as f64;
    let inconsistent_score = (inconsistent_ratio * 100.0).clamp(0.0, 30.0);

    let overall_naming = (length_score * 0.25 + abbr_score * 0.25 + single_char_score * 0.25 + inconsistent_score * 0.25)
        .clamp(0.0, 100.0);

    overall_naming
}

fn extract_identifier(line: &str, keyword: &str) -> Option<String> {
    let idx = line.find(keyword)?;
    let rest = &line[idx + keyword.len()..];
    let trimmed = rest.trim_start();
    let end = trimmed.find(|c: char| !c.is_alphanumeric() && c != '_')?;
    if end == 0 {
        None
    } else {
        Some(trimmed[..end].to_string())
    }
}

fn extract_let_name(line: &str) -> Option<String> {
    let idx = line.find("let ")?;
    let rest = &line[idx + 4..];
    let trimmed = rest.trim_start();
    let trimmed = trimmed.trim_start_matches("mut ");
    let end = trimmed.find(|c: char| !c.is_alphanumeric() && c != '_')?;
    if end == 0 {
        None
    } else {
        Some(trimmed[..end].to_string())
    }
}

fn is_abbreviation(s: &str) -> bool {
    if s.len() <= 3 {
        return s.chars().all(|c| c.is_ascii_uppercase());
    }
    let upper_count = s.chars().filter(|c| c.is_ascii_uppercase()).count();
    upper_count == s.len() && s.len() <= 5
}

fn has_inconsistent_casing(s: &str) -> bool {
    let has_upper = s.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = s.chars().any(|c| c.is_ascii_lowercase());
    if !has_upper || !has_lower {
        return false;
    }
    let chars: Vec<char> = s.chars().collect();
    for i in 1..chars.len() {
        if chars[i].is_ascii_uppercase() && chars[i-1].is_ascii_lowercase() {
            if i + 1 < chars.len() && chars[i+1].is_ascii_lowercase() {
                return false;
            }
        }
    }
    let snake_case = s.contains('_');
    let camel_case = has_upper && has_lower;
    snake_case && camel_case
}

pub fn calculate_coupling(files: &[FileCouplingInfo]) -> f64 {
    if files.is_empty() {
        return 0.0;
    }

    let mut in_degrees = Vec::new();
    let mut out_degrees = Vec::new();

    for f in files {
        in_degrees.push(f.incoming as f64);
        out_degrees.push(f.outgoing as f64);
    }

    let in_entropy = shannon_entropy(&in_degrees);
    let out_entropy = shannon_entropy(&out_degrees);
    let max_entropy = (files.len() as f64).log2().max(1.0);

    let in_score = normalize_entropy(in_entropy, max_entropy);
    let out_score = normalize_entropy(out_entropy, max_entropy);

    let cross_layer_count = files.iter().filter(|f| f.cross_layer).count();
    let cross_layer_ratio = cross_layer_count as f64 / files.len() as f64;
    let cross_layer_score = (cross_layer_ratio * 100.0).clamp(0.0, 50.0);

    let circular_count = files.iter().filter(|f| f.in_cycle).count();
    let circular_score = if circular_count > 0 {
        (circular_count as f64 * 20.0).clamp(0.0, 30.0)
    } else {
        0.0
    };

    let overall_coupling = (in_score * 0.2 + out_score * 0.2 + cross_layer_score * 0.35 + circular_score * 0.25)
        .clamp(0.0, 100.0);

    overall_coupling
}

pub struct FileCouplingInfo {
    pub path: String,
    pub incoming: usize,
    pub outgoing: usize,
    pub cross_layer: bool,
    pub in_cycle: bool,
}

pub fn calculate_test_entropy(files: &[TestFileInfo]) -> f64 {
    if files.is_empty() {
        return 50.0;
    }

    let total_code_lines: usize = files.iter().map(|f| f.code_lines).sum();
    let total_test_lines: usize = files.iter().map(|f| f.test_lines).sum();
    let total_assertions: usize = files.iter().map(|f| f.assertion_count).sum();
    let total_tests: usize = files.iter().map(|f| f.test_count).sum();

    let test_ratio = if total_code_lines > 0 {
        total_test_lines as f64 / total_code_lines as f64
    } else {
        0.0
    };
    let coverage_estimate = (test_ratio * 200.0).clamp(0.0, 100.0);
    let coverage_score = 100.0 - coverage_estimate;

    let assertion_density = if total_tests > 0 {
        total_assertions as f64 / total_tests as f64
    } else {
        0.0
    };
    let assertion_score = if assertion_density < 1.0 {
        (1.0 - assertion_density) * 40.0
    } else {
        0.0
    };

    let mut test_distribution = [0.0; 5];
    for f in files {
        if f.test_count > 0 {
            let bucket = match f.test_count {
                1..=2 => 0,
                3..=5 => 1,
                6..=10 => 2,
                11..=20 => 3,
                _ => 4,
            };
            test_distribution[bucket] += 1.0;
        }
    }
    let test_entropy_val = shannon_entropy(&test_distribution);
    let max_test_entropy = (5.0_f64).log2();
    let distribution_score = normalize_entropy(test_entropy_val, max_test_entropy);

    let overall_test = (coverage_score * 0.5 + assertion_score * 0.25 + distribution_score * 0.25)
        .clamp(0.0, 100.0);

    overall_test
}

pub struct TestFileInfo {
    pub path: String,
    pub code_lines: usize,
    pub test_lines: usize,
    pub test_count: usize,
    pub assertion_count: usize,
    pub is_test_file: bool,
}

pub fn calculate_overall_score(dimensions: &EntropyDimensions, weights: &DimensionWeights) -> f64 {
    dimensions.structural * weights.structural
        + dimensions.complexity * weights.complexity
        + dimensions.coupling * weights.coupling
        + dimensions.naming * weights.naming
        + dimensions.test * weights.test
}

pub fn aggregate_dimensions(file_scores: &[f64]) -> f64 {
    if file_scores.is_empty() {
        return 0.0;
    }
    let sum: f64 = file_scores.iter().sum();
    let avg = sum / file_scores.len() as f64;

    let max = file_scores.iter().cloned().fold(f64::NAN, f64::max);
    let p90 = percentile_90(file_scores);

    (avg * 0.4 + max * 0.3 + p90 * 0.3).clamp(0.0, 100.0)
}

fn percentile_90(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() as f64 - 1.0) * 0.9).round() as usize;
    sorted[idx.min(values.len() - 1)]
}

pub fn is_high_risk_file(file: &FileEntropy) -> bool {
    file.complexity_score > 60.0
        || file.structural_score > 60.0
        || file.naming_score > 60.0
        || file.lines > 500
}

pub fn build_file_metrics(
    file_path: &str,
    content: &str,
) -> (FileEntropy, FileComplexityInfo, FileStructuralInfo, f64) {
    let structural_info = calculate_structural_for_file(content, file_path, 1);
    let complexity_info = calculate_complexity_for_file(content);
    let naming_score = calculate_naming_for_file(content);

    let file_entropy = FileEntropy {
        path: file_path.to_string(),
        lines: structural_info.line_count,
        complexity_score: complexity_info.score,
        structural_score: structural_info.score,
        naming_score,
        function_count: complexity_info.function_count,
        max_nesting_depth: complexity_info.max_nesting_depth,
    };

    (file_entropy, complexity_info, structural_info, naming_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy_uniform() {
        let probs = vec![1.0, 1.0, 1.0, 1.0];
        let entropy = shannon_entropy(&probs);
        assert!((entropy - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_shannon_entropy_single() {
        let probs = vec![1.0, 0.0, 0.0];
        let entropy = shannon_entropy(&probs);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_shannon_entropy_empty() {
        let probs: Vec<f64> = vec![];
        assert_eq!(shannon_entropy(&probs), 0.0);
    }

    #[test]
    fn test_normalize_entropy() {
        let entropy = 1.0;
        let max = 2.0;
        let normalized = normalize_entropy(entropy, max);
        assert_eq!(normalized, 50.0);
    }

    #[test]
    fn test_entropy_grade_healthy() {
        assert_eq!(EntropyGrade::from_score(10.0), EntropyGrade::Healthy);
    }

    #[test]
    fn test_entropy_grade_warning() {
        assert_eq!(EntropyGrade::from_score(50.0), EntropyGrade::Warning);
    }

    #[test]
    fn test_entropy_grade_critical() {
        assert_eq!(EntropyGrade::from_score(90.0), EntropyGrade::Critical);
    }

    #[test]
    fn test_empty_content() {
        let (file_entropy, complexity, structural, naming) = build_file_metrics("test.rs", "");
        assert_eq!(file_entropy.lines, 0);
        assert_eq!(complexity.cyclomatic_complexity, 1);
        assert_eq!(structural.function_count, 0);
        assert_eq!(naming, 0.0);
    }

    #[test]
    fn test_simple_function() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let (file_entropy, complexity, _, _) = build_file_metrics("test.rs", code);
        assert_eq!(file_entropy.function_count, 1);
        assert!(complexity.score >= 0.0);
        assert!(complexity.score < 50.0);
    }

    #[test]
    fn test_nested_complexity() {
        let code = r#"
fn nested() {
    if true {
        if true {
            if true {
                if true {
                    println!("deep");
                }
            }
        }
    }
}
"#;
        let (_, complexity, _, _) = build_file_metrics("test.rs", code);
        assert!(complexity.max_nesting_depth >= 4);
        assert!(complexity.score > 20.0);
    }

    #[test]
    fn test_naming_detection() {
        let code = r#"
fn good_name() {
    let x = 1;
    let user_id = 2;
    struct Abc {}
    enum XYZ {}
}
"#;
        let score = calculate_naming_for_file(code);
        assert!(score >= 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_coupling_calculation() {
        let files = vec![
            FileCouplingInfo { path: "a.rs".to_string(), incoming: 5, outgoing: 2, cross_layer: false, in_cycle: false },
            FileCouplingInfo { path: "b.rs".to_string(), incoming: 1, outgoing: 5, cross_layer: true, in_cycle: false },
            FileCouplingInfo { path: "c.rs".to_string(), incoming: 3, outgoing: 3, cross_layer: false, in_cycle: true },
        ];
        let score = calculate_coupling(&files);
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_test_entropy_no_tests() {
        let files = vec![
            TestFileInfo { path: "a.rs".to_string(), code_lines: 100, test_lines: 0, test_count: 0, assertion_count: 0, is_test_file: false },
        ];
        let score = calculate_test_entropy(&files);
        assert!(score > 0.0);
    }

    #[test]
    fn test_overall_score_weights() {
        let dims = EntropyDimensions {
            structural: 20.0,
            complexity: 40.0,
            coupling: 30.0,
            naming: 10.0,
            test: 50.0,
        };
        let weights = DimensionWeights::default();
        let score = calculate_overall_score(&dims, &weights);
        let expected = 20.0 * 0.25 + 40.0 * 0.25 + 30.0 * 0.20 + 10.0 * 0.15 + 50.0 * 0.15;
        assert!((score - expected).abs() < 0.01);
    }

    #[test]
    fn test_is_high_risk_file() {
        let file = FileEntropy {
            path: "big.rs".to_string(),
            lines: 600,
            complexity_score: 70.0,
            structural_score: 30.0,
            naming_score: 20.0,
            function_count: 20,
            max_nesting_depth: 5,
        };
        assert!(is_high_risk_file(&file));
    }
}
