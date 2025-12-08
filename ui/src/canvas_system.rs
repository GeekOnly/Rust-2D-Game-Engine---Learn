//! Canvas management system
//!
//! This module provides the CanvasSystem which manages Canvas entities,
//! handles screen resolution changes, and updates scale factors.

use crate::{Canvas, CanvasScaler};
use std::collections::HashMap;

/// Entity ID type (using u64 as a simple entity identifier)
pub type Entity = u64;

/// Canvas management system
/// 
/// Handles canvas creation, initialization, resolution changes, and scale factor updates.
pub struct CanvasSystem {
    /// Map of entity IDs to Canvas components
    canvases: HashMap<Entity, Canvas>,
    
    /// Current screen dimensions
    screen_width: u32,
    screen_height: u32,
    
    /// Current screen DPI
    screen_dpi: f32,
    
    /// Next entity ID to assign
    next_entity_id: Entity,
}

impl CanvasSystem {
    /// Create a new CanvasSystem with default screen settings
    pub fn new() -> Self {
        Self {
            canvases: HashMap::new(),
            screen_width: 1920,
            screen_height: 1080,
            screen_dpi: 96.0,
            next_entity_id: 1,
        }
    }

    /// Create a new CanvasSystem with specified screen settings
    pub fn with_screen_settings(width: u32, height: u32, dpi: f32) -> Self {
        Self {
            canvases: HashMap::new(),
            screen_width: width,
            screen_height: height,
            screen_dpi: dpi,
            next_entity_id: 1,
        }
    }

    /// Create a new canvas entity with default settings
    /// 
    /// Returns the entity ID of the created canvas
    pub fn create_canvas(&mut self) -> Entity {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        let mut canvas = Canvas::new();
        
        // Initialize with current screen size
        canvas.cached_screen_size = (self.screen_width, self.screen_height);
        
        // Calculate initial scale factor
        canvas.scaler.calculate_scale_factor(
            self.screen_width as f32,
            self.screen_height as f32,
            self.screen_dpi,
        );
        
        // Mark as dirty for initial render
        canvas.dirty = true;

        self.canvases.insert(entity_id, canvas);
        
        entity_id
    }

    /// Create a new canvas entity with a specific Canvas configuration
    /// 
    /// Returns the entity ID of the created canvas
    pub fn create_canvas_with_config(&mut self, mut canvas: Canvas) -> Entity {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;

        // Initialize with current screen size
        canvas.cached_screen_size = (self.screen_width, self.screen_height);
        
        // Calculate initial scale factor
        canvas.scaler.calculate_scale_factor(
            self.screen_width as f32,
            self.screen_height as f32,
            self.screen_dpi,
        );
        
        // Mark as dirty for initial render
        canvas.dirty = true;

        self.canvases.insert(entity_id, canvas);
        
        entity_id
    }

    /// Get a reference to a canvas by entity ID
    pub fn get_canvas(&self, entity: Entity) -> Option<&Canvas> {
        self.canvases.get(&entity)
    }

    /// Get a mutable reference to a canvas by entity ID
    pub fn get_canvas_mut(&mut self, entity: Entity) -> Option<&mut Canvas> {
        self.canvases.get_mut(&entity)
    }

    /// Remove a canvas entity
    pub fn remove_canvas(&mut self, entity: Entity) -> Option<Canvas> {
        self.canvases.remove(&entity)
    }

    /// Get all canvas entities sorted by sort order
    /// 
    /// Returns a vector of (entity_id, sort_order) tuples sorted in ascending order
    pub fn get_sorted_canvases(&self) -> Vec<(Entity, i32)> {
        let mut canvases: Vec<(Entity, i32)> = self.canvases
            .iter()
            .map(|(entity, canvas)| (*entity, canvas.sort_order))
            .collect();
        
        // Sort by sort order (ascending - lower values render first)
        canvases.sort_by_key(|(_, sort_order)| *sort_order);
        
        canvases
    }

    /// Update screen resolution
    /// 
    /// This will update all canvases with the new screen size, recalculate scale factors,
    /// and mark them as dirty if the resolution changed.
    /// 
    /// Returns true if the resolution changed, false otherwise
    pub fn update_screen_resolution(&mut self, width: u32, height: u32) -> bool {
        if self.screen_width == width && self.screen_height == height {
            return false;
        }

        self.screen_width = width;
        self.screen_height = height;

        // Update all canvases
        for canvas in self.canvases.values_mut() {
            // Update cached screen size and mark dirty
            canvas.update_screen_size(width, height);
            
            // Recalculate scale factor
            canvas.scaler.calculate_scale_factor(
                width as f32,
                height as f32,
                self.screen_dpi,
            );
        }

        true
    }

    /// Update screen DPI
    /// 
    /// This will recalculate scale factors for all canvases using ConstantPhysicalSize mode
    /// and mark them as dirty.
    /// 
    /// Returns true if the DPI changed, false otherwise
    pub fn update_screen_dpi(&mut self, dpi: f32) -> bool {
        if (self.screen_dpi - dpi).abs() < f32::EPSILON {
            return false;
        }

        self.screen_dpi = dpi;

        // Update all canvases
        for canvas in self.canvases.values_mut() {
            // Recalculate scale factor
            canvas.scaler.calculate_scale_factor(
                self.screen_width as f32,
                self.screen_height as f32,
                self.screen_dpi,
            );
            
            // Mark as dirty
            canvas.mark_dirty();
        }

        true
    }

    /// Get current screen width
    pub fn screen_width(&self) -> u32 {
        self.screen_width
    }

    /// Get current screen height
    pub fn screen_height(&self) -> u32 {
        self.screen_height
    }

    /// Get current screen DPI
    pub fn screen_dpi(&self) -> f32 {
        self.screen_dpi
    }

    /// Get the number of canvases
    pub fn canvas_count(&self) -> usize {
        self.canvases.len()
    }

    /// Check if any canvas is dirty
    pub fn has_dirty_canvases(&self) -> bool {
        self.canvases.values().any(|canvas| canvas.is_dirty())
    }

    /// Get all dirty canvas entities
    pub fn get_dirty_canvases(&self) -> Vec<Entity> {
        self.canvases
            .iter()
            .filter(|(_, canvas)| canvas.is_dirty())
            .map(|(entity, _)| *entity)
            .collect()
    }

    /// Clear dirty flags for all canvases
    pub fn clear_all_dirty_flags(&mut self) {
        for canvas in self.canvases.values_mut() {
            canvas.clear_dirty();
        }
    }

    /// Clear dirty flag for a specific canvas
    pub fn clear_dirty_flag(&mut self, entity: Entity) {
        if let Some(canvas) = self.canvases.get_mut(&entity) {
            canvas.clear_dirty();
        }
    }

    /// Mark all canvases as dirty
    pub fn mark_all_dirty(&mut self) {
        for canvas in self.canvases.values_mut() {
            canvas.mark_dirty();
        }
    }

    /// Mark a specific canvas as dirty
    pub fn mark_dirty(&mut self, entity: Entity) {
        if let Some(canvas) = self.canvases.get_mut(&entity) {
            canvas.mark_dirty();
        }
    }
}

impl Default for CanvasSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CanvasRenderMode;

    #[test]
    fn test_canvas_system_creation() {
        let system = CanvasSystem::new();
        assert_eq!(system.screen_width(), 1920);
        assert_eq!(system.screen_height(), 1080);
        assert_eq!(system.screen_dpi(), 96.0);
        assert_eq!(system.canvas_count(), 0);
    }

    #[test]
    fn test_canvas_system_with_screen_settings() {
        let system = CanvasSystem::with_screen_settings(1280, 720, 120.0);
        assert_eq!(system.screen_width(), 1280);
        assert_eq!(system.screen_height(), 720);
        assert_eq!(system.screen_dpi(), 120.0);
    }

    #[test]
    fn test_create_canvas() {
        let mut system = CanvasSystem::new();
        let entity = system.create_canvas();
        
        assert_eq!(system.canvas_count(), 1);
        
        let canvas = system.get_canvas(entity).unwrap();
        assert_eq!(canvas.cached_screen_size, (1920, 1080));
        assert!(canvas.is_dirty());
    }

    #[test]
    fn test_create_multiple_canvases() {
        let mut system = CanvasSystem::new();
        let entity1 = system.create_canvas();
        let entity2 = system.create_canvas();
        
        assert_eq!(system.canvas_count(), 2);
        assert_ne!(entity1, entity2);
    }

    #[test]
    fn test_create_canvas_with_config() {
        let mut system = CanvasSystem::new();
        let mut canvas = Canvas::new();
        canvas.sort_order = 10;
        canvas.render_mode = CanvasRenderMode::WorldSpace;
        
        let entity = system.create_canvas_with_config(canvas);
        
        let stored_canvas = system.get_canvas(entity).unwrap();
        assert_eq!(stored_canvas.sort_order, 10);
        assert_eq!(stored_canvas.render_mode, CanvasRenderMode::WorldSpace);
    }

    #[test]
    fn test_remove_canvas() {
        let mut system = CanvasSystem::new();
        let entity = system.create_canvas();
        
        assert_eq!(system.canvas_count(), 1);
        
        let removed = system.remove_canvas(entity);
        assert!(removed.is_some());
        assert_eq!(system.canvas_count(), 0);
    }

    #[test]
    fn test_get_sorted_canvases() {
        let mut system = CanvasSystem::new();
        
        let mut canvas1 = Canvas::new();
        canvas1.sort_order = 5;
        let entity1 = system.create_canvas_with_config(canvas1);
        
        let mut canvas2 = Canvas::new();
        canvas2.sort_order = 1;
        let entity2 = system.create_canvas_with_config(canvas2);
        
        let mut canvas3 = Canvas::new();
        canvas3.sort_order = 10;
        let entity3 = system.create_canvas_with_config(canvas3);
        
        let sorted = system.get_sorted_canvases();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], (entity2, 1));
        assert_eq!(sorted[1], (entity1, 5));
        assert_eq!(sorted[2], (entity3, 10));
    }

    #[test]
    fn test_update_screen_resolution() {
        let mut system = CanvasSystem::new();
        let entity = system.create_canvas();
        
        // Clear initial dirty flag
        system.clear_dirty_flag(entity);
        assert!(!system.get_canvas(entity).unwrap().is_dirty());
        
        // Update resolution
        let changed = system.update_screen_resolution(1280, 720);
        assert!(changed);
        assert_eq!(system.screen_width(), 1280);
        assert_eq!(system.screen_height(), 720);
        
        // Canvas should be marked dirty
        let canvas = system.get_canvas(entity).unwrap();
        assert!(canvas.is_dirty());
        assert_eq!(canvas.cached_screen_size, (1280, 720));
    }

    #[test]
    fn test_update_screen_resolution_no_change() {
        let mut system = CanvasSystem::new();
        let entity = system.create_canvas();
        system.clear_dirty_flag(entity);
        
        let changed = system.update_screen_resolution(1920, 1080);
        assert!(!changed);
        
        // Canvas should not be marked dirty
        assert!(!system.get_canvas(entity).unwrap().is_dirty());
    }

    #[test]
    fn test_update_screen_dpi() {
        let mut system = CanvasSystem::new();
        let entity = system.create_canvas();
        system.clear_dirty_flag(entity);
        
        let changed = system.update_screen_dpi(120.0);
        assert!(changed);
        assert_eq!(system.screen_dpi(), 120.0);
        
        // Canvas should be marked dirty
        assert!(system.get_canvas(entity).unwrap().is_dirty());
    }

    #[test]
    fn test_scale_factor_calculation_constant_pixel_size() {
        let mut system = CanvasSystem::new();
        let entity = system.create_canvas();
        
        let canvas = system.get_canvas(entity).unwrap();
        assert_eq!(canvas.scaler.get_scale_factor(), 1.0);
    }

    #[test]
    fn test_scale_factor_calculation_scale_with_screen_size() {
        let mut system = CanvasSystem::with_screen_settings(1920, 1080, 96.0);
        
        let mut canvas = Canvas::new();
        canvas.scaler = CanvasScaler::scale_with_screen_size(1920.0, 1080.0);
        canvas.scaler.set_match_width_or_height(0.0); // Match width
        
        let entity = system.create_canvas_with_config(canvas);
        
        // At reference resolution, scale should be 1.0
        let canvas = system.get_canvas(entity).unwrap();
        assert!((canvas.scaler.get_scale_factor() - 1.0).abs() < 0.001);
        
        // Change resolution to half
        system.update_screen_resolution(960, 540);
        let canvas = system.get_canvas(entity).unwrap();
        assert!((canvas.scaler.get_scale_factor() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_scale_factor_calculation_constant_physical_size() {
        let mut system = CanvasSystem::with_screen_settings(1920, 1080, 96.0);
        
        let mut canvas = Canvas::new();
        canvas.scaler = CanvasScaler::constant_physical_size(96.0);
        
        let entity = system.create_canvas_with_config(canvas);
        
        // At reference DPI, scale should be 1.0
        let canvas = system.get_canvas(entity).unwrap();
        assert_eq!(canvas.scaler.get_scale_factor(), 1.0);
        
        // Change DPI to double
        system.update_screen_dpi(192.0);
        let canvas = system.get_canvas(entity).unwrap();
        assert_eq!(canvas.scaler.get_scale_factor(), 2.0);
    }

    #[test]
    fn test_dirty_flag_management() {
        let mut system = CanvasSystem::new();
        let entity1 = system.create_canvas();
        let _entity2 = system.create_canvas();
        
        assert!(system.has_dirty_canvases());
        assert_eq!(system.get_dirty_canvases().len(), 2);
        
        system.clear_dirty_flag(entity1);
        assert!(system.has_dirty_canvases());
        assert_eq!(system.get_dirty_canvases().len(), 1);
        
        system.clear_all_dirty_flags();
        assert!(!system.has_dirty_canvases());
        assert_eq!(system.get_dirty_canvases().len(), 0);
        
        system.mark_dirty(entity1);
        assert!(system.has_dirty_canvases());
        assert_eq!(system.get_dirty_canvases().len(), 1);
        
        system.mark_all_dirty();
        assert_eq!(system.get_dirty_canvases().len(), 2);
    }

    #[test]
    fn test_scale_factor_clamping() {
        let mut system = CanvasSystem::with_screen_settings(1920, 1080, 96.0);
        
        let mut canvas = Canvas::new();
        canvas.scaler = CanvasScaler::scale_with_screen_size(1920.0, 1080.0);
        canvas.scaler.set_min_scale(0.5);
        canvas.scaler.set_max_scale(2.0);
        
        let entity = system.create_canvas_with_config(canvas);
        
        // Test minimum clamping - very small resolution
        system.update_screen_resolution(100, 100);
        let canvas = system.get_canvas(entity).unwrap();
        assert!(canvas.scaler.get_scale_factor() >= 0.5);
        
        // Test maximum clamping - very large resolution
        system.update_screen_resolution(10000, 10000);
        let canvas = system.get_canvas(entity).unwrap();
        assert!(canvas.scaler.get_scale_factor() <= 2.0);
    }
}
