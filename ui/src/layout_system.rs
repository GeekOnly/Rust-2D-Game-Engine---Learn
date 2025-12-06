//! Layout calculation system for automatic UI element arrangement
//!
//! This module provides systems for calculating and applying layout to UI elements
//! based on layout group components (Horizontal, Vertical, Grid).

use std::collections::HashMap;
use glam::Vec2;
use crate::{
    RectTransform, UIElement,
    layout::{HorizontalLayoutGroup, VerticalLayoutGroup, GridLayoutGroup, Alignment, Corner, Axis, GridConstraint},
};

/// Entity ID type (matches ecs crate)
pub type Entity = u64;

/// Layout calculation system
pub struct LayoutSystem {
    /// Cached layout calculations to avoid redundant work
    layout_cache: HashMap<Entity, LayoutCache>,
}

#[derive(Clone, Debug)]
struct LayoutCache {
    /// Last calculated positions for children
    child_positions: HashMap<Entity, Vec2>,
    /// Last calculated sizes for children
    child_sizes: HashMap<Entity, Vec2>,
}

impl LayoutSystem {
    /// Create a new layout system
    pub fn new() -> Self {
        Self {
            layout_cache: HashMap::new(),
        }
    }

    /// Update all layouts in the scene
    ///
    /// This should be called after RectTransform updates and before rendering.
    /// It processes all layout groups and updates their children's positions and sizes.
    pub fn update_layouts(
        &mut self,
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        ui_elements: &HashMap<Entity, UIElement>,
        horizontal_layouts: &HashMap<Entity, HorizontalLayoutGroup>,
        vertical_layouts: &HashMap<Entity, VerticalLayoutGroup>,
        grid_layouts: &HashMap<Entity, GridLayoutGroup>,
        children: &HashMap<Entity, Vec<Entity>>,
    ) {
        // Process horizontal layouts
        for (&entity, layout) in horizontal_layouts.iter() {
            if let Some(child_list) = children.get(&entity) {
                self.apply_horizontal_layout(
                    entity,
                    layout,
                    child_list,
                    rect_transforms,
                    ui_elements,
                );
            }
        }

        // Process vertical layouts
        for (&entity, layout) in vertical_layouts.iter() {
            if let Some(child_list) = children.get(&entity) {
                self.apply_vertical_layout(
                    entity,
                    layout,
                    child_list,
                    rect_transforms,
                    ui_elements,
                );
            }
        }

        // Process grid layouts
        for (&entity, layout) in grid_layouts.iter() {
            if let Some(child_list) = children.get(&entity) {
                self.apply_grid_layout(
                    entity,
                    layout,
                    child_list,
                    rect_transforms,
                    ui_elements,
                );
            }
        }
    }

    /// Apply horizontal layout to children
    fn apply_horizontal_layout(
        &mut self,
        parent_entity: Entity,
        layout: &HorizontalLayoutGroup,
        children: &[Entity],
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        ui_elements: &HashMap<Entity, UIElement>,
    ) {
        // Get parent rect
        let parent_rect = match rect_transforms.get(&parent_entity) {
            Some(rt) => rt.rect,
            None => return,
        };

        // Filter children that should be laid out
        let layout_children: Vec<Entity> = children
            .iter()
            .copied()
            .filter(|&child| {
                ui_elements
                    .get(&child)
                    .map(|ui| !ui.ignore_layout)
                    .unwrap_or(false)
            })
            .collect();

        if layout_children.is_empty() {
            return;
        }

        // Calculate available space
        let padding = layout.padding;
        let available_width = parent_rect.width - padding.x - padding.z; // left and right padding
        let available_height = parent_rect.height - padding.y - padding.w; // bottom and top padding

        // Calculate total spacing
        let total_spacing = layout.spacing * (layout_children.len() as f32 - 1.0).max(0.0);

        // Calculate child sizes
        let mut child_sizes: Vec<(Entity, Vec2)> = Vec::new();
        let mut total_child_width = 0.0;

        for &child in &layout_children {
            if let Some(child_rt) = rect_transforms.get(&child) {
                let mut size = child_rt.size_delta;
                
                // Apply child control
                if layout.child_control_height {
                    size.y = available_height;
                }
                
                child_sizes.push((child, size));
                total_child_width += size.x;
            }
        }

        // Apply force expand width
        if layout.child_force_expand_width && total_child_width + total_spacing < available_width {
            let extra_space = available_width - total_spacing - total_child_width;
            let extra_per_child = extra_space / layout_children.len() as f32;
            
            for (_, size) in &mut child_sizes {
                size.x += extra_per_child;
            }
        }

        // Calculate positions based on alignment
        let mut current_x = padding.x; // Start from left padding
        
        for (child, size) in child_sizes {
            if let Some(child_rt) = rect_transforms.get_mut(&child) {
                // Calculate Y position based on alignment
                let y_pos = match layout.child_alignment {
                    Alignment::UpperLeft | Alignment::UpperCenter | Alignment::UpperRight => {
                        parent_rect.y + parent_rect.height - padding.w - size.y
                    }
                    Alignment::MiddleLeft | Alignment::MiddleCenter | Alignment::MiddleRight => {
                        parent_rect.y + (parent_rect.height - size.y) * 0.5
                    }
                    Alignment::LowerLeft | Alignment::LowerCenter | Alignment::LowerRight => {
                        parent_rect.y + padding.y
                    }
                };

                // Set anchors to top-left corner for absolute positioning
                child_rt.anchor_min = Vec2::new(0.0, 1.0);
                child_rt.anchor_max = Vec2::new(0.0, 1.0);
                child_rt.pivot = Vec2::new(0.0, 1.0);
                
                // Set position relative to parent's top-left
                let x_pos = parent_rect.x + current_x;
                child_rt.anchored_position = Vec2::new(x_pos - parent_rect.x, -(y_pos - (parent_rect.y + parent_rect.height)));
                child_rt.size_delta = size;
                child_rt.dirty = true;

                // Update rect for immediate use
                child_rt.rect.x = x_pos;
                child_rt.rect.y = y_pos;
                child_rt.rect.width = size.x;
                child_rt.rect.height = size.y;

                current_x += size.x + layout.spacing;
            }
        }
    }

    /// Apply vertical layout to children
    fn apply_vertical_layout(
        &mut self,
        parent_entity: Entity,
        layout: &VerticalLayoutGroup,
        children: &[Entity],
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        ui_elements: &HashMap<Entity, UIElement>,
    ) {
        // Get parent rect
        let parent_rect = match rect_transforms.get(&parent_entity) {
            Some(rt) => rt.rect,
            None => return,
        };

        // Filter children that should be laid out
        let layout_children: Vec<Entity> = children
            .iter()
            .copied()
            .filter(|&child| {
                ui_elements
                    .get(&child)
                    .map(|ui| !ui.ignore_layout)
                    .unwrap_or(false)
            })
            .collect();

        if layout_children.is_empty() {
            return;
        }

        // Calculate available space
        let padding = layout.padding;
        let available_width = parent_rect.width - padding.x - padding.z; // left and right padding
        let available_height = parent_rect.height - padding.y - padding.w; // bottom and top padding

        // Calculate total spacing
        let total_spacing = layout.spacing * (layout_children.len() as f32 - 1.0).max(0.0);

        // Calculate child sizes
        let mut child_sizes: Vec<(Entity, Vec2)> = Vec::new();
        let mut total_child_height = 0.0;

        for &child in &layout_children {
            if let Some(child_rt) = rect_transforms.get(&child) {
                let mut size = child_rt.size_delta;
                
                // Apply child control
                if layout.child_control_width {
                    size.x = available_width;
                }
                
                child_sizes.push((child, size));
                total_child_height += size.y;
            }
        }

        // Apply force expand height
        if layout.child_force_expand_height && total_child_height + total_spacing < available_height {
            let extra_space = available_height - total_spacing - total_child_height;
            let extra_per_child = extra_space / layout_children.len() as f32;
            
            for (_, size) in &mut child_sizes {
                size.y += extra_per_child;
            }
        }

        // Calculate positions based on alignment
        let mut current_y = parent_rect.height - padding.w; // Start from top padding
        
        for (child, size) in child_sizes {
            if let Some(child_rt) = rect_transforms.get_mut(&child) {
                // Calculate X position based on alignment
                let x_pos = match layout.child_alignment {
                    Alignment::UpperLeft | Alignment::MiddleLeft | Alignment::LowerLeft => {
                        parent_rect.x + padding.x
                    }
                    Alignment::UpperCenter | Alignment::MiddleCenter | Alignment::LowerCenter => {
                        parent_rect.x + (parent_rect.width - size.x) * 0.5
                    }
                    Alignment::UpperRight | Alignment::MiddleRight | Alignment::LowerRight => {
                        parent_rect.x + parent_rect.width - padding.z - size.x
                    }
                };

                // Set anchors to top-left corner for absolute positioning
                child_rt.anchor_min = Vec2::new(0.0, 1.0);
                child_rt.anchor_max = Vec2::new(0.0, 1.0);
                child_rt.pivot = Vec2::new(0.0, 1.0);
                
                // Set position relative to parent's top-left
                let y_pos = parent_rect.y + current_y - size.y;
                child_rt.anchored_position = Vec2::new(x_pos - parent_rect.x, -current_y);
                child_rt.size_delta = size;
                child_rt.dirty = true;

                // Update rect for immediate use
                child_rt.rect.x = x_pos;
                child_rt.rect.y = y_pos;
                child_rt.rect.width = size.x;
                child_rt.rect.height = size.y;

                current_y -= size.y + layout.spacing;
            }
        }
    }

    /// Apply grid layout to children
    fn apply_grid_layout(
        &mut self,
        parent_entity: Entity,
        layout: &GridLayoutGroup,
        children: &[Entity],
        rect_transforms: &mut HashMap<Entity, RectTransform>,
        ui_elements: &HashMap<Entity, UIElement>,
    ) {
        // Get parent rect
        let parent_rect = match rect_transforms.get(&parent_entity) {
            Some(rt) => rt.rect,
            None => return,
        };

        // Filter children that should be laid out
        let layout_children: Vec<Entity> = children
            .iter()
            .copied()
            .filter(|&child| {
                ui_elements
                    .get(&child)
                    .map(|ui| !ui.ignore_layout)
                    .unwrap_or(false)
            })
            .collect();

        if layout_children.is_empty() {
            return;
        }

        // Calculate available space
        let padding = layout.padding;
        let available_width = parent_rect.width - padding.x - padding.z;
        let available_height = parent_rect.height - padding.y - padding.w;

        // Calculate grid dimensions
        let (columns, rows) = self.calculate_grid_dimensions(
            layout_children.len(),
            &layout.constraint,
            layout.constraint_count,
            available_width,
            available_height,
            layout.cell_size,
            layout.spacing,
        );

        if columns == 0 || rows == 0 {
            return;
        }

        // Calculate starting position based on start corner
        let (start_x, start_y, x_dir, y_dir) = match layout.start_corner {
            Corner::UpperLeft => (padding.x, parent_rect.height - padding.w, 1.0, -1.0),
            Corner::UpperRight => (parent_rect.width - padding.z, parent_rect.height - padding.w, -1.0, -1.0),
            Corner::LowerLeft => (padding.x, padding.y, 1.0, 1.0),
            Corner::LowerRight => (parent_rect.width - padding.z, padding.y, -1.0, 1.0),
        };

        // Place children in grid
        for (index, &child) in layout_children.iter().enumerate() {
            if let Some(child_rt) = rect_transforms.get_mut(&child) {
                // Calculate grid position
                let (col, row) = match layout.start_axis {
                    Axis::Horizontal => (index % columns, index / columns),
                    Axis::Vertical => (index / rows, index % rows),
                };

                // Calculate cell position
                let cell_x = start_x + x_dir * (col as f32 * (layout.cell_size.x + layout.spacing.x));
                let cell_y = start_y + y_dir * (row as f32 * (layout.cell_size.y + layout.spacing.y));

                // Apply alignment within cell
                let (offset_x, offset_y) = self.calculate_cell_alignment_offset(
                    &layout.child_alignment,
                    layout.cell_size,
                    x_dir,
                    y_dir,
                );

                let final_x = cell_x + offset_x;
                let final_y = cell_y + offset_y;

                // Set anchors to top-left corner for absolute positioning
                child_rt.anchor_min = Vec2::new(0.0, 1.0);
                child_rt.anchor_max = Vec2::new(0.0, 1.0);
                child_rt.pivot = Vec2::new(0.0, 1.0);
                
                // Set position and size
                let world_x = parent_rect.x + final_x;
                let world_y = parent_rect.y + final_y - (if y_dir > 0.0 { layout.cell_size.y } else { 0.0 });
                
                child_rt.anchored_position = Vec2::new(
                    final_x,
                    -(parent_rect.height - final_y + (if y_dir > 0.0 { layout.cell_size.y } else { 0.0 }))
                );
                child_rt.size_delta = layout.cell_size;
                child_rt.dirty = true;

                // Update rect for immediate use
                child_rt.rect.x = world_x;
                child_rt.rect.y = world_y;
                child_rt.rect.width = layout.cell_size.x;
                child_rt.rect.height = layout.cell_size.y;
            }
        }
    }

    /// Calculate grid dimensions based on constraint
    fn calculate_grid_dimensions(
        &self,
        child_count: usize,
        constraint: &GridConstraint,
        constraint_count: i32,
        available_width: f32,
        available_height: f32,
        cell_size: Vec2,
        spacing: Vec2,
    ) -> (usize, usize) {
        match constraint {
            GridConstraint::Flexible => {
                // Calculate how many columns fit
                let columns = ((available_width + spacing.x) / (cell_size.x + spacing.x)).floor() as usize;
                let columns = columns.max(1);
                let rows = (child_count + columns - 1) / columns;
                (columns, rows)
            }
            GridConstraint::FixedColumnCount => {
                let columns = constraint_count.max(1) as usize;
                let rows = (child_count + columns - 1) / columns;
                (columns, rows)
            }
            GridConstraint::FixedRowCount => {
                let rows = constraint_count.max(1) as usize;
                let columns = (child_count + rows - 1) / rows;
                (columns, rows)
            }
        }
    }

    /// Calculate alignment offset within a grid cell
    fn calculate_cell_alignment_offset(
        &self,
        alignment: &Alignment,
        cell_size: Vec2,
        x_dir: f32,
        y_dir: f32,
    ) -> (f32, f32) {
        let (h_align, v_align) = match alignment {
            Alignment::UpperLeft => (0.0, 0.0),
            Alignment::UpperCenter => (0.5, 0.0),
            Alignment::UpperRight => (1.0, 0.0),
            Alignment::MiddleLeft => (0.0, 0.5),
            Alignment::MiddleCenter => (0.5, 0.5),
            Alignment::MiddleRight => (1.0, 0.5),
            Alignment::LowerLeft => (0.0, 1.0),
            Alignment::LowerCenter => (0.5, 1.0),
            Alignment::LowerRight => (1.0, 1.0),
        };

        // Adjust for direction
        let offset_x = if x_dir > 0.0 {
            h_align * cell_size.x
        } else {
            -h_align * cell_size.x
        };

        let offset_y = if y_dir > 0.0 {
            v_align * cell_size.y
        } else {
            -v_align * cell_size.y
        };

        (offset_x, offset_y)
    }
}

impl Default for LayoutSystem {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rect;

    fn create_test_rect_transform(x: f32, y: f32, width: f32, height: f32) -> RectTransform {
        let mut rt = RectTransform::default();
        rt.rect = Rect::new(x, y, width, height);
        rt.size_delta = Vec2::new(width, height);
        rt
    }

    fn create_test_ui_element() -> UIElement {
        UIElement::default()
    }

    #[test]
    fn test_horizontal_layout_basic() {
        let mut system = LayoutSystem::new();
        let mut rect_transforms = HashMap::new();
        let mut ui_elements = HashMap::new();
        let mut horizontal_layouts = HashMap::new();
        let mut children_map = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;
        let child3 = 4;

        // Create parent with 300x100 rect
        rect_transforms.insert(parent, create_test_rect_transform(0.0, 0.0, 300.0, 100.0));
        
        // Create children with 50x50 size
        rect_transforms.insert(child1, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child2, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child3, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));

        ui_elements.insert(parent, create_test_ui_element());
        ui_elements.insert(child1, create_test_ui_element());
        ui_elements.insert(child2, create_test_ui_element());
        ui_elements.insert(child3, create_test_ui_element());

        // Create horizontal layout with 10px spacing
        let mut layout = HorizontalLayoutGroup::default();
        layout.spacing = 10.0;
        layout.padding = glam::Vec4::ZERO;
        layout.child_force_expand_width = false;
        layout.child_force_expand_height = false;
        layout.child_control_width = false;
        layout.child_control_height = false;
        horizontal_layouts.insert(parent, layout);

        children_map.insert(parent, vec![child1, child2, child3]);

        system.update_layouts(
            &mut rect_transforms,
            &ui_elements,
            &horizontal_layouts,
            &HashMap::new(),
            &HashMap::new(),
            &children_map,
        );

        // Check that children are positioned horizontally with spacing
        let rt1 = rect_transforms.get(&child1).unwrap();
        let rt2 = rect_transforms.get(&child2).unwrap();
        let rt3 = rect_transforms.get(&child3).unwrap();

        // Child1 should be at x=0
        assert_eq!(rt1.rect.x, 0.0);
        // Child2 should be at x=60 (50 + 10 spacing)
        assert_eq!(rt2.rect.x, 60.0);
        // Child3 should be at x=120 (50 + 10 + 50 + 10)
        assert_eq!(rt3.rect.x, 120.0);
    }

    #[test]
    fn test_vertical_layout_basic() {
        let mut system = LayoutSystem::new();
        let mut rect_transforms = HashMap::new();
        let mut ui_elements = HashMap::new();
        let mut vertical_layouts = HashMap::new();
        let mut children_map = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;

        // Create parent with 100x300 rect
        rect_transforms.insert(parent, create_test_rect_transform(0.0, 0.0, 100.0, 300.0));
        
        // Create children with 50x50 size
        rect_transforms.insert(child1, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child2, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));

        ui_elements.insert(parent, create_test_ui_element());
        ui_elements.insert(child1, create_test_ui_element());
        ui_elements.insert(child2, create_test_ui_element());

        // Create vertical layout with 10px spacing
        let mut layout = VerticalLayoutGroup::default();
        layout.spacing = 10.0;
        layout.padding = glam::Vec4::ZERO;
        layout.child_force_expand_width = false;
        layout.child_force_expand_height = false;
        layout.child_control_width = false;
        layout.child_control_height = false;
        vertical_layouts.insert(parent, layout);

        children_map.insert(parent, vec![child1, child2]);

        system.update_layouts(
            &mut rect_transforms,
            &ui_elements,
            &HashMap::new(),
            &vertical_layouts,
            &HashMap::new(),
            &children_map,
        );

        // Check that children are positioned vertically with spacing
        let rt1 = rect_transforms.get(&child1).unwrap();
        let rt2 = rect_transforms.get(&child2).unwrap();

        // Child1 should be at top (y = 300 - 50 = 250)
        assert_eq!(rt1.rect.y, 250.0);
        // Child2 should be below with spacing (y = 300 - 50 - 10 - 50 = 190)
        assert_eq!(rt2.rect.y, 190.0);
    }

    #[test]
    fn test_grid_layout_basic() {
        let mut system = LayoutSystem::new();
        let mut rect_transforms = HashMap::new();
        let mut ui_elements = HashMap::new();
        let mut grid_layouts = HashMap::new();
        let mut children_map = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;
        let child3 = 4;
        let child4 = 5;

        // Create parent with 300x300 rect
        rect_transforms.insert(parent, create_test_rect_transform(0.0, 0.0, 300.0, 300.0));
        
        // Create children
        rect_transforms.insert(child1, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child2, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child3, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child4, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));

        ui_elements.insert(parent, create_test_ui_element());
        ui_elements.insert(child1, create_test_ui_element());
        ui_elements.insert(child2, create_test_ui_element());
        ui_elements.insert(child3, create_test_ui_element());
        ui_elements.insert(child4, create_test_ui_element());

        // Create grid layout with 2 columns
        let mut layout = GridLayoutGroup::default();
        layout.cell_size = Vec2::new(50.0, 50.0);
        layout.spacing = Vec2::new(10.0, 10.0);
        layout.padding = glam::Vec4::ZERO;
        layout.constraint = GridConstraint::FixedColumnCount;
        layout.constraint_count = 2;
        grid_layouts.insert(parent, layout);

        children_map.insert(parent, vec![child1, child2, child3, child4]);

        system.update_layouts(
            &mut rect_transforms,
            &ui_elements,
            &HashMap::new(),
            &HashMap::new(),
            &grid_layouts,
            &children_map,
        );

        // Check grid positions (2x2 grid)
        let rt1 = rect_transforms.get(&child1).unwrap();
        let rt2 = rect_transforms.get(&child2).unwrap();
        let rt3 = rect_transforms.get(&child3).unwrap();
        let rt4 = rect_transforms.get(&child4).unwrap();

        // First row
        assert_eq!(rt1.rect.x, 0.0);
        assert_eq!(rt2.rect.x, 60.0); // 50 + 10 spacing

        // Second row
        assert_eq!(rt3.rect.x, 0.0);
        assert_eq!(rt4.rect.x, 60.0);
    }

    #[test]
    fn test_horizontal_layout_with_padding() {
        let mut system = LayoutSystem::new();
        let mut rect_transforms = HashMap::new();
        let mut ui_elements = HashMap::new();
        let mut horizontal_layouts = HashMap::new();
        let mut children_map = HashMap::new();

        let parent = 1;
        let child1 = 2;

        rect_transforms.insert(parent, create_test_rect_transform(0.0, 0.0, 200.0, 100.0));
        rect_transforms.insert(child1, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));

        ui_elements.insert(parent, create_test_ui_element());
        ui_elements.insert(child1, create_test_ui_element());

        // Create layout with padding (left=10, bottom=10, right=10, top=10)
        let mut layout = HorizontalLayoutGroup::default();
        layout.padding = glam::Vec4::new(10.0, 10.0, 10.0, 10.0);
        layout.child_force_expand_width = false;
        layout.child_force_expand_height = false;
        layout.child_control_width = false;
        layout.child_control_height = false;
        horizontal_layouts.insert(parent, layout);

        children_map.insert(parent, vec![child1]);

        system.update_layouts(
            &mut rect_transforms,
            &ui_elements,
            &horizontal_layouts,
            &HashMap::new(),
            &HashMap::new(),
            &children_map,
        );

        let rt1 = rect_transforms.get(&child1).unwrap();
        
        // Child should start at x=10 (left padding)
        assert_eq!(rt1.rect.x, 10.0);
    }

    #[test]
    fn test_ignore_layout_flag() {
        let mut system = LayoutSystem::new();
        let mut rect_transforms = HashMap::new();
        let mut ui_elements = HashMap::new();
        let mut horizontal_layouts = HashMap::new();
        let mut children_map = HashMap::new();

        let parent = 1;
        let child1 = 2;
        let child2 = 3;

        rect_transforms.insert(parent, create_test_rect_transform(0.0, 0.0, 300.0, 100.0));
        rect_transforms.insert(child1, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));
        rect_transforms.insert(child2, create_test_rect_transform(0.0, 0.0, 50.0, 50.0));

        ui_elements.insert(parent, create_test_ui_element());
        
        // Child1 ignores layout
        let mut ui1 = create_test_ui_element();
        ui1.ignore_layout = true;
        ui_elements.insert(child1, ui1);
        
        ui_elements.insert(child2, create_test_ui_element());

        let mut layout = HorizontalLayoutGroup::default();
        layout.spacing = 10.0;
        layout.padding = glam::Vec4::ZERO;
        layout.child_force_expand_width = false;
        layout.child_force_expand_height = false;
        horizontal_layouts.insert(parent, layout);

        children_map.insert(parent, vec![child1, child2]);

        system.update_layouts(
            &mut rect_transforms,
            &ui_elements,
            &horizontal_layouts,
            &HashMap::new(),
            &HashMap::new(),
            &children_map,
        );

        // Child1 should not be affected by layout (still at 0,0)
        let rt1 = rect_transforms.get(&child1).unwrap();
        assert_eq!(rt1.rect.x, 0.0);
        
        // Child2 should be positioned as if it's the first child
        let rt2 = rect_transforms.get(&child2).unwrap();
        assert_eq!(rt2.rect.x, 0.0);
    }
}
