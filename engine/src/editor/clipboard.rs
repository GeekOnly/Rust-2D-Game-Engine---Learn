//! Clipboard System
//!
//! Handles copy/paste/duplicate operations for entities.
//! Supports:
//! - Copy (Ctrl+C)
//! - Paste (Ctrl+V)
//! - Duplicate (Ctrl+D)
//! - Multi-entity operations
//! - Component preservation
//! - Hierarchy preservation

use ecs::{World, Entity};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Clipboard data structure
#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub entities: Vec<EntityClipboardData>,
    pub hierarchy: Vec<(usize, usize)>, // (child_index, parent_index)
}

/// Entity data for clipboard
#[derive(Clone, Serialize, Deserialize)]
pub struct EntityClipboardData {
    pub name: String,
    pub transform: Option<ecs::Transform>,
    pub sprite: Option<ecs::Sprite>,
    pub collider: Option<ecs::Collider>,
    pub camera: Option<ecs::Camera>,
    pub mesh: Option<ecs::Mesh>,
    pub velocity: Option<(f32, f32)>,
    pub tag: Option<ecs::EntityTag>,
    pub script: Option<ecs::Script>,
    pub active: bool,
    pub layer: u8,
}

impl EntityClipboardData {
    /// Capture entity data from world
    pub fn from_world(entity: Entity, world: &World, entity_names: &HashMap<Entity, String>) -> Self {
        Self {
            name: entity_names.get(&entity).cloned().unwrap_or_else(|| format!("Entity {}", entity)),
            transform: world.transforms.get(&entity).cloned(),
            sprite: world.sprites.get(&entity).cloned(),
            collider: world.colliders.get(&entity).cloned(),
            camera: world.cameras.get(&entity).cloned(),
            mesh: world.meshes.get(&entity).cloned(),
            velocity: world.velocities.get(&entity).copied(),
            tag: world.tags.get(&entity).cloned(),
            script: world.scripts.get(&entity).cloned(),
            active: world.active.get(&entity).copied().unwrap_or(true),
            layer: world.layers.get(&entity).copied().unwrap_or(0),
        }
    }
    
    /// Create new entity in world with this data
    pub fn create_in_world(
        &self,
        world: &mut World,
        entity_names: &mut HashMap<Entity, String>,
        name_suffix: Option<&str>,
    ) -> Entity {
        let entity = world.spawn();
        
        // Generate unique name
        let name = if let Some(suffix) = name_suffix {
            format!("{} {}", self.name, suffix)
        } else {
            self.name.clone()
        };
        
        entity_names.insert(entity, name.clone());
        world.names.insert(entity, name);
        
        // Copy components
        if let Some(transform) = &self.transform {
            world.transforms.insert(entity, transform.clone());
        }
        if let Some(sprite) = &self.sprite {
            world.sprites.insert(entity, sprite.clone());
        }
        if let Some(collider) = &self.collider {
            world.colliders.insert(entity, collider.clone());
        }
        if let Some(camera) = &self.camera {
            world.cameras.insert(entity, camera.clone());
        }
        if let Some(mesh) = &self.mesh {
            world.meshes.insert(entity, mesh.clone());
        }
        if let Some(velocity) = self.velocity {
            world.velocities.insert(entity, velocity);
        }
        if let Some(tag) = &self.tag {
            world.tags.insert(entity, tag.clone());
        }
        if let Some(script) = &self.script {
            world.scripts.insert(entity, script.clone());
        }
        world.active.insert(entity, self.active);
        world.layers.insert(entity, self.layer);
        
        entity
    }
}

/// Clipboard manager
pub struct Clipboard {
    data: Option<ClipboardData>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self { data: None }
    }
    
    // ========================================================================
    // COPY OPERATIONS
    // ========================================================================
    
    /// Copy single entity
    pub fn copy_entity(
        &mut self,
        entity: Entity,
        world: &World,
        entity_names: &HashMap<Entity, String>,
    ) {
        let entity_data = EntityClipboardData::from_world(entity, world, entity_names);
        
        self.data = Some(ClipboardData {
            entities: vec![entity_data],
            hierarchy: Vec::new(),
        });
    }
    
    /// Copy multiple entities
    pub fn copy_entities(
        &mut self,
        entities: &[Entity],
        world: &World,
        entity_names: &HashMap<Entity, String>,
    ) {
        if entities.is_empty() {
            return;
        }
        
        let mut entity_data = Vec::new();
        let mut hierarchy = Vec::new();
        let mut entity_to_index = HashMap::new();
        
        // Capture entity data
        for (index, &entity) in entities.iter().enumerate() {
            entity_data.push(EntityClipboardData::from_world(entity, world, entity_names));
            entity_to_index.insert(entity, index);
        }
        
        // Capture hierarchy (only if both parent and child are in selection)
        for (index, &entity) in entities.iter().enumerate() {
            if let Some(&parent) = world.parents.get(&entity) {
                if let Some(&parent_index) = entity_to_index.get(&parent) {
                    hierarchy.push((index, parent_index));
                }
            }
        }
        
        self.data = Some(ClipboardData {
            entities: entity_data,
            hierarchy,
        });
    }
    
    // ========================================================================
    // PASTE OPERATIONS
    // ========================================================================
    
    /// Paste entities at offset
    pub fn paste(
        &self,
        world: &mut World,
        entity_names: &mut HashMap<Entity, String>,
        offset: Option<[f32; 3]>,
    ) -> Vec<Entity> {
        if let Some(data) = &self.data {
            self.paste_data(data, world, entity_names, offset, Some("(Copy)"))
        } else {
            Vec::new()
        }
    }
    
    /// Paste clipboard data
    fn paste_data(
        &self,
        data: &ClipboardData,
        world: &mut World,
        entity_names: &mut HashMap<Entity, String>,
        offset: Option<[f32; 3]>,
        name_suffix: Option<&str>,
    ) -> Vec<Entity> {
        let mut new_entities = Vec::new();
        
        // Create entities
        for entity_data in &data.entities {
            let mut entity_data = entity_data.clone();
            
            // Apply offset to transform
            if let Some(offset) = offset {
                if let Some(transform) = &mut entity_data.transform {
                    transform.position[0] += offset[0];
                    transform.position[1] += offset[1];
                    transform.position[2] += offset[2];
                }
            }
            
            let new_entity = entity_data.create_in_world(world, entity_names, name_suffix);
            new_entities.push(new_entity);
        }
        
        // Restore hierarchy
        for &(child_index, parent_index) in &data.hierarchy {
            if child_index < new_entities.len() && parent_index < new_entities.len() {
                let child = new_entities[child_index];
                let parent = new_entities[parent_index];
                
                world.parents.insert(child, parent);
                world.children.entry(parent).or_default().push(child);
            }
        }
        
        new_entities
    }
    
    // ========================================================================
    // DUPLICATE OPERATIONS
    // ========================================================================
    
    /// Duplicate single entity
    pub fn duplicate_entity(
        &self,
        entity: Entity,
        world: &mut World,
        entity_names: &HashMap<Entity, String>,
    ) -> Option<Entity> {
        let entity_data = EntityClipboardData::from_world(entity, world, entity_names);
        
        let data = ClipboardData {
            entities: vec![entity_data],
            hierarchy: Vec::new(),
        };
        
        let new_entities = self.paste_data(&data, world, entity_names, Some([10.0, 10.0, 0.0]), Some("(Copy)"));
        new_entities.first().copied()
    }
    
    /// Duplicate multiple entities
    pub fn duplicate_entities(
        &self,
        entities: &[Entity],
        world: &mut World,
        entity_names: &HashMap<Entity, String>,
    ) -> Vec<Entity> {
        if entities.is_empty() {
            return Vec::new();
        }
        
        let mut entity_data = Vec::new();
        let mut hierarchy = Vec::new();
        let mut entity_to_index = HashMap::new();
        
        // Capture entity data
        for (index, &entity) in entities.iter().enumerate() {
            entity_data.push(EntityClipboardData::from_world(entity, world, entity_names));
            entity_to_index.insert(entity, index);
        }
        
        // Capture hierarchy
        for (index, &entity) in entities.iter().enumerate() {
            if let Some(&parent) = world.parents.get(&entity) {
                if let Some(&parent_index) = entity_to_index.get(&parent) {
                    hierarchy.push((index, parent_index));
                }
            }
        }
        
        let data = ClipboardData {
            entities: entity_data,
            hierarchy,
        };
        
        self.paste_data(&data, world, entity_names, Some([10.0, 10.0, 0.0]), Some("(Copy)"))
    }
    
    // ========================================================================
    // QUERY OPERATIONS
    // ========================================================================
    
    /// Check if clipboard has data
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
    
    /// Get number of entities in clipboard
    pub fn count(&self) -> usize {
        self.data.as_ref().map(|d| d.entities.len()).unwrap_or(0)
    }
    
    /// Clear clipboard
    pub fn clear(&mut self) {
        self.data = None;
    }
    
    // ========================================================================
    // SERIALIZATION (for system clipboard)
    // ========================================================================
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Option<String> {
        self.data.as_ref().and_then(|data| {
            serde_json::to_string(data).ok()
        })
    }
    
    /// Deserialize from JSON
    pub fn from_json(&mut self, json: &str) -> Result<(), String> {
        match serde_json::from_str::<ClipboardData>(json) {
            Ok(data) => {
                self.data = Some(data);
                Ok(())
            }
            Err(e) => Err(format!("Failed to parse clipboard data: {}", e)),
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Copy selected entities to clipboard
pub fn copy_selected(
    clipboard: &mut Clipboard,
    selected: &[Entity],
    world: &World,
    entity_names: &HashMap<Entity, String>,
) {
    if selected.is_empty() {
        return;
    }
    
    if selected.len() == 1 {
        clipboard.copy_entity(selected[0], world, entity_names);
    } else {
        clipboard.copy_entities(selected, world, entity_names);
    }
}

/// Paste from clipboard
pub fn paste_from_clipboard(
    clipboard: &Clipboard,
    world: &mut World,
    entity_names: &mut HashMap<Entity, String>,
    offset: Option<[f32; 3]>,
) -> Vec<Entity> {
    clipboard.paste(world, entity_names, offset)
}

/// Duplicate selected entities
pub fn duplicate_selected(
    clipboard: &Clipboard,
    selected: &[Entity],
    world: &mut World,
    entity_names: &HashMap<Entity, String>,
) -> Vec<Entity> {
    if selected.is_empty() {
        return Vec::new();
    }
    
    if selected.len() == 1 {
        if let Some(entity) = clipboard.duplicate_entity(selected[0], world, entity_names) {
            vec![entity]
        } else {
            Vec::new()
        }
    } else {
        clipboard.duplicate_entities(selected, world, entity_names)
    }
}

// ============================================================================
// KEYBOARD SHORTCUTS HANDLER
// ============================================================================

/// Handle clipboard keyboard shortcuts
pub fn handle_clipboard_shortcuts(
    ctx: &egui::Context,
    clipboard: &mut Clipboard,
    selected: &[Entity],
    world: &mut World,
    entity_names: &mut HashMap<Entity, String>,
    undo_stack: &mut crate::editor::UndoStack,
) -> ClipboardAction {
    ctx.input(|i| {
        // Ctrl+C: Copy
        if i.modifiers.ctrl && i.key_pressed(egui::Key::C) {
            if !selected.is_empty() {
                copy_selected(clipboard, selected, world, entity_names);
                return ClipboardAction::Copied(selected.len());
            }
        }
        
        // Ctrl+V: Paste
        if i.modifiers.ctrl && i.key_pressed(egui::Key::V) {
            if clipboard.has_data() {
                let new_entities = paste_from_clipboard(clipboard, world, entity_names, Some([10.0, 10.0, 0.0]));
                
                if !new_entities.is_empty() {
                    // Create undo command
                    let mut batch = crate::editor::BatchCommand::new("Paste");
                    for &entity in &new_entities {
                        batch.add(Box::new(crate::editor::CreateEntityCommand::new(entity, world, entity_names)));
                    }
                    undo_stack.execute(Box::new(batch), world, entity_names);
                    
                    return ClipboardAction::Pasted(new_entities);
                }
            }
        }
        
        // Ctrl+D: Duplicate
        if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
            if !selected.is_empty() {
                let new_entities = duplicate_selected(clipboard, selected, world, entity_names);
                
                if !new_entities.is_empty() {
                    // Create undo command
                    let mut batch = crate::editor::BatchCommand::new("Duplicate");
                    for &entity in &new_entities {
                        batch.add(Box::new(crate::editor::CreateEntityCommand::new(entity, world, entity_names)));
                    }
                    undo_stack.execute(Box::new(batch), world, entity_names);
                    
                    return ClipboardAction::Duplicated(new_entities);
                }
            }
        }
        
        // Ctrl+X: Cut (Copy + Delete)
        if i.modifiers.ctrl && i.key_pressed(egui::Key::X) {
            if !selected.is_empty() {
                // Copy first
                copy_selected(clipboard, selected, world, entity_names);
                
                // Then delete with undo
                let mut batch = crate::editor::BatchCommand::new("Cut");
                for &entity in selected {
                    batch.add(Box::new(crate::editor::DeleteEntityCommand::new(entity, world, entity_names)));
                }
                undo_stack.execute(Box::new(batch), world, entity_names);
                
                return ClipboardAction::Cut(selected.len());
            }
        }
        
        ClipboardAction::None
    })
}

/// Clipboard action result
#[derive(Debug, Clone)]
pub enum ClipboardAction {
    None,
    Copied(usize),              // Number of entities copied
    Pasted(Vec<Entity>),        // New entities created
    Duplicated(Vec<Entity>),    // New entities created
    Cut(usize),                 // Number of entities cut
}

impl ClipboardAction {
    /// Get user-friendly message
    pub fn message(&self) -> Option<String> {
        match self {
            ClipboardAction::None => None,
            ClipboardAction::Copied(count) => Some(format!("Copied {} entity(ies)", count)),
            ClipboardAction::Pasted(entities) => Some(format!("Pasted {} entity(ies)", entities.len())),
            ClipboardAction::Duplicated(entities) => Some(format!("Duplicated {} entity(ies)", entities.len())),
            ClipboardAction::Cut(count) => Some(format!("Cut {} entity(ies)", count)),
        }
    }
    
    /// Check if action was performed
    pub fn is_some(&self) -> bool {
        !matches!(self, ClipboardAction::None)
    }
}

// ============================================================================
// SYSTEM CLIPBOARD INTEGRATION (Optional)
// ============================================================================

#[cfg(feature = "system-clipboard")]
pub mod system_clipboard {
    use super::*;
    use clipboard::{ClipboardProvider, ClipboardContext};
    
    /// Copy to system clipboard
    pub fn copy_to_system(clipboard: &Clipboard) -> Result<(), String> {
        if let Some(json) = clipboard.to_json() {
            let mut ctx: ClipboardContext = ClipboardProvider::new()
                .map_err(|e| format!("Failed to access system clipboard: {}", e))?;
            
            ctx.set_contents(json)
                .map_err(|e| format!("Failed to set clipboard contents: {}", e))?;
            
            Ok(())
        } else {
            Err("No data to copy".to_string())
        }
    }
    
    /// Paste from system clipboard
    pub fn paste_from_system(clipboard: &mut Clipboard) -> Result<(), String> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| format!("Failed to access system clipboard: {}", e))?;
        
        let contents = ctx.get_contents()
            .map_err(|e| format!("Failed to get clipboard contents: {}", e))?;
        
        clipboard.from_json(&contents)
    }
}
