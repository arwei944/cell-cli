use super::*;
use crate::arch_linter::{ArchitectureLinter, LintCategory, LintSeverity};
use std::fs;

fn has_rule_violation(result: &crate::arch_linter::LintResult, rule_id: &str) -> bool {
    result.violations.iter().any(|v| v.rule_id == rule_id)
}

fn get_rule_violations<'a>(result: &'a crate::arch_linter::LintResult, rule_id: &str) -> Vec<&'a crate::arch_linter::LintViolation> {
    result.violations.iter().filter(|v| v.rule_id == rule_id).collect()
}

mod domain_forbidden_deps {
    use super::*;

    #[test]
    fn test_domain_depends_on_application_detected_use() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
use crate::application::Service;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L001");

        assert!(
            !violations.is_empty(),
            "Should detect L001 (domain → application) violation, but got {} violations. All violations: {:?}",
            violations.len(),
            result.violations.iter().map(|v| (&v.rule_id, &v.file, v.line)).collect::<Vec<_>>()
        );
        assert_eq!(violations[0].severity, LintSeverity::Error);
        assert!(violations[0].file.contains("domain"), "File path should contain domain: {}", violations[0].file);
        assert!(violations[0].line > 0, "Line number should be positive");
    }

    #[test]
    fn test_domain_depends_on_application_detected_pub_use() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
pub use crate::application::Service;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L001");

        assert!(
            !violations.is_empty(),
            "Should detect L001 (domain → application) violation with pub use, but got {} violations",
            violations.len()
        );
    }

    #[test]
    fn test_domain_depends_on_application_detected_nested_module() {
        let dir = setup_test_project();
        let domain_sub = dir.path().join("src/domain/services");
        fs::create_dir_all(&domain_sub).unwrap();
        fs::write(
            domain_sub.join("mod.rs"),
            r"
use crate::application::Service;
pub fn do_something() { let _s = Service::do_work(); }
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L001");

        assert!(
            !violations.is_empty(),
            "Should detect L001 violation in nested domain module, but got {} violations",
            violations.len()
        );
        assert!(
            violations.iter().any(|v| v.file.contains("services")),
            "Violation should be in services submodule"
        );
    }

    #[test]
    fn test_domain_depends_on_adapters_detected_use() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
use crate::adapters::Adapter;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L002");

        assert!(
            !violations.is_empty(),
            "Should detect L002 (domain → adapters) violation, but got {} violations",
            violations.len()
        );
        assert_eq!(violations[0].severity, LintSeverity::Error);
        assert!(violations[0].line > 0);
    }

    #[test]
    fn test_domain_depends_on_adapters_detected_pub_use() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
pub use crate::adapters::Adapter;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        assert!(
            has_rule_violation(&result, "L002"),
            "Should detect L002 violation with pub use"
        );
    }

    #[test]
    fn test_domain_depends_on_adapters_detected_nested_module() {
        let dir = setup_test_project();
        let domain_sub = dir.path().join("src/domain/entities");
        fs::create_dir_all(&domain_sub).unwrap();
        fs::write(
            domain_sub.join("mod.rs"),
            r"
use crate::adapters::Adapter;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L002");

        assert!(
            !violations.is_empty(),
            "Should detect L002 violation in nested domain module"
        );
        assert!(
            violations.iter().any(|v| v.file.contains("entities")),
            "Violation should be in entities submodule"
        );
    }

    #[test]
    fn test_domain_depends_on_interfaces_detected_use() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
use crate::interfaces::cli;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L003");

        assert!(
            !violations.is_empty(),
            "Should detect L003 (domain → interfaces) violation, but got {} violations",
            violations.len()
        );
        assert_eq!(violations[0].severity, LintSeverity::Error);
    }

    #[test]
    fn test_domain_depends_on_interfaces_detected_pub_use() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
pub use crate::interfaces::cli;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        assert!(
            has_rule_violation(&result, "L003"),
            "Should detect L003 violation with pub use"
        );
    }

    #[test]
    fn test_domain_depends_on_interfaces_detected_nested_module() {
        let dir = setup_test_project();
        let domain_sub = dir.path().join("src/domain/value_objects");
        fs::create_dir_all(&domain_sub).unwrap();
        fs::write(
            domain_sub.join("mod.rs"),
            r"
use crate::interfaces::cli;
pub struct ValueObject;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L003");

        assert!(
            !violations.is_empty(),
            "Should detect L003 violation in nested domain module"
        );
    }
}

mod application_forbidden_deps {
    use super::*;

    #[test]
    fn test_application_depends_on_adapters_detected_use() {
        let dir = setup_test_project();
        let app_mod = dir.path().join("src/application/mod.rs");
        fs::write(
            app_mod,
            r"
use crate::adapters::Adapter;
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L005");

        assert!(
            !violations.is_empty(),
            "Should detect L005 (application → adapters) violation, but got {} violations",
            violations.len()
        );
        assert_eq!(violations[0].severity, LintSeverity::Error);
        assert!(violations[0].line > 0);
    }

    #[test]
    fn test_application_depends_on_adapters_detected_pub_use() {
        let dir = setup_test_project();
        let app_mod = dir.path().join("src/application/mod.rs");
        fs::write(
            app_mod,
            r"
pub use crate::adapters::Adapter;
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        assert!(
            has_rule_violation(&result, "L005"),
            "Should detect L005 violation with pub use"
        );
    }

    #[test]
    fn test_application_depends_on_adapters_detected_nested_module() {
        let dir = setup_test_project();
        let app_sub = dir.path().join("src/application/usecases");
        fs::create_dir_all(&app_sub).unwrap();
        fs::write(
            app_sub.join("mod.rs"),
            r"
use crate::adapters::Adapter;
pub struct UseCase;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L005");

        assert!(
            !violations.is_empty(),
            "Should detect L005 violation in nested application module"
        );
        assert!(
            violations.iter().any(|v| v.file.contains("usecases")),
            "Violation should be in usecases submodule"
        );
    }

    #[test]
    fn test_application_depends_on_interfaces_detected_use() {
        let dir = setup_test_project();
        let app_mod = dir.path().join("src/application/mod.rs");
        fs::write(
            app_mod,
            r"
use crate::interfaces::cli;
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L004");

        assert!(
            !violations.is_empty(),
            "Should detect L004 (application → interfaces) violation, but got {} violations",
            violations.len()
        );
        assert_eq!(violations[0].severity, LintSeverity::Error);
    }

    #[test]
    fn test_application_depends_on_interfaces_detected_pub_use() {
        let dir = setup_test_project();
        let app_mod = dir.path().join("src/application/mod.rs");
        fs::write(
            app_mod,
            r"
pub use crate::interfaces::cli::run;
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        assert!(
            has_rule_violation(&result, "L004"),
            "Should detect L004 violation with pub use"
        );
    }

    #[test]
    fn test_application_depends_on_interfaces_detected_nested_module() {
        let dir = setup_test_project();
        let app_sub = dir.path().join("src/application/ports");
        fs::create_dir_all(&app_sub).unwrap();
        fs::write(
            app_sub.join("mod.rs"),
            r"
use crate::interfaces::cli;
pub trait InputPort;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L004");

        assert!(
            !violations.is_empty(),
            "Should detect L004 violation in nested application module"
        );
    }
}

mod adapters_forbidden_deps {
    use super::*;

    #[test]
    fn test_adapters_depends_on_interfaces_detected_use() {
        let dir = setup_test_project();
        let adapter_mod = dir.path().join("src/adapters/mod.rs");
        fs::write(
            adapter_mod,
            r"
use crate::interfaces::cli;
use crate::application::Service;
pub struct Adapter;
impl Adapter {
    pub fn run() { Service::do_work(); }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L006");

        assert!(
            !violations.is_empty(),
            "Should detect L006 (adapters → interfaces) violation, but got {} violations",
            violations.len()
        );
        assert_eq!(violations[0].severity, LintSeverity::Error);
        assert!(violations[0].line > 0);
    }

    #[test]
    fn test_adapters_depends_on_interfaces_detected_pub_use() {
        let dir = setup_test_project();
        let adapter_mod = dir.path().join("src/adapters/mod.rs");
        fs::write(
            adapter_mod,
            r"
pub use crate::interfaces::cli::run;
use crate::application::Service;
pub struct Adapter;
impl Adapter {
    pub fn run() { Service::do_work(); }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        assert!(
            has_rule_violation(&result, "L006"),
            "Should detect L006 violation with pub use"
        );
    }

    #[test]
    fn test_adapters_depends_on_interfaces_detected_nested_module() {
        let dir = setup_test_project();
        let adapter_sub = dir.path().join("src/adapters/persistence");
        fs::create_dir_all(&adapter_sub).unwrap();
        fs::write(
            adapter_sub.join("mod.rs"),
            r"
use crate::interfaces::cli;
pub struct Repository;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());
        let violations = get_rule_violations(&result, "L006");

        assert!(
            !violations.is_empty(),
            "Should detect L006 violation in nested adapters module"
        );
        assert!(
            violations.iter().any(|v| v.file.contains("persistence")),
            "Violation should be in persistence submodule"
        );
    }
}

mod allowed_dependencies {
    use super::*;

    #[test]
    fn test_application_depends_on_domain_allowed() {
        let dir = setup_test_project();
        let app_mod = dir.path().join("src/application/mod.rs");
        fs::write(
            app_mod,
            r"
use crate::domain::Entity;
pub struct Service;
impl Service {
    pub fn do_work() -> Entity { Entity::new() }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "application → domain should be allowed, but got layer errors: {:?}",
            layer_errors.iter().map(|v| (&v.rule_id, &v.file, v.line)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_application_depends_on_domain_allowed_nested() {
        let dir = setup_test_project();
        let app_sub = dir.path().join("src/application/usecases");
        fs::create_dir_all(&app_sub).unwrap();
        fs::write(
            app_sub.join("mod.rs"),
            r"
use crate::domain::Entity;
pub struct UseCase;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "application → domain (nested) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_adapters_depends_on_domain_allowed() {
        let dir = setup_test_project();
        let adapter_mod = dir.path().join("src/adapters/mod.rs");
        fs::write(
            adapter_mod,
            r"
use crate::domain::Entity;
use crate::application::Service;
pub struct Adapter;
impl Adapter {
    pub fn run() -> Entity { Service::do_work() }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "adapters → domain should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_adapters_depends_on_domain_allowed_nested() {
        let dir = setup_test_project();
        let adapter_sub = dir.path().join("src/adapters/persistence");
        fs::create_dir_all(&adapter_sub).unwrap();
        fs::write(
            adapter_sub.join("mod.rs"),
            r"
use crate::domain::Entity;
pub struct Repository;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "adapters → domain (nested) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_adapters_depends_on_application_allowed() {
        let dir = setup_test_project();
        let adapter_mod = dir.path().join("src/adapters/mod.rs");
        fs::write(
            adapter_mod,
            r"
use crate::application::Service;
pub struct Adapter;
impl Adapter {
    pub fn run() { Service::do_work(); }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "adapters → application should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_adapters_depends_on_application_allowed_nested() {
        let dir = setup_test_project();
        let adapter_sub = dir.path().join("src/adapters/persistence");
        fs::create_dir_all(&adapter_sub).unwrap();
        fs::write(
            adapter_sub.join("mod.rs"),
            r"
use crate::application::Service;
pub struct Repository;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "adapters → application (nested) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_interfaces_depends_on_domain_allowed() {
        let dir = setup_test_project();
        let cli_mod = dir.path().join("src/interfaces/cli.rs");
        fs::write(
            cli_mod,
            r"
use crate::domain::Entity;
use crate::application::Service;
pub fn run() -> Entity { Service::do_work() }
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "interfaces → domain should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_interfaces_depends_on_domain_allowed_nested() {
        let dir = setup_test_project();
        let iface_sub = dir.path().join("src/interfaces/api");
        fs::create_dir_all(&iface_sub).unwrap();
        fs::write(
            iface_sub.join("mod.rs"),
            r"
use crate::domain::Entity;
pub struct ApiHandler;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "interfaces → domain (nested) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_interfaces_depends_on_application_allowed() {
        let dir = setup_test_project();
        let cli_mod = dir.path().join("src/interfaces/cli.rs");
        fs::write(
            cli_mod,
            r"
use crate::application::Service;
pub fn run() { Service::do_work(); }
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "interfaces → application should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_interfaces_depends_on_application_allowed_nested() {
        let dir = setup_test_project();
        let iface_sub = dir.path().join("src/interfaces/api");
        fs::create_dir_all(&iface_sub).unwrap();
        fs::write(
            iface_sub.join("mod.rs"),
            r"
use crate::application::Service;
pub struct ApiHandler;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "interfaces → application (nested) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_interfaces_depends_on_adapters_allowed() {
        let dir = setup_test_project();
        let cli_mod = dir.path().join("src/interfaces/cli.rs");
        fs::write(
            cli_mod,
            r"
use crate::adapters::Adapter;
use crate::application::Service;
pub fn run() { Adapter::run(); }
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "interfaces → adapters should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_interfaces_depends_on_adapters_allowed_nested() {
        let dir = setup_test_project();
        let iface_sub = dir.path().join("src/interfaces/api");
        fs::create_dir_all(&iface_sub).unwrap();
        fs::write(
            iface_sub.join("mod.rs"),
            r"
use crate::adapters::Adapter;
pub struct ApiHandler;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "interfaces → adapters (nested) should be allowed, but got: {layer_errors:?}"
        );
    }
}

mod same_layer_deps {
    use super::*;

    #[test]
    fn test_domain_same_layer_deps_allowed() {
        let dir = setup_test_project();
        let domain_sub = dir.path().join("src/domain/entities");
        fs::create_dir_all(&domain_sub).unwrap();
        fs::write(
            domain_sub.join("mod.rs"),
            r"
use crate::domain::Entity;
pub struct AnotherEntity;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "domain → domain (same layer) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_application_same_layer_deps_allowed() {
        let dir = setup_test_project();
        let app_sub = dir.path().join("src/application/usecases");
        fs::create_dir_all(&app_sub).unwrap();
        fs::write(
            app_sub.join("mod.rs"),
            r"
use crate::application::Service;
pub struct UseCase;
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        let layer_errors: Vec<_> = result
            .violations
            .iter()
            .filter(|v| v.category == LintCategory::Layering && v.severity == LintSeverity::Error)
            .collect();

        assert!(
            layer_errors.is_empty(),
            "application → application (same layer) should be allowed, but got: {layer_errors:?}"
        );
    }

    #[test]
    fn test_multiple_violations_detected_in_same_file() {
        let dir = setup_test_project();
        let domain_mod = dir.path().join("src/domain/mod.rs");
        fs::write(
            domain_mod,
            r"
use crate::application::Service;
use crate::adapters::Adapter;
use crate::interfaces::cli;
pub struct Entity;
impl Entity {
    pub fn new() -> Self { Entity }
}
",
        )
        .unwrap();

        let linter = ArchitectureLinter::new();
        let result = linter.lint(dir.path());

        assert!(
            has_rule_violation(&result, "L001"),
            "Should detect L001 (domain → application)"
        );
        assert!(
            has_rule_violation(&result, "L002"),
            "Should detect L002 (domain → adapters)"
        );
        assert!(
            has_rule_violation(&result, "L003"),
            "Should detect L003 (domain → interfaces)"
        );
    }
}
