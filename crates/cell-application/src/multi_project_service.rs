use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub created_at: String,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub current_project: Option<String>,
    pub projects: Vec<ProjectInfo>,
}

pub struct MultiProjectService;

impl MultiProjectService {
    pub fn new() -> Self {
        Self
    }

    pub fn list_projects(&self, root_path: &str) -> CellResult<Vec<ProjectInfo>> {
        let config = self.load_config(root_path)?;
        Ok(config.projects)
    }

    pub fn get_current_project(&self, root_path: &str) -> CellResult<Option<ProjectInfo>> {
        let config = self.load_config(root_path)?;
        if let Some(name) = &config.current_project {
            Ok(config.projects.into_iter().find(|p| p.name == *name))
        } else {
            Ok(None)
        }
    }

    pub fn switch_project(&self, root_path: &str, name: &str) -> CellResult<()> {
        let mut config = self.load_config(root_path)?;
        
        if !config.projects.iter().any(|p| p.name == name) {
            return Err(cell_domain::errors::CellError::Config(format!(
                "Project '{name}' not found"
            )));
        }

        config.current_project = Some(name.to_string());
        self.save_config(root_path, &config)?;

        Ok(())
    }

    pub fn add_project(&self, root_path: &str, name: &str, path: &str, description: Option<&str>) -> CellResult<ProjectInfo> {
        let mut config = self.load_config(root_path)?;
        
        if config.projects.iter().any(|p| p.name == name) {
            return Err(cell_domain::errors::CellError::Config(format!(
                "Project '{name}' already exists"
            )));
        }

        let project = ProjectInfo {
            name: name.to_string(),
            path: path.to_string(),
            description: description.map(std::string::ToString::to_string),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_opened_at: None,
        };

        config.projects.push(project.clone());
        
        if config.current_project.is_none() {
            config.current_project = Some(name.to_string());
        }

        self.save_config(root_path, &config)?;
        Ok(project)
    }

    pub fn remove_project(&self, root_path: &str, name: &str) -> CellResult<()> {
        let mut config = self.load_config(root_path)?;
        
        let initial_len = config.projects.len();
        config.projects.retain(|p| p.name != name);
        
        if config.projects.len() == initial_len {
            return Err(cell_domain::errors::CellError::Config(format!(
                "Project '{name}' not found"
            )));
        }

        if config.current_project.as_deref() == Some(name) {
            config.current_project = config.projects.first().map(|p| p.name.clone());
        }

        self.save_config(root_path, &config)?;
        Ok(())
    }

    fn config_path(root_path: &str) -> std::path::PathBuf {
        Path::new(root_path).join(".cell/projects.json")
    }

    fn load_config(&self, root_path: &str) -> CellResult<ProjectConfig> {
        let path = Self::config_path(root_path);
        if !path.exists() {
            return Ok(ProjectConfig {
                current_project: None,
                projects: Vec::new(),
            });
        }
        let content = std::fs::read_to_string(&path)?;
        let config: ProjectConfig = serde_json::from_str(&content)
            .map_err(|e| cell_domain::errors::CellError::Config(format!("Invalid projects config: {e}")))?;
        Ok(config)
    }

    fn save_config(&self, root_path: &str, config: &ProjectConfig) -> CellResult<()> {
        let path = Self::config_path(root_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for MultiProjectService {
    fn default() -> Self {
        Self::new()
    }
}
