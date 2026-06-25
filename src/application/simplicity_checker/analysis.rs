//! 代码质量检测器：代码分析工具函数
//! Code quality checker: code analysis utility functions

use super::types::FnInfo;

/// 从函数声明行提取函数名
pub fn fn_name(line: &str) -> String {
    let after_fn = line
        .trim_start_matches("pub ")
        .trim_start_matches("async ")
        .trim_start_matches("fn ");
    after_fn.split('(').next().unwrap_or("unknown").trim().to_string()
}

/// 统计函数参数个数（仅第一层括号内）
pub fn count_args(line: &str) -> usize {
    let mut depth = 0;
    let mut count = 0;
    let mut found_first = false;
    for ch in line.chars() {
        match ch {
            '(' => depth += 1,
            ')' => { if depth > 0 { break; } }
            ',' if depth > 0 => { count += 1; found_first = true; }
            _ => if depth > 0 && !ch.is_whitespace() { found_first = true; }
        }
    }
    if found_first { count + 1 } else { 0 }
}

/// 统计代码块中的 clone() 调用次数
pub fn count_clones(content: &str) -> usize {
    content.matches(".clone()").count()
}

/// 计算代码块最大嵌套深度
pub fn max_nesting(content: &str) -> usize {
    let mut max: usize = 0;
    let mut current: usize = 0;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("if ") || t.starts_with("for ") || t.starts_with("while ") ||
           t.starts_with("match ") || t.starts_with("loop ") {
            current += 1;
            max = max.max(current);
        }
        if t == "}" || t.starts_with('}') {
            current = current.saturating_sub(1);
        }
    }
    max
}

/// 计算圈复杂度（if/match/?/&&/|| 计数）
pub fn complexity(content: &str) -> usize {
    let mut c = 1;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("if ") || t.starts_with("else if ") { c += 1; }
        if t.starts_with("else") && !t.starts_with("else if") { c += 1; }
        if t.starts_with("for ") || t.starts_with("while ") || t.starts_with("match ") { c += 1; }
        if t.contains("&&") { c += 1; }
        if t.contains("||") { c += 1; }
        if t.contains('?') && !t.starts_with("//") && !t.starts_with("/*") { c += 1; }
    }
    c
}

/// 统计注释行数和代码行数
pub fn count_comments(lines: &[&str]) -> (usize, usize) {
    let mut comments = 0;
    let mut code = 0;
    let mut in_block = false;

    for line in lines {
        let t = line.trim();
        if t.starts_with("/*") { in_block = true; comments += 1; continue; }
        if in_block {
            comments += 1;
            if t.contains("*/") { in_block = false; }
            continue;
        }
        if t.starts_with("//") || t.starts_with("///") { comments += 1; continue; }
        if !t.is_empty() { code += 1; }
    }
    (comments, code)
}

/// 从代码内容中提取所有函数信息
pub fn extract_fns(content: &str) -> Vec<FnInfo> {
    let mut fns = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        let is_fn = line.starts_with("fn ") || line.starts_with("pub fn ") ||
                   line.starts_with("async fn ") || line.starts_with("pub async fn ");

        if is_fn && line.contains('(') && line.contains('{') {
            let name = fn_name(line);
            let args = count_args(line);
            let start = i + 1;
            let mut braces = 0;
            let mut started = false;
            let j = i;

            while i < lines.len() {
                for ch in lines[i].chars() {
                    match ch {
                        '{' => { braces += 1; started = true; }
                        '}' => { braces -= 1; }
                        _ => {}
                    }
                }
                i += 1;
                if started && braces == 0 { break; }
            }

            let fn_body = lines[j..i].join("\n");
            let clone_count = count_clones(&fn_body);

            fns.push(FnInfo {
                name,
                start_line: start,
                lines: i - j,
                args,
                nesting: max_nesting(&fn_body),
                complexity: complexity(&fn_body),
                clone_count,
            });
        } else {
            i += 1;
        }
    }
    fns
}

/// 从代码内容中提取所有结构体信息
pub fn extract_structs(content: &str) -> Vec<super::types::StructInfo> {
    use super::types::StructInfo;
    let mut structs = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        let is_struct = line.starts_with("struct ") || line.starts_with("pub struct ");

        if is_struct {
            let name = line
                .trim_start_matches("pub ")
                .trim_start_matches("struct ")
                .split(|c: char| c == ' ' || c == '{' || c == '(')
                .next()
                .unwrap_or("unknown")
                .to_string();

            let start = i + 1;
            let mut braces = 0;
            let mut started = false;
            let mut field_count = 0;

            while i < lines.len() {
                let l = lines[i].trim();
                if l.contains('{') { started = true; braces += l.matches('{').count(); }
                if l.contains('}') { braces -= l.matches('}').count(); }

                if started && braces > 0 && !l.starts_with("//") && !l.is_empty()
                   && !l.starts_with("pub struct") && !l.starts_with("struct")
                   && l.contains(':') && !l.starts_with("/*") && !l.starts_with("///") && !l.starts_with("#[") {
                    field_count += 1;
                }

                i += 1;
                if started && braces == 0 { break; }
            }

            structs.push(StructInfo { name, start_line: start, field_count });
        } else {
            i += 1;
        }
    }
    structs
}
