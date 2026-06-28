use cell_application::fingerprint_service::FingerprintService;
use cell_domain::errors::CellResult;
use cell_domain::fingerprint::{FingerprintCategory, FingerprintSeverity, ProblemFingerprint};
use crate::cli::{DiagnoseArgs, DiagnoseSub};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn cmd_diagnose(args: DiagnoseArgs) -> CellResult<()> {
    let service = FingerprintService::new();

    match args.sub {
        DiagnoseSub::Scan { path, category, severity } => {
            let scan_path = path.unwrap_or_else(|| ".".to_string());
            let base_path = Path::new(&scan_path);

            let category_filter = category.as_deref().and_then(parse_category);
            let severity_filter = severity.as_deref().and_then(parse_severity);

            println!("\n🔍 问题诊断扫描\n{}", "─".repeat(60));
            println!("  扫描路径: {scan_path}");

            let mut total_files = 0;
            let mut all_matches = Vec::new();

            for entry in WalkDir::new(base_path)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs")
                    && let Ok(content) = fs::read_to_string(path) {
                        total_files += 1;
                        let matches = service.scan_log(&content);
                        for m in matches {
                            if let Some(cat) = &category_filter
                                && &m.fingerprint.category != cat {
                                    continue;
                                }
                            if let Some(sev) = &severity_filter
                                && &m.fingerprint.severity != sev {
                                    continue;
                                }
                            all_matches.push((path.to_string_lossy().to_string(), m));
                        }
                    }
            }

            println!("  扫描文件: {total_files} 个");
            println!("  发现问题: {} 个\n", all_matches.len());

            if all_matches.is_empty() {
                println!("  ✅ 未发现已知问题模式\n");
            } else {
                println!("  {:-<5} {:-<8} {:-<20} {:-<12} {:-<10}", "ID", "严重度", "名称", "类别", "置信度");
                println!("  {}", "─".repeat(58));

                for (file_path, m) in &all_matches {
                    let sev = match m.fingerprint.severity {
                        FingerprintSeverity::Critical => "🔴 严重",
                        FingerprintSeverity::High => "🟠 高",
                        FingerprintSeverity::Medium => "🟡 中",
                        FingerprintSeverity::Low => "🔵 低",
                        FingerprintSeverity::Info => "⚪ 信息",
                    };
                    let cat = m.fingerprint.category.description();
                    let conf = format!("{:.0}%", m.confidence * 100.0);
                    println!("  {:<5} {:<8} {:<20} {:<12} {:<10}",
                        m.fingerprint.id, sev, m.fingerprint.name, cat, conf);
                    println!("    📄 {file_path}");
                    if !m.matched_lines.is_empty() {
                        let lines: Vec<String> = m.matched_lines.iter().map(|l| format!("L{l}")).collect();
                        println!("    📍 行: {}", lines.join(", "));
                    }
                }
                println!();
            }
        }
        DiagnoseSub::List { category, severity } => {
            let category_filter = category.as_deref().and_then(parse_category);
            let severity_filter = severity.as_deref().and_then(parse_severity);

            let fingerprints: Vec<ProblemFingerprint> = service.list_fingerprints()
                .into_iter()
                .filter(|f| {
                    if let Some(cat) = &category_filter {
                        &f.category != cat
                    } else { true }
                })
                .filter(|f| {
                    if let Some(sev) = &severity_filter {
                        &f.severity != sev
                    } else { true }
                })
                .collect();

            println!("\n🔬 问题指纹库\n{}", "─".repeat(60));
            println!("  共 {} 个指纹\n", fingerprints.len());

            println!("  {:-<5} {:-<8} {:-<20} {:-<14}", "ID", "严重度", "名称", "类别");
            println!("  {}", "─".repeat(50));

            for fp in &fingerprints {
                let sev = match fp.severity {
                    FingerprintSeverity::Critical => "🔴 严重",
                    FingerprintSeverity::High => "🟠 高",
                    FingerprintSeverity::Medium => "🟡 中",
                    FingerprintSeverity::Low => "🔵 低",
                    FingerprintSeverity::Info => "⚪ 信息",
                };
                let cat = fp.category.description();
                println!("  {:<5} {:<8} {:<20} {:<14}", fp.id, sev, fp.name, cat);
            }
            println!();
        }
        DiagnoseSub::Detail { id } => {
            if let Some(fp) = service.get_fingerprint_detail(&id) {
                println!("\n🔬 问题指纹详情\n{}", "─".repeat(60));
                println!("  ID: {}", fp.id);
                println!("  名称: {}", fp.name);
                println!("  类别: {}", fp.category.description());
                println!("  严重度: {}", fp.severity.description());
                println!("  可自动修复: {}", if fp.auto_fixable { "✅ 是" } else { "❌ 否" });
                println!();
                println!("  🌡️  常见症状:");
                for (i, symptom) in fp.symptoms.iter().enumerate() {
                    println!("    {}. {}", i + 1, symptom);
                }
                println!();
                println!("  🔍 根因分析:");
                println!("    {}", fp.root_cause);
                println!();
                println!("  💡 修复建议:");
                println!("    {}", fp.fix_suggestion);
                println!();
                if !fp.patterns.is_empty() {
                    println!("  📝 代码模式:");
                    for p in &fp.patterns {
                        println!("    - \"{p}\"");
                    }
                    println!();
                }
                if !fp.error_patterns.is_empty() {
                    println!("  ⚠️  错误模式:");
                    for p in &fp.error_patterns {
                        println!("    - \"{p}\"");
                    }
                    println!();
                }
            } else {
                println!("❌ 未找到指纹: {id}");
                println!("   使用 `cell diagnose list` 查看所有指纹");
            }
        }
        DiagnoseSub::Error { message } => {
            println!("\n🔍 错误诊断\n{}", "─".repeat(60));
            println!("  错误信息: {message}\n");

            let matches = service.scan_log(&message);

            if matches.is_empty() {
                println!("  ❓ 未匹配到已知问题指纹");
                println!("  提示: 可能是新的问题模式，建议记录并分析");
            } else {
                println!("  🎯 匹配到 {} 个相关问题:\n", matches.len());
                for (i, m) in matches.iter().enumerate() {
                    let fp = &m.fingerprint;
                    println!("  {}. [{}] {} ({})", i + 1, fp.id, fp.name, fp.severity.description());
                    println!("     根因: {}", fp.root_cause);
                    println!("     修复: {}", fp.fix_suggestion.lines().next().unwrap_or(""));
                    println!();
                }
            }
        }
        DiagnoseSub::Fix { id } => {
            let mut service = FingerprintService::new();
            match service.fix_problem(&id) {
                Ok(result) => {
                    println!("\n🔧 修复结果\n{}", "─".repeat(60));
                    println!("  指纹 ID: {}", result.fingerprint_id);
                    if result.success {
                        println!("  ✅ 修复成功");
                        println!("  应用修复: {}", result.applied_fix);
                    } else {
                        println!("  ⚠️  {}", result.message);
                        if let Some(fp) = service.get_fingerprint_detail(&id) {
                            println!();
                            println!("  💡 手动修复建议:");
                            println!("    {}", fp.fix_suggestion);
                        }
                    }
                    println!();
                }
                Err(e) => {
                    println!("❌ 修复失败: {e}");
                    println!("   使用 `cell diagnose detail {id}` 查看指纹详情");
                }
            }
        }
    }

    Ok(())
}

fn parse_category(s: &str) -> Option<FingerprintCategory> {
    match s.to_lowercase().as_str() {
        "architecture" | "arch" => Some(FingerprintCategory::Architecture),
        "performance" | "perf" => Some(FingerprintCategory::Performance),
        "security" | "sec" => Some(FingerprintCategory::Security),
        "maintainability" | "maint" => Some(FingerprintCategory::Maintainability),
        "testing" | "test" => Some(FingerprintCategory::Testing),
        "dependency" | "dep" => Some(FingerprintCategory::Dependency),
        "configuration" | "config" => Some(FingerprintCategory::Configuration),
        _ => None,
    }
}

fn parse_severity(s: &str) -> Option<FingerprintSeverity> {
    match s.to_lowercase().as_str() {
        "critical" | "crit" => Some(FingerprintSeverity::Critical),
        "high" => Some(FingerprintSeverity::High),
        "medium" | "med" => Some(FingerprintSeverity::Medium),
        "low" => Some(FingerprintSeverity::Low),
        "info" => Some(FingerprintSeverity::Info),
        _ => None,
    }
}
