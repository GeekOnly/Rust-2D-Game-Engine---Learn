//! UI batch builder for efficient rendering
//!
//! This module collects visible UI elements, sorts them by render order,
//! and groups them into batches for efficient GPU rendering.

use std::collections::HashMap;
use glam::Vec2;
use crate::{
    Canvas, RectTransform, UIElement, UIImage, UIText, ImageType,
    rendering::{UIMesh, UIVertex, generate_nine_slice_mesh, generate_simple_mesh},
    Rect,
};

/// Entity ID type
pub type Entity = u64;

/// Check if two rectangles overlap
fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width
        && a.x + a.width > b.x
        && a.y < b.y + b.height
        && a.y + a.height > b.y
}

/// A batch of UI elements that can be rendered together
#[derive(Clone, Debug)]
pub struct UIBatch {
    /// Texture ID for this batch
    pub texture_id: Option<String>,
    
    /// Vertex data
    pub vertices: Vec<UIVertex>,
    
    /// Index data
    pub indices: Vec<u32>,
    
    /// Canvas sort order
    pub canvas_sort_order: i32,
    
    /// Minimum Z-order in this batch
    pub min_z_order: i32,
    
    /// Maximum Z-order in this batch
    pub max_z_order: i32,
}

impl UIBatch {
    /// Create a new empty batch
    pub fn new(texture_id: Option<String>, canvas_sort_order: i32) -> Self {
        Self {
            texture_id,
            vertices: Vec::new(),
            indices: Vec::new(),
            canvas_sort_order,
            min_z_order: i32::MAX,
            max_z_order: i32::MIN,
        }
    }

    /// Add a mesh to this batch
    pub fn add_mesh(&mut self, mesh: &UIMesh, z_order: i32) {
        let base_index = self.vertices.len() as u32;
        
        // Add vertices
        self.vertices.extend_from_slice(&mesh.vertices);
        
        // Add indices with offset
        for &index in &mesh.indices {
            self.indices.push(base_index + index);
        }
        
        // Update Z-order range
        self.min_z_order = self.min_z_order.min(z_order);
        self.max_z_order = self.max_z_order.max(z_order);
    }

    /// Check if this batch is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Get the number of triangles in this batch
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Represents a UI element ready for batching
#[derive(Clone, Debug)]
pub struct BatchableElement {
    /// Entity ID
    pub entity: Entity,
    
    /// Canvas sort order
    pub canvas_sort_order: i32,
    
    /// Z-order within canvas
    pub z_order: i32,
    
    /// Texture ID (for batching)
    pub texture_id: Option<String>,
    
    /// Generated mesh
    pub mesh: UIMesh,
    
    /// Whether this element is visible
    pub visible: bool,
}

/// UI Batch Builder
///
/// Collects visible UI elements from the hierarchy, sorts them by render order,
/// and groups them into batches for efficient rendering.
pub struct UIBatchBuilder {
    /// Collected elements ready for batching
    elements: Vec<BatchableElement>,
    
    /// Generated batches
    batches: Vec<UIBatch>,
    
    /// Dirty flag - set when elements need re-batching
    dirty: bool,
}

impl UIBatchBuilder {
    /// Create a new batch builder
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            batches: Vec::new(),
            dirty: true,
        }
    }

    /// Mark the builder as dirty (needs rebuild)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if the builder is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear all collected elements and batches
    pub fn clear(&mut self) {
        self.elements.clear();
        self.batches.clear();
        self.dirty = true;
    }

    /// Collect a UI element for batching
    ///
    /// This should be called for each visible UI element in the hierarchy.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity ID
    /// * `canvas` - The canvas this element belongs to
    /// * `rect_transform` - The element's rect transform
    /// * `ui_element` - The UI element component
    /// * `image` - Optional image component
    /// * `_text` - Optional text component (for future text rendering)
    /// * `viewport` - Optional viewport rect for culling (elements outside are culled)
    pub fn collect_element(
        &mut self,
        entity: Entity,
        canvas: &Canvas,
        rect_transform: &RectTransform,
        ui_element: &UIElement,
        image: Option<&UIImage>,
        _text: Option<&UIText>,
        viewport: Option<Rect>,
    ) {
        // Skip if not visible or not a raycast target
        if !ui_element.raycast_target && image.is_none() {
            return;
        }

        // Cull elements outside viewport
        if let Some(vp) = viewport {
            if !rects_overlap(&rect_transform.rect, &vp) {
                return; // Element is outside viewport, skip it
            }
        }

        // Calculate color with alpha
        let mut color = ui_element.color;
        color[3] *= ui_element.alpha;

        // Generate mesh based on component type
        let (texture_id, mesh) = if let Some(img) = image {
            let texture_id = img.sprite.clone();
            
            let mesh = match img.image_type {
                ImageType::Simple => {
                    generate_simple_mesh(rect_transform.rect, color)
                }
                ImageType::Sliced => {
                    // For 9-slice, we need the texture size
                    // For now, use a default size (this should come from texture manager)
                    let texture_size = Vec2::new(256.0, 256.0);
                    generate_nine_slice_mesh(
                        rect_transform.rect,
                        img.slice_borders,
                        texture_size,
                        color,
                    )
                }
                ImageType::Tiled => {
                    // TODO: Implement tiled rendering
                    generate_simple_mesh(rect_transform.rect, color)
                }
                ImageType::Filled => {
                    // TODO: Implement filled rendering
                    generate_simple_mesh(rect_transform.rect, color)
                }
            };
            
            (texture_id, mesh)
        } else {
            // No image component, create a simple colored quad
            (None, generate_simple_mesh(rect_transform.rect, color))
        };

        // Add to elements list
        self.elements.push(BatchableElement {
            entity,
            canvas_sort_order: canvas.sort_order,
            z_order: ui_element.z_order,
            texture_id,
            mesh,
            visible: true,
        });

        self.dirty = true;
    }

    /// Build batches from collected elements
    ///
    /// This sorts elements by render order and groups them into batches
    /// based on material/texture compatibility and Z-order constraints.
    pub fn build_batches(&mut self) {
        if !self.dirty {
            return;
        }

        self.batches.clear();

        // Sort elements by render order
        // Primary: Canvas sort order (ascending)
        // Secondary: Z-order (ascending for back-to-front rendering)
        // Tertiary: Entity ID (for stable sorting)
        self.elements.sort_by(|a, b| {
            a.canvas_sort_order
                .cmp(&b.canvas_sort_order)
                .then(a.z_order.cmp(&b.z_order))
                .then(a.entity.cmp(&b.entity))
        });

        // Group elements into batches
        let mut current_batch: Option<UIBatch> = None;
        let mut last_texture: Option<String> = None;
        let mut last_canvas_sort_order = i32::MIN;
        let mut last_z_order = i32::MIN;

        for element in &self.elements {
            if !element.visible {
                continue;
            }

            // Check if we need to start a new batch
            let needs_new_batch = if let Some(ref batch) = current_batch {
                // Different canvas sort order always breaks batch
                element.canvas_sort_order != last_canvas_sort_order
                // Different texture breaks batch
                || element.texture_id != last_texture
                // Z-order change breaks batch (to maintain render order)
                || element.z_order != last_z_order
            } else {
                true
            };

            if needs_new_batch {
                // Save current batch if it exists
                if let Some(batch) = current_batch.take() {
                    if !batch.is_empty() {
                        self.batches.push(batch);
                    }
                }

                // Start new batch
                current_batch = Some(UIBatch::new(
                    element.texture_id.clone(),
                    element.canvas_sort_order,
                ));
                last_texture = element.texture_id.clone();
                last_canvas_sort_order = element.canvas_sort_order;
                last_z_order = element.z_order;
            }

            // Add element to current batch
            if let Some(ref mut batch) = current_batch {
                batch.add_mesh(&element.mesh, element.z_order);
            }
        }

        // Save final batch
        if let Some(batch) = current_batch {
            if !batch.is_empty() {
                self.batches.push(batch);
            }
        }

        self.dirty = false;
    }

    /// Get the generated batches
    ///
    /// This will build batches if they are dirty.
    pub fn get_batches(&mut self) -> &[UIBatch] {
        if self.dirty {
            self.build_batches();
        }
        &self.batches
    }

    /// Get the number of batches
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Get the total number of triangles across all batches
    pub fn total_triangle_count(&self) -> usize {
        self.batches.iter().map(|b| b.triangle_count()).sum()
    }

    /// Get statistics about batching efficiency
    pub fn get_stats(&self) -> BatchStats {
        BatchStats {
            element_count: self.elements.len(),
            batch_count: self.batches.len(),
            total_vertices: self.batches.iter().map(|b| b.vertices.len()).sum(),
            total_indices: self.batches.iter().map(|b| b.indices.len()).sum(),
            total_triangles: self.total_triangle_count(),
        }
    }
}

impl Default for UIBatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about batching efficiency
#[derive(Clone, Debug)]
pub struct BatchStats {
    /// Number of UI elements
    pub element_count: usize,
    
    /// Number of batches
    pub batch_count: usize,
    
    /// Total vertices across all batches
    pub total_vertices: usize,
    
    /// Total indices across all batches
    pub total_indices: usize,
    
    /// Total triangles across all batches
    pub total_triangles: usize,
}

/// UI Render System
///
/// Manages the rendering of UI elements by collecting them from the ECS world,
/// building batches, and submitting them to the renderer.
pub struct UIRenderSystem {
    /// Batch builder
    batch_builder: UIBatchBuilder,
    
    /// Canvas entities (for tracking dirty state)
    canvas_entities: HashMap<Entity, bool>, // entity -> dirty
}

impl UIRenderSystem {
    /// Create a new UI render system
    pub fn new() -> Self {
        Self {
            batch_builder: UIBatchBuilder::new(),
            canvas_entities: HashMap::new(),
        }
    }

    /// Mark a canvas as dirty (needs rebuild)
    pub fn mark_canvas_dirty(&mut self, entity: Entity) {
        self.canvas_entities.insert(entity, true);
        self.batch_builder.mark_dirty();
    }

    /// Check if any canvas is dirty
    pub fn is_dirty(&self) -> bool {
        self.batch_builder.is_dirty() || self.canvas_entities.values().any(|&dirty| dirty)
    }

    /// Clear all dirty flags
    pub fn clear_dirty(&mut self) {
        for dirty in self.canvas_entities.values_mut() {
            *dirty = false;
        }
    }

    /// Get the batch builder
    pub fn batch_builder(&self) -> &UIBatchBuilder {
        &self.batch_builder
    }

    /// Get the batch builder mutably
    pub fn batch_builder_mut(&mut self) -> &mut UIBatchBuilder {
        &mut self.batch_builder
    }

    /// Collect all UI elements for rendering
    ///
    /// This should be called once per frame to collect all visible UI elements.
    pub fn collect_elements<F>(&mut self, mut collector: F)
    where
        F: FnMut(&mut UIBatchBuilder),
    {
        self.batch_builder.clear();
        collector(&mut self.batch_builder);
    }

    /// Build batches and get them for rendering
    pub fn get_batches(&mut self) -> &[UIBatch] {
        self.batch_builder.get_batches()
    }

    /// Get batching statistics
    pub fn get_stats(&self) -> BatchStats {
        self.batch_builder.get_stats()
    }
}

impl Default for UIRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Canvas, CanvasRenderMode, CanvasScaler, ScaleMode};

    fn create_test_canvas(sort_order: i32) -> Canvas {
        Canvas {
            render_mode: CanvasRenderMode::ScreenSpaceOverlay,
            sort_order,
            camera_entity: None,
            plane_distance: 100.0,
            scaler: CanvasScaler {
                mode: ScaleMode::ConstantPixelSize,
                reference_resolution: (1920.0, 1080.0),
                match_width_or_height: 0.5,
                reference_dpi: 96.0,
                min_scale: 0.5,
                max_scale: 2.0,
                scale_factor: 1.0,
            },
            blocks_raycasts: true,
            cached_screen_size: (1920, 1080),
            dirty: false,
        }
    }

    fn create_test_rect_transform(x: f32, y: f32, width: f32, height: f32) -> RectTransform {
        let mut rt = RectTransform::default();
        rt.rect = Rect { x, y, width, height };
        rt
    }

    fn create_test_ui_element(z_order: i32) -> UIElement {
        UIElement {
            raycast_target: true,
            blocks_raycasts: true,
            z_order,
            color: [1.0, 1.0, 1.0, 1.0],
            alpha: 1.0,
            interactable: true,
            ignore_layout: false,
            canvas_entity: None,
        }
    }

    #[test]
    fn test_batch_builder_basic() {
        let mut builder = UIBatchBuilder::new();
        
        let canvas = create_test_canvas(0);
        let rect = create_test_rect_transform(0.0, 0.0, 100.0, 100.0);
        let element = create_test_ui_element(0);
        
        builder.collect_element(1, &canvas, &rect, &element, None, None, None);
        
        assert_eq!(builder.elements.len(), 1);
        assert!(builder.is_dirty());
    }

    #[test]
    fn test_batch_builder_sorting() {
        let mut builder = UIBatchBuilder::new();
        
        let canvas = create_test_canvas(0);
        let rect = create_test_rect_transform(0.0, 0.0, 100.0, 100.0);
        
        // Add elements with different Z-orders
        let element1 = create_test_ui_element(2);
        let element2 = create_test_ui_element(0);
        let element3 = create_test_ui_element(1);
        
        builder.collect_element(1, &canvas, &rect, &element1, None, None, None);
        builder.collect_element(2, &canvas, &rect, &element2, None, None, None);
        builder.collect_element(3, &canvas, &rect, &element3, None, None, None);
        
        builder.build_batches();
        
        // Elements should be sorted by Z-order
        assert_eq!(builder.elements[0].z_order, 0);
        assert_eq!(builder.elements[1].z_order, 1);
        assert_eq!(builder.elements[2].z_order, 2);
    }

    #[test]
    fn test_batch_builder_canvas_sort_order() {
        let mut builder = UIBatchBuilder::new();
        
        let canvas1 = create_test_canvas(0);
        let canvas2 = create_test_canvas(1);
        let rect = create_test_rect_transform(0.0, 0.0, 100.0, 100.0);
        let element = create_test_ui_element(0);
        
        // Add elements from different canvases
        builder.collect_element(1, &canvas2, &rect, &element, None, None, None);
        builder.collect_element(2, &canvas1, &rect, &element, None, None, None);
        
        builder.build_batches();
        
        // Elements should be sorted by canvas sort order first
        assert_eq!(builder.elements[0].canvas_sort_order, 0);
        assert_eq!(builder.elements[1].canvas_sort_order, 1);
    }

    #[test]
    fn test_batch_breaking_on_z_order() {
        let mut builder = UIBatchBuilder::new();
        
        let canvas = create_test_canvas(0);
        let rect = create_test_rect_transform(0.0, 0.0, 100.0, 100.0);
        
        // Add elements with different Z-orders
        let element1 = create_test_ui_element(0);
        let element2 = create_test_ui_element(1);
        
        builder.collect_element(1, &canvas, &rect, &element1, None, None, None);
        builder.collect_element(2, &canvas, &rect, &element2, None, None, None);
        
        let batches = builder.get_batches();
        
        // Should create separate batches for different Z-orders
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_batch_stats() {
        let mut builder = UIBatchBuilder::new();
        
        let canvas = create_test_canvas(0);
        let rect = create_test_rect_transform(0.0, 0.0, 100.0, 100.0);
        let element = create_test_ui_element(0);
        
        builder.collect_element(1, &canvas, &rect, &element, None, None, None);
        builder.collect_element(2, &canvas, &rect, &element, None, None, None);
        
        builder.build_batches();
        
        let stats = builder.get_stats();
        assert_eq!(stats.element_count, 2);
        assert!(stats.batch_count > 0);
        assert!(stats.total_vertices > 0);
        assert!(stats.total_triangles > 0);
    }

    #[test]
    fn test_culling() {
        let mut builder = UIBatchBuilder::new();
        
        let canvas = create_test_canvas(0);
        let element = create_test_ui_element(0);
        
        // Element inside viewport
        let rect_inside = create_test_rect_transform(50.0, 50.0, 100.0, 100.0);
        // Element outside viewport
        let rect_outside = create_test_rect_transform(2000.0, 2000.0, 100.0, 100.0);
        
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        
        builder.collect_element(1, &canvas, &rect_inside, &element, None, None, Some(viewport));
        builder.collect_element(2, &canvas, &rect_outside, &element, None, None, Some(viewport));
        
        // Only the element inside viewport should be collected
        assert_eq!(builder.elements.len(), 1);
        assert_eq!(builder.elements[0].entity, 1);
    }
}
