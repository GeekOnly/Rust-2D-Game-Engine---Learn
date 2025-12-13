use ecs::{World, Entity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Prefab - A reusable template for creating entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefab {
    /// Prefab name
    pub name: String,
    
    /// Root entity data
    pub root: PrefabEntity,
    
    /// Child entities (hierarchical)
    pub children: Vec<PrefabEntity>,
    
    /// Metadata
    pub metadata: PrefabMetadata,
}

/// Metadata about the prefab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabMetadata {
    /// Creation timestamp
    pub created_at: String,
    
    /// Last modified timestamp
    pub modified_at: String,
    
    /// Prefab version
    pub version: u32,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Serialized entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabEntity {
    /// Entity name
    pub name: String,
    
    /// Transform component (always present)
    pub transform: ecs::Transform,
    
    /// Optional components
    pub sprite: Option<ecs::Sprite>,
    pub camera: Option<ecs::Camera>,
    pub mesh: Option<ecs::Mesh>,
    pub collider: Option<ecs::Collider>,
    pub rigidbody: Option<ecs::Rigidbody2D>,
    pub tilemap: Option<ecs::Tilemap>,
    pub tilemap_renderer: Option<ecs::TilemapRenderer>,
    pub tileset: Option<ecs::TileSet>,
    pub grid: Option<ecs::Grid>,
    pub script: Option<ecs::Script>,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Layer
    pub layer: i32,
    
    /// Active state
    pub active: bool,
    
    /// Child entities
    pub children: Vec<PrefabEntity>,
}

impl Prefab {
    /// Create a new prefab from an entity
    pub fn from_entity(
        entity: Entity,
        world: &World,
        entity_names: &HashMap<Entity, String>,
        name: String,
    ) -> Result<Self, String> {
        let root = Self::serialize_entity(entity, world, entity_names)?;
        
        let metadata = PrefabMetadata {
            created_at: chrono::Local::now().to_rfc3339(),
            modified_at: chrono::Local::now().to_rfc3339(),
            version: 1,
            tags: Vec::new(),
        };
        
        Ok(Self {
            name,
            root,
            children: Vec::new(),
            metadata,
        })
    }
    
    /// Serialize an entity and its children
    fn serialize_entity(
        entity: Entity,
        world: &World,
        entity_names: &HashMap<Entity, String>,
    ) -> Result<PrefabEntity, String> {
        let name = entity_names.get(&entity)
            .cloned()
            .unwrap_or_else(|| format!("Entity {}", entity));
        
        let transform = world.transforms.get(&entity)
            .cloned()
            .unwrap_or_default();
        
        let sprite = world.sprites.get(&entity).cloned();
        let camera = world.cameras.get(&entity).cloned();
        let mesh = world.meshes.get(&entity).cloned();
        let collider = world.colliders.get(&entity).cloned();
        let rigidbody = world.rigidbodies.get(&entity).cloned();
        let tilemap = world.tilemaps.get(&entity).cloned();
        let tilemap_renderer = world.tilemap_renderers.get(&entity).cloned();
        let tileset = world.tilesets.get(&entity).cloned();
        let grid = world.grids.get(&entity).cloned();
        let script = world.scripts.get(&entity).cloned();
        
        let tags = world.tags.get(&entity)
            .map(|tag| vec![format!("{:?}", tag)])
            .unwrap_or_default();
        
        let layer = world.layers.get(&entity)
            .copied()
            .unwrap_or(0) as i32;
        
        let active = world.active.get(&entity)
            .copied()
            .unwrap_or(true);
        
        // Serialize children recursively
        let mut children = Vec::new();
        for &child in world.get_children(entity) {
            children.push(Self::serialize_entity(child, world, entity_names)?);
        }
        
        Ok(PrefabEntity {
            name,
            transform,
            sprite,
            camera,
            mesh,
            collider,
            rigidbody,
            tilemap,
            tilemap_renderer,
            tileset,
            grid,
            script,
            tags,
            layer,
            active,
            children,
        })
    }
    
    /// Instantiate prefab into the world
    pub fn instantiate(
        &self,
        world: &mut World,
        entity_names: &mut HashMap<Entity, String>,
        parent: Option<Entity>,
    ) -> Result<Entity, String> {
        self.instantiate_entity(&self.root, world, entity_names, parent)
    }
    
    /// Instantiate a prefab entity
    fn instantiate_entity(
        &self,
        prefab_entity: &PrefabEntity,
        world: &mut World,
        entity_names: &mut HashMap<Entity, String>,
        parent: Option<Entity>,
    ) -> Result<Entity, String> {
        let entity = world.spawn();
        
        // Set name
        entity_names.insert(entity, prefab_entity.name.clone());
        world.names.insert(entity, prefab_entity.name.clone());
        
        // Set parent
        if let Some(parent_entity) = parent {
            world.set_parent(entity, Some(parent_entity));
        }
        
        // Add components
        world.transforms.insert(entity, prefab_entity.transform.clone());
        
        if let Some(sprite) = &prefab_entity.sprite {
            world.sprites.insert(entity, sprite.clone());
        }
        
        if let Some(camera) = &prefab_entity.camera {
            world.cameras.insert(entity, camera.clone());
        }
        
        if let Some(mesh) = &prefab_entity.mesh {
            world.meshes.insert(entity, mesh.clone());
        }
        
        if let Some(collider) = &prefab_entity.collider {
            world.colliders.insert(entity, collider.clone());
        }
        
        if let Some(rigidbody) = &prefab_entity.rigidbody {
            world.rigidbodies.insert(entity, rigidbody.clone());
        }
        
        if let Some(tilemap) = &prefab_entity.tilemap {
            world.tilemaps.insert(entity, tilemap.clone());
        }
        
        if let Some(tilemap_renderer) = &prefab_entity.tilemap_renderer {
            world.tilemap_renderers.insert(entity, tilemap_renderer.clone());
        }
        
        if let Some(tileset) = &prefab_entity.tileset {
            world.tilesets.insert(entity, tileset.clone());
        }
        
        if let Some(grid) = &prefab_entity.grid {
            world.grids.insert(entity, grid.clone());
        }
        
        if let Some(script) = &prefab_entity.script {
            world.scripts.insert(entity, script.clone());
        }
        
        // Convert tags back to EntityTag enum (for now, just use first tag or default to Player)
        if let Some(first_tag) = prefab_entity.tags.first() {
            let entity_tag = match first_tag.as_str() {
                "Player" => ecs::EntityTag::Player,
                "Item" => ecs::EntityTag::Item,
                _ => ecs::EntityTag::Player, // Default fallback
            };
            world.tags.insert(entity, entity_tag);
        }
        world.layers.insert(entity, prefab_entity.layer as u8);
        world.active.insert(entity, prefab_entity.active);
        
        // Instantiate children
        for child_prefab in &prefab_entity.children {
            self.instantiate_entity(child_prefab, world, entity_names, Some(entity))?;
        }
        
        Ok(entity)
    }
    
    /// Save prefab to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize prefab: {}", e))?;
        
        std::fs::write(path.as_ref(), json)
            .map_err(|e| format!("Failed to write prefab file: {}", e))?;
        
        log::info!("Saved prefab: {:?}", path.as_ref());
        Ok(())
    }
    
    /// Load prefab from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let json = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read prefab file: {}", e))?;
        
        let prefab: Prefab = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize prefab: {}", e))?;
        
        log::info!("Loaded prefab: {:?}", path.as_ref());
        Ok(prefab)
    }
}

/// Prefab Manager - Manages all prefabs in the project
pub struct PrefabManager {
    /// Loaded prefabs (path -> prefab)
    pub prefabs: HashMap<PathBuf, Prefab>,
    
    /// Available prefab files
    pub available_files: Vec<PathBuf>,
    
    /// Project path
    pub project_path: Option<PathBuf>,
    
    /// Selected prefab
    pub selected_prefab: Option<PathBuf>,
}

impl PrefabManager {
    pub fn new() -> Self {
        Self {
            prefabs: HashMap::new(),
            available_files: Vec::new(),
            project_path: None,
            selected_prefab: None,
        }
    }
    
    /// Set project path and scan for prefabs
    pub fn set_project_path(&mut self, path: PathBuf) {
        self.project_path = Some(path);
        self.scan_prefabs();
    }
    
    /// Scan project directory for .prefab files
    pub fn scan_prefabs(&mut self) {
        self.available_files.clear();
        
        if let Some(project_path) = &self.project_path {
            // Scan prefabs folder
            let prefabs_dir = project_path.join("prefabs");
            if prefabs_dir.exists() {
                self.scan_directory(&prefabs_dir);
            }
        }
        
        log::info!("Found {} prefab files", self.available_files.len());
    }
    
    /// Recursively scan directory for .prefab files
    fn scan_directory(&mut self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    self.scan_directory(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("prefab") {
                    self.available_files.push(path);
                }
            }
        }
    }
    
    /// Create prefab from entity
    pub fn create_prefab(
        &mut self,
        entity: Entity,
        world: &World,
        entity_names: &HashMap<Entity, String>,
        name: String,
    ) -> Result<PathBuf, String> {
        let prefab = Prefab::from_entity(entity, world, entity_names, name.clone())?;
        
        // Ensure prefabs directory exists
        let project_path = self.project_path.as_ref()
            .ok_or("No project path set")?;
        
        let prefabs_dir = project_path.join("prefabs");
        std::fs::create_dir_all(&prefabs_dir)
            .map_err(|e| format!("Failed to create prefabs directory: {}", e))?;
        
        // Generate file path
        let file_name = format!("{}.prefab", name.replace(" ", "_"));
        let file_path = prefabs_dir.join(file_name);
        
        // Save prefab
        prefab.save(&file_path)?;
        
        // Add to manager
        self.prefabs.insert(file_path.clone(), prefab);
        self.available_files.push(file_path.clone());
        
        Ok(file_path)
    }
    
    /// Load a prefab file
    pub fn load_prefab(&mut self, path: &PathBuf) -> Result<(), String> {
        let prefab = Prefab::load(path)?;
        self.prefabs.insert(path.clone(), prefab);
        Ok(())
    }
    
    /// Instantiate a prefab
    pub fn instantiate_prefab(
        &self,
        path: &PathBuf,
        world: &mut World,
        entity_names: &mut HashMap<Entity, String>,
        parent: Option<Entity>,
    ) -> Result<Entity, String> {
        let prefab = self.prefabs.get(path)
            .ok_or("Prefab not loaded")?;
        
        prefab.instantiate(world, entity_names, parent)
    }
    
    /// Delete a prefab file
    pub fn delete_prefab(&mut self, path: &PathBuf) -> Result<(), String> {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to delete prefab: {}", e))?;
        
        self.prefabs.remove(path);
        self.available_files.retain(|p| p != path);
        
        if self.selected_prefab.as_ref() == Some(path) {
            self.selected_prefab = None;
        }
        
        Ok(())
    }
}

impl Default for PrefabManager {
    fn default() -> Self {
        Self::new()
    }
}
