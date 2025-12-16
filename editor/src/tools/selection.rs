//! Multi-Selection System
//!
//! Handles multiple entity selection with various selection modes:
//! - Single selection (click)
//! - Add to selection (Ctrl+Click)
//! - Range selection (Shift+Click)
//! - Box selection (drag)
//! - Select all (Ctrl+A)

#![allow(dead_code)]

use ecs::{World, Entity};
use egui;
use std::collections::HashSet;

/// Selection mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionMode {
    /// Replace current selection
    Replace,
    /// Add to current selection (Ctrl)
    Add,
    /// Toggle selection (Ctrl)
    Toggle,
    /// Range selection (Shift)
    Range,
}

/// Selection manager
pub struct SelectionManager {
    /// Currently selected entities
    selected: HashSet<Entity>,
    
    /// Last selected entity (for range selection)
    last_selected: Option<Entity>,
    
    /// Box selection state
    box_selection: Option<BoxSelection>,
    
    /// Selection history for undo/redo
    history: Vec<HashSet<Entity>>,
    history_index: usize,
}

/// Box selection state
#[derive(Debug, Clone)]
pub struct BoxSelection {
    pub start_pos: egui::Pos2,
    pub current_pos: egui::Pos2,
    pub mode: SelectionMode,
}

impl BoxSelection {
    pub fn new(start_pos: egui::Pos2, mode: SelectionMode) -> Self {
        Self {
            start_pos,
            current_pos: start_pos,
            mode,
        }
    }
    
    pub fn update(&mut self, pos: egui::Pos2) {
        self.current_pos = pos;
    }
    
    pub fn get_rect(&self) -> egui::Rect {
        egui::Rect::from_two_pos(self.start_pos, self.current_pos)
    }
}

impl SelectionManager {
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            last_selected: None,
            box_selection: None,
            history: vec![HashSet::new()],
            history_index: 0,
        }
    }
    
    // ========================================================================
    // SELECTION OPERATIONS
    // ========================================================================
    
    /// Select a single entity
    pub fn select(&mut self, entity: Entity, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.selected.clear();
                self.selected.insert(entity);
                self.last_selected = Some(entity);
            }
            SelectionMode::Add => {
                self.selected.insert(entity);
                self.last_selected = Some(entity);
            }
            SelectionMode::Toggle => {
                if self.selected.contains(&entity) {
                    self.selected.remove(&entity);
                } else {
                    self.selected.insert(entity);
                    self.last_selected = Some(entity);
                }
            }
            SelectionMode::Range => {
                // Range selection handled separately
            }
        }
        
        self.push_history();
    }
    
    /// Select multiple entities
    pub fn select_multiple(&mut self, entities: &[Entity], mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.selected.clear();
                self.selected.extend(entities.iter());
            }
            SelectionMode::Add => {
                self.selected.extend(entities.iter());
            }
            SelectionMode::Toggle => {
                for &entity in entities {
                    if self.selected.contains(&entity) {
                        self.selected.remove(&entity);
                    } else {
                        self.selected.insert(entity);
                    }
                }
            }
            SelectionMode::Range => {
                // Not applicable for multiple
            }
        }
        
        if let Some(&last) = entities.last() {
            self.last_selected = Some(last);
        }
        
        self.push_history();
    }
    
    /// Select range between last selected and target
    pub fn select_range(&mut self, target: Entity, all_entities: &[Entity]) {
        if let Some(last) = self.last_selected {
            // Find indices
            let last_idx = all_entities.iter().position(|&e| e == last);
            let target_idx = all_entities.iter().position(|&e| e == target);
            
            if let (Some(start), Some(end)) = (last_idx, target_idx) {
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                
                // Select range
                for &entity in &all_entities[start..=end] {
                    self.selected.insert(entity);
                }
                
                self.push_history();
            }
        } else {
            // No last selected, just select target
            self.select(target, SelectionMode::Replace);
        }
    }
    
    /// Select all entities
    pub fn select_all(&mut self, entities: &[Entity]) {
        self.selected.clear();
        self.selected.extend(entities.iter());
        self.push_history();
    }
    
    /// Clear selection
    pub fn clear(&mut self) {
        if !self.selected.is_empty() {
            self.selected.clear();
            self.last_selected = None;
            self.push_history();
        }
    }
    
    /// Deselect entity
    pub fn deselect(&mut self, entity: Entity) {
        if self.selected.remove(&entity) {
            self.push_history();
        }
    }
    
    // ========================================================================
    // BOX SELECTION
    // ========================================================================
    
    /// Start box selection
    pub fn start_box_selection(&mut self, start_pos: egui::Pos2, mode: SelectionMode) {
        self.box_selection = Some(BoxSelection::new(start_pos, mode));
    }
    
    /// Update box selection
    pub fn update_box_selection(&mut self, current_pos: egui::Pos2) {
        if let Some(box_sel) = &mut self.box_selection {
            box_sel.update(current_pos);
        }
    }
    
    /// Finish box selection
    pub fn finish_box_selection(&mut self, world: &World, scene_camera: &crate::SceneCamera, center: egui::Pos2) -> Vec<Entity> {
        if let Some(box_sel) = self.box_selection.take() {
            let rect = box_sel.get_rect();
            let selected_entities = self.get_entities_in_rect(rect, world, scene_camera, center);
            
            if !selected_entities.is_empty() {
                self.select_multiple(&selected_entities, box_sel.mode);
            }
            
            selected_entities
        } else {
            Vec::new()
        }
    }
    
    /// Cancel box selection
    pub fn cancel_box_selection(&mut self) {
        self.box_selection = None;
    }
    
    /// Get entities within rectangle
    fn get_entities_in_rect(
        &self,
        rect: egui::Rect,
        world: &World,
        scene_camera: &crate::SceneCamera,
        center: egui::Pos2,
    ) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for (&entity, transform) in &world.transforms {
            let world_pos = glam::Vec2::new(transform.x(), transform.y());
            let screen_pos = scene_camera.world_to_screen(world_pos);
            let screen_x = center.x + screen_pos.x;
            let screen_y = center.y + screen_pos.y;
            
            if rect.contains(egui::pos2(screen_x, screen_y)) {
                entities.push(entity);
            }
        }
        
        entities
    }
    
    // ========================================================================
    // QUERY OPERATIONS
    // ========================================================================
    
    /// Check if entity is selected
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected.contains(&entity)
    }
    
    /// Get selected entities
    pub fn get_selected(&self) -> Vec<Entity> {
        self.selected.iter().copied().collect()
    }
    
    /// Get selected entities as set
    pub fn get_selected_set(&self) -> &HashSet<Entity> {
        &self.selected
    }
    
    /// Get count of selected entities
    pub fn count(&self) -> usize {
        self.selected.len()
    }
    
    /// Check if has selection
    pub fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }
    
    /// Get first selected entity
    pub fn get_first(&self) -> Option<Entity> {
        self.selected.iter().next().copied()
    }
    
    /// Get last selected entity
    pub fn get_last(&self) -> Option<Entity> {
        self.last_selected
    }
    
    /// Get box selection state
    pub fn get_box_selection(&self) -> Option<&BoxSelection> {
        self.box_selection.as_ref()
    }
    
    // ========================================================================
    // HISTORY (for undo/redo)
    // ========================================================================
    
    fn push_history(&mut self) {
        // Remove any history after current index
        self.history.truncate(self.history_index + 1);
        
        // Add new state
        self.history.push(self.selected.clone());
        self.history_index += 1;
        
        // Limit history size
        const MAX_HISTORY: usize = 50;
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
            self.history_index -= 1;
        }
    }
    
    pub fn undo_selection(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.selected = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }
    
    pub fn redo_selection(&mut self) -> bool {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.selected = self.history[self.history_index].clone();
            true
        } else {
            false
        }
    }
    
    // ========================================================================
    // UTILITY
    // ========================================================================
    
    /// Get selection mode from modifiers
    pub fn get_selection_mode(modifiers: &egui::Modifiers) -> SelectionMode {
        if modifiers.shift {
            SelectionMode::Range
        } else if modifiers.ctrl || modifiers.command {
            SelectionMode::Toggle
        } else {
            SelectionMode::Replace
        }
    }
    
    /// Render box selection visual
    pub fn render_box_selection(&self, painter: &egui::Painter) {
        if let Some(box_sel) = &self.box_selection {
            let rect = box_sel.get_rect();
            
            // Fill
            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_premultiplied(100, 150, 255, 30),
            );
            
            // Border
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255)),
            );
        }
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Handle selection in scene view
pub fn handle_scene_selection(
    response: &egui::Response,
    selection: &mut SelectionManager,
    hovered_entity: Option<Entity>,
    all_entities: &[Entity],
    world: &World,
    scene_camera: &crate::SceneCamera,
    center: egui::Pos2,
) {
    let modifiers = response.ctx.input(|i| i.modifiers);
    
    // Box selection
    if response.drag_started_by(egui::PointerButton::Primary) && !modifiers.alt {
        if hovered_entity.is_none() {
            // Start box selection if not hovering entity
            if let Some(pos) = response.interact_pointer_pos() {
                let mode = SelectionManager::get_selection_mode(&modifiers);
                selection.start_box_selection(pos, mode);
            }
        }
    }
    
    if response.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            selection.update_box_selection(pos);
        }
    }
    
    if response.drag_stopped_by(egui::PointerButton::Primary) {
        selection.finish_box_selection(world, scene_camera, center);
    }
    
    // Click selection
    if response.clicked() && !response.dragged() {
        if let Some(entity) = hovered_entity {
            let mode = SelectionManager::get_selection_mode(&modifiers);
            
            if mode == SelectionMode::Range {
                selection.select_range(entity, all_entities);
            } else {
                selection.select(entity, mode);
            }
        } else if !modifiers.ctrl && !modifiers.command {
            // Click on empty space clears selection (unless Ctrl held)
            selection.clear();
        }
    }
    
    // Keyboard shortcuts
    response.ctx.input(|i| {
        // Ctrl+A: Select all
        if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
            selection.select_all(all_entities);
        }
        
        // Escape: Clear selection
        if i.key_pressed(egui::Key::Escape) {
            selection.clear();
        }
    });
}

/// Handle selection in hierarchy
pub fn handle_hierarchy_selection(
    ui: &mut egui::Ui,
    entity: Entity,
    selection: &mut SelectionManager,
    all_entities: &[Entity],
) -> egui::Response {
    let is_selected = selection.is_selected(entity);
    let response = ui.selectable_label(is_selected, format!("Entity {}", entity));
    
    if response.clicked() {
        let modifiers = ui.input(|i| i.modifiers);
        let mode = SelectionManager::get_selection_mode(&modifiers);
        
        if mode == SelectionMode::Range {
            selection.select_range(entity, all_entities);
        } else {
            selection.select(entity, mode);
        }
    }
    
    response
}

// ============================================================================
// MULTI-ENTITY OPERATIONS
// ============================================================================

/// Get common transform values from selected entities
pub fn get_common_transform(
    selected: &[Entity],
    world: &World,
) -> Option<(Option<[f32; 3]>, Option<[f32; 3]>, Option<[f32; 3]>)> {
    if selected.is_empty() {
        return None;
    }
    
    let mut common_pos: Option<[f32; 3]> = None;
    let mut common_rot: Option<[f32; 3]> = None;
    let mut common_scale: Option<[f32; 3]> = None;
    
    for &entity in selected {
        if let Some(transform) = world.transforms.get(&entity) {
            // Check position
            if let Some(pos) = common_pos {
                if pos != transform.position {
                    common_pos = None;
                }
            } else {
                common_pos = Some(transform.position);
            }
            
            // Check rotation
            if let Some(rot) = common_rot {
                if rot != transform.rotation {
                    common_rot = None;
                }
            } else {
                common_rot = Some(transform.rotation);
            }
            
            // Check scale
            if let Some(scale) = common_scale {
                if scale != transform.scale {
                    common_scale = None;
                }
            } else {
                common_scale = Some(transform.scale);
            }
        }
    }
    
    Some((common_pos, common_rot, common_scale))
}

/// Apply transform to multiple entities
pub fn apply_transform_to_selected(
    selected: &[Entity],
    world: &mut World,
    position: Option<[f32; 3]>,
    rotation: Option<[f32; 3]>,
    scale: Option<[f32; 3]>,
) {
    for &entity in selected {
        if let Some(transform) = world.transforms.get_mut(&entity) {
            if let Some(pos) = position {
                transform.position = pos;
            }
            if let Some(rot) = rotation {
                transform.rotation = rot;
            }
            if let Some(scl) = scale {
                transform.scale = scl;
            }
        }
    }
}

/// Move multiple entities by delta
pub fn move_selected_by_delta(
    selected: &[Entity],
    world: &mut World,
    delta: [f32; 3],
) {
    for &entity in selected {
        if let Some(transform) = world.transforms.get_mut(&entity) {
            transform.position[0] += delta[0];
            transform.position[1] += delta[1];
            transform.position[2] += delta[2];
        }
    }
}

/// Rotate multiple entities by delta
pub fn rotate_selected_by_delta(
    selected: &[Entity],
    world: &mut World,
    delta: [f32; 3],
) {
    for &entity in selected {
        if let Some(transform) = world.transforms.get_mut(&entity) {
            transform.rotation[0] += delta[0];
            transform.rotation[1] += delta[1];
            transform.rotation[2] += delta[2];
        }
    }
}

/// Scale multiple entities by factor
pub fn scale_selected_by_factor(
    selected: &[Entity],
    world: &mut World,
    factor: [f32; 3],
) {
    for &entity in selected {
        if let Some(transform) = world.transforms.get_mut(&entity) {
            transform.scale[0] *= factor[0];
            transform.scale[1] *= factor[1];
            transform.scale[2] *= factor[2];
        }
    }
}
