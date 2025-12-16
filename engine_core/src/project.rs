use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub last_modified: String,
    pub path: PathBuf,
    pub is_example: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub editor_startup_scene: Option<PathBuf>,  // Scene to load when opening project in Editor
    #[serde(default)]
    pub game_startup_scene: Option<PathBuf>,    // Scene to load when running exported game
    #[serde(default)]
    pub last_opened_scene: Option<PathBuf>,     // Last scene that was open (for auto-restore)
    // Legacy field for backward compatibility
    #[serde(default)]
    pub startup_scene: Option<PathBuf>,
}

pub struct ProjectManager {
    projects_dir: PathBuf,
    current_project: Option<ProjectMetadata>,
}

impl ProjectManager {
    pub fn new() -> Result<Self> {
        let projects_dir = PathBuf::from("./projects");
        if !projects_dir.exists() {
            fs::create_dir_all(&projects_dir)?;
        }

        Ok(Self {
            projects_dir,
            current_project: None,
        })
    }

    pub fn create_project(&mut self, name: &str, description: &str) -> Result<ProjectMetadata> {
        let project_path = self.projects_dir.join(name);

        if project_path.exists() {
            return Err(anyhow::anyhow!("Project '{}' already exists", name));
        }

        fs::create_dir_all(&project_path)?;

        let config = ProjectConfig {
            name: name.to_string(),
            description: description.to_string(),
            version: "0.1.0".to_string(),
            editor_startup_scene: None,
            game_startup_scene: None,
            last_opened_scene: None,
            startup_scene: None,
        };

        let config_path = project_path.join("project.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;

        // Create project directories
        fs::create_dir_all(project_path.join("assets"))?;
        fs::create_dir_all(project_path.join("scenes"))?;
        fs::create_dir_all(project_path.join("scripts"))?;

        let metadata = ProjectMetadata {
            name: name.to_string(),
            description: description.to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            last_modified: chrono::Local::now().to_rfc3339(),
            path: project_path,
            is_example: false,
        };

        self.current_project = Some(metadata.clone());
        Ok(metadata)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectMetadata>> {
        let mut projects = Vec::new();

        if !self.projects_dir.exists() {
            return Ok(projects);
        }

        for entry in fs::read_dir(&self.projects_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let config_path = path.join("project.json");
                if config_path.exists() {
                    let config_str = fs::read_to_string(&config_path)?;
                    let config: ProjectConfig = serde_json::from_str(&config_str)?;

                    let metadata_path = path.join("metadata.json");
                    let metadata = if metadata_path.exists() {
                        let metadata_str = fs::read_to_string(&metadata_path)?;
                        serde_json::from_str(&metadata_str)?
                    } else {
                        ProjectMetadata {
                            name: config.name.clone(),
                            description: config.description.clone(),
                            created_at: "Unknown".to_string(),
                            last_modified: "Unknown".to_string(),
                            path: path.clone(),
                            is_example: false,
                        }
                    };

                    projects.push(metadata);
                }
            }
        }

        Ok(projects)
    }

    pub fn open_project(&mut self, path: &Path) -> Result<ProjectMetadata> {
        let config_path = path.join("project.json");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Not a valid project directory"));
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_json::from_str(&config_str)?;

        let metadata = ProjectMetadata {
            name: config.name.clone(),
            description: config.description.clone(),
            created_at: "Unknown".to_string(),
            last_modified: chrono::Local::now().to_rfc3339(),
            path: path.to_path_buf(),
            is_example: false,
        };

        self.current_project = Some(metadata.clone());
        Ok(metadata)
    }

    pub fn delete_project(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("Project does not exist"));
        }

        fs::remove_dir_all(path)?;

        if let Some(current) = &self.current_project {
            if current.path == path {
                self.current_project = None;
            }
        }

        Ok(())
    }

    pub fn current_project(&self) -> Option<&ProjectMetadata> {
        self.current_project.as_ref()
    }

    pub fn close_project(&mut self) {
        self.current_project = None;
    }

    // Legacy method - uses editor_startup_scene or falls back to startup_scene
    pub fn get_startup_scene(&self, project_path: &Path) -> Result<Option<PathBuf>> {
        self.get_editor_startup_scene(project_path)
    }

    pub fn get_editor_startup_scene(&self, project_path: &Path) -> Result<Option<PathBuf>> {
        let config_path = project_path.join("project.json");
        if !config_path.exists() {
            return Ok(None);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_json::from_str(&config_str)?;

        // Try new field first, then fall back to legacy field
        Ok(config.editor_startup_scene.or(config.startup_scene))
    }

    pub fn set_editor_startup_scene(&self, project_path: &Path, scene_path: Option<PathBuf>) -> Result<()> {
        let config_path = project_path.join("project.json");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Project config not found"));
        }

        let config_str = fs::read_to_string(&config_path)?;
        let mut config: ProjectConfig = serde_json::from_str(&config_str)?;
        config.editor_startup_scene = scene_path;

        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    pub fn get_game_startup_scene(&self, project_path: &Path) -> Result<Option<PathBuf>> {
        let config_path = project_path.join("project.json");
        if !config_path.exists() {
            return Ok(None);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_json::from_str(&config_str)?;
        Ok(config.game_startup_scene)
    }

    pub fn set_game_startup_scene(&self, project_path: &Path, scene_path: Option<PathBuf>) -> Result<()> {
        let config_path = project_path.join("project.json");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Project config not found"));
        }

        let config_str = fs::read_to_string(&config_path)?;
        let mut config: ProjectConfig = serde_json::from_str(&config_str)?;
        config.game_startup_scene = scene_path;

        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    // Legacy method - sets editor_startup_scene
    pub fn set_startup_scene(&self, project_path: &Path, scene_path: Option<PathBuf>) -> Result<()> {
        self.set_editor_startup_scene(project_path, scene_path)
    }

    // Get last opened scene
    pub fn get_last_opened_scene(&self, project_path: &Path) -> Result<Option<PathBuf>> {
        let config_path = project_path.join("project.json");
        if !config_path.exists() {
            return Ok(None);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_json::from_str(&config_str)?;
        Ok(config.last_opened_scene)
    }

    // Set last opened scene
    pub fn set_last_opened_scene(&self, project_path: &Path, scene_path: Option<PathBuf>) -> Result<()> {
        let config_path = project_path.join("project.json");
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Project config not found"));
        }

        let config_str = fs::read_to_string(&config_path)?;
        let mut config: ProjectConfig = serde_json::from_str(&config_str)?;
        config.last_opened_scene = scene_path;

        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;
        Ok(())
    }

    pub fn get_example_projects() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Celeste Demo", "Platformer demo with Celeste-style movement (Run, Jump, Dash)"),
            ("FPS 3D Example", "First Person Shooter 3D Example"),
            ("Item Collection Game", "Simple 2D game with player movement and item collection"),
            ("Empty Project", "Start from scratch with an empty project"),
        ]
    }
}
