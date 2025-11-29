//! Undo/Redo System
//!
//! Command pattern implementation for editor actions.
//! Supports unlimited undo/redo with memory management.

use ecs::{World, Entity, Transform, Sprite, Collider, Camera, Mesh, EntityTag, Script};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[allow(dead_code)]

/// Maximum number of undo steps to keep in memory
const MAX_UNDO_STEPS: usize = 100;

/// Command trait for undo/redo operations
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>);
    
    /// Undo the command
    fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>);
    
    /// Redo the command (default: same as execute)
    fn redo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        self.execute(world, entity_names);
    }
    
    /// Get command description for UI
    fn description(&self) -> String;
    
    /// Check if command can be merged with another (for optimization)
    fn can_merge(&self, _other: &dyn Command) -> bool {
        false
    }
    
    /// Merge with another command (for optimization)
    fn merge(&mut self, _other: Box<dyn Command>) {}

    /// Helper for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Helper for downcasting
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
}

/// Undo/Redo stack manager
pub struct UndoStack {
    commands: Vec<Box<dyn Command>>,
    current_index: usize,
    max_size: usize,
    saved_index: Option<usize>, // Index when last saved
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
            max_size: MAX_UNDO_STEPS,
            saved_index: Some(0),
        }
    }
    
    /// Execute and push a new command
    pub fn execute(&mut self, mut command: Box<dyn Command>, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        // Execute the command
        command.execute(world, entity_names);
        
        // Remove any commands after current index (they're now invalid)
        self.commands.truncate(self.current_index);
        
        // Try to merge with previous command if possible
        if let Some(last_cmd) = self.commands.last_mut() {
            if last_cmd.can_merge(command.as_ref()) {
                last_cmd.merge(command);
                return;
            }
        }
        
        // Add new command
        self.commands.push(command);
        self.current_index += 1;
        
        // Limit stack size
        if self.commands.len() > self.max_size {
            self.commands.remove(0);
            self.current_index -= 1;
            if let Some(saved) = self.saved_index {
                self.saved_index = saved.checked_sub(1);
            }
        }
    }
    
    /// Undo the last command
    pub fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) -> bool {
        if self.can_undo() {
            self.current_index -= 1;
            self.commands[self.current_index].undo(world, entity_names);
            true
        } else {
            false
        }
    }
    
    /// Redo the next command
    pub fn redo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) -> bool {
        if self.can_redo() {
            self.commands[self.current_index].redo(world, entity_names);
            self.current_index += 1;
            true
        } else {
            false
        }
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.current_index < self.commands.len()
    }
    
    /// Get description of command that would be undone
    pub fn undo_description(&self) -> Option<String> {
        if self.can_undo() {
            Some(self.commands[self.current_index - 1].description())
        } else {
            None
        }
    }
    
    /// Get description of command that would be redone
    pub fn redo_description(&self) -> Option<String> {
        if self.can_redo() {
            Some(self.commands[self.current_index].description())
        } else {
            None
        }
    }
    
    /// Mark current state as saved
    pub fn mark_saved(&mut self) {
        self.saved_index = Some(self.current_index);
    }
    
    /// Check if current state is saved
    pub fn is_saved(&self) -> bool {
        self.saved_index == Some(self.current_index)
    }
    
    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_index = 0;
        self.saved_index = Some(0);
    }
    
    /// Get undo history for UI
    pub fn get_history(&self) -> Vec<String> {
        self.commands.iter().map(|cmd| cmd.description()).collect()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// COMMAND IMPLEMENTATIONS
// ============================================================================

/// Complete entity data for serialization
#[derive(Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub entity: Entity,
    pub name: String,
    pub transform: Option<Transform>,
    pub sprite: Option<Sprite>,
    pub collider: Option<Collider>,
    pub camera: Option<Camera>,
    pub mesh: Option<Mesh>,
    pub velocity: Option<(f32, f32)>,
    pub tag: Option<EntityTag>,
    pub script: Option<Script>,
    pub active: bool,
    pub layer: u8,
    pub parent: Option<Entity>,
}

impl EntityData {
    /// Capture entity data from world
    pub fn from_world(entity: Entity, world: &World, entity_names: &HashMap<Entity, String>) -> Self {
        Self {
            entity,
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
            parent: world.parents.get(&entity).copied(),
        }
    }
    
    /// Restore entity data to world
    pub fn restore_to_world(&self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        entity_names.insert(self.entity, self.name.clone());
        world.names.insert(self.entity, self.name.clone());
        
        if let Some(transform) = &self.transform {
            world.transforms.insert(self.entity, transform.clone());
        }
        if let Some(sprite) = &self.sprite {
            world.sprites.insert(self.entity, sprite.clone());
        }
        if let Some(collider) = &self.collider {
            world.colliders.insert(self.entity, collider.clone());
        }
        if let Some(camera) = &self.camera {
            world.cameras.insert(self.entity, camera.clone());
        }
        if let Some(mesh) = &self.mesh {
            world.meshes.insert(self.entity, mesh.clone());
        }
        if let Some(velocity) = self.velocity {
            world.velocities.insert(self.entity, velocity);
        }
        if let Some(tag) = &self.tag {
            world.tags.insert(self.entity, tag.clone());
        }
        if let Some(script) = &self.script {
            world.scripts.insert(self.entity, script.clone());
        }
        world.active.insert(self.entity, self.active);
        world.layers.insert(self.entity, self.layer);
        
        if let Some(parent) = self.parent {
            world.parents.insert(self.entity, parent);
            world.children.entry(parent).or_default().push(self.entity);
        }
    }
    
    /// Remove entity from world
    pub fn remove_from_world(&self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        entity_names.remove(&self.entity);
        world.names.remove(&self.entity);
        world.transforms.remove(&self.entity);
        world.sprites.remove(&self.entity);
        world.colliders.remove(&self.entity);
        world.cameras.remove(&self.entity);
        world.meshes.remove(&self.entity);
        world.velocities.remove(&self.entity);
        world.tags.remove(&self.entity);
        world.scripts.remove(&self.entity);
        world.active.remove(&self.entity);
        world.layers.remove(&self.entity);
        
        if let Some(parent) = world.parents.remove(&self.entity) {
            if let Some(children) = world.children.get_mut(&parent) {
                children.retain(|&e| e != self.entity);
            }
        }
        world.children.remove(&self.entity);
    }
}

// ============================================================================
// CREATE ENTITY COMMAND
// ============================================================================

pub struct CreateEntityCommand {
    entity_data: Option<EntityData>,
}

impl CreateEntityCommand {
    pub fn new(entity: Entity, world: &World, entity_names: &HashMap<Entity, String>) -> Self {
        Self {
            entity_data: Some(EntityData::from_world(entity, world, entity_names)),
        }
    }
}

impl Command for CreateEntityCommand {
    fn execute(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        if let Some(data) = &self.entity_data {
            data.restore_to_world(world, entity_names);
        }
    }
    
    fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        if let Some(data) = &self.entity_data {
            data.remove_from_world(world, entity_names);
        }
    }
    
    fn description(&self) -> String {
        if let Some(data) = &self.entity_data {
            format!("Create {}", data.name)
        } else {
            "Create Entity".to_string()
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// ============================================================================
// DELETE ENTITY COMMAND
// ============================================================================

pub struct DeleteEntityCommand {
    entity_data: Option<EntityData>,
}

impl DeleteEntityCommand {
    pub fn new(entity: Entity, world: &World, entity_names: &HashMap<Entity, String>) -> Self {
        Self {
            entity_data: Some(EntityData::from_world(entity, world, entity_names)),
        }
    }
}

impl Command for DeleteEntityCommand {
    fn execute(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        if let Some(data) = &self.entity_data {
            data.remove_from_world(world, entity_names);
        }
    }
    
    fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        if let Some(data) = &self.entity_data {
            data.restore_to_world(world, entity_names);
        }
    }
    
    fn description(&self) -> String {
        if let Some(data) = &self.entity_data {
            format!("Delete {}", data.name)
        } else {
            "Delete Entity".to_string()
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// ============================================================================
// MOVE ENTITY COMMAND
// ============================================================================

pub struct MoveEntityCommand {
    entity: Entity,
    old_position: [f32; 3],
    new_position: [f32; 3],
    merge_threshold: f32,
}

impl MoveEntityCommand {
    pub fn new(entity: Entity, old_position: [f32; 3], new_position: [f32; 3]) -> Self {
        Self {
            entity,
            old_position,
            new_position,
            merge_threshold: 0.1, // Merge if moved less than 0.1 units
        }
    }
}

impl Command for MoveEntityCommand {
    fn execute(&mut self, world: &mut World, _entity_names: &mut HashMap<Entity, String>) {
        if let Some(transform) = world.transforms.get_mut(&self.entity) {
            transform.position = self.new_position;
        }
    }
    
    fn undo(&mut self, world: &mut World, _entity_names: &mut HashMap<Entity, String>) {
        if let Some(transform) = world.transforms.get_mut(&self.entity) {
            transform.position = self.old_position;
        }
    }
    
    fn description(&self) -> String {
        format!("Move Entity {}", self.entity)
    }
    
    fn can_merge(&self, other: &dyn Command) -> bool {
        // Try to downcast to MoveEntityCommand
        if let Some(other_move) = other.as_any().downcast_ref::<MoveEntityCommand>() {
            // Can merge if same entity and positions are close
            if self.entity == other_move.entity {
                let dx = self.new_position[0] - other_move.old_position[0];
                let dy = self.new_position[1] - other_move.old_position[1];
                let dz = self.new_position[2] - other_move.old_position[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                return dist < self.merge_threshold;
            }
        }
        false
    }
    
    fn merge(&mut self, other: Box<dyn Command>) {
        if let Ok(other_move) = other.into_any().downcast::<MoveEntityCommand>() {
            // Keep old_position from self, update new_position from other
            self.new_position = other_move.new_position;
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// ============================================================================
// ROTATE ENTITY COMMAND
// ============================================================================

pub struct RotateEntityCommand {
    entity: Entity,
    old_rotation: [f32; 3],
    new_rotation: [f32; 3],
}

impl RotateEntityCommand {
    pub fn new(entity: Entity, old_rotation: [f32; 3], new_rotation: [f32; 3]) -> Self {
        Self {
            entity,
            old_rotation,
            new_rotation,
        }
    }
}

impl Command for RotateEntityCommand {
    fn execute(&mut self, world: &mut World, _entity_names: &mut HashMap<Entity, String>) {
        if let Some(transform) = world.transforms.get_mut(&self.entity) {
            transform.rotation = self.new_rotation;
        }
    }
    
    fn undo(&mut self, world: &mut World, _entity_names: &mut HashMap<Entity, String>) {
        if let Some(transform) = world.transforms.get_mut(&self.entity) {
            transform.rotation = self.old_rotation;
        }
    }
    
    fn description(&self) -> String {
        format!("Rotate Entity {}", self.entity)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// ============================================================================
// SCALE ENTITY COMMAND
// ============================================================================

pub struct ScaleEntityCommand {
    entity: Entity,
    old_scale: [f32; 3],
    new_scale: [f32; 3],
}

impl ScaleEntityCommand {
    pub fn new(entity: Entity, old_scale: [f32; 3], new_scale: [f32; 3]) -> Self {
        Self {
            entity,
            old_scale,
            new_scale,
        }
    }
}

impl Command for ScaleEntityCommand {
    fn execute(&mut self, world: &mut World, _entity_names: &mut HashMap<Entity, String>) {
        if let Some(transform) = world.transforms.get_mut(&self.entity) {
            transform.scale = self.new_scale;
        }
    }
    
    fn undo(&mut self, world: &mut World, _entity_names: &mut HashMap<Entity, String>) {
        if let Some(transform) = world.transforms.get_mut(&self.entity) {
            transform.scale = self.old_scale;
        }
    }
    
    fn description(&self) -> String {
        format!("Scale Entity {}", self.entity)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// ============================================================================
// RENAME ENTITY COMMAND
// ============================================================================

pub struct RenameEntityCommand {
    entity: Entity,
    old_name: String,
    new_name: String,
}

impl RenameEntityCommand {
    pub fn new(entity: Entity, old_name: String, new_name: String) -> Self {
        Self {
            entity,
            old_name,
            new_name,
        }
    }
}

impl Command for RenameEntityCommand {
    fn execute(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        entity_names.insert(self.entity, self.new_name.clone());
        world.names.insert(self.entity, self.new_name.clone());
    }
    
    fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        entity_names.insert(self.entity, self.old_name.clone());
        world.names.insert(self.entity, self.old_name.clone());
    }
    
    fn description(&self) -> String {
        format!("Rename {} to {}", self.old_name, self.new_name)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// ============================================================================
// BATCH COMMAND (for multiple operations)
// ============================================================================

pub struct BatchCommand {
    commands: Vec<Box<dyn Command>>,
    description: String,
}

impl BatchCommand {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            commands: Vec::new(),
            description: description.into(),
        }
    }
    
    pub fn add(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }
}

impl Command for BatchCommand {
    fn execute(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        for cmd in &mut self.commands {
            cmd.execute(world, entity_names);
        }
    }
    
    fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>) {
        // Undo in reverse order
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(world, entity_names);
        }
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}
