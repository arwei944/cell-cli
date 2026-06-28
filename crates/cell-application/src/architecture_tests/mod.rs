#![cfg(test)]
#![allow(clippy::unwrap_used)]

pub mod best_practices;
pub mod code_quality;
pub mod layer_deps;
pub mod visibility;

use crate::arch_linter::ArchitectureLinter;
use std::fs;
use tempfile::TempDir;

pub fn setup_test_project() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");

    fs::create_dir_all(src.join("domain")).unwrap();
    fs::create_dir_all(src.join("application")).unwrap();
    fs::create_dir_all(src.join("adapters")).unwrap();
    fs::create_dir_all(src.join("interfaces")).unwrap();

    fs::write(
        src.join("domain").join("mod.rs"),
        r"
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
    )
    .unwrap();

    fs::write(
        src.join("application").join("mod.rs"),
        r"
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
",
    )
    .unwrap();

    fs::write(
        src.join("adapters").join("mod.rs"),
        r"
use crate::application::Service;
pub struct Adapter;
impl Adapter {
    pub fn run() { Service::do_work(); }
}
",
    )
    .unwrap();

    fs::write(
        src.join("interfaces").join("cli.rs"),
        r"
use crate::application::Service;
pub fn run() { Service::do_work(); }
",
    )
    .unwrap();

    fs::write(
        src.join("lib.rs"),
        r"
pub mod domain;
pub mod application;
pub mod adapters;
pub mod interfaces;
",
    )
    .unwrap();

    dir
}

#[allow(dead_code)]
pub fn count_layer_violations(linter: &ArchitectureLinter, dir: &TempDir, from: &str, to: &str) -> usize {
    let result = linter.lint(dir.path());
    result
        .violations
        .iter()
        .filter(|v| {
            let file = v.file.replace('\\', "/");
            let from_match = file.contains(&format!("src/{from}/")) || file.starts_with(&format!("{from}/"));
            let to_match = v.message.contains(to);
            from_match && to_match
        })
        .count()
}
