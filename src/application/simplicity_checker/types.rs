//! 代码质量检测器：所有数据类型定义
//! Code quality checker: all data type definitions

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplicityReport {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_functions: usize,
    pub score: f64,
    pub grade: Grade,
    pub files: Vec<FileReport>,
    pub issues: Vec<Issue>,
    pub summary: Summary,
    pub dimension_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReport {
    pub path: String,
    pub lines: usize,
    pub fn_count: usize,
    pub struct_count: usize,
    pub avg_fn_lines: f64,
    pub max_fn_lines: usize,
    pub comment_ratio: f64,
    pub score: f64,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub path: String,
    pub line: Option<usize>,
    pub severity: Severity,
    pub category: Category,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Summary {
    pub long_files: usize,
    pub long_functions: usize,
    pub large_structs: usize,
    pub low_comment_files: usize,
    pub complex_functions: usize,
    pub deep_nesting: usize,
    pub many_args: usize,
    pub magic_numbers: usize,
    pub todo_markers: usize,
    pub duplicate_imports: usize,
    pub unwrap_usage: usize,
    pub unsafe_usage: usize,
    pub clone_overuse: usize,
    pub string_concat_inefficient: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Grade { S, A, B, C, D, F }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity { Info, Warning, Error }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    LongFile,
    LongFunction,
    LargeStruct,
    LowComments,
    ComplexFunction,
    DeepNesting,
    ManyArgs,
    MagicNumber,
    TodoMarker,
    UnwrapUsage,
    UnsafeCode,
    CloneOveruse,
    DuplicateImport,
    InefficientString,
}

impl Category {
    pub fn display_name(&self) -> &'static str {
        match self {
            Category::LongFile => "Long file",
            Category::LongFunction => "Long function",
            Category::LargeStruct => "Large struct",
            Category::LowComments => "Low comments",
            Category::ComplexFunction => "High complexity",
            Category::DeepNesting => "Deep nesting",
            Category::ManyArgs => "Many args",
            Category::MagicNumber => "Magic number",
            Category::TodoMarker => "TODO marker",
            Category::UnwrapUsage => "unwrap() usage",
            Category::UnsafeCode => "unsafe code",
            Category::CloneOveruse => "clone() overuse",
            Category::DuplicateImport => "Duplicate import",
            Category::InefficientString => "Inefficient string",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnInfo {
    pub name: String,
    pub start_line: usize,
    pub lines: usize,
    pub args: usize,
    pub nesting: usize,
    pub complexity: usize,
    pub clone_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub start_line: usize,
    pub field_count: usize,
}

use serde::{Deserialize, Serialize};
