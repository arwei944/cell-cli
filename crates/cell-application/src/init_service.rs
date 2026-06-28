use cell_domain::errors::{CellError, CellResult};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct InitInput {
    pub name: Option<String>,
    pub path: Option<String>,
    pub template: Option<String>,
    pub yes: bool,
    pub force: bool,
}

pub fn run_init(input: &InitInput) -> CellResult<String> {
    let project_name = input.name.as_deref().unwrap_or("my-cell");
    let base_path = input.path.as_deref().unwrap_or(".");
    let project_path = Path::new(base_path).join(project_name);

    if project_path.exists() && !input.force {
        return Err(CellError::AlreadyExists(format!(
            "Directory {} already exists (use --force to overwrite)",
            project_path.display()
        )));
    }

    fs::create_dir_all(&project_path)?;
    create_cell_structure(&project_path, project_name)?;

    Ok(project_path.display().to_string())
}

fn create_cell_structure(path: &Path, name: &str) -> CellResult<()> {
    let dirs = vec![
        "src/domain",
        "src/application",
        "src/adapters",
        "src/interfaces",
        "tests",
        "specs",
        "docs/adr",
    ];

    for dir in dirs {
        fs::create_dir_all(path.join(dir))?;
    }

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    let readme = format!(
        r"# {name}

Cell 架构项目

## 目录结构

```
src/
├── domain/       # 领域层（纯业务逻辑）
├── application/  # 应用层（用例编排）
├── adapters/     # 适配器层（外部技术实现）
└── interfaces/   # 接口层（输入端口实现）
specs/            # 规范定义
tests/            # 测试
docs/             # 文档
```
"
    );
    fs::write(path.join("README.md"), readme)?;

    let lib_rs = r"pub mod domain;
pub mod application;
pub mod adapters;
pub mod interfaces;
";
    fs::write(path.join("src/lib.rs"), lib_rs)?;

    fs::write(path.join("src/domain/mod.rs"), "// Domain layer\n")?;
    fs::write(
        path.join("src/application/mod.rs"),
        "// Application layer\n",
    )?;
    fs::write(path.join("src/adapters/mod.rs"), "// Adapters layer\n")?;
    fs::write(path.join("src/interfaces/mod.rs"), "// Interfaces layer\n")?;

    let gitignore = r"/target
Cargo.lock
*.swp
.DS_Store
";
    fs::write(path.join(".gitignore"), gitignore)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_creates_structure() {
        let dir = tempfile::tempdir().unwrap();
        let input = InitInput {
            name: Some("test-project".to_string()),
            path: Some(dir.path().to_str().unwrap().to_string()),
            template: None,
            yes: true,
            force: false,
        };

        let result = run_init(&input);
        assert!(result.is_ok());

        let project_path = dir.path().join("test-project");
        assert!(project_path.join("src/domain").exists());
        assert!(project_path.join("src/application").exists());
        assert!(project_path.join("src/adapters").exists());
        assert!(project_path.join("src/interfaces").exists());
        assert!(project_path.join("Cargo.toml").exists());
    }

    #[test]
    fn test_init_fails_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path().join("existing");
        fs::create_dir_all(&project_path).unwrap();

        let input = InitInput {
            name: Some("existing".to_string()),
            path: Some(dir.path().to_str().unwrap().to_string()),
            template: None,
            yes: true,
            force: false,
        };

        let result = run_init(&input);
        assert!(result.is_err());
    }
}
