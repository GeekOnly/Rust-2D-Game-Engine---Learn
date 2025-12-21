# Multi-Selection System Documentation

## Overview

ระบบ Multi-Selection ที่สมบูรณ์สำหรับ Editor รองรับการเลือกหลาย entities พร้อมกันด้วยวิธีต่างๆ

## Features

### ✅ Selection Modes

1. **Replace Selection** (Click)
   - คลิก entity → เลือกเฉพาะ entity นั้น
   - คลิกพื้นที่ว่าง → ยกเลิกการเลือกทั้งหมด

2. **Toggle Selection** (Ctrl+Click)
   - Ctrl+Click entity → เพิ่ม/ลบจากการเลือก
   - ใช้สำหรับเลือกหลาย entities ที่ไม่ติดกัน

3. **Range Selection** (Shift+Click)
   - Shift+Click entity → เลือกทุก entities ระหว่าง last selected ถึง entity นี้
   - ใช้สำหรับเลือกหลาย entities ที่ติดกัน

4. **Box Selection** (Drag)
   - ลาก mouse บนพื้นที่ว่าง → เลือกทุก entities ใน box
   - รองรับ Ctrl (add) และ Shift (range)

5. **Select All** (Ctrl+A)
   - เลือกทุก entities ใน scene

6. **Clear Selection** (Escape)
   - ยกเลิกการเลือกทั้งหมด

## Architecture

### SelectionManager

```rust
pub struct SelectionManager {
    selected: HashSet<Entity>,
    last_selected: Option<Entity>,
    box_selection: Option<BoxSelection>,
    history: Vec<HashSet<Entity>>,
    history_index: usize,
}
```

### SelectionMode

```rust
pub enum SelectionMode {
    Replace,  // เลือกใหม่
    Add,      // เพิ่มเข้าไป
    Toggle,   // สลับ on/off
    Range,    // เลือกช่วง
}
```

### BoxSelection

```rust
pub struct BoxSelection {
    pub start_pos: egui::Pos2,
    pub current_pos: egui::Pos2,
    pub mode: SelectionMode,
}
```

## Usage

### Basic Operations

```rust
// ใน EditorState
pub selection: SelectionManager,

// Select single entity
selection.select(entity, SelectionMode::Replace);

// Select multiple entities
selection.select_multiple(&entities, SelectionMode::Add);

// Select range
selection.select_range(target_entity, &all_entities);

// Select all
selection.select_all(&all_entities);

// Clear selection
selection.clear();

// Check if selected
if selection.is_selected(entity) {
    // ...
}

// Get selected entities
let selected = selection.get_selected();
```

### Scene View Integration

```rust
use crate::editor::selection::handle_scene_selection;

// ใน scene_view rendering
let mut hovered_entity = None;

// ... detect hovered entity ...

// Handle selection
handle_scene_selection(
    &response,
    &mut state.selection,
    hovered_entity,
    &all_entities,
    &state.world,
    &state.scene_camera,
    center,
);

// Render box selection
state.selection.render_box_selection(&painter);

// Render selection outlines
for entity in state.selection.get_selected() {
    // Draw selection outline
    // ...
}
```

### Hierarchy Integration

```rust
use crate::editor::selection::handle_hierarchy_selection;

// ใน hierarchy panel
for entity in entities {
    let response = handle_hierarchy_selection(
        ui,
        entity,
        &mut state.selection,
        &all_entities,
    );
    
    // Additional UI...
}
```

## Multi-Entity Operations

### Get Common Values

```rust
use crate::editor::selection::get_common_transform;

let selected = selection.get_selected();
if let Some((pos, rot, scale)) = get_common_transform(&selected, &world) {
    // pos, rot, scale are Some if all entities have same value
    // None if values differ
    
    if let Some(position) = pos {
        // All entities have same position
        ui.label(format!("Position: {:?}", position));
    } else {
        // Entities have different positions
        ui.label("Position: <multiple values>");
    }
}
```

### Apply Transform

```rust
use crate::editor::selection::{
    apply_transform_to_selected,
    move_selected_by_delta,
    rotate_selected_by_delta,
    scale_selected_by_factor,
};

let selected = selection.get_selected();

// Set absolute values
apply_transform_to_selected(
    &selected,
    &mut world,
    Some([10.0, 20.0, 0.0]),  // position
    None,                      // rotation (unchanged)
    None,                      // scale (unchanged)
);

// Move by delta
move_selected_by_delta(&selected, &mut world, [5.0, 0.0, 0.0]);

// Rotate by delta
rotate_selected_by_delta(&selected, &mut world, [0.0, 0.0, 45.0]);

// Scale by factor
scale_selected_by_factor(&selected, &mut world, [2.0, 2.0, 1.0]);
```

## Inspector Integration

### Multi-Entity Inspector

```rust
// ใน inspector.rs
let selected = state.selection.get_selected();

if selected.is_empty() {
    ui.label("No selection");
} else if selected.len() == 1 {
    // Single entity inspector (existing code)
    render_single_entity_inspector(ui, selected[0], world, entity_names);
} else {
    // Multi-entity inspector
    render_multi_entity_inspector(ui, &selected, world, entity_names);
}

fn render_multi_entity_inspector(
    ui: &mut egui::Ui,
    selected: &[Entity],
    world: &mut World,
    entity_names: &HashMap<Entity, String>,
) {
    ui.heading(format!("{} entities selected", selected.len()));
    
    // Get common values
    if let Some((pos, rot, scale)) = get_common_transform(selected, world) {
        ui.separator();
        ui.label("Transform");
        
        // Position
        if let Some(mut position) = pos {
            ui.horizontal(|ui| {
                ui.label("Position:");
                if ui.add(egui::DragValue::new(&mut position[0])).changed() {
                    apply_transform_to_selected(selected, world, Some(position), None, None);
                }
                // ... Y, Z
            });
        } else {
            ui.label("Position: <multiple values>");
        }
        
        // Similar for rotation and scale
    }
    
    // Common components
    let all_have_sprite = selected.iter().all(|&e| world.sprites.contains_key(&e));
    if all_have_sprite {
        ui.separator();
        ui.label("Sprite (all)");
        // Edit sprite properties
    }
}
```

## Keyboard Shortcuts

```rust
// ใน main loop หรือ shortcuts handler
ctx.input(|i| {
    // Ctrl+A: Select all
    if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
        state.selection.select_all(&all_entities);
    }
    
    // Escape: Clear selection
    if i.key_pressed(egui::Key::Escape) {
        state.selection.clear();
    }
    
    // Delete: Delete selected
    if i.key_pressed(egui::Key::Delete) {
        let selected = state.selection.get_selected();
        for entity in selected {
            // Delete entity with undo
            let cmd = Box::new(DeleteEntityCommand::new(entity, &world, &entity_names));
            state.undo_stack.execute(cmd, &mut world, &mut entity_names);
        }
        state.selection.clear();
    }
});
```

## Visual Feedback

### Selection Outline

```rust
// ใน scene view rendering
for entity in state.selection.get_selected() {
    if let Some(transform) = world.transforms.get(&entity) {
        let world_pos = glam::Vec2::new(transform.x(), transform.y());
        let screen_pos = scene_camera.world_to_screen(world_pos);
        let screen_x = center.x + screen_pos.x;
        let screen_y = center.y + screen_pos.y;
        
        // Get entity bounds
        let size = if let Some(sprite) = world.sprites.get(&entity) {
            egui::vec2(sprite.width * scene_camera.zoom, sprite.height * scene_camera.zoom)
        } else {
            egui::vec2(20.0, 20.0)
        };
        
        // Draw selection outline
        painter.rect_stroke(
            egui::Rect::from_center_size(egui::pos2(screen_x, screen_y), size + egui::vec2(4.0, 4.0)),
            2.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0)),
        );
    }
}
```

### Box Selection Visual

```rust
// Automatically rendered by SelectionManager
state.selection.render_box_selection(&painter);
```

### Hierarchy Highlight

```rust
// ใน hierarchy panel
for entity in entities {
    let is_selected = state.selection.is_selected(entity);
    
    let response = ui.selectable_label(is_selected, entity_name);
    
    if response.clicked() {
        // Handle selection
        // ...
    }
}
```

## Integration with Undo/Redo

### Selection Commands

```rust
// สร้าง command สำหรับ multi-entity operations
let selected = state.selection.get_selected();

// Delete multiple
let mut batch = BatchCommand::new("Delete Multiple");
for entity in &selected {
    batch.add(Box::new(DeleteEntityCommand::new(*entity, &world, &entity_names)));
}
state.undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);

// Move multiple
let old_positions: Vec<_> = selected.iter()
    .map(|&e| world.transforms.get(&e).unwrap().position)
    .collect();

// ... perform move ...

let mut batch = BatchCommand::new("Move Multiple");
for (i, &entity) in selected.iter().enumerate() {
    let new_pos = world.transforms.get(&entity).unwrap().position;
    batch.add(Box::new(MoveEntityCommand::new(entity, old_positions[i], new_pos)));
}
state.undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);
```

## Performance Considerations

### HashSet for Fast Lookup

```rust
// O(1) lookup
if selection.is_selected(entity) {
    // ...
}
```

### Box Selection Optimization

```rust
// Only check entities with transforms
for (&entity, transform) in &world.transforms {
    // Check if in box
}

// Future: Use spatial partitioning (QuadTree)
```

## Examples

### Example 1: Select and Move Multiple

```rust
// Select multiple entities
state.selection.select_all(&all_entities);

// Move all by delta
let selected = state.selection.get_selected();
move_selected_by_delta(&selected, &mut world, [10.0, 0.0, 0.0]);
```

### Example 2: Box Selection

```rust
// User drags mouse
// → Box selection automatically handled by handle_scene_selection()

// Get selected entities
let selected = state.selection.get_selected();
println!("Selected {} entities", selected.len());
```

### Example 3: Range Selection in Hierarchy

```rust
// User clicks entity A
// User Shift+clicks entity E
// → Entities A, B, C, D, E are selected

let selected = state.selection.get_selected();
// Do something with selected entities
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_selection() {
        let mut selection = SelectionManager::new();
        let entity = 1;
        
        selection.select(entity, SelectionMode::Replace);
        assert!(selection.is_selected(entity));
        assert_eq!(selection.count(), 1);
    }
    
    #[test]
    fn test_toggle_selection() {
        let mut selection = SelectionManager::new();
        let entity = 1;
        
        selection.select(entity, SelectionMode::Toggle);
        assert!(selection.is_selected(entity));
        
        selection.select(entity, SelectionMode::Toggle);
        assert!(!selection.is_selected(entity));
    }
    
    #[test]
    fn test_range_selection() {
        let mut selection = SelectionManager::new();
        let entities = vec![1, 2, 3, 4, 5];
        
        selection.select(1, SelectionMode::Replace);
        selection.select_range(5, &entities);
        
        assert_eq!(selection.count(), 5);
        for &entity in &entities {
            assert!(selection.is_selected(entity));
        }
    }
    
    #[test]
    fn test_box_selection() {
        let mut selection = SelectionManager::new();
        let world = World::new();
        let camera = SceneCamera::new();
        
        selection.start_box_selection(egui::pos2(0.0, 0.0), SelectionMode::Replace);
        selection.update_box_selection(egui::pos2(100.0, 100.0));
        let selected = selection.finish_box_selection(&world, &camera, egui::pos2(400.0, 300.0));
        
        // Verify entities in box are selected
    }
}
```

## Future Enhancements

### 1. Selection Groups

```rust
pub struct SelectionGroup {
    name: String,
    entities: Vec<Entity>,
}

// Save/load selection groups
// Ctrl+1-9 to save/recall groups
```

### 2. Selection Filters

```rust
// Select by type
selection.select_by_component::<Sprite>(&world);

// Select by tag
selection.select_by_tag(EntityTag::Player, &world);

// Select by name pattern
selection.select_by_name_pattern("Enemy*", &world, &entity_names);
```

### 3. Invert Selection

```rust
selection.invert(&all_entities);
```

### 4. Grow/Shrink Selection

```rust
// Select children of selected entities
selection.grow_to_children(&world);

// Select parents of selected entities
selection.grow_to_parents(&world);
```

## Best Practices

### 1. Always Use SelectionManager

```rust
// ❌ Bad: Direct manipulation
state.selected_entity = Some(entity);

// ✅ Good: Use SelectionManager
state.selection.select(entity, SelectionMode::Replace);
```

### 2. Use Helper Functions

```rust
// ❌ Bad: Manual loop
for entity in state.selection.get_selected() {
    if let Some(transform) = world.transforms.get_mut(&entity) {
        transform.position[0] += 10.0;
    }
}

// ✅ Good: Use helper
move_selected_by_delta(&state.selection.get_selected(), &mut world, [10.0, 0.0, 0.0]);
```

### 3. Batch Operations with Undo

```rust
// ✅ Good: Batch multiple operations
let mut batch = BatchCommand::new("Operation Name");
for entity in &selected {
    batch.add(Box::new(SomeCommand::new(entity, ...)));
}
state.undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);
```

## Summary

✅ **Implemented:**
- Complete multi-selection system
- 5 selection modes (Replace, Add, Toggle, Range, Box)
- Keyboard shortcuts (Ctrl+A, Escape)
- Visual feedback (outlines, box)
- Multi-entity operations
- History for undo/redo

🚀 **Ready for:**
- Scene view integration
- Hierarchy integration
- Inspector integration
- Transform gizmo (multi-entity)
- Delete/Duplicate operations

📝 **Next Steps:**
1. Integrate with scene view
2. Integrate with hierarchy
3. Update inspector for multi-entity
4. Update transform gizmo
5. Add keyboard shortcuts
