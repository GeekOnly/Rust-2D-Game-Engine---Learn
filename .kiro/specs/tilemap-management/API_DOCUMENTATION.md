# Tilemap Management API Documentation

## Overview

This document provides technical documentation for extending and integrating with the Tilemap Management System.

## Core Components

### MapManager

The central manager for all tilemap operations.

```rust
pub struct MapManager {
    pub loaded_maps: HashMap<PathBuf, LoadedMap>,
    pub available_files: Vec<PathBuf>,
    pub selected_map: Option<PathBuf>,
    pub project_path: Option<PathBuf>,
    pub hot_reload_watcher: Option<HotReloadWatcher>,
    pub hot_reload_enabled: bool,
    pub collision_value: i64,
    pub auto_regenerate_colliders: bool,
    pub settings: TilemapSettings,
}
```

#### Key Methods

**Initialization**:
```rust
// Create new MapManager
let mut map_manager = MapManager::new();

// Set project path and scan for files
map_manager.set_project_path(project_path);
```

**Map Operations**:
```rust
// Load a map with auto-generated colliders
map_manager.load_map(&path, &mut world)?;

// Reload a map (preserves Grid Entity ID)
map_manager.reload_map(&path, &mut world)?;

// Unload a map (despawns Grid and all children)
map_manager.unload_map(&path, &mut world);
```

**Collider Operations**:
```rust
// Regenerate colliders for a map
let count = map_manager.regenerate_colliders(&path, &mut world)?;

// Clean up colliders for specific map
let removed = map_manager.clean_up_colliders(&path, &mut world);

// Clean up all colliders
let total_removed = map_manager.clean_up_all_colliders(&mut world);
```

**Layer Operations**:
```rust
// Toggle layer visibility
map_manager.toggle_layer_visibility(entity, &mut world);

// Check if entity belongs to a map
let is_map_entity = map_manager.is_map_entity(entity);

// Get map by entity
let map = map_manager.get_map_by_entity(entity);
```

**Hot-Reload**:
```rust
// Enable hot-reload
map_manager.enable_hot_reload()?;

// Disable hot-reload
map_manager.disable_hot_reload();

// Process hot-reload events (call each frame)
let reloaded_files = map_manager.process_hot_reload(&mut world);
```

**Settings**:
```rust
// Set collision value
map_manager.set_collision_value(1);

// Set auto-regenerate flag
map_manager.set_auto_regenerate_colliders(true);

// Save settings to project
map_manager.save_settings()?;

// Load settings from project
map_manager.load_settings();
```

### LoadedMap

Information about a loaded map.

```rust
pub struct LoadedMap {
    pub grid_entity: Entity,
    pub layer_entities: Vec<LayerInfo>,
    pub collider_entities: Vec<Entity>,
    pub file_path: PathBuf,
    pub last_modified: SystemTime,
}
```

### LayerInfo

Information about a tilemap layer.

```rust
pub struct LayerInfo {
    pub entity: Entity,
    pub name: String,
    pub size: (u32, u32),
    pub visible: bool,
    pub z_order: i32,
}
```

### LdtkLoader

Low-level loader for LDtk files.

```rust
pub struct LdtkLoader;

impl LdtkLoader {
    // Load with Grid and auto-generated colliders
    pub fn load_project_with_grid_and_colliders(
        path: impl AsRef<Path>,
        world: &mut World,
        auto_generate_colliders: bool,
        collision_value: i64,
    ) -> Result<(Entity, Vec<Entity>, Vec<Entity>), String>;
    
    // Load with Grid only
    pub fn load_project_with_grid(
        path: impl AsRef<Path>,
        world: &mut World,
    ) -> Result<(Entity, Vec<Entity>), String>;
    
    // Generate composite colliders
    pub fn generate_composite_colliders_from_intgrid(
        path: impl AsRef<Path>,
        world: &mut World,
        collision_value: i64,
    ) -> Result<Vec<Entity>, String>;
}
```

## UI Panels

### Maps Panel

```rust
// Render as standalone window
render_maps_panel(ctx, map_manager, world, &mut open);

// Render content for docking system
render_maps_panel_content(ui, map_manager, world);
```

### Layer Properties Panel

```rust
pub struct LayerPropertiesPanel {
    pub selected_layer: Option<Entity>,
}

impl LayerPropertiesPanel {
    pub fn new() -> Self;
    
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        world: &mut World,
        map_manager: &mut MapManager,
        open: &mut bool,
    );
    
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        map_manager: &mut MapManager,
    );
}
```

### Layer Ordering Panel

```rust
pub struct LayerOrderingPanel {
    pub selected_map: Option<PathBuf>,
}

impl LayerOrderingPanel {
    pub fn new() -> Self;
    
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        world: &mut World,
        map_manager: &mut MapManager,
        open: &mut bool,
    );
    
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        map_manager: &mut MapManager,
    );
}
```

### Performance Panel

```rust
pub struct PerformancePanel {
    pub thresholds: PerformanceThresholds,
}

impl PerformancePanel {
    pub fn new() -> Self;
    
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        world: &World,
        map_manager: &MapManager,
        open: &mut bool,
    );
    
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        world: &World,
        map_manager: &MapManager,
    );
}
```

### Collider Settings Panel

```rust
pub struct ColliderSettingsPanel {
    pub configuration: ColliderConfiguration,
    pub has_changes: bool,
}

impl ColliderSettingsPanel {
    pub fn new() -> Self;
    
    pub fn with_configuration(configuration: ColliderConfiguration) -> Self;
    
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        map_manager: &mut MapManager,
        open: &mut bool,
    );
    
    pub fn render_content(
        &mut self,
        ui: &mut egui::Ui,
        map_manager: &mut MapManager,
    );
    
    pub fn get_configuration(&self) -> &ColliderConfiguration;
    pub fn set_configuration(&mut self, configuration: ColliderConfiguration);
}
```

## Error Handling

### TilemapError

```rust
pub enum TilemapError {
    FileNotFound(PathBuf),
    InvalidFormat(String),
    ValidationError(String),
    EntityNotFound(Entity),
    ComponentMissing(String),
    IoError(String),
    JsonError(String),
    ColliderGenerationFailed(String),
}

impl TilemapError {
    pub fn display_message(&self) -> String;
    pub fn log_error(&self);
    pub fn log_error_with_context(&self, context: &str);
}
```

### Error Handling Pattern

```rust
match map_manager.load_map(&path, &mut world) {
    Ok(()) => {
        console.info(format!("Loaded map: {:?}", path));
    }
    Err(e) => {
        console.error(format!("Failed to load map: {}", e.display_message()));
        e.log_error();
    }
}
```

## Settings and Configuration

### TilemapSettings

```rust
pub struct TilemapSettings {
    pub auto_generate_colliders: bool,
    pub collision_value: i64,
    pub collider_type: ColliderType,
    pub hot_reload_enabled: bool,
    pub pixels_per_unit: f32,
}

impl TilemapSettings {
    pub fn load(project_path: &Path) -> Self;
    pub fn save(&self, project_path: &Path) -> Result<(), String>;
}
```

Settings are stored in `.kiro/settings/tilemap.json`.

### ColliderConfiguration

```rust
pub struct ColliderConfiguration {
    pub collider_type: ColliderType,
    pub collision_value: i64,
    pub auto_regenerate: bool,
}

pub enum ColliderType {
    Composite,
    Individual,
    Polygon,
}
```

## Integration Examples

### Example 1: Loading a Map Programmatically

```rust
use crate::editor::map_manager::MapManager;
use ecs::World;
use std::path::PathBuf;

fn load_map_example() {
    let mut world = World::new();
    let mut map_manager = MapManager::new();
    
    // Set project path
    let project_path = PathBuf::from("path/to/project");
    map_manager.set_project_path(project_path);
    
    // Load a specific map
    let map_path = PathBuf::from("path/to/project/levels/Level_01.ldtk");
    match map_manager.load_map(&map_path, &mut world) {
        Ok(()) => println!("Map loaded successfully"),
        Err(e) => eprintln!("Failed to load map: {}", e.display_message()),
    }
}
```

### Example 2: Custom Collider Generation

```rust
use ecs::{World, Entity, Collider, Transform};
use std::path::Path;

fn custom_collider_generation(
    ldtk_path: &Path,
    world: &mut World,
) -> Result<Vec<Entity>, String> {
    // Load the LDtk file
    let project_data = std::fs::read_to_string(ldtk_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let project: serde_json::Value = serde_json::from_str(&project_data)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let mut colliders = Vec::new();
    
    // Custom logic to generate colliders
    // ... your implementation here ...
    
    Ok(colliders)
}
```

### Example 3: Listening to Hot-Reload Events

```rust
fn update_loop(
    map_manager: &mut MapManager,
    world: &mut World,
    console: &mut Console,
) {
    // Process hot-reload events each frame
    let reloaded_files = map_manager.process_hot_reload(world);
    
    for file in reloaded_files {
        console.info(format!("Hot-reloaded: {:?}", file));
    }
    
    // Check for errors
    if let Some(error) = map_manager.get_last_hot_reload_error() {
        console.error(format!("Hot-reload error: {}", error));
        map_manager.clear_hot_reload_error();
    }
}
```

### Example 4: Custom Layer Property Editor

```rust
use egui;
use ecs::{World, Entity};

fn custom_layer_editor(
    ui: &mut egui::Ui,
    layer_entity: Entity,
    world: &mut World,
) {
    ui.heading("Custom Layer Properties");
    
    // Get layer components
    if let Some(transform) = world.transforms.get_mut(&layer_entity) {
        ui.horizontal(|ui| {
            ui.label("Custom Property:");
            // Add your custom UI here
        });
    }
}
```

## Performance Considerations

### Best Practices

1. **Batch Operations**: Use batch operations when modifying multiple entities
2. **Lazy Loading**: Only load maps when needed
3. **Composite Colliders**: Always use composite colliders for better performance
4. **Memory Management**: Unload unused maps to free memory
5. **Hot-Reload**: Enable hot-reload for development, disable for production

### Performance Metrics

- Map loading: <1 second for 100x100 tiles
- Collider generation: <500ms for 1000 tiles
- UI responsiveness: <100ms for all actions
- Hot-reload detection: <1 second
- Frame rate: 60 FPS with 10 loaded maps

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_map_loading() {
        let mut world = World::new();
        let mut map_manager = MapManager::new();
        
        // Test map loading
        // ... your test implementation ...
    }
}
```

### Property-Based Testing

See `tests/tilemap_management_properties.rs` for property-based tests using QuickCheck.

## Extending the System

### Adding Custom Collider Types

1. Add new variant to `ColliderType` enum
2. Implement generation logic in `LdtkLoader`
3. Update UI in `ColliderSettingsPanel`

### Adding Custom Layer Properties

1. Add new component to ECS
2. Update `LayerPropertiesPanel` to edit the component
3. Ensure serialization/deserialization works

### Adding Custom Panels

1. Create new panel struct
2. Implement `render_window` and `render_content` methods
3. Add to `EditorTab` enum in `dock_layout.rs`
4. Update `TabViewer` to render the panel

## Troubleshooting

### Common Issues

**Issue**: Colliders not generating
- Check collision value matches IntGrid values in LDtk
- Verify IntGrid layer has tiles
- Check console for error messages

**Issue**: Hot-reload not working
- Verify hot-reload is enabled
- Check file watcher is active
- Ensure file path is correct

**Issue**: Performance degradation
- Check Performance Panel for metrics
- Reduce number of loaded maps
- Use composite colliders
- Unload unused maps

## Conclusion

The Tilemap Management System provides a complete API for working with LDtk maps. The system is designed to be extensible and can be customized to fit your specific needs.

For more information, see:
- `USER_GUIDE.md` - User-facing documentation
- `PERFORMANCE_OPTIMIZATION.md` - Performance details
- Source code in `engine/src/editor/`
