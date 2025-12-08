# UI Prefab Instantiation System

## Overview

The prefab instantiation system allows you to create reusable UI templates (prefabs) and instantiate them at runtime with optional parameter overrides. This is essential for creating dynamic UIs where you need to spawn UI elements programmatically.

## Implementation

The system is implemented in `ui/src/prefab/mod.rs` and provides the following key components:

### Core Types

1. **`PrefabInstantiator`** - Main system for instantiating prefabs
   - Manages entity creation and component storage
   - Handles parent-child relationships
   - Supports entity destruction

2. **`PrefabParameters`** - Parameter overrides for instantiation
   - Allows customizing prefab instances without modifying the template
   - Supports text, color, sprite, position, and size overrides

3. **`InstantiatedPrefab`** - Result of instantiation
   - Contains the root entity ID
   - Provides name-to-entity mapping for easy access

4. **`PrefabValue`** - Enum for parameter values
   - Text, Color, Sprite, Position, Size

## Features

### ✅ Create entities from UIPrefab hierarchy
The system recursively creates entities for all elements in a prefab hierarchy, maintaining the parent-child relationships.

### ✅ Apply all component values from prefab
All component types are supported:
- RectTransform
- UIElement
- UIImage, UIText, UIButton, UIPanel
- UISlider, UIToggle, UIDropdown, UIInputField
- UIScrollView, UIMask
- Layout groups (Horizontal, Vertical, Grid)

### ✅ Set up parent-child relationships
The system automatically establishes parent-child relationships and maintains bidirectional mappings (parents and children HashMaps).

### ✅ Support parameterization
Parameters can override specific properties when instantiating:
- Text content
- Colors
- Sprites
- Positions
- Sizes

## Usage Examples

### Basic Instantiation

```rust
use ui::{UIPrefab, PrefabInstantiator};

let mut instantiator = PrefabInstantiator::new();
let prefab = load_prefab("button.prefab");

// Instantiate without parameters
let result = instantiator.instantiate(&prefab);
println!("Created entity: {}", result.root_entity);
```

### Instantiation with Parameters

```rust
use ui::{PrefabParameters, Vec2};

let mut params = PrefabParameters::new();
params
    .set_text("ButtonText", "Click Me!".to_string())
    .set_color("ButtonBackground", [0.2, 0.6, 1.0, 1.0])
    .set_size("ButtonBackground", Vec2::new(150.0, 50.0));

let result = instantiator.instantiate_with_params(&prefab, &params);
```

### Accessing Named Entities

```rust
// Get entity by name
if let Some(entity) = instantiator.get_entity_by_name(&result, "ButtonText") {
    // Access the entity's components
    if let Some(text) = instantiator.texts.get(&entity) {
        println!("Text: {}", text.text);
    }
}
```

### Destroying Entities

```rust
// Destroy an entity and all its descendants
instantiator.destroy_entity(result.root_entity);
```

## Component Storage

The `PrefabInstantiator` maintains HashMaps for all component types:

```rust
pub struct PrefabInstantiator {
    // Component storage
    pub rect_transforms: HashMap<Entity, RectTransform>,
    pub ui_elements: HashMap<Entity, UIElement>,
    pub images: HashMap<Entity, UIImage>,
    pub texts: HashMap<Entity, UIText>,
    pub buttons: HashMap<Entity, UIButton>,
    // ... and more
    
    // Hierarchy
    pub parents: HashMap<Entity, Entity>,
    pub children: HashMap<Entity, Vec<Entity>>,
}
```

This design allows for efficient component queries and updates.

## Testing

The implementation includes comprehensive unit tests covering:

- ✅ Simple prefab instantiation
- ✅ Hierarchical prefab instantiation
- ✅ Parameter overrides (text, color, position, size, sprite)
- ✅ Multiple instantiations
- ✅ Entity name lookup
- ✅ Entity destruction
- ✅ Multiple parameter overrides

All tests pass successfully.

## Example Demo

Run the prefab demo to see the system in action:

```bash
cargo run --package ui --example prefab_demo
```

The demo shows:
1. Basic instantiation
2. Instantiation with parameters
3. Multiple instances
4. Entity destruction
5. Complex hierarchical prefabs

## Requirements Validation

This implementation satisfies the following requirements from the design document:

- **Requirement 14.1**: UI prefab instantiation creates all elements with configured properties ✅
- **Requirement 14.2**: All elements in the prefab hierarchy are created ✅
- **Requirement 14.3**: Parameters can override specific properties ✅

## Future Enhancements

Potential improvements for future iterations:

1. **Prefab Serialization** - Save/load prefabs from JSON files
2. **Nested Prefabs** - Support prefabs that reference other prefabs
3. **Prefab Variants** - Create variations of a base prefab
4. **Visual Editor** - GUI tool for creating and editing prefabs
5. **Prefab Pooling** - Reuse destroyed prefab instances for performance
6. **Prefab Events** - Callbacks for instantiation and destruction
7. **Prefab Validation** - Validate prefab structure before instantiation

## Integration Notes

The prefab system is designed to integrate with:

- **Canvas System** - Prefabs can be instantiated as children of canvases
- **Hierarchy System** - Maintains proper parent-child relationships
- **Layout System** - Prefab elements can use layout groups
- **Event System** - Prefab elements can receive input events
- **Lua Scripting** - Future Lua bindings will allow script-based instantiation

## Performance Considerations

- Entity IDs are simple u64 values for fast lookups
- Component storage uses HashMaps for O(1) access
- Recursive instantiation is efficient for typical UI hierarchies (2-5 levels deep)
- Destruction properly cleans up all descendants to prevent memory leaks

## Conclusion

The prefab instantiation system provides a robust foundation for creating dynamic UIs in the XS 2D Game Engine. It supports all required features and is ready for integration with the rest of the UI system.
