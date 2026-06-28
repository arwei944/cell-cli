use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct PubItem {
    name: String,
    file: PathBuf,
    line: usize,
}

#[derive(Debug, Default)]
struct PubItemsStats {
    structs: Vec<PubItem>,
    enums: Vec<PubItem>,
    traits: Vec<PubItem>,
    fns: Vec<PubItem>,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files
}

fn strip_comments_and_strings(line: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            if c == '\\' {
                chars.next();
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        result.push(c);
    }
    result
}

fn count_pub_items_in_file(file_path: &Path, stats: &mut PubItemsStats) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut in_test_module = false;
    let mut brace_depth: i32 = 0;
    let mut test_brace_depth: i32 = 0;

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let clean_line = strip_comments_and_strings(line);
        let trimmed = clean_line.trim();

        if (trimmed.contains("#[cfg(test)]")
            || trimmed.contains("#[test]")
            || trimmed.contains("#[cfg(all(test"))
            && !in_test_module {
                in_test_module = true;
                test_brace_depth = brace_depth;
            }

        let open_braces = trimmed.matches('{').count() as i32;
        let close_braces = trimmed.matches('}').count() as i32;
        brace_depth += open_braces - close_braces;

        if in_test_module && brace_depth <= test_brace_depth && open_braces == 0 {
            in_test_module = false;
        }

        if in_test_module {
            continue;
        }

        if trimmed.starts_with("pub struct ") {
            let name = extract_identifier(trimmed, "struct");
            if let Some(name) = name {
                stats.structs.push(PubItem {
                    name,
                    file: file_path.to_path_buf(),
                    line: line_num,
                });
            }
        } else if trimmed.starts_with("pub enum ") {
            let name = extract_identifier(trimmed, "enum");
            if let Some(name) = name {
                stats.enums.push(PubItem {
                    name,
                    file: file_path.to_path_buf(),
                    line: line_num,
                });
            }
        } else if trimmed.starts_with("pub trait ") {
            let name = extract_identifier(trimmed, "trait");
            if let Some(name) = name {
                stats.traits.push(PubItem {
                    name,
                    file: file_path.to_path_buf(),
                    line: line_num,
                });
            }
        } else if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ") {
            let name = extract_fn_identifier(trimmed);
            if let Some(name) = name {
                stats.fns.push(PubItem {
                    name,
                    file: file_path.to_path_buf(),
                    line: line_num,
                });
            }
        }
    }
}

fn extract_identifier(line: &str, keyword: &str) -> Option<String> {
    let pattern = format!("{keyword} ");
    if let Some(pos) = line.find(&pattern) {
        let rest = &line[pos + pattern.len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn extract_fn_identifier(line: &str) -> Option<String> {
    let keywords = ["pub async fn ", "pub fn "];
    for kw in &keywords {
        if let Some(pos) = line.find(kw) {
            let rest = &line[pos + kw.len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn count_pub_items_in_crate(crate_name: &str) -> PubItemsStats {
    let crate_path = workspace_root()
        .join("crates")
        .join(crate_name)
        .join("src");
    let mut stats = PubItemsStats::default();
    let files = collect_rs_files(&crate_path);
    for file in files {
        count_pub_items_in_file(&file, &mut stats);
    }
    stats
}

fn format_items(items: &[PubItem]) -> String {
    items
        .iter()
        .map(|item| {
            format!(
                "  {}:{} - {}",
                item.file.file_name().unwrap().to_string_lossy(),
                item.line,
                item.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn count_service_pub_fns() -> HashMap<String, usize> {
    let crate_path = workspace_root()
        .join("crates")
        .join("cell-application")
        .join("src");
    let mut result = HashMap::new();
    let files = collect_rs_files(&crate_path);
    for file in files {
        let file_name = file.file_name().unwrap().to_string_lossy().to_string();
        if !file_name.ends_with("_service.rs") {
            continue;
        }
        let mut service_stats = PubItemsStats::default();
        count_pub_items_in_file(&file, &mut service_stats);
        let count = service_stats.fns.len();
        result.insert(file_name, count);
    }
    result
}

mod domain {
    use super::*;

    const MAX_PUB_STRUCTS: usize = 350;
    const MAX_PUB_ENUMS: usize = 150;
    const MAX_PUB_TRAITS: usize = 15;
    const MAX_PUB_FNS: usize = 1000;

    #[test]
    fn domain_pub_structs_under_limit() {
        let stats = count_pub_items_in_crate("cell-domain");
        assert!(
            stats.structs.len() <= MAX_PUB_STRUCTS,
            "Domain crate has {} pub structs, limit is {}\nItems:\n{}",
            stats.structs.len(),
            MAX_PUB_STRUCTS,
            format_items(&stats.structs)
        );
    }

    #[test]
    fn domain_pub_enums_under_limit() {
        let stats = count_pub_items_in_crate("cell-domain");
        assert!(
            stats.enums.len() <= MAX_PUB_ENUMS,
            "Domain crate has {} pub enums, limit is {}\nItems:\n{}",
            stats.enums.len(),
            MAX_PUB_ENUMS,
            format_items(&stats.enums)
        );
    }

    #[test]
    fn domain_pub_traits_under_limit() {
        let stats = count_pub_items_in_crate("cell-domain");
        assert!(
            stats.traits.len() <= MAX_PUB_TRAITS,
            "Domain crate has {} pub traits, limit is {}\nItems:\n{}",
            stats.traits.len(),
            MAX_PUB_TRAITS,
            format_items(&stats.traits)
        );
    }

    #[test]
    fn domain_pub_fns_under_limit() {
        let stats = count_pub_items_in_crate("cell-domain");
        assert!(
            stats.fns.len() <= MAX_PUB_FNS,
            "Domain crate has {} pub fns, limit is {}\nItems:\n{}",
            stats.fns.len(),
            MAX_PUB_FNS,
            format_items(&stats.fns)
        );
    }

    #[test]
    fn domain_pub_items_have_reasonable_names() {
        let stats = count_pub_items_in_crate("cell-domain");
        let all_items: Vec<&PubItem> = stats
            .structs
            .iter()
            .chain(stats.enums.iter())
            .chain(stats.traits.iter())
            .chain(stats.fns.iter())
            .collect();

        let bad_names: Vec<PubItem> = all_items
            .iter()
            .filter(|item| {
                item.name.len() < 2
                    || item.name.chars().next().is_none_or(|c| !c.is_alphabetic())
            })
            .map(|item| (*item).clone())
            .collect();

        assert!(
            bad_names.is_empty(),
            "Domain has {} pub items with suspicious names:\n{}",
            bad_names.len(),
            format_items(&bad_names)
        );
    }
}

mod application {
    use super::*;

    const MAX_PUB_STRUCTS: usize = 350;
    const MAX_PORT_TRAITS: usize = 10;
    const MAX_AVG_SERVICE_PUB_FNS: usize = 12;

    #[test]
    fn application_pub_structs_under_limit() {
        let stats = count_pub_items_in_crate("cell-application");
        assert!(
            stats.structs.len() <= MAX_PUB_STRUCTS,
            "Application crate has {} pub structs, limit is {}\nItems:\n{}",
            stats.structs.len(),
            MAX_PUB_STRUCTS,
            format_items(&stats.structs)
        );
    }

    #[test]
    fn application_port_traits_under_limit() {
        let ports_dir = workspace_root()
            .join("crates")
            .join("cell-application")
            .join("src")
            .join("ports");
        let mut port_stats = PubItemsStats::default();
        let files = collect_rs_files(&ports_dir);
        for file in files {
            count_pub_items_in_file(&file, &mut port_stats);
        }
        assert!(
            port_stats.traits.len() <= MAX_PORT_TRAITS,
            "Application has {} Port traits, limit is {}\nItems:\n{}",
            port_stats.traits.len(),
            MAX_PORT_TRAITS,
            format_items(&port_stats.traits)
        );
    }

    #[test]
    fn application_service_pub_fns_average_under_limit() {
        let service_fns = count_service_pub_fns();
        if service_fns.is_empty() {
            return;
        }
        let total: usize = service_fns.values().sum();
        let count = service_fns.len();
        let avg = total as f64 / count as f64;
        assert!(
            avg <= MAX_AVG_SERVICE_PUB_FNS as f64,
            "Application services have average {:.1} pub fns per service, limit is {}\nPer service:\n{}",
            avg,
            MAX_AVG_SERVICE_PUB_FNS,
            service_fns
                .iter()
                .map(|(name, n)| format!("  {name}: {n} pub fns"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

mod adapters {
    use super::*;

    const MAX_PUB_STRUCTS: usize = 25;

    #[test]
    fn adapters_pub_structs_reasonable() {
        let stats = count_pub_items_in_crate("cell-adapters");
        assert!(
            stats.structs.len() <= MAX_PUB_STRUCTS,
            "Adapters crate has {} pub structs, limit is {}\nItems:\n{}",
            stats.structs.len(),
            MAX_PUB_STRUCTS,
            format_items(&stats.structs)
        );
    }

    #[test]
    fn adapters_have_pub_structs_for_constructors() {
        let stats = count_pub_items_in_crate("cell-adapters");
        let adapter_structs: Vec<&PubItem> = stats
            .structs
            .iter()
            .filter(|s| {
                s.name.contains("File")
                    || s.name.contains("Json")
                    || s.name.contains("Tera")
                    || s.name.contains("Web")
                    || s.name.contains("Ast")
                    || s.name.contains("Template")
            })
            .collect();
        assert!(
            !adapter_structs.is_empty(),
            "Adapters should have public structs for adapter implementations"
        );
    }
}

mod interfaces {
    use super::*;

    const MAX_CLI_COMMANDS: usize = 80;

    #[test]
    fn interfaces_cli_commands_reasonable() {
        let commands_dir = workspace_root()
            .join("crates")
            .join("cell-interfaces")
            .join("src")
            .join("commands");
        let mut cmd_stats = PubItemsStats::default();
        let files = collect_rs_files(&commands_dir);
        for file in files {
            count_pub_items_in_file(&file, &mut cmd_stats);
        }
        let total_pub_fns = cmd_stats.fns.len();
        assert!(
            total_pub_fns <= MAX_CLI_COMMANDS,
            "Interfaces crate has {} pub command fns, limit is {}\nItems:\n{}",
            total_pub_fns,
            MAX_CLI_COMMANDS,
            format_items(&cmd_stats.fns)
        );
    }

    #[test]
    fn interfaces_cli_types_reasonable() {
        let stats = count_pub_items_in_crate("cell-interfaces");
        let cli_structs_count = stats.structs.len();
        assert!(
            cli_structs_count <= 80,
            "Interfaces crate has {} pub structs, limit is 80\nItems:\n{}",
            cli_structs_count,
            format_items(&stats.structs)
        );
    }
}
