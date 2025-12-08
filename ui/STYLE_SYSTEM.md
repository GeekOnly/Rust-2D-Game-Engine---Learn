# UI Style System

The UI Style System provides a comprehensive solution for managing visual styling and theming in the XS 2D Game Engine's UI system. It supports style application, inheritance, theme changes, and smooth style transitions.

## Features

- **Style Definition**: Define reusable UI styles with colors, fonts, sprites, and spacing
- **Theme Management**: Organize multiple styles into themes and switch between them
- **Style Inheritance**: Child elements can inherit styles from their parents
- **Style Transitions**: Smoothly animate between different styles
- **Component Support**: Apply styles to all UI component types

## Core Components

### UIStyle

A `UIStyle` defines the visual appearance of UI elements:

```rust
pub struct UIStyle {
    pub name: String,
    
    // Colors
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background_color: Color,
    pub text_color: Color,
    pub disabled_color: Color,
    
    // Fonts
    pub default_font: String,
    pub default_font_size: f32,
    
    // Sprites
    pub button_sprite: Option<String>,
    pub panel_sprite: Option<String>,
    // ... more sprite options
    
    // Spacing
    pub default_spacing: f32,
    pub default_padding: Vec4,
}
```

### UITheme

A `UITheme` is a collection of styles with one active style:

```rust
pub struct UITheme {
    pub name: String,
    pub styles: HashMap<String, UIStyle>,
    pub active_style: String,
}
```

### StyledElement

The `StyledElement` component marks a UI element as styled:

```rust
pub struct StyledElement {
    /// Style name to apply (None means inherit from parent)
    pub style_name: Option<String>,
    
    /// Whether to inherit style from parent
    pub inherit_from_parent: bool,
    
    /// Cached resolved style name
    pub resolved_style_name: Option<String>,
    
    /// Whether this element needs style update
    pub dirty: bool,
}
```

### StyleTransition

The `StyleTransition` component enables smooth animated transitions between styles:

```rust
pub struct StyleTransition {
    pub duration: f32,
    // ... internal state for interpolation
}
```

### StyleSystem

The `StyleSystem` manages themes and applies styles to UI elements:

```rust
pub struct StyleSystem {
    theme: UITheme,
    theme_changed: bool,
}
```

## Usage Examples

### Creating a Custom Style

```rust
use ui::{UIStyle, StyleSystem};
use glam::Vec4;

// Create a dark theme style
let mut dark_style = UIStyle::default();
dark_style.name = "dark".to_string();
dark_style.primary_color = [0.2, 0.2, 0.3, 1.0];
dark_style.secondary_color = [0.3, 0.3, 0.4, 1.0];
dark_style.background_color = [0.1, 0.1, 0.15, 1.0];
dark_style.text_color = [0.9, 0.9, 0.9, 1.0];
dark_style.default_font_size = 16.0;
dark_style.default_spacing = 8.0;
dark_style.default_padding = Vec4::new(12.0, 12.0, 12.0, 12.0);

// Add to style system
let mut style_system = StyleSystem::new();
style_system.theme_mut().add_style(dark_style);
```

### Applying Styles to Elements

```rust
use ui::{UIElement, UIButton, UIText};

let mut element = UIElement::default();
let mut button = UIButton::default();
let mut text = UIText::default();

// Get the active style
let style = style_system.theme().get_active_style().unwrap();

// Apply to components
style_system.apply_style_to_element(style, &mut element);
style_system.apply_style_to_button(style, &mut button);
style_system.apply_style_to_text(style, &mut text);
```

### Style Inheritance

```rust
use ui::StyledElement;

// Parent with explicit style
let parent_styled = StyledElement::with_style("dark".to_string());

// Child that inherits from parent
let child_styled = StyledElement::inheriting();

// Resolve the child's style (will be "dark")
let parent_style_name = style_system.resolve_style_name(&parent_styled, None);
let child_style_name = style_system.resolve_style_name(
    &child_styled, 
    parent_style_name.as_deref()
);

// Child with explicit style (doesn't inherit)
let child_explicit = StyledElement::with_style("light".to_string());
```

### Changing Themes

```rust
// Change the active style in the current theme
style_system.set_active_style("dark".to_string());

// Check if theme changed (to trigger updates)
if style_system.has_theme_changed() {
    // Update all styled elements
    // ...
    
    style_system.clear_theme_changed();
}
```

### Style Transitions

```rust
use ui::StyleTransition;

let mut transition = StyleTransition::default();
transition.duration = 0.5; // 0.5 second transition

// Start transition from one style to another
style_system.start_transition(&mut transition, "default", "dark");

// In your update loop
let delta_time = 0.016; // ~60 FPS
if style_system.update_transition(&mut transition, delta_time) {
    // Transition is still active, apply interpolated colors
    style_system.apply_transition_to_element(&transition, &mut element);
    style_system.apply_transition_to_button(&transition, &mut button);
    style_system.apply_transition_to_text(&transition, &mut text);
}
```

## Integration with ECS

The style system is designed to work with the ECS architecture:

1. **Add StyledElement component** to entities that should be styled
2. **Add StyleTransition component** to entities that should animate style changes
3. **Create a style update system** that:
   - Resolves style names (handles inheritance)
   - Applies styles to components
   - Updates transitions
   - Marks dirty elements for re-rendering

### Example ECS Integration

```rust
// Pseudo-code for ECS integration
fn update_styles(
    style_system: &mut StyleSystem,
    styled_elements: &mut [(Entity, &mut StyledElement)],
    ui_elements: &mut [(Entity, &mut UIElement)],
    buttons: &mut [(Entity, &mut UIButton)],
    texts: &mut [(Entity, &mut UIText)],
    // ... other component types
) {
    // Check if theme changed
    if style_system.has_theme_changed() {
        // Mark all styled elements as dirty
        for (_, styled) in styled_elements.iter_mut() {
            styled.mark_dirty();
        }
        style_system.clear_theme_changed();
    }
    
    // Update dirty elements
    for (entity, styled) in styled_elements.iter_mut() {
        if styled.dirty {
            // Resolve style name (with inheritance)
            let parent_style = get_parent_style(entity);
            let style_name = style_system.resolve_style_name(styled, parent_style);
            
            if let Some(style_name) = style_name {
                if let Some(style) = style_system.get_style(&style_name) {
                    // Apply to all component types
                    if let Some((_, ui_element)) = ui_elements.iter_mut().find(|(e, _)| *e == entity) {
                        style_system.apply_style_to_element(style, ui_element);
                    }
                    if let Some((_, button)) = buttons.iter_mut().find(|(e, _)| *e == entity) {
                        style_system.apply_style_to_button(style, button);
                    }
                    if let Some((_, text)) = texts.iter_mut().find(|(e, _)| *e == entity) {
                        style_system.apply_style_to_text(style, text);
                    }
                    // ... other component types
                }
            }
            
            styled.dirty = false;
        }
    }
}
```

## Supported Component Types

The style system can apply styles to:

- **UIElement**: Base color
- **UIButton**: Normal, highlighted, pressed, and disabled colors
- **UIPanel**: Background sprite and padding
- **UIText**: Text color, font, and font size
- **UIImage**: (Reserved for future use)
- **HorizontalLayoutGroup**: Spacing and padding
- **VerticalLayoutGroup**: Spacing and padding
- **GridLayoutGroup**: Spacing and padding

## Best Practices

### 1. Use Style Inheritance

Let child elements inherit styles from their parents to maintain consistency:

```rust
// Parent defines the style
let parent = StyledElement::with_style("dark".to_string());

// Children inherit automatically
let child = StyledElement::inheriting();
```

### 2. Organize Styles into Themes

Group related styles into themes for easy switching:

```rust
let mut theme = UITheme::default();
theme.add_style(dark_style);
theme.add_style(light_style);
theme.add_style(high_contrast_style);
```

### 3. Use Transitions for Smooth Changes

Always use transitions when changing themes to avoid jarring visual changes:

```rust
let mut transition = StyleTransition::default();
transition.duration = 0.3; // 300ms is a good default
style_system.start_transition(&mut transition, old_style, new_style);
```

### 4. Cache Style Lookups

The `StyledElement` component caches the resolved style name to avoid repeated lookups:

```rust
// The resolved_style_name is cached automatically
if styled.resolved_style_name.is_some() {
    // Use cached value
}
```

### 5. Mark Elements Dirty on Changes

Always mark elements as dirty when their style needs to be updated:

```rust
styled.mark_dirty();
```

## Performance Considerations

- **Dirty Flagging**: Only update elements marked as dirty
- **Cached Resolution**: Style names are cached to avoid repeated inheritance resolution
- **Batch Updates**: When theme changes, update all elements in a single pass
- **Transition Pooling**: Reuse `StyleTransition` components instead of creating new ones

## Future Enhancements

Potential future additions to the style system:

1. **CSS-like Selectors**: Apply styles based on element type or tags
2. **Style Variants**: Define style variants (e.g., "button-primary", "button-secondary")
3. **Runtime Style Editing**: Hot-reload styles during development
4. **Style Presets**: Pre-defined style collections for common use cases
5. **Advanced Transitions**: Support for custom easing functions and property-specific durations
6. **Style Validation**: Validate style definitions at load time
7. **Style Inheritance Overrides**: Fine-grained control over which properties inherit

## See Also

- [UI System Overview](README.md)
- [Layout System](LAYOUT_SYSTEM.md)
- [Prefab System](PREFAB_INSTANTIATION.md)
- [Animation System](../src/animation/mod.rs)
