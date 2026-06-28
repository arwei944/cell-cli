use crate::enforcement_service::CiProvider;
use cell_domain::errors::CellResult;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiTemplate {
    pub provider: CiProvider,
    pub name: String,
    pub content: String,
    pub path: String,
}

pub struct CiTemplateService;

impl CiTemplateService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_github_actions(&self, _project_path: &str) -> CellResult<CiTemplate> {
        let content = [
            "name: Cell Architecture Gate",
            "on:",
            "  pull_request:",
            "    branches: [ main, master, develop ]",
            "  push:",
            "    branches: [ main, master, develop ]",
            "",
            "jobs:",
            "  architecture-gate:",
            "    runs-on: ubuntu-latest",
            "    steps:",
            "      - uses: actions/checkout@v4",
            "        with:",
            "          fetch-depth: 0",
            "",
            "      - name: Install Rust",
            "        uses: dtolnay/rust-toolchain@stable",
            "",
            "      - name: Cache cargo",
            "        uses: Swatinem/rust-cache@v2",
            "",
            "      - name: Install cell CLI",
            "        run: cargo install --path .",
            "",
            "      - name: Architecture Check",
            "        run: cell arch lint",
            "        continue-on-error: false",
            "",
            "      - name: Entropy Check",
            "        run: cell entropy",
            "        continue-on-error: false",
            "",
            "      - name: Test Suite",
            "        run: cargo test --release",
            "        continue-on-error: false",
            "",
            "      - name: Code Review",
            "        run: cell review --deep",
            "        continue-on-error: true",
            "",
            "      - name: Entropy Trend",
            "        run: cell entropy trend",
            "        continue-on-error: true",
            "",
            "      - name: Post Results",
            "        if: always()",
            "        run: |",
            "          echo \"## Cell Architecture Gate Results\" >> $GITHUB_STEP_SUMMARY",
            "          echo \"\" >> $GITHUB_STEP_SUMMARY",
            "          cell arch status --format md >> $GITHUB_STEP_SUMMARY",
            "          cell entropy --format md >> $GITHUB_STEP_SUMMARY",
            "",
        ].join("\n");

        Ok(CiTemplate {
            provider: CiProvider::Github,
            name: "GitHub Actions".to_string(),
            content,
            path: ".github/workflows/cell-gate.yml".to_string(),
        })
    }

    pub fn generate_gitlab_ci(&self, _project_path: &str) -> CellResult<CiTemplate> {
        let content = [
            "stages:",
            "  - check",
            "  - test",
            "  - review",
            "",
            "cell-architecture-check:",
            "  stage: check",
            "  image: rust:latest",
            "  script:",
            "    - cargo install --path .",
            "    - cell arch lint",
            "    - cell entropy",
            "  rules:",
            "    - if: '$CI_PIPELINE_SOURCE == \"merge_request_event\"'",
            "    - if: '$CI_COMMIT_BRANCH == \"main\"'",
            "    - if: '$CI_COMMIT_BRANCH == \"develop\"'",
            "",
            "cell-test-suite:",
            "  stage: test",
            "  image: rust:latest",
            "  script:",
            "    - cargo test --release",
            "  artifacts:",
            "    reports:",
            "      junit: target/test-results.xml",
            "",
            "cell-code-review:",
            "  stage: review",
            "  image: rust:latest",
            "  script:",
            "    - cargo install --path .",
            "    - cell review --deep",
            "  allow_failure: true",
            "  rules:",
            "    - if: '$CI_PIPELINE_SOURCE == \"merge_request_event\"'",
            "",
        ].join("\n");

        Ok(CiTemplate {
            provider: CiProvider::Gitlab,
            name: "GitLab CI".to_string(),
            content,
            path: ".gitlab-ci.yml".to_string(),
        })
    }

    pub fn generate_jenkinsfile(&self, _project_path: &str) -> CellResult<CiTemplate> {
        let content = [
            "pipeline {",
            "    agent any",
            "    ",
            "    stages {",
            "        stage('Architecture Check') {",
            "            steps {",
            "                sh 'cargo install --path .'",
            "                sh 'cell arch lint'",
            "                sh 'cell entropy'",
            "            }",
            "        }",
            "        ",
            "        stage('Test') {",
            "            steps {",
            "                sh 'cargo test --release'",
            "            }",
            "            post {",
            "                always {",
            "                    junit 'target/test-results.xml'",
            "                }",
            "            }",
            "        }",
            "        ",
            "        stage('Code Review') {",
            "            steps {",
            "                sh 'cell review --deep'",
            "            }",
            "        }",
            "    }",
            "    ",
            "    post {",
            "        success {",
            "            echo 'All checks passed!'",
            "        }",
            "        failure {",
            "            echo 'Some checks failed'",
            "        }",
            "    }",
            "}",
            "",
        ].join("\n");

        Ok(CiTemplate {
            provider: CiProvider::Jenkins,
            name: "Jenkins".to_string(),
            content,
            path: "Jenkinsfile".to_string(),
        })
    }

    pub fn generate_gitee_workflow(&self, _project_path: &str) -> CellResult<CiTemplate> {
        let content = [
            "name: Cell Architecture Gate",
            "",
            "on:",
            "  pull_request:",
            "    branches: [ main, master, develop ]",
            "  push:",
            "    branches: [ main, master, develop ]",
            "",
            "jobs:",
            "  architecture-gate:",
            "    runs-on: gitee-go",
            "    steps:",
            "      - uses: actions/checkout@v4",
            "        with:",
            "          fetch-depth: 0",
            "",
            "      - name: Install Rust",
            "        run: |",
            "          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
            "          source $HOME/.cargo/env",
            "",
            "      - name: Install cell CLI",
            "        run: cargo install --path .",
            "",
            "      - name: Architecture Check",
            "        run: cell arch lint",
            "",
            "      - name: Entropy Check",
            "        run: cell entropy",
            "",
            "      - name: Test Suite",
            "        run: cargo test --release",
            "",
        ].join("\n");

        Ok(CiTemplate {
            provider: CiProvider::Gitee,
            name: "Gitee Go".to_string(),
            content,
            path: ".gitee/workflows/cell-gate.yml".to_string(),
        })
    }

    pub fn generate_all(&self, project_path: &str) -> CellResult<Vec<CiTemplate>> {
        let templates = vec![
            self.generate_github_actions(project_path)?,
            self.generate_gitlab_ci(project_path)?,
            self.generate_jenkinsfile(project_path)?,
            self.generate_gitee_workflow(project_path)?,
        ];
        Ok(templates)
    }

    pub fn apply_template(&self, project_path: &str, provider: &CiProvider) -> CellResult<CiTemplate> {
        let template = match provider {
            CiProvider::Github => self.generate_github_actions(project_path)?,
            CiProvider::Gitlab => self.generate_gitlab_ci(project_path)?,
            CiProvider::Jenkins => self.generate_jenkinsfile(project_path)?,
            CiProvider::Gitee => self.generate_gitee_workflow(project_path)?,
            CiProvider::None => return Err(cell_domain::errors::CellError::Config("No provider selected".to_string())),
        };

        let output_path = Path::new(project_path).join(&template.path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, &template.content)?;

        Ok(template)
    }

    pub fn apply_all(&self, project_path: &str) -> CellResult<Vec<CiTemplate>> {
        let templates = self.generate_all(project_path)?;
        let mut applied = Vec::new();
        
        for template in templates {
            let output_path = Path::new(project_path).join(&template.path);
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&output_path, &template.content)?;
            applied.push(template);
        }
        
        Ok(applied)
    }
}

impl Default for CiTemplateService {
    fn default() -> Self {
        Self::new()
    }
}