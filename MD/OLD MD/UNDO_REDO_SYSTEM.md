# Undo/Redo System Documentation

## Overview

ระบบ Undo/Redo ที่สมบูรณ์สำหรับ Editor ใช้ Command Pattern เพื่อให้สามารถ undo/redo การกระทำต่างๆ ได้

## Architecture

### Command Pattern

```rust
pub trait Command {
    fn execute(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>);
    fn undo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>);
    fn redo(&mut self, world: &mut World, entity_names: &mut HashMap<Entity, String>);
    fn description(&self) -> String;
    fn can_merge(&self, other: &dyn Command) -> bool;
    fn merge(&mut self, other: Box<dyn Command>);
}
```

### UndoStack

```rust
pub struct UndoStack {
    commands: Vec<Box<dyn Command>>,
    current_index: usize,
    max_size: usize,
    saved_index: Option<usize>,
}
```

## Features

### ✅ Implemented Commands

1. **CreateEntityCommand** - สร้าง entity ใหม่
2. **DeleteEntityCommand** - ลบ entity
3. **MoveEntityCommand** - เคลื่อนที่ entity (รองรับ merge)
4. **RotateEntityCommand** - หมุน entity
5. **ScaleEntityCommand** - ขยาย/ย่อ entity
6. **RenameEntityCommand** - เปลี่ยนชื่อ entity
7. **BatchCommand** - รวมหลาย commands

### ✅ Key Features

- **Unlimited Undo/Redo** (จำกัดที่ 100 steps)
- **Command Merging** - รวม commands ที่ต่อเนื่องกัน (เช่น drag)
- **Saved State Tracking** - ตรวจสอบว่า scene ถูก save แล้วหรือยัง
- **Batch Operations** - ทำหลาย operations พร้อมกัน
- **Memory Management** - จำกัดจำนวน commands ใน memory

## Usage

### Basic Usage

```rust
// ใน EditorState
pub undo_stack: UndoStack,

// Execute command
let command = Box::new(MoveEntityCommand::new(entity, old_pos, new_pos));
state.undo_stack.execute(command, &mut state.world, &mut state.entity_names);

// Undo
if state.undo_stack.can_undo() {
    state.undo_stack.undo(&mut state.world, &mut state.entity_names);
}

// Redo
if state.undo_stack.can_redo() {
    state.undo_stack.redo(&mut state.world, &mut state.entity_names);
}
```

### Keyboard Shortcuts

```rust
// ใน shortcuts.rs หรือ main loop
if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
    // Undo
    state.undo_stack.undo(&mut state.world, &mut state.entity_names);
}

if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
    // Redo
    state.undo_stack.redo(&mut state.world, &mut state.entity_names);
}

// หรือ Ctrl+Y สำหรับ Redo
if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
    state.undo_stack.redo(&mut state.world, &mut state.entity_names);
}
```

### Menu Integration

```rust
// ใน menu_bar.rs
ui.menu_button("Edit", |ui| {
    // Undo
    let undo_text = if let Some(desc) = state.undo_stack.undo_description() {
        format!("Undo {}", desc)
    } else {
        "Undo".to_string()
    };
    
    if ui.add_enabled(state.undo_stack.can_undo(), egui::Button::new(undo_text))
        .on_hover_text("Ctrl+Z")
        .clicked() 
    {
        state.undo_stack.undo(&mut state.world, &mut state.entity_names);
    }
    
    // Redo
    let redo_text = if let Some(desc) = state.undo_stack.redo_description() {
        format!("Redo {}", desc)
    } else {
        "Redo".to_string()
    };
    
    if ui.add_enabled(state.undo_stack.can_redo(), egui::Button::new(redo_text))
        .on_hover_text("Ctrl+Shift+Z or Ctrl+Y")
        .clicked() 
    {
        state.undo_stack.redo(&mut state.world, &mut state.entity_names);
    }
});
```

## Command Examples

### 1. Create Entity

```rust
// เมื่อสร้าง entity ใหม่
let entity = world.spawn();
// ... setup entity components ...

let command = Box::new(CreateEntityCommand::new(entity, &world, &entity_names));
undo_stack.execute(command, &mut world, &mut entity_names);
```

### 2. Delete Entity

```rust
// เมื่อลบ entity
let command = Box::new(DeleteEntityCommand::new(entity, &world, &entity_names));
undo_stack.execute(command, &mut world, &mut entity_names);
```

### 3. Move Entity

```rust
// เมื่อเริ่ม drag
let old_pos = transform.position;

// เมื่อจบ drag
let new_pos = transform.position;
let command = Box::new(MoveEntityCommand::new(entity, old_pos, new_pos));
undo_stack.execute(command, &mut world, &mut entity_names);
```

### 4. Batch Operations

```rust
// สำหรับ operations ที่ต้องทำพร้อมกัน
let mut batch = BatchCommand::new("Delete Multiple Entities");

for entity in selected_entities {
    let cmd = Box::new(DeleteEntityCommand::new(entity, &world, &entity_names));
    batch.add(cmd);
}

undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);
```

## Command Merging

### How It Works

```rust
impl Command for MoveEntityCommand {
    fn can_merge(&self, other: &dyn Command) -> bool {
        // ตรวจสอบว่าเป็น entity เดียวกันและ position ใกล้กัน
        if let Some(other_move) = other.downcast_ref::<MoveEntityCommand>() {
            if self.entity == other_move.entity {
                let dist = calculate_distance(self.new_position, other_move.old_position);
                return dist < 0.1; // Merge threshold
            }
        }
        false
    }
    
    fn merge(&mut self, other: Box<dyn Command>) {
        // รวม commands โดยเก็บ old_position จาก self และ new_position จาก other
        if let Ok(other_move) = other.downcast::<MoveEntityCommand>() {
            self.new_position = other_move.new_position;
        }
    }
}
```

### Benefits

- ลดจำนวน undo steps เมื่อ drag
- ประหยัด memory
- UX ดีขึ้น (undo ทั้ง drag แทนที่จะเป็นทีละ pixel)

## Saved State Tracking

```rust
// เมื่อ save scene
undo_stack.mark_saved();

// ตรวจสอบว่า scene modified หรือไม่
let is_modified = !undo_stack.is_saved();

// แสดง * ใน title bar ถ้า modified
let title = if is_modified {
    format!("{}* - Editor", scene_name)
} else {
    format!("{} - Editor", scene_name)
};
```

## Integration Points

### 1. Transform Gizmo

```rust
// ใน interaction/transform.rs
pub fn handle_gizmo_interaction_with_undo(
    response: &egui::Response,
    entity: Entity,
    world: &mut World,
    undo_stack: &mut UndoStack,
    entity_names: &mut HashMap<Entity, String>,
    // ... other params
) {
    // เมื่อเริ่ม drag
    if response.drag_started() {
        // เก็บ old position
        let old_pos = world.transforms.get(&entity).unwrap().position;
        // ... store in state
    }
    
    // เมื่อ drag
    if response.dragged() {
        // อัพเดท position
        // ...
    }
    
    // เมื่อจบ drag
    if response.drag_released() {
        let new_pos = world.transforms.get(&entity).unwrap().position;
        let command = Box::new(MoveEntityCommand::new(entity, old_pos, new_pos));
        undo_stack.execute(command, world, entity_names);
    }
}
```

### 2. Hierarchy Panel

```rust
// ใน hierarchy.rs
if ui.button("Delete").clicked() {
    let command = Box::new(DeleteEntityCommand::new(entity, world, entity_names));
    undo_stack.execute(command, world, entity_names);
}

if ui.button("Duplicate").clicked() {
    // Create new entity
    let new_entity = world.spawn();
    // ... copy components ...
    
    let command = Box::new(CreateEntityCommand::new(new_entity, world, entity_names));
    undo_stack.execute(command, world, entity_names);
}
```

### 3. Inspector

```rust
// ใน inspector.rs
// เมื่อแก้ไข component
if ui.button("Apply").clicked() {
    // สร้าง command ที่เหมาะสม
    // ...
    undo_stack.execute(command, world, entity_names);
}
```

## Performance Considerations

### Memory Usage

```rust
// จำกัดจำนวน commands
const MAX_UNDO_STEPS: usize = 100;

// ประมาณการ memory:
// - EntityData: ~500 bytes per entity
// - 100 commands × 500 bytes = 50 KB
// - ยอมรับได้สำหรับ editor
```

### Command Merging

```rust
// ลด commands จาก 1000+ (drag) → 1 (merged)
// ประหยัด memory 99%
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut world = World::new();
        let mut entity_names = HashMap::new();
        let mut undo_stack = UndoStack::new();
        
        let entity = world.spawn();
        let old_pos = [0.0, 0.0, 0.0];
        let new_pos = [10.0, 0.0, 0.0];
        
        // Execute
        let command = Box::new(MoveEntityCommand::new(entity, old_pos, new_pos));
        undo_stack.execute(command, &mut world, &mut entity_names);
        
        // Verify
        assert_eq!(world.transforms.get(&entity).unwrap().position, new_pos);
        
        // Undo
        undo_stack.undo(&mut world, &mut entity_names);
        assert_eq!(world.transforms.get(&entity).unwrap().position, old_pos);
        
        // Redo
        undo_stack.redo(&mut world, &mut entity_names);
        assert_eq!(world.transforms.get(&entity).unwrap().position, new_pos);
    }
    
    #[test]
    fn test_command_merging() {
        let mut undo_stack = UndoStack::new();
        
        // Execute multiple move commands
        for i in 0..10 {
            let old_pos = [i as f32, 0.0, 0.0];
            let new_pos = [(i + 1) as f32, 0.0, 0.0];
            let command = Box::new(MoveEntityCommand::new(entity, old_pos, new_pos));
            undo_stack.execute(command, &mut world, &mut entity_names);
        }
        
        // Should be merged into fewer commands
        assert!(undo_stack.commands.len() < 10);
    }
}
```

## Future Enhancements

### 1. Undo History Panel

```rust
// แสดง history ของ commands
pub fn render_undo_history(ui: &mut egui::Ui, undo_stack: &UndoStack) {
    ui.heading("Undo History");
    
    let history = undo_stack.get_history();
    for (i, desc) in history.iter().enumerate() {
        let is_current = i == undo_stack.current_index;
        let text = if is_current {
            format!("→ {}", desc)
        } else {
            desc.clone()
        };
        
        if ui.selectable_label(is_current, text).clicked() {
            // Jump to this state
            // ...
        }
    }
}
```

### 2. Component-Level Undo

```rust
pub struct ModifyComponentCommand<T: Component> {
    entity: Entity,
    old_value: T,
    new_value: T,
}
```

### 3. Undo Groups

```rust
// เริ่ม group
undo_stack.begin_group("Complex Operation");

// ทำหลาย operations
// ...

// จบ group
undo_stack.end_group();

// Undo ทั้ง group พร้อมกัน
```

## Best Practices

### 1. Always Use Commands

```rust
// ❌ Bad: แก้ไขโดยตรง
world.transforms.get_mut(&entity).unwrap().position = new_pos;

// ✅ Good: ใช้ command
let command = Box::new(MoveEntityCommand::new(entity, old_pos, new_pos));
undo_stack.execute(command, &mut world, &mut entity_names);
```

### 2. Batch Related Operations

```rust
// ❌ Bad: แยก commands
for entity in entities {
    let cmd = Box::new(DeleteEntityCommand::new(entity, &world, &entity_names));
    undo_stack.execute(cmd, &mut world, &mut entity_names);
}

// ✅ Good: ใช้ batch
let mut batch = BatchCommand::new("Delete Multiple");
for entity in entities {
    batch.add(Box::new(DeleteEntityCommand::new(entity, &world, &entity_names)));
}
undo_stack.execute(Box::new(batch), &mut world, &mut entity_names);
```

### 3. Clear Stack on Scene Load

```rust
// เมื่อโหลด scene ใหม่
undo_stack.clear();
undo_stack.mark_saved();
```

## Summary

✅ **Implemented:**
- Complete undo/redo system
- Command pattern
- Command merging
- Saved state tracking
- Memory management

🚀 **Ready for:**
- Integration with editor
- Keyboard shortcuts
- Menu items
- Transform gizmo
- Hierarchy operations

📝 **Next Steps:**
1. เพิ่ม keyboard shortcuts (Ctrl+Z, Ctrl+Y)
2. เพิ่ม menu items (Edit → Undo/Redo)
3. Integrate กับ transform gizmo
4. Integrate กับ hierarchy operations
5. เพิ่ม undo history panel (optional)
