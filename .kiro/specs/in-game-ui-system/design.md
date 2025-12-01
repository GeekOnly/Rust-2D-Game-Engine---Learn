# In-Game UI System Design Document

## Overview

The In-Game UI System provides a comprehensive, production-ready user interface framework for the XS 2D Game Engine, offering capabilities comparable to Unity's Canvas UI and Unreal Engine's UMG. The system is designed as a native ECS-based solution that integrates seamlessly with the engine's existing sprite rendering pipeline, physics system, and Lua scripting capabilities.

The UI system follows a retained-mode architecture with immediate-mode rendering, combining the best of both approaches: clean component-based design with high-performance batch rendering. All UI elements are entities in the ECS world, enabling full integration with the engine's existing systems while maintaining specialized UI-specific behaviors.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Game Application                         │
│                    (Lua Scripts / Rust)                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    UI System API                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Canvas       │  │ UI Builder   │  │ Event        │      │
│  │ Manager      │  │              │  │ Dispatcher   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    ECS World (ecs crate)                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  UI Components:                                       │   │
│  │  - Canvas, RectTransform, UIElement                  │   │
│  │  - UIImage, UIText, UIButton, UIPanel                │   │
│  │  - Layout Groups, Scroll View, Mask                  │   │
│  │  - UISlider, UIToggle, UIDropdown, UIInputField     │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Rendering Pipeline                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ UI Batch     │  │ Text         │  │ Clipping &   │      │
│  │ Builder      │  │ Renderer     │  │ Masking      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         Sprite Renderer (render crate)                │   │
│  │         WGPU Backend                                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Module Structure

The UI system will be implemented as a new crate `ui` with the following module structure:

```
ui/
├── src/
│   ├── lib.rs                 # Public API and re-exports
│   ├── canvas.rs              # Canvas component and management
│   ├── rect_transform.rs      # RectTransform positioning system
│   ├── components/
│   │   ├── mod.rs
│   │   ├── image.rs           # UIImage component
│   │   ├── text.rs            # UIText component
│   │   ├── button.rs          # UIButton component
│   │   ├── panel.rs           # UIPanel component
│   │   ├── slider.rs          # UISlider component
│   │   ├── toggle.rs          # UIToggle component
│   │   ├── dropdown.rs        # UIDropdown component
│   │   ├── input_field.rs     # UIInputField component
│   │   └── scroll_view.rs     # UIScrollView component
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── horizontal.rs      # Horizontal layout group
│   │   ├── vertical.rs        # Vertical layout group
│   │   ├── grid.rs            # Grid layout group
│   │   └── content_size_fitter.rs
│   ├── events/
│   │   ├── mod.rs
│   │   ├── event_system.rs    # Event dispatching
│   │   ├── raycast.rs         # UI raycasting
│   │   └── input_handler.rs   # Input event handling
│   ├── rendering/
│   │   ├── mod.rs
│   │   ├── batch_builder.rs   # UI batch construction
│   │   ├── text_renderer.rs   # Text rendering
│   │   ├── nine_slice.rs      # 9-slice sprite rendering
│   │   └── mask.rs            # Clipping and masking
│   ├── animation/
│   │   ├── mod.rs
│   │   ├── tween.rs           # Tween animations
│   │   └── easing.rs          # Easing functions
│   ├── prefab.rs              # UI prefab system
│   ├── style.rs               # UI styling and themes
│   └── lua_bindings.rs        # Lua API bindings
└── Cargo.toml
```

## Components and Interfaces

### Core UI Components

#### Canvas Component

The Canvas is the root component for all UI rendering, defining the coordinate space and render mode.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Canvas {
    /// Render mode determines how UI is positioned and rendered
    pub render_mode: CanvasRenderMode,
    
    /// Sort order for multiple canvases (higher renders on top)
    pub sort_order: i32,
    
    /// Reference camera for Screen Space Camera and World Space modes
    pub camera_entity: Option<Entity>,
    
    /// Plane distance for Screen Space Camera mode
    pub plane_distance: f32,
    
    /// Canvas scaler for resolution independence
    pub scaler: CanvasScaler,
    
    /// Whether this canvas blocks raycasts to canvases behind it
    pub blocks_raycasts: bool,
    
    /// Cached screen size for dirty checking
    #[serde(skip)]
    pub cached_screen_size: (u32, u32),
    
    /// Dirty flag for rebuild
    #[serde(skip)]
    pub dirty: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasRenderMode {
    /// UI rendered in screen space on top of everything
    ScreenSpaceOverlay,
    
    /// UI rendered in screen space at a distance from camera
    ScreenSpaceCamera,
    
    /// UI rendered as part of the 3D world
    WorldSpace,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasScaler {
    /// Scaling mode
    pub mode: ScaleMode,
    
    /// Reference resolution for Scale With Screen Size mode
    pub reference_resolution: (f32, f32),
    
    /// Match width (0.0) or height (1.0) or blend
    pub match_width_or_height: f32,
    
    /// Reference DPI for Constant Physical Size mode
    pub reference_dpi: f32,
    
    /// Minimum and maximum scale factors
    pub min_scale: f32,
    pub max_scale: f32,
    
    /// Cached scale factor
    #[serde(skip)]
    pub scale_factor: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScaleMode {
    ConstantPixelSize,
    ScaleWithScreenSize,
    ConstantPhysicalSize,
}
```

#### RectTransform Component

RectTransform defines the position, size, and anchoring of UI elements.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RectTransform {
    /// Anchor minimum (normalized 0-1 in parent space)
    pub anchor_min: Vec2,
    
    /// Anchor maximum (normalized 0-1 in parent space)
    pub anchor_max: Vec2,
    
    /// Pivot point (normalized 0-1 in local space)
    pub pivot: Vec2,
    
    /// Anchored position (offset from anchor point)
    pub anchored_position: Vec2,
    
    /// Size delta (additional size beyond anchors)
    pub size_delta: Vec2,
    
    /// Local rotation (Z-axis rotation in degrees)
    pub rotation: f32,
    
    /// Local scale
    pub scale: Vec2,
    
    /// Cached world corners (updated by layout system)
    #[serde(skip)]
    pub world_corners: [Vec2; 4], // Bottom-left, top-left, top-right, bottom-right
    
    /// Cached rect (updated by layout system)
    #[serde(skip)]
    pub rect: Rect,
    
    /// Dirty flag
    #[serde(skip)]
    pub dirty: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectTransform {
    /// Create with anchored position (for fixed-size elements)
    pub fn anchored(anchor: Vec2, position: Vec2, size: Vec2) -> Self;
    
    /// Create with stretched anchors (for responsive elements)
    pub fn stretched(anchor_min: Vec2, anchor_max: Vec2, margins: Vec4) -> Self;
    
    /// Get the calculated size
    pub fn get_size(&self) -> Vec2;
    
    /// Set the size (updates size_delta)
    pub fn set_size(&mut self, size: Vec2);
    
    /// Get world position
    pub fn get_world_position(&self) -> Vec2;
    
    /// Check if point is inside rect (for raycasting)
    pub fn contains_point(&self, point: Vec2) -> bool;
}
```

#### UIElement Component

Base component for all UI elements, providing common properties.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIElement {
    /// Whether this element can receive raycast events
    pub raycast_target: bool,
    
    /// Whether this element blocks raycasts to elements behind it
    pub blocks_raycasts: bool,
    
    /// Z-order within siblings (higher renders on top)
    pub z_order: i32,
    
    /// Color tint applied to this element
    pub color: Color,
    
    /// Alpha transparency (0.0 = fully transparent, 1.0 = fully opaque)
    pub alpha: f32,
    
    /// Whether this element is interactable
    pub interactable: bool,
    
    /// Whether to ignore parent groups (for layout)
    pub ignore_layout: bool,
    
    /// Cached canvas entity (updated by hierarchy system)
    #[serde(skip)]
    pub canvas_entity: Option<Entity>,
}

pub type Color = [f32; 4]; // RGBA
```

### UI Component Types

#### UIImage

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIImage {
    /// Sprite or texture to display
    pub sprite: Option<String>, // Texture ID
    
    /// Image type (simple, sliced, tiled, filled)
    pub image_type: ImageType,
    
    /// 9-slice borders (for sliced type)
    pub slice_borders: Vec4, // left, bottom, right, top
    
    /// Fill method (for filled type)
    pub fill_method: FillMethod,
    
    /// Fill amount (0.0 to 1.0 for filled type)
    pub fill_amount: f32,
    
    /// Fill origin (for filled type)
    pub fill_origin: i32,
    
    /// Whether to preserve aspect ratio
    pub preserve_aspect: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImageType {
    Simple,
    Sliced,
    Tiled,
    Filled,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FillMethod {
    Horizontal,
    Vertical,
    Radial90,
    Radial180,
    Radial360,
}
```

#### UIText

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIText {
    /// Text content
    pub text: String,
    
    /// Font asset ID
    pub font: String,
    
    /// Font size in points
    pub font_size: f32,
    
    /// Text color (overrides UIElement color)
    pub color: Color,
    
    /// Text alignment
    pub alignment: TextAlignment,
    
    /// Horizontal overflow mode
    pub horizontal_overflow: OverflowMode,
    
    /// Vertical overflow mode
    pub vertical_overflow: OverflowMode,
    
    /// Whether to enable rich text markup
    pub rich_text: bool,
    
    /// Line spacing multiplier
    pub line_spacing: f32,
    
    /// Whether to use best fit
    pub best_fit: bool,
    
    /// Min and max font size for best fit
    pub best_fit_min_size: f32,
    pub best_fit_max_size: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, MiddleCenter, MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OverflowMode {
    Wrap,
    Overflow,
    Truncate,
}
```

#### UIButton

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIButton {
    /// Button state
    #[serde(skip)]
    pub state: ButtonState,
    
    /// Transition type
    pub transition: ButtonTransition,
    
    /// Color tint for each state (for ColorTint transition)
    pub normal_color: Color,
    pub highlighted_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    
    /// Color fade duration
    pub fade_duration: f32,
    
    /// Sprite swap (for SpriteSwap transition)
    pub highlighted_sprite: Option<String>,
    pub pressed_sprite: Option<String>,
    pub disabled_sprite: Option<String>,
    
    /// Animation trigger (for Animation transition)
    pub normal_trigger: String,
    pub highlighted_trigger: String,
    pub pressed_trigger: String,
    pub disabled_trigger: String,
    
    /// Lua callback function name
    pub on_click: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Highlighted,
    Pressed,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ButtonTransition {
    None,
    ColorTint,
    SpriteSwap,
    Animation,
}
```

#### UIPanel

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPanel {
    /// Background sprite
    pub background: Option<String>,
    
    /// Whether to use 9-slice
    pub use_nine_slice: bool,
    
    /// 9-slice borders
    pub slice_borders: Vec4,
    
    /// Padding inside the panel
    pub padding: Vec4, // left, bottom, right, top
}
```

### Layout Components

#### HorizontalLayoutGroup

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HorizontalLayoutGroup {
    /// Padding around the layout
    pub padding: Vec4,
    
    /// Spacing between elements
    pub spacing: f32,
    
    /// Child alignment
    pub child_alignment: Alignment,
    
    /// Whether to force expand children width
    pub child_force_expand_width: bool,
    
    /// Whether to force expand children height
    pub child_force_expand_height: bool,
    
    /// Whether to control child width
    pub child_control_width: bool,
    
    /// Whether to control child height
    pub child_control_height: bool,
}
```

#### VerticalLayoutGroup

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerticalLayoutGroup {
    /// Padding around the layout
    pub padding: Vec4,
    
    /// Spacing between elements
    pub spacing: f32,
    
    /// Child alignment
    pub child_alignment: Alignment,
    
    /// Whether to force expand children width
    pub child_force_expand_width: bool,
    
    /// Whether to force expand children height
    pub child_force_expand_height: bool,
    
    /// Whether to control child width
    pub child_control_width: bool,
    
    /// Whether to control child height
    pub child_control_height: bool,
}
```

#### GridLayoutGroup

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridLayoutGroup {
    /// Padding around the layout
    pub padding: Vec4,
    
    /// Cell size
    pub cell_size: Vec2,
    
    /// Spacing between cells
    pub spacing: Vec2,
    
    /// Start corner
    pub start_corner: Corner,
    
    /// Start axis
    pub start_axis: Axis,
    
    /// Child alignment
    pub child_alignment: Alignment,
    
    /// Constraint mode
    pub constraint: GridConstraint,
    
    /// Constraint count (for FixedColumnCount or FixedRowCount)
    pub constraint_count: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GridConstraint {
    Flexible,
    FixedColumnCount,
    FixedRowCount,
}
```

### Advanced Components

#### UIScrollView

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIScrollView {
    /// Content entity (the scrollable content container)
    pub content: Option<Entity>,
    
    /// Viewport entity (the visible area)
    pub viewport: Option<Entity>,
    
    /// Horizontal scrollbar
    pub horizontal_scrollbar: Option<Entity>,
    
    /// Vertical scrollbar
    pub vertical_scrollbar: Option<Entity>,
    
    /// Movement type
    pub movement_type: MovementType,
    
    /// Elasticity (for elastic movement)
    pub elasticity: f32,
    
    /// Inertia
    pub inertia: bool,
    
    /// Deceleration rate
    pub deceleration_rate: f32,
    
    /// Scroll sensitivity
    pub scroll_sensitivity: f32,
    
    /// Horizontal scroll enabled
    pub horizontal: bool,
    
    /// Vertical scroll enabled
    pub vertical: bool,
    
    /// Current scroll position (0-1)
    #[serde(skip)]
    pub normalized_position: Vec2,
    
    /// Velocity for inertia
    #[serde(skip)]
    pub velocity: Vec2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MovementType {
    Unrestricted,
    Elastic,
    Clamped,
}
```

#### UIMask

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIMask {
    /// Whether to show the mask graphic
    pub show_mask_graphic: bool,
    
    /// Whether to use sprite alpha for masking
    pub use_sprite_alpha: bool,
}
```

#### UISlider

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UISlider {
    /// Fill rect entity
    pub fill_rect: Option<Entity>,
    
    /// Handle rect entity
    pub handle_rect: Option<Entity>,
    
    /// Direction
    pub direction: SliderDirection,
    
    /// Min value
    pub min_value: f32,
    
    /// Max value
    pub max_value: f32,
    
    /// Current value
    pub value: f32,
    
    /// Whether to use whole numbers
    pub whole_numbers: bool,
    
    /// Lua callback for value changed
    pub on_value_changed: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SliderDirection {
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}
```

#### UIToggle

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIToggle {
    /// Checkmark graphic entity
    pub graphic: Option<Entity>,
    
    /// Whether toggle is on
    pub is_on: bool,
    
    /// Toggle transition
    pub toggle_transition: ToggleTransition,
    
    /// Lua callback for value changed
    pub on_value_changed: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ToggleTransition {
    None,
    Fade,
}
```

#### UIDropdown

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIDropdown {
    /// Template entity (the dropdown list)
    pub template: Option<Entity>,
    
    /// Caption text entity
    pub caption_text: Option<Entity>,
    
    /// Item text entity (in template)
    pub item_text: Option<Entity>,
    
    /// Options
    pub options: Vec<DropdownOption>,
    
    /// Current selected index
    pub value: i32,
    
    /// Lua callback for value changed
    pub on_value_changed: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DropdownOption {
    pub text: String,
    pub image: Option<String>,
}
```

#### UIInputField

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIInputField {
    /// Text component entity
    pub text_component: Option<Entity>,
    
    /// Placeholder entity
    pub placeholder: Option<Entity>,
    
    /// Current text
    pub text: String,
    
    /// Character limit (0 = unlimited)
    pub character_limit: i32,
    
    /// Content type
    pub content_type: ContentType,
    
    /// Line type
    pub line_type: LineType,
    
    /// Input type (for mobile keyboards)
    pub input_type: InputType,
    
    /// Keyboard type (for mobile)
    pub keyboard_type: KeyboardType,
    
    /// Character validation
    pub character_validation: CharacterValidation,
    
    /// Caret blink rate
    pub caret_blink_rate: f32,
    
    /// Caret width
    pub caret_width: i32,
    
    /// Selection color
    pub selection_color: Color,
    
    /// Read only
    pub read_only: bool,
    
    /// Lua callbacks
    pub on_value_changed: Option<String>,
    pub on_end_edit: Option<String>,
    
    /// Runtime state
    #[serde(skip)]
    pub caret_position: i32,
    #[serde(skip)]
    pub selection_anchor: i32,
    #[serde(skip)]
    pub is_focused: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    Standard,
    Autocorrected,
    IntegerNumber,
    DecimalNumber,
    Alphanumeric,
    Name,
    EmailAddress,
    Password,
    Pin,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    SingleLine,
    MultiLineSubmit,
    MultiLineNewline,
}
```

## Data Models

### Event System Data

```rust
/// UI Event types
#[derive(Clone, Debug)]
pub enum UIEvent {
    PointerEnter(Entity),
    PointerExit(Entity),
    PointerDown(Entity, Vec2),
    PointerUp(Entity, Vec2),
    PointerClick(Entity, Vec2),
    BeginDrag(Entity, Vec2),
    Drag(Entity, Vec2, Vec2), // entity, position, delta
    EndDrag(Entity, Vec2),
    Scroll(Entity, f32), // entity, delta
}

/// UI Event handler
pub struct UIEventHandler {
    /// Registered event listeners
    listeners: HashMap<Entity, Vec<UIEventListener>>,
    
    /// Current hover state
    hovered_elements: HashSet<Entity>,
    
    /// Current pressed state
    pressed_elements: HashMap<Entity, Vec2>,
    
    /// Current drag state
    dragging_element: Option<(Entity, Vec2)>,
}

pub struct UIEventListener {
    pub event_type: UIEventType,
    pub callback: String, // Lua function name
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UIEventType {
    OnPointerEnter,
    OnPointerExit,
    OnPointerDown,
    OnPointerUp,
    OnPointerClick,
    OnBeginDrag,
    OnDrag,
    OnEndDrag,
    OnScroll,
}
```

### Animation Data

```rust
/// UI Animation (tween-based)
#[derive(Clone, Debug)]
pub struct UIAnimation {
    /// Target entity
    pub entity: Entity,
    
    /// Property to animate
    pub property: AnimatedProperty,
    
    /// Start value
    pub from: AnimationValue,
    
    /// End value
    pub to: AnimationValue,
    
    /// Duration in seconds
    pub duration: f32,
    
    /// Easing function
    pub easing: EasingFunction,
    
    /// Delay before starting
    pub delay: f32,
    
    /// Loop mode
    pub loop_mode: LoopMode,
    
    /// Completion callback
    pub on_complete: Option<String>,
    
    /// Runtime state
    pub elapsed: f32,
    pub started: bool,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimatedProperty {
    AnchoredPosition,
    Scale,
    Rotation,
    Color,
    Alpha,
    SizeDelta,
}

#[derive(Clone, Debug)]
pub enum AnimationValue {
    Vec2(Vec2),
    Float(f32),
    Color(Color),
}

#[derive(Clone, Debug, PartialEq)]
pub enum EasingFunction {
    Linear,
    EaseInQuad, EaseOutQuad, EaseInOutQuad,
    EaseInCubic, EaseOutCubic, EaseInOutCubic,
    EaseInQuart, EaseOutQuart, EaseInOutQuart,
    EaseInQuint, EaseOutQuint, EaseInOutQuint,
    EaseInSine, EaseOutSine, EaseInOutSine,
    EaseInExpo, EaseOutExpo, EaseInOutExpo,
    EaseInCirc, EaseOutCirc, EaseInOutCirc,
    EaseInElastic, EaseOutElastic, EaseInOutElastic,
    EaseInBack, EaseOutBack, EaseInOutBack,
    EaseInBounce, EaseOutBounce, EaseInOutBounce,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}
```

### UI Prefab Data

```rust
/// UI Prefab for reusable UI templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPrefab {
    /// Prefab name
    pub name: String,
    
    /// Root element data
    pub root: UIPrefabElement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPrefabElement {
    /// Element name
    pub name: String,
    
    /// Components
    pub rect_transform: RectTransform,
    pub ui_element: UIElement,
    pub image: Option<UIImage>,
    pub text: Option<UIText>,
    pub button: Option<UIButton>,
    pub panel: Option<UIPanel>,
    pub slider: Option<UISlider>,
    pub toggle: Option<UIToggle>,
    pub dropdown: Option<UIDropdown>,
    pub input_field: Option<UIInputField>,
    pub scroll_view: Option<UIScrollView>,
    pub mask: Option<UIMask>,
    pub horizontal_layout: Option<HorizontalLayoutGroup>,
    pub vertical_layout: Option<VerticalLayoutGroup>,
    pub grid_layout: Option<GridLayoutGroup>,
    
    /// Children
    pub children: Vec<UIPrefabElement>,
}
```

### Style and Theme Data

```rust
/// UI Style definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIStyle {
    /// Style name
    pub name: String,
    
    /// Colors
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background_color: Color,
    pub text_color: Color,
    pub disabled_color: Color,
    
    /// Fonts
    pub default_font: String,
    pub default_font_size: f32,
    
    /// Sprites
    pub button_sprite: Option<String>,
    pub panel_sprite: Option<String>,
    pub input_field_sprite: Option<String>,
    pub slider_background_sprite: Option<String>,
    pub slider_fill_sprite: Option<String>,
    pub slider_handle_sprite: Option<String>,
    pub toggle_background_sprite: Option<String>,
    pub toggle_checkmark_sprite: Option<String>,
    pub dropdown_sprite: Option<String>,
    pub scrollbar_sprite: Option<String>,
    
    /// Spacing
    pub default_spacing: f32,
    pub default_padding: Vec4,
}

/// UI Theme (collection of styles)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UITheme {
    pub name: String,
    pub styles: HashMap<String, UIStyle>,
    pub active_style: String,
}
```

## Co
rrectness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing all acceptance criteria, several properties can be consolidated to reduce redundancy:

**Animation Properties (8.2-8.6)** can be consolidated into a single comprehensive property about animation interpolation, as they all test the same interpolation mechanism with different property types.

**Layout Arrangement Properties (5.1-5.3)** share the same core behavior (arranging children with spacing) and can be tested with a unified property that covers all layout types.

**Event Triggering Properties (6.2-6.5)** all test event callback invocation and can be consolidated into a property about event system correctness.

**Hierarchy Propagation Properties (3.2-3.4)** all test transform propagation and can be unified into a single property about hierarchical transform updates.

### Core Properties

**Property 1: Canvas initialization completeness**
*For any* Canvas creation, the resulting Canvas entity should have all required components (Canvas, RectTransform, UIElement) properly initialized with valid default values.
**Validates: Requirements 1.1**

**Property 2: Resolution change preserves anchored layout**
*For any* UI element with anchor configuration and any screen resolution change, the element's position and size relative to its anchors should remain consistent according to the anchor rules.
**Validates: Requirements 1.5, 2.6, 7.5**

**Property 3: Canvas sort order determines render priority**
*For any* collection of Canvases with different sort orders, rendering should occur in ascending sort order (lower values render first, higher values render on top).
**Validates: Requirements 1.6**

**Property 4: RectTransform anchor clamping**
*For any* anchor min/max values set on a RectTransform, the stored values should be clamped to the range [0,0] to [1,1].
**Validates: Requirements 2.7**

**Property 5: Fixed anchor positioning**
*For any* UI element where anchor_min equals anchor_max, the element's world position should be the anchor point plus the anchored_position offset, regardless of parent size changes.
**Validates: Requirements 2.2**

**Property 6: Stretched anchor sizing**
*For any* UI element with stretched anchors (anchor_min != anchor_max), the element's size in the stretched dimension should scale proportionally with the parent's size.
**Validates: Requirements 2.3, 2.4**

**Property 7: Pivot change preserves visual position**
*For any* UI element, changing the pivot point should not change the element's world corner positions (visual location remains the same).
**Validates: Requirements 2.5**

**Property 8: Hierarchical transform propagation**
*For any* parent-child UI hierarchy, transforming the parent (position, scale, or rotation) should correctly propagate to all descendants' world transforms.
**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

**Property 9: Hierarchical visibility propagation**
*For any* parent-child UI hierarchy, hiding a parent element should result in all descendant elements being hidden from rendering.
**Validates: Requirements 3.5**

**Property 10: Hierarchical destruction propagation**
*For any* parent-child UI hierarchy, destroying a parent element should result in all descendant elements being destroyed and removed from the ECS world.
**Validates: Requirements 3.6**

**Property 11: Sibling index determines render order**
*For any* collection of sibling UI elements, rendering should occur in sibling index order (lower indices render first).
**Validates: Requirements 3.7**

**Property 12: 9-slice corner preservation**
*For any* UIImage with 9-slice enabled and any size, the corner regions should maintain their original pixel dimensions without distortion.
**Validates: Requirements 4.5**

**Property 13: Text overflow handling**
*For any* UIText component with content exceeding bounds, the rendered text should respect the overflow mode (wrap, truncate, or overflow).
**Validates: Requirements 4.6**

**Property 14: Color tint application**
*For any* UI component with a color tint, the rendered output should have the tint color multiplied with the source color.
**Validates: Requirements 4.7**

**Property 15: Layout spacing consistency**
*For any* layout group (Horizontal, Vertical, or Grid) with N children and spacing S, the distance between adjacent children should be exactly S units.
**Validates: Requirements 5.1, 5.2, 5.3**

**Property 16: Layout padding application**
*For any* layout group with padding P, the first child should be offset by P from the container's edge, and the last child should end P units before the opposite edge.
**Validates: Requirements 5.4**

**Property 17: Layout child alignment**
*For any* layout group with alignment setting A, all children should be aligned according to A within their allocated space.
**Validates: Requirements 5.5**

**Property 18: Layout force expand**
*For any* layout group with force expand enabled, children should expand to fill all available space in the expand dimension.
**Validates: Requirements 5.6**

**Property 19: Layout recalculation on size change**
*For any* layout group, when a child's size changes, the layout should recalculate positions for all affected children on the next frame.
**Validates: Requirements 5.7**

**Property 20: Raycast target inclusion**
*For any* UI element marked as raycast_target=true, the element should be included in raycast tests and can receive input events.
**Validates: Requirements 6.1**

**Property 21: Event delivery to topmost element**
*For any* collection of overlapping UI elements at a point, input events should be delivered only to the element with the highest Z-order.
**Validates: Requirements 6.6**

**Property 22: Raycast blocking**
*For any* UI element with blocks_raycasts=true, elements behind it (lower Z-order) should not receive raycast events at overlapping points.
**Validates: Requirements 6.7**

**Property 23: Event callback invocation**
*For any* UI element with a registered event callback, when the corresponding event occurs, the callback should be invoked with correct parameters.
**Validates: Requirements 6.2, 6.3, 6.4, 6.5, 6.8, 10.5, 10.6, 10.7**

**Property 24: Canvas scaler maintains pixel size**
*For any* Canvas with ScaleMode::ConstantPixelSize, UI elements should maintain their specified pixel dimensions regardless of screen resolution.
**Validates: Requirements 7.1**

**Property 25: Canvas scaler proportional scaling**
*For any* Canvas with ScaleMode::ScaleWithScreenSize, the scale factor should be proportional to the ratio between current resolution and reference resolution.
**Validates: Requirements 7.2**

**Property 26: Canvas scaler DPI consistency**
*For any* Canvas with ScaleMode::ConstantPhysicalSize, UI elements should maintain consistent physical dimensions (in mm/inches) across different DPI settings.
**Validates: Requirements 7.3**

**Property 27: Canvas scaler aspect ratio matching**
*For any* Canvas with aspect ratio different from reference, the scale factor should blend between width-based and height-based scaling according to the match parameter.
**Validates: Requirements 7.4**

**Property 28: Canvas scaler clamps scale factor**
*For any* Canvas with min/max scale limits, the calculated scale factor should be clamped within [min_scale, max_scale].
**Validates: Requirements 7.6**

**Property 29: Animation interpolation correctness**
*For any* UI animation at time t (0 ≤ t ≤ duration), the animated property value should equal lerp(from, to, easing(t/duration)).
**Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7**

**Property 30: Animation completion callback**
*For any* UI animation with a completion callback, when the animation reaches its duration, the callback should be invoked exactly once.
**Validates: Requirements 8.8**

**Property 31: Scroll view viewport clipping**
*For any* scroll view, content elements outside the viewport bounds should not be rendered.
**Validates: Requirements 9.1, 9.8**

**Property 32: Scroll view drag scrolling**
*For any* scroll view with scrolling enabled in direction D, dragging by delta Δ should move the content by exactly Δ in direction D.
**Validates: Requirements 9.3**

**Property 33: Scrollbar position reflects content**
*For any* scroll view with scrollbars, the scrollbar handle position should accurately reflect the normalized scroll position (0-1).
**Validates: Requirements 9.4**

**Property 34: Programmatic scroll positioning**
*For any* scroll view, setting normalized_position to P should move the content such that the visible portion starts at position P of the total content.
**Validates: Requirements 9.5**

**Property 35: Elastic scroll spring-back**
*For any* scroll view with elastic movement, scrolling beyond bounds should result in the content springing back to the nearest valid position.
**Validates: Requirements 9.6**

**Property 36: Scroll inertia deceleration**
*For any* scroll view with inertia enabled, releasing a drag should cause the content to continue scrolling with exponential deceleration.
**Validates: Requirements 9.7**

**Property 37: Slider value clamping**
*For any* slider with min/max values, the slider's value should always be clamped within [min_value, max_value].
**Validates: Requirements 10.1**

**Property 38: Slider handle position reflects value**
*For any* slider, the handle's position should accurately reflect the normalized value (value - min) / (max - min).
**Validates: Requirements 10.1**

**Property 39: Toggle state consistency**
*For any* toggle, the visual state (checkmark visibility) should match the is_on boolean value.
**Validates: Requirements 10.2, 10.6**

**Property 40: Dropdown displays selected option**
*For any* dropdown, the caption text should display the text of the option at index 'value'.
**Validates: Requirements 10.3, 10.7**

**Property 41: Input field content type validation**
*For any* input field with content type C, all characters in the text should satisfy the validation rules for content type C.
**Validates: Requirements 10.8**

**Property 42: Mask clips children to bounds**
*For any* mask component, all child elements should be clipped to the mask's RectTransform bounds.
**Validates: Requirements 11.1**

**Property 43: Sprite alpha masking**
*For any* mask using sprite alpha, child pixels should only be visible where the mask sprite's alpha > 0.
**Validates: Requirements 11.2**

**Property 44: Nested mask intersection**
*For any* nested masks, a child element should only be visible in the intersection of all ancestor mask regions.
**Validates: Requirements 11.3**

**Property 45: Mask graphic visibility**
*For any* mask, the mask's own graphic should be rendered if and only if show_mask_graphic is true.
**Validates: Requirements 11.4, 11.5**

**Property 46: UI batching reduces draw calls**
*For any* collection of UI elements sharing material and texture, they should be batched into a single draw call when Z-order permits.
**Validates: Requirements 12.1**

**Property 47: Z-order breaks batches**
*For any* two UI elements with different Z-orders and an element between them in Z-order, they should not be batched together.
**Validates: Requirements 12.2**

**Property 48: Property changes mark dirty**
*For any* UI element, modifying a visual property should set the dirty flag, causing re-batching on the next frame.
**Validates: Requirements 12.3**

**Property 49: Transparent elements render back-to-front**
*For any* collection of transparent UI elements, rendering should occur in ascending Z-order (back-to-front).
**Validates: Requirements 12.4**

**Property 50: Canvas dirty triggers rebuild**
*For any* Canvas marked as dirty, the render batches should be rebuilt before the next frame's rendering.
**Validates: Requirements 12.5**

**Property 51: Culled elements excluded from rendering**
*For any* UI element outside all camera viewports, it should be excluded from rendering (not submitted to GPU).
**Validates: Requirements 12.6**

**Property 52: Lua element creation**
*For any* Lua call to create a UI element, the element should be instantiated in the ECS world and added to the specified parent's children.
**Validates: Requirements 13.1**

**Property 53: Lua property modification**
*For any* Lua call to modify a UI property, the property value should be updated and the element marked dirty for re-rendering.
**Validates: Requirements 13.2**

**Property 54: Lua callback registration**
*For any* Lua callback registered for an event, the callback should be invoked when that event occurs on the element.
**Validates: Requirements 13.3**

**Property 55: Lua element destruction**
*For any* Lua call to destroy a UI element, the element and all its descendants should be removed from the ECS world.
**Validates: Requirements 13.4**

**Property 56: Lua property queries**
*For any* Lua query for a UI property, the returned value should match the current property value in the ECS component.
**Validates: Requirements 13.5**

**Property 57: Lua animation execution**
*For any* Lua call to animate a property, an animation should be created and executed with the specified parameters.
**Validates: Requirements 13.6**

**Property 58: Prefab serialization round-trip**
*For any* UI prefab, serializing to JSON and then deserializing should produce an equivalent prefab with the same hierarchy and component values.
**Validates: Requirements 14.4, 14.5**

**Property 59: Prefab instantiation completeness**
*For any* UI prefab instantiation, all elements in the prefab hierarchy should be created with their configured components and properties.
**Validates: Requirements 14.1, 14.2**

**Property 60: Prefab parameterization**
*For any* UI prefab instantiated with parameters P, the created elements should have the parameter values applied to the specified properties.
**Validates: Requirements 14.3**

**Property 61: Style application updates visuals**
*For any* UI element with a style applied, the element's visual properties (colors, fonts, sprites) should match the style's configuration.
**Validates: Requirements 15.1, 15.2**

**Property 62: Theme change updates all elements**
*For any* theme change, all UI elements using the theme should update their visual properties to match the new theme.
**Validates: Requirements 15.3**

**Property 63: Style inheritance**
*For any* UI element without explicit style overrides, it should inherit style properties from its parent.
**Validates: Requirements 15.4**

**Property 64: Style animation transitions**
*For any* style property change with animation enabled, the property should smoothly interpolate from the old value to the new value over the transition duration.
**Validates: Requirements 15.5**

## Error Handling

### Input Validation

- **Invalid Anchor Values**: Anchor min/max values outside [0,1] should be clamped automatically
- **Null References**: Missing required entity references (e.g., Canvas camera, Slider handle) should log warnings and use safe defaults
- **Circular Hierarchies**: Attempting to parent an element to its own descendant should be rejected with an error
- **Invalid Indices**: Out-of-bounds sibling indices should be clamped to valid range

### Runtime Errors

- **Missing Components**: Accessing a component that doesn't exist should return None/null rather than crashing
- **Destroyed Entities**: Operations on destroyed entities should be no-ops with optional warnings
- **Resource Loading**: Missing textures/fonts should fall back to default resources with error logging
- **Lua Errors**: Lua callback errors should be caught, logged, and not crash the engine

### Performance Safeguards

- **Layout Recursion**: Detect and break infinite layout loops (e.g., parent size depends on child, child size depends on parent)
- **Batch Limits**: If a single batch exceeds vertex/index buffer limits, split into multiple batches
- **Event Flooding**: Rate-limit rapid event firing to prevent performance degradation

## Testing Strategy

### Unit Testing

The UI system will use standard unit tests for:

- **Component Creation**: Verify components initialize with correct default values
- **Anchor Calculations**: Test anchor-to-position calculations with known inputs
- **Layout Algorithms**: Test layout calculations with specific configurations
- **Event Routing**: Test event delivery with known UI hierarchies
- **Serialization**: Test JSON serialization/deserialization of UI components
- **Lua Bindings**: Test Lua API functions with specific inputs

### Property-Based Testing

The UI system will use **proptest** (Rust's property-based testing library) to verify universal properties:

**Testing Framework**: proptest 1.4+

**Test Configuration**: Each property test should run a minimum of 100 iterations to ensure thorough coverage of the input space.

**Property Test Tagging**: Each property-based test must include a comment explicitly referencing the correctness property from this design document using the format: `// Feature: in-game-ui-system, Property N: <property text>`

**Property Implementation**: Each correctness property listed above must be implemented as a single property-based test.

**Key Property Tests**:

1. **Anchor System Properties** (Properties 4-7)
   - Generate random anchor configurations and parent sizes
   - Verify positioning and sizing calculations are correct
   - Test that pivot changes don't affect visual position

2. **Hierarchy Properties** (Properties 8-11)
   - Generate random UI hierarchies
   - Verify transform, visibility, and destruction propagation
   - Test sibling ordering

3. **Layout Properties** (Properties 15-19)
   - Generate random layout configurations with varying child counts
   - Verify spacing, padding, and alignment calculations
   - Test layout recalculation on size changes

4. **Event System Properties** (Properties 20-23)
   - Generate random UI hierarchies with overlapping elements
   - Verify correct event routing and blocking
   - Test callback invocation

5. **Canvas Scaler Properties** (Properties 24-28)
   - Generate random screen resolutions and reference resolutions
   - Verify scale factor calculations for all modes
   - Test clamping behavior

6. **Animation Properties** (Properties 29-30)
   - Generate random animation parameters (duration, easing, values)
   - Verify interpolation correctness at random time points
   - Test callback invocation on completion

7. **Scroll View Properties** (Properties 31-36)
   - Generate random content sizes and viewport sizes
   - Verify clipping, scrolling, and scrollbar behavior
   - Test elastic and inertia physics

8. **Component Properties** (Properties 37-45)
   - Generate random component configurations
   - Verify value clamping, state consistency, and masking
   - Test validation rules

9. **Rendering Properties** (Properties 46-51)
   - Generate random UI hierarchies with varying materials
   - Verify batching decisions and render order
   - Test culling behavior

10. **Lua Integration Properties** (Properties 52-57)
    - Generate random Lua operations
    - Verify ECS state matches Lua operations
    - Test callback invocation

11. **Prefab Properties** (Properties 58-60)
    - Generate random UI hierarchies
    - Verify serialization round-trip preserves structure
    - Test parameterized instantiation

12. **Style Properties** (Properties 61-64)
    - Generate random style configurations
    - Verify style application and inheritance
    - Test theme changes affect all elements

**Generator Strategies**:

- **Smart Anchor Generation**: Generate anchors that cover common patterns (fixed, stretched, mixed)
- **Realistic Hierarchies**: Generate UI hierarchies with realistic depth (2-5 levels) and breadth (2-10 children)
- **Valid Configurations**: Ensure generated configurations are valid (e.g., min < max for sliders)
- **Edge Cases**: Include edge cases in generators (empty strings, zero sizes, boundary values)

### Integration Testing

- **Full UI Workflows**: Test complete UI interactions (e.g., button click → callback → UI update)
- **Multi-Canvas Scenarios**: Test multiple canvases with different render modes
- **Performance Testing**: Measure frame time with large UI hierarchies (1000+ elements)
- **Lua Integration**: Test end-to-end Lua scripts that create and manipulate UI

### Visual Testing

- **Screenshot Comparison**: Capture and compare rendered UI against reference images
- **Layout Verification**: Verify layout calculations produce expected visual results
- **Animation Smoothness**: Verify animations interpolate smoothly without jitter

## Implementation Notes

### Performance Considerations

1. **Dirty Flagging**: Only recalculate layouts and rebuild batches for dirty elements
2. **Spatial Hashing**: Use spatial hashing for efficient raycasting with many elements
3. **Object Pooling**: Pool frequently created/destroyed UI elements (e.g., dropdown items)
4. **Batch Caching**: Cache batch data and only rebuild when necessary
5. **Culling**: Cull off-screen UI elements before batching

### Integration with Existing Systems

1. **ECS Integration**: All UI components are ECS components, enabling queries and systems
2. **Rendering Integration**: UI rendering uses the existing sprite renderer with UI-specific batching
3. **Input Integration**: UI event system integrates with the engine's input system
4. **Scripting Integration**: Lua bindings provide full access to UI system from scripts

### Future Enhancements

1. **UI Animation Timeline**: Visual timeline editor for complex UI animations
2. **UI Templates**: Visual template editor for creating reusable UI patterns
3. **Accessibility**: Screen reader support, keyboard navigation, high contrast modes
4. **Localization**: Built-in text localization system
5. **UI Effects**: Blur, glow, outline, shadow effects for UI elements
6. **Rich Text**: Full rich text support with inline images and custom tags
7. **UI Particles**: Particle effects integrated with UI (e.g., button press effects)
8. **UI Sound**: Automatic sound effects for UI interactions

