# 🎨 Unity-Like Editor Upgrade Plan

## 📊 Current Status Analysis

### ✅ What We Already Have (Good!)
- Basic hierarchy panel with entity tree
- Inspector with component editing
- Scene view with gizmos
- Console system
- Asset browser
- Play-in-editor mode
- Project manager
- Transform tools (Move, Rotate, Scale)

### ❌ What's Missing (Unity Features)

#### 1. **Layout & Docking System**
- ❌ No dockable panels
- ❌ Fixed layout (can't rearrange)
- ❌ No layout presets (2 by 3, Tall, Wide)
- ❌ Can't maximize panels

#### 2. **Scene View Improvements**
- ❌ No grid snapping
- ❌ No scene camera controls (pan, zoom, rotate)
- ❌ No gizmo size adjustment
- ❌ No scene lighting preview
- ❌ No wireframe mode
- ❌ No scene statistics (FPS, draw calls)

#### 3. **Inspector Enhancements**
- ❌ No component reordering (drag & drop)
- ❌ No component copy/paste
- ❌ No component presets
- ❌ No multi-object editing
- ❌ No property locking
- ❌ No debug mode

#### 4. **Hierarchy Improvements**
- ❌ No multi-selection
- ❌ No drag & drop to reorder
- ❌ No search/filter
- ❌ No visibility toggles (eye icon)
- ❌ No lock toggles (lock icon)

#### 5. **Toolbar Missing Features**
- ❌ No transform pivot mode (Center/Pivot)
- ❌ No transform space (Local/Global)
- ❌ No play mode tint
- ❌ No pause button
- ❌ No step frame button

#### 6. **Asset Browser Upgrades**
- ❌ No thumbnail previews
- ❌ No asset import settings
- ❌ No asset labels/favorites
- ❌ No asset search
- ❌ No drag & drop to scene

#### 7. **Project Settings**
- ❌ No physics settings
- ❌ No input settings
- ❌ No quality settings
- ❌ No build settings

---

## 🎯 Upgrade Roadmap

### 🔴 Phase 1: Core Layout System (Week 1-2)

**Goal:** Dockable panels like Unity

#### 1.1 Implement egui_dock
```toml
# engine/Cargo.toml
[dependencies]
egui_dock = "0.11"
```

**Features:**
- Drag panels to dock
- Split panels horizontally/vertically
- Tab system for multiple panels in same area
- Save/load layout
- Layout presets (Default, 2 by 3, Tall, Wide)

#### 1.2 Panel System

```rust
// New panel types
enum EditorPanel {
    Hierarchy,
    Inspector,
    Scene,
    Game,
    Console,
    Project,
    Animation,
    Profiler,
}

struct EditorLayout {
    dock_state: egui_dock::DockState<EditorPanel>,
    current_preset: LayoutPreset,
}

enum LayoutPreset {
    Default,    // Unity default layout
    TwoByThree, // 2 columns, 3 rows
    Tall,       // Vertical focus
    Wide,       // Horizontal focus
    Custom,
}
```

---

### 🟡 Phase 2: Scene View Enhancements (Week 3-4)

#### 2.1 Camera Controls
```rust
struct SceneCamera {
    position: Vec2,
    zoom: f32,
    rotation: f32,
}

// Controls:
// - Middle Mouse: Pan
// - Scroll: Zoom
// - Alt + Left Mouse: Orbit (for 3D later)
// - F: Frame selected object
```

#### 2.2 Grid System
```rust
struct SceneGrid {
    enabled: bool,
    size: f32,        // Grid cell size
    snap: bool,       // Snap to grid
    color: Color,
}
```

#### 2.3 Gizmo Improvements

```rust
struct GizmoSettings {
    size: f32,           // Gizmo size multiplier
    show_labels: bool,   // Show X, Y, Z labels
    pivot_mode: PivotMode,
    space_mode: SpaceMode,
}

enum PivotMode {
    Center,  // Object center
    Pivot,   // Object pivot point
}

enum SpaceMode {
    Local,   // Local coordinates
    Global,  // World coordinates
}
```

#### 2.4 Scene Statistics
```rust
struct SceneStats {
    fps: f32,
    entities: usize,
    draw_calls: usize,
    triangles: usize,
}
```

---

### 🟢 Phase 3: Hierarchy Upgrades (Week 5)

#### 3.1 Multi-Selection
```rust
struct HierarchyState {
    selected_entities: Vec<Entity>,  // Multiple selection
    last_selected: Option<Entity>,
}

// Keyboard shortcuts:
// - Ctrl + Click: Add to selection
// - Shift + Click: Range selection
// - Ctrl + A: Select all
```

#### 3.2 Visibility & Lock Toggles
```rust
// Add to World
pub struct EntityVisibility {
    pub visible: bool,
    pub locked: bool,
}

// UI: Eye icon and Lock icon next to each entity
```

#### 3.3 Search & Filter

```rust
struct HierarchyFilter {
    search_text: String,
    filter_by_tag: Option<EntityTag>,
    filter_by_component: Option<ComponentType>,
}

// Search bar at top of hierarchy
// Filter by: Name, Tag, Component type
```

#### 3.4 Drag & Drop Reordering
```rust
// Drag entity to reorder in hierarchy
// Drag entity onto another to make it a child
// Visual feedback during drag
```

---

### 🔵 Phase 4: Inspector Enhancements (Week 6-7)

#### 4.1 Component Reordering
```rust
// Drag component headers to reorder
// Visual feedback (highlight drop zone)
```

#### 4.2 Component Copy/Paste
```rust
struct ComponentClipboard {
    component_type: ComponentType,
    data: Vec<u8>,  // Serialized component data
}

// Right-click menu:
// - Copy Component
// - Paste Component
// - Paste Component Values
```

#### 4.3 Multi-Object Editing
```rust
// When multiple entities selected:
// - Show common components
// - Edit all at once
// - Show "Mixed..." for different values
```

#### 4.4 Component Presets
```rust
struct ComponentPreset {
    name: String,
    component_type: ComponentType,
    values: HashMap<String, Value>,
}

// Save/load component configurations
// Useful for common setups (Player, Enemy, etc.)
```

---

### 🟣 Phase 5: Toolbar & Shortcuts (Week 8)

#### 5.1 Enhanced Toolbar

```rust
struct Toolbar {
    // Transform tools
    current_tool: TransformTool,  // Q, W, E, R
    pivot_mode: PivotMode,        // Center/Pivot
    space_mode: SpaceMode,        // Local/Global
    
    // Play controls
    is_playing: bool,
    is_paused: bool,
    step_frame: bool,
    
    // View options
    show_grid: bool,
    show_gizmos: bool,
    show_wireframe: bool,
}
```

#### 5.2 Keyboard Shortcuts
```rust
// Unity-like shortcuts
Q - View tool (hand)
W - Move tool
E - Rotate tool
R - Scale tool
T - Rect tool (for UI)

F - Frame selected
Ctrl+D - Duplicate
Ctrl+Z - Undo
Ctrl+Y - Redo
Ctrl+S - Save
Ctrl+N - New scene
Ctrl+O - Open scene

Space - Play/Pause
Ctrl+P - Play
Ctrl+Shift+P - Pause

Delete - Delete selected
Ctrl+A - Select all
Ctrl+Shift+A - Deselect all
```

---

### 🟠 Phase 6: Asset Browser Upgrade (Week 9-10)

#### 6.1 Thumbnail Previews
```rust
struct AssetThumbnail {
    asset_path: PathBuf,
    texture: Option<egui::TextureHandle>,
    size: Vec2,
}

// Generate thumbnails for:
// - Images (show actual image)
// - Scenes (show icon)
// - Scripts (show icon)
// - Prefabs (show icon)
```

#### 6.2 Asset Import Settings
```rust
struct ImageImportSettings {
    max_size: u32,
    compression: CompressionType,
    filter_mode: FilterMode,
    generate_mipmaps: bool,
}

// Right-click asset → Import Settings
```

#### 6.3 Asset Labels & Favorites

```rust
struct AssetMetadata {
    path: PathBuf,
    labels: Vec<String>,
    is_favorite: bool,
    last_modified: SystemTime,
}

// Star icon for favorites
// Color-coded labels
// Filter by label
```

#### 6.4 Drag & Drop to Scene
```rust
// Drag sprite from asset browser to scene
// → Creates entity with sprite component
// Drag script to entity in hierarchy
// → Adds script component
```

---

### 🔴 Phase 7: Project Settings Panel (Week 11)

#### 7.1 Physics Settings
```rust
struct PhysicsSettings {
    gravity: Vec2,
    fixed_timestep: f32,
    max_velocity: f32,
    collision_layers: Vec<CollisionLayer>,
}
```

#### 7.2 Input Settings
```rust
struct InputSettings {
    axes: Vec<InputAxis>,
    actions: Vec<InputAction>,
}

struct InputAxis {
    name: String,
    positive_key: Key,
    negative_key: Key,
    sensitivity: f32,
    dead_zone: f32,
}
```

#### 7.3 Quality Settings
```rust
struct QualitySettings {
    vsync: bool,
    target_fps: u32,
    anti_aliasing: AntiAliasingMode,
    texture_quality: TextureQuality,
}
```

#### 7.4 Build Settings
```rust
struct BuildSettings {
    target_platform: Platform,
    output_path: PathBuf,
    optimization_level: OptimizationLevel,
    include_debug_symbols: bool,
}
```

---

## 🛠️ Implementation Details

### Using egui_dock for Docking

```rust
use egui_dock::{DockArea, DockState, NodeIndex, Style};

struct EditorApp {
    dock_state: DockState<EditorPanel>,
}

impl EditorApp {
    fn new() -> Self {
        let mut dock_state = DockState::new(vec![EditorPanel::Scene]);
        
        // Create Unity-like default layout
        let [left, main] = dock_state.main_surface_mut()
            .split_left(NodeIndex::root(), 0.2, vec![EditorPanel::Hierarchy]);
        
        let [main, right] = dock_state.main_surface_mut()
            .split_right(main, 0.25, vec![EditorPanel::Inspector]);
        
        let [_top, bottom] = dock_state.main_surface_mut()
            .split_below(main, 0.7, vec![EditorPanel::Console, EditorPanel::Project]);
        
        Self { dock_state }
    }
    
    fn ui(&mut self, ctx: &egui::Context) {
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer {});
    }
}

struct TabViewer;

impl egui_dock::TabViewer for TabViewer {
    type Tab = EditorPanel;
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorPanel::Hierarchy => render_hierarchy(ui),
            EditorPanel::Inspector => render_inspector(ui),
            EditorPanel::Scene => render_scene_view(ui),
            EditorPanel::Game => render_game_view(ui),
            EditorPanel::Console => render_console(ui),
            EditorPanel::Project => render_project_browser(ui),
            EditorPanel::Animation => render_animation_editor(ui),
            EditorPanel::Profiler => render_profiler(ui),
        }
    }
    
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            EditorPanel::Hierarchy => "Hierarchy".into(),
            EditorPanel::Inspector => "Inspector".into(),
            EditorPanel::Scene => "Scene".into(),
            EditorPanel::Game => "Game".into(),
            EditorPanel::Console => "Console".into(),
            EditorPanel::Project => "Project".into(),
            EditorPanel::Animation => "Animation".into(),
            EditorPanel::Profiler => "Profiler".into(),
        }
    }
}
```

---

## 📊 Priority Matrix

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| **Docking System** | 🔥 High | 🔨 Medium | 🔴 Critical |
| **Scene Camera Controls** | 🔥 High | 🔨 Low | 🔴 Critical |
| **Multi-Selection** | 🔥 High | 🔨 Medium | 🔴 Critical |
| **Keyboard Shortcuts** | 🔥 High | 🔨 Low | 🔴 Critical |
| **Grid Snapping** | 🔥 High | 🔨 Low | 🟡 High |
| **Component Copy/Paste** | 🔥 Medium | 🔨 Medium | 🟡 High |
| **Asset Thumbnails** | 🔥 Medium | 🔨 High | 🟡 High |
| **Visibility Toggles** | 🔥 Medium | 🔨 Low | 🟡 High |
| **Search/Filter** | 🔥 Medium | 🔨 Low | 🟡 High |
| **Component Reordering** | 🔥 Low | 🔨 Medium | 🟢 Medium |
| **Multi-Object Edit** | 🔥 Medium | 🔨 High | 🟢 Medium |
| **Component Presets** | 🔥 Low | 🔨 Medium | 🟢 Medium |
| **Project Settings** | 🔥 High | 🔨 High | 🟢 Medium |

---

## 🎯 Quick Wins (Do First!)

### Week 1: Essential Improvements
1. ✅ Add keyboard shortcuts (Q, W, E, R, F, Ctrl+D, Delete)
2. ✅ Scene camera pan & zoom (Middle mouse + Scroll)
3. ✅ Grid snapping toggle
4. ✅ Multi-selection (Ctrl+Click)
5. ✅ Search bar in hierarchy

**Impact:** Massive UX improvement with minimal effort!

### Week 2: Docking System
1. ✅ Install egui_dock
2. ✅ Convert panels to dockable tabs
3. ✅ Add layout presets
4. ✅ Save/load layout

**Impact:** Professional editor feel!

---

## 📝 Code Structure Changes

### New Files to Create
```
engine/src/editor/
├── layout.rs          # Docking system
├── shortcuts.rs       # Keyboard shortcuts
├── camera.rs          # Scene camera
├── gizmos.rs          # Enhanced gizmos
├── clipboard.rs       # Copy/paste system
├── settings/
│   ├── mod.rs
│   ├── physics.rs
│   ├── input.rs
│   ├── quality.rs
│   └── build.rs
└── panels/
    ├── mod.rs
    ├── hierarchy.rs   # Enhanced hierarchy
    ├── inspector.rs   # Enhanced inspector
    ├── scene.rs       # Enhanced scene view
    ├── game.rs        # Game view
    ├── console.rs     # Console
    ├── project.rs     # Asset browser
    ├── animation.rs   # Animation editor
    └── profiler.rs    # Profiler
```

---

## 🚀 Getting Started

### Step 1: Add Dependencies
```toml
# engine/Cargo.toml
[dependencies]
egui_dock = "0.11"
glam = "0.25"
```

### Step 2: Create Layout System
```bash
# Create new file
touch engine/src/editor/layout.rs
```

### Step 3: Implement Keyboard Shortcuts
```bash
# Create new file
touch engine/src/editor/shortcuts.rs
```

---

## 📚 Resources

### egui_dock Examples
- https://github.com/Adanos020/egui_dock
- https://docs.rs/egui_dock/latest/egui_dock/

### Unity Editor Reference
- Unity Manual: https://docs.unity3d.com/Manual/UsingTheEditor.html
- Unity Shortcuts: https://docs.unity3d.com/Manual/UnityHotkeys.html

---

**Last Updated:** 2025-11-26
**Status:** Planning Phase
**Estimated Time:** 11 weeks for full implementation
**Quick Wins:** 1-2 weeks for major UX improvements
