use crate::application::ports::code_generator::{CodeGeneratorPort, GeneratedFile};
use crate::domain::cell_spec::{CellSpec, PortKind, AdapterKind};
use crate::domain::errors::{CellError, CellResult};
use std::fs;
use std::path::Path;

pub struct GenerateService<T: CodeGeneratorPort> {
    code_generator: T,
}

impl<T: CodeGeneratorPort> GenerateService<T> {
    pub fn new(code_generator: T) -> Self {
        Self { code_generator }
    }

    pub fn generate_cell_from_spec(&self, spec_path: &str, output: &str, force: bool) -> CellResult<Vec<String>> {
        let spec = Self::load_spec(spec_path)?;
        let output_path = Path::new(output);
        if output_path.exists() && !force {
            return Err(CellError::AlreadyExists(format!(
                "Output directory {} already exists (use --force to overwrite)",
                output
            )));
        }
        let files = self.code_generator.render_cell_structure(&spec);
        Self::write_files(&files, output_path)?;
        Ok(files.iter().map(|f| f.path.clone()).collect())
    }

    pub fn generate_cell_from_name(
        &self,
        name: &str,
        output: &str,
        force: bool,
    ) -> CellResult<Vec<String>> {
        let spec = CellSpec {
            name: name.to_string(),
            ..Default::default()
        };
        let output_path = Path::new(output);
        if output_path.exists() && !force {
            return Err(CellError::AlreadyExists(format!(
                "Output directory {} already exists (use --force to overwrite)",
                output
            )));
        }
        let files = self.code_generator.render_cell_structure(&spec);
        Self::write_files(&files, output_path)?;
        Ok(files.iter().map(|f| f.path.clone()).collect())
    }

    pub fn generate_port(
        &self,
        name: &str,
        kind: PortKind,
        output: &str,
    ) -> CellResult<String> {
        use crate::domain::cell_spec::PortSpec;

        let port = PortSpec {
            name: name.to_string(),
            kind,
            description: String::new(),
            input: None,
            output: None,
            is_async: true,
        };

        let port_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/application/ports/{}.rs", port_name));
        let content = render_port_trait(&port);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    pub fn generate_adapter(
        &self,
        name: &str,
        kind: AdapterKind,
        port: &str,
        output: &str,
    ) -> CellResult<String> {
        let adapter_name = to_snake_case(name);
        let file_path = Path::new(output).join(format!("src/adapters/{}.rs", adapter_name));
        let content = render_adapter_struct(name, kind, port);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, &content)?;

        Ok(file_path.to_string_lossy().to_string())
    }

    fn load_spec(path: &str) -> CellResult<CellSpec> {
        let content = fs::read_to_string(path)?;
        let spec: CellSpec = if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)?
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)?
        } else if path.ends_with(".toml") {
            toml::from_str(&content)?
        } else {
            return Err(CellError::Config(format!(
                "Unsupported spec format: {}. Use yaml, json, or toml.",
                path
            )));
        };
        Ok(spec)
    }

    fn write_files(files: &[GeneratedFile], base: &Path) -> CellResult<()> {
        for file in files {
            let file_path = base.join(&file.path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, &file.content)?;
        }
        Ok(())
    }
}

fn render_port_trait(port: &crate::domain::cell_spec::PortSpec) -> String {
    let input_type = port.input.as_deref().unwrap_or("()");
    let output_type = port.output.as_deref().unwrap_or("()");
    let async_kw = if port.is_async { "async " } else { "" };

    format!(
        r#"use crate::domain::errors::CellResult;

pub trait {} {{
    fn {}execute(&self, input: {}) -> CellResult<{}>;
}}
"#,
        to_pascal_case(&port.name),
        async_kw,
        input_type,
        output_type
    )
}

fn render_adapter_struct(name: &str, kind: AdapterKind, port: &str) -> String {
    format!(
        r#"// Adapter: {} (kind: {:?})
// Implements port: {}

pub struct {}Adapter;

impl {}Adapter {{
    pub fn new() -> Self {{
        Self
    }}
}}
"#,
        name,
        kind,
        port,
        to_pascal_case(name),
        to_pascal_case(name),
    )
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut upper = true;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            result.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockGenerator;

    impl CodeGeneratorPort for MockGenerator {
        fn render_cell_structure(&self, spec: &CellSpec) -> Vec<GeneratedFile> {
            vec![
                GeneratedFile {
                    path: format!("{}/Cargo.toml", spec.name),
                    content: format!("[package]\nname = \"{}\"\n", spec.name),
                },
                GeneratedFile {
                    path: format!("{}/src/lib.rs", spec.name),
                    content: "pub mod domain;\n".to_string(),
                },
            ]
        }
    }

    #[test]
    fn test_generate_from_name() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("output");
        let files = service
            .generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), false)
            .unwrap();
        assert!(files.len() > 1);
        assert!(output_dir.join("test-cell/Cargo.toml").exists());
        assert!(output_dir.join("test-cell/src/lib.rs").exists());
    }

    #[test]
    fn test_generate_port() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/application/ports")).unwrap();

        let result = service.generate_port("CreateUser", PortKind::UseCase, dir.path().to_str().unwrap());
        assert!(result.is_ok());

        let file_path = dir.path().join("src/application/ports/create_user.rs");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("trait CreateUser"));
    }

    #[test]
    fn test_generate_adapter() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/adapters")).unwrap();

        let result = service.generate_adapter(
            "InMemoryUserRepo",
            AdapterKind::InMemory,
            "UserRepository",
            dir.path().to_str().unwrap(),
        );
        assert!(result.is_ok());

        let file_path = dir.path().join("src/adapters/in_memory_user_repo.rs");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("InMemoryUserRepoAdapter"));
    }

    #[test]
    fn test_generate_force_flag() {
        let mock_gen = MockGenerator;
        let service = GenerateService::new(mock_gen);
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("output");

        service
            .generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), false)
            .unwrap();

        let result =
            service.generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), false);
        assert!(result.is_err());

        let result2 =
            service.generate_cell_from_name("test-cell", output_dir.to_str().unwrap(), true);
        assert!(result2.is_ok());
    }
}
