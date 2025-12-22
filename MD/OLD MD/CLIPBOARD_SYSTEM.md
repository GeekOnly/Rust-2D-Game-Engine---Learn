# Copy/Paste/Duplicate System Documentation

## Overview

ระบบ Clipboard ที่สมบูรณ์สำหรับ Editor รองรับการ copy, paste, duplicate entities พร้อมทั้ง components และ hierarchy

## Features

### ✅ Operations

1. **Copy (Ctrl+C)**
   - คัดลอก selected entities ไปยัง clipboard
   - รักษา components ทั้งหมด
   - รักษา hierarchy (parent-child relationships)

2. **Paste (Ctrl+V)**
   - วาง entities จาก clipboard
   - สร้าง entities ใหม่พร้อม components
   - Offset position เพื่อไม่ให้ซ้อนกัน
   - รักษา hierarchy

3. **Duplicate (Ctrl+D)**
   - Duplicate selected entities ทันที
   - ไม่ต้องผ่าน clipboard
   - Offset position อัตโนมัติ

4. **Cut (Ctrl+X)**
   - Copy + Delete
   - ใช้ undo/redo system

## Architecture

### Clipboard

```rust
pub struct Clipboard {
    data: Option<ClipboardData>,
}

pub struct ClipboardData {
    pub entities: Vec<EntityClipboardData>,
    pub hierarchy: Vec<(usize, usize)>,
}

pub struct EntityClipboardData {
    pub name: String,
    pub transform: Option<Transform>,
    pub sprite: Option<Sprite>,
    // ... all components
}
```

### ClipboardAction

```rust
pub enum ClipboardAction {
    None,
    Copied(usize),
    Pasted(Vec<Entity>),
    Duplicated(Vec<Entity>),
    Cut(usize),
}
```

## Usage

### Basic Operations

```rust
// ใน EditorState
pub clipboard: Clipboard,

// Copy
clipboard.copy_entity(entity, &world, &entity_names);
clipboard.copy_entities(&entities, &world, &entity_names);

// Paste
let new_entities = clipboard.paste(&mut world, &mut entity_names, Some([10.0, 10.0, 0.0]));

// Duplicate
let new_entity = clipboard.duplicate_entity(entity, &mut world, &entity_names);
let new_entities = clipboard.duplicate_entities(&entities, &mut world, &entity_names);

// Check clipboard
if clipboard.has_data() {
    println!("Clipboard has {} entities", clipboard.count());
}
```

### Keyboard Shortcuts

```rust
use crate::editor::clipboard::handle_clipboard_shortcuts;

// ใน main loop
let action = handle_clipboard_shortcuts(
    ctx,
    &mut state.clipboard,
    &state.selection.get_selected(),
    &mut state.world,
    &mut state.entity_names,
    &mut state.undo_stack,
);

// Handle action result
if let Some(message) = action.message() {
    state.console.info(message);
}

match action {
    ClipboardAction::Pasted(entities) => {
        // Select pasted entities
        state.selection.select_multiple(&entities, SelectionMode::Replace);
    }
    ClipboardAction::Duplicated(entities) => {
        // Select duplicated entities
        state.selection.select_multiple(&entities, SelectionMode::Replace);
    }
    _ => {}
}
```

### Menu Integration

```rust
// ใน menu_bar.rs
ui.menu_button("Edit", |ui| {
    // Copy
    if ui.add_enabled(
        state.selection.has_selection(),
        egui::Button::new("Copy")
    ).on_hover_text("Ctrl+C").clicked() {
        copy_selected(
            &mut state.clipboard,
            &state.selection.get_selected(),
            &state.world,
            &state.entity_names,
        );
        ui.close_menu();
    }
    
    // Paste
    if ui.add_enabled(
        state.clipboard.has_data(),
        egui::Button::new("Paste")
    ).on_hover_text("Ctrl+V").clicked() {
        let new_entities = paste_from_clipboard(
            &state.clipboard,
            &mut state.world,
            &mut state.entity_names,
            Some([10.0, 10.0, 0.0]),
        );
        
        // Create undo command
        let mut batch = BatchCommand::new("Paste");
        for &entity in &new_entities {
            batch.add(Box::new(CreateEntityCommand::new(entity, &state.world, &state.entity_names)));
        }
        state.undo_stack.execute(Box::new(batch), &mut state.world, &mut state.entity_names);
        
        // Select pasted entities
        state.selection.select_multiple(&new_entities, SelectionMode::Replace);
        ui.close_menu();
    }
    
    // Duplicate
    if ui.add_enabled(
        state.selection.has_selection(),
        egui::Button::new("Duplicate")
    ).on_hover_text("Ctrl+D").clicked() {
        let selected = state.selection.get_selected();
        let new_entities = duplicate_selected(
            &state.clipboard,
            &selected,
            &mut state.world,
            &state.entity_names,
        );
        
        // Create undo command
        let mut batch = BatchCommand::new("Duplicate");
        for &entity in &new_entities {
            batch.add(Box::new(CreateEntityCommand::new(entity, &state.world, &state.entity_names)));
        }
        state.undo_stack.execute(Box::new(batch), &mut state.world, &mut state.entity_names);
        
        // Select duplicated entities
        state.selection.select_multiple(&new_entities, SelectionMode::Replace);
        ui.close_menu();
    }
    
    // Cut
    if ui.add_enabled(
        state.selection.has_selection(),
        egui::Button::new("Cut")
    ).on_hover_text("Ctrl+X").clicked() {
        let selected = state.selection.get_selected();
        
        // Copy first
        copy_selected(&mut state.clipboard, &selected, &state.world, &state.entity_names);
        
        // Then delete
        let mut batch = BatchCommand::new("Cut");
        for &entity in &selected {
            batch.add(Box::new(DeleteEntityCommand::new(entity, &state.world, &state.entity_names)));
        }
        state.undo_stack.execute(Box::new(batch), &mut state.world, &mut state.entity_names);
        
        state.selection.clear();
        ui.close_menu();
    }
});
```

## Component Preservation

### All Components Copied

```rust
// Transform
pub transform: Option<Transform>,

// Rendering
pub sprite: Option<Sprite>,
pub mesh: Option<Mesh>,
pub camera: Option<Camera>,

// Physics
pub collider: Option<Collider>,
pub velocity: Option<(f32, f32)>,

// Gameplay
pub tag: Option<EntityTag>,
pub script: Option<Script>,

// Metadata
pub active: bool,
pub layer: u8,
```

### Example

```rust
// Original entity
Entity {
    name: "Player",
    transform: Transform { position: [0, 0, 0], ... },
    sprite: Sprite { texture: "player.png", ... },
    collider: Collider { width: 40, height: 40 },
    velocity: (0, 0),
    tag: EntityTag::Player,
    script: Script { path: "player.lua" },
}

// After paste
Entity {
    name: "Player (Copy)",
    transform: Transform { position: [10, 10, 0], ... }, // Offset
    sprite: Sprite { texture: "player.png", ... },       // Same
    collider: Collider { width: 40, height: 40 },        // Same
    velocity: (0, 0),                                     // Same
    tag: EntityTag::Player,                               // Same
    script: Script { path: "player.lua" },               // Same
}
```

## Hierarchy Preservation

### Parent-Child Relationships

```rust
// Original hierarchy
Parent
├── Child A
└── Child B
    └── Grandchild

// After copy & paste (all selected)
Parent (Copy)
├── Child A (Copy)
└── Child B (Copy)
    └── Grandchild (Copy)

// Hierarchy is preserved!
```

### Partial Selection

```rust
// Original
Parent
├── Child A (selected)
└── Child B (selected)

// After paste
Child A (Copy)  // No parent (parent not selected)
Child B (Copy)  // No parent
```

## Offset Behavior

### Default Offset

```rust
// Paste with default offset
let offset = Some([10.0, 10.0, 0.0]);
clipboard.paste(&mut world, &mut entity_names, offset);

// Original at (0, 0, 0)
// Pasted at (10, 10, 0)
```

### No Offset

```rust
// Paste at same position
clipboard.paste(&mut world, &mut entity_names, None);

// Original at (0, 0, 0)
// Pasted at (0, 0, 0) - overlapping!
```

### Custom Offset

```rust
// Paste at mouse position
let mouse_world_pos = scene_camera.screen_to_world(mouse_pos);
let offset = Some([mouse_world_pos.x, mouse_world_pos.y, 0.0]);
clipboard.paste(&mut world, &mut entity_names, offset);
```

## Name Generation

### Automatic Naming

```rust
// Original: "Player"
// Copy 1: "Player (Copy)"
// Copy 2: "Player (Copy)" // Same name (user can rename)

// Duplicate: "Player (Copy)"
```

### Custom Suffix

```rust
// Internal API
entity_data.create_in_world(world, entity_names, Some("(Duplicate)"));
// Result: "Player (Duplicate)"
```

## Integration with Undo/Redo

### Paste with Undo

```rust
let new_entities = paste_from_clipboard(&clipboard, &mut world, &mut entity_names, offset);

// Create undo command
let mut batch = BatchCommand::new("Paste");
for &entity in &new_entities {
    batch.add(Box::new(CreateEntityCommand::new(entity, &world, &entity_names)));
}
undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);

// Now can undo paste!
```

### Duplicate with Undo

```rust
let new_entities = duplicate_selected(&clipboard, &selected, &mut world, &entity_names);

let mut batch = BatchCommand::new("Duplicate");
for &entity in &new_entities {
    batch.add(Box::new(CreateEntityCommand::new(entity, &world, &entity_names)));
}
undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);
```

### Cut with Undo

```rust
// Copy first
copy_selected(&mut clipboard, &selected, &world, &entity_names);

// Delete with undo
let mut batch = BatchCommand::new("Cut");
for &entity in &selected {
    batch.add(Box::new(DeleteEntityCommand::new(entity, &world, &entity_names)));
}
undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);
```

## Serialization

### JSON Format

```json
{
  "entities": [
    {
      "name": "Player",
      "transform": {
        "position": [0.0, 0.0, 0.0],
        "rotation": [0.0, 0.0, 0.0],
        "scale": [1.0, 1.0, 1.0]
      },
      "sprite": {
        "texture_id": "player",
        "width": 40.0,
        "height": 40.0,
        "color": [1.0, 1.0, 1.0, 1.0]
      },
      "active": true,
      "layer": 0
    }
  ],
  "hierarchy": []
}
```

### System Clipboard (Optional)

```rust
#[cfg(feature = "system-clipboard")]
use crate::editor::clipboard::system_clipboard;

// Copy to system clipboard
system_clipboard::copy_to_system(&clipboard)?;

// Paste from system clipboard
system_clipboard::paste_from_system(&mut clipboard)?;

// Can paste between different editor instances!
```

## Performance Considerations

### Memory Usage

```rust
// EntityClipboardData: ~500 bytes per entity
// 100 entities = 50 KB
// Acceptable for clipboard
```

### Deep Copy

```rust
// All components are cloned
// No shared references
// Safe for modification
```

## Examples

### Example 1: Copy & Paste

```rust
// Select entities
state.selection.select_all(&all_entities);

// Copy (Ctrl+C)
copy_selected(&mut state.clipboard, &state.selection.get_selected(), &state.world, &state.entity_names);

// Paste (Ctrl+V)
let new_entities = paste_from_clipboard(&state.clipboard, &mut state.world, &mut state.entity_names, Some([10.0, 10.0, 0.0]));

// Select pasted
state.selection.select_multiple(&new_entities, SelectionMode::Replace);
```

### Example 2: Duplicate

```rust
// Select entity
state.selection.select(entity, SelectionMode::Replace);

// Duplicate (Ctrl+D)
let new_entities = duplicate_selected(&state.clipboard, &state.selection.get_selected(), &mut state.world, &state.entity_names);

// Select duplicated
state.selection.select_multiple(&new_entities, SelectionMode::Replace);
```

### Example 3: Cut & Paste

```rust
// Select entities
state.selection.select_multiple(&entities, SelectionMode::Replace);

// Cut (Ctrl+X)
copy_selected(&mut state.clipboard, &state.selection.get_selected(), &state.world, &state.entity_names);
// ... delete entities ...

// Move to different location
// ...

// Paste (Ctrl+V)
let new_entities = paste_from_clipboard(&state.clipboard, &mut state.world, &mut state.entity_names, None);
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_paste() {
        let mut world = World::new();
        let mut entity_names = HashMap::new();
        let mut clipboard = Clipboard::new();
        
        // Create entity
        let entity = world.spawn();
        entity_names.insert(entity, "Test".to_string());
        world.transforms.insert(entity, Transform::default());
        
        // Copy
        clipboard.copy_entity(entity, &world, &entity_names);
        assert!(clipboard.has_data());
        
        // Paste
        let new_entities = clipboard.paste(&mut world, &mut entity_names, Some([10.0, 0.0, 0.0]));
        assert_eq!(new_entities.len(), 1);
        
        // Verify
        let new_entity = new_entities[0];
        assert!(world.transforms.contains_key(&new_entity));
        assert_eq!(world.transforms.get(&new_entity).unwrap().position[0], 10.0);
    }
    
    #[test]
    fn test_duplicate() {
        let mut world = World::new();
        let entity_names = HashMap::new();
        let clipboard = Clipboard::new();
        
        let entity = world.spawn();
        world.transforms.insert(entity, Transform::default());
        
        let new_entity = clipboard.duplicate_entity(entity, &mut world, &entity_names);
        assert!(new_entity.is_some());
    }
}
```

## Best Practices

### 1. Always Use Undo

```rust
// ✅ Good: With undo
let new_entities = paste_from_clipboard(...);
let mut batch = BatchCommand::new("Paste");
for &entity in &new_entities {
    batch.add(Box::new(CreateEntityCommand::new(entity, ...)));
}
undo_stack.execute(Box::new(batch), ...);
```

### 2. Select After Paste

```rust
// ✅ Good: Select pasted entities
let new_entities = paste_from_clipboard(...);
state.selection.select_multiple(&new_entities, SelectionMode::Replace);
```

### 3. Show Feedback

```rust
// ✅ Good: Show message
let action = handle_clipboard_shortcuts(...);
if let Some(message) = action.message() {
    state.console.info(message);
}
```

## Summary

✅ **Implemented:**
- Complete clipboard system
- Copy/Paste/Cut/Duplicate operations
- Component preservation
- Hierarchy preservation
- Undo/Redo integration
- Keyboard shortcuts
- JSON serialization

🚀 **Ready for:**
- Menu integration
- Keyboard shortcuts (Ctrl+C/V/D/X)
- Multi-entity operations
- System clipboard (optional)

📝 **Next Steps:**
1. Add keyboard shortcuts to main loop
2. Add menu items (Edit → Copy/Paste/etc.)
3. Add console feedback
4. Test with complex hierarchies
5. Optional: System clipboard integration
