#![cfg(test)]
#![allow(clippy::unwrap_used)]

use crate::application::arch_service::{ArchitectureRules, validate_architecture};
use std::path::Path;

fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn domain_must_not_depend_on_application() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.from_module.contains("domain") && v.to_module.contains("application"))
        .collect();
    assert!(
        violations.is_empty(),
        "Domain → Application FORBIDDEN: {:#?}",
        violations
    );
}

#[test]
fn domain_must_not_depend_on_adapters() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.from_module.contains("domain") && v.to_module.contains("adapters"))
        .collect();
    assert!(
        violations.is_empty(),
        "Domain → Adapters FORBIDDEN: {:#?}",
        violations
    );
}

#[test]
fn domain_must_not_depend_on_interfaces() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.from_module.contains("domain") && v.to_module.contains("interfaces"))
        .collect();
    assert!(
        violations.is_empty(),
        "Domain → Interfaces FORBIDDEN: {:#?}",
        violations
    );
}

#[test]
fn application_must_not_depend_on_interfaces() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.from_module.contains("application") && v.to_module.contains("interfaces"))
        .collect();
    assert!(
        violations.is_empty(),
        "Application → Interfaces FORBIDDEN: {:#?}",
        violations
    );
}

#[test]
fn application_must_not_depend_on_adapters() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.from_module.contains("application") && v.to_module.contains("adapters"))
        .collect();
    assert!(
        violations.is_empty(),
        "Application → Adapters FORBIDDEN (must use Port): {:#?}",
        violations
    );
}

#[test]
fn adapters_must_not_depend_on_interfaces() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.from_module.contains("adapters") && v.to_module.contains("interfaces"))
        .collect();
    assert!(
        violations.is_empty(),
        "Adapters → Interfaces FORBIDDEN: {:#?}",
        violations
    );
}

#[test]
fn architecture_must_pass_all_rules() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    assert!(
        result.passed,
        "Architecture validation FAILED. Violations: {:#?}",
        result.violations
    );
}

#[test]
fn domain_has_zero_external_deps() {
    let rules = ArchitectureRules::default();
    let result = validate_architecture(project_root(), &rules);
    let domain_stats = result.layer_stats.get("domain").unwrap();
    assert_eq!(
        domain_stats.external_deps, 0,
        "Domain layer must have ZERO external deps, found {}",
        domain_stats.external_deps
    );
}
