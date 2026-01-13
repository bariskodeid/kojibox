use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(rename = "schemaVersion")]
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub path: String,
    pub domain: String,
    pub stack: String,
    #[serde(default)]
    pub overrides: HashMap<String, String>,
}

fn default_schema_version() -> u32 {
    0
}

#[derive(Debug)]
pub struct ProjectStore {
    root: PathBuf,
}

impl ProjectStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn list(&self) -> Result<Vec<ProjectConfig>, String> {
        let dir = self.projects_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut projects = Vec::new();
        let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path().join("config.json");
            if !path.exists() {
                continue;
            }
            let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let project: ProjectConfig = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            let project = self.migrate_project(project)?;
            projects.push(project);
        }
        Ok(projects)
    }

    pub fn save(&self, project: &ProjectConfig) -> Result<(), String> {
        let dir = self.projects_dir().join(&project.id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let raw = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
        fs::write(dir.join("config.json"), raw).map_err(|e| e.to_string())
    }

    pub fn save_raw(&self, raw: &Value) -> Result<(), String> {
        crate::schema::validate_project_config(raw)?;
        let project: ProjectConfig =
            serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
        self.validate_project_path(&project.path)?;
        self.save(&project)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let dir = self.projects_dir().join(id);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn validate_project_path(&self, path: &str) -> Result<(), String> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err("project path not found".to_string());
        }
        if !path.is_dir() {
            return Err("project path is not a directory".to_string());
        }
        Ok(())
    }

    fn migrate_project(&self, mut project: ProjectConfig) -> Result<ProjectConfig, String> {
        if project.schema_version == 0 {
            project.schema_version = 1;
            self.save(&project)?;
        }
        if project.overrides.is_empty() && project.schema_version == 1 {
            self.save(&project)?;
        }
        Ok(project)
    }

    fn projects_dir(&self) -> PathBuf {
        self.root.join("app/projects")
    }
}
