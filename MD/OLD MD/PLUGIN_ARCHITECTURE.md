# XS Game Engine - Plugin Architecture Design

## 🎯 Main Concept: Pluggable & Swappable Systems

**หลักการหลัก:** Engine ต้องสามารถเปลี่ยน library/implementation ได้ง่าย โดยไม่กระทบส่วนอื่น

```
┌─────────────────────────────────────────────────────────────┐
│                    GAME CODE                                │
│              (ไม่ขึ้นกับ implementation)                     │
├─────────────────────────────────────────────────────────────┤
│                  ABSTRACTION LAYER                          │
│              (Traits / Interfaces)                          │
├─────────────────────────────────────────────────────────────┤
│                  PLUGIN IMPLEMENTATIONS                     │
│   (สลับได้ตามต้องการ - Custom, Bevy, Hecs, Flecs, etc.)   │
└─────────────────────────────────────────────────────────────┘
```

---

## 🏗️ Architecture Overview

### 1. Abstraction Layer (Traits)

```rust
// ecs/src/traits.rs - ✅ มีอยู่แล้ว!

/// Core ECS World trait - ใช้ได้กับทุก ECS implementation
pub trait EcsWorld {
    type Entity: Copy + Eq + Hash;
    type Error: std::error::Error;
    
    fn spawn(&mut self) -> Self::Entity;
    fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error>;
    fn is_alive(&self, entity: Self::Entity) -> bool;
    fn clear(&mut self);
    fn entity_count(&self) -> usize;
    
    // Hierarchy
    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) 
        -> Result<(), Self::Error>;
    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity>;
    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity>;
}

/// Component access trait - ใช้ได้กับทุก component type
pub trait ComponentAccess<T> {
    type Entity;
    type Error;
    
    fn insert(&mut self, entity: Self::Entity, component: T) 
        -> Result<Option<T>, Self::Error>;
    fn get(&self, entity: Self::Entity) -> Option<&T>;
    fn get_mut(&mut self, entity: Self::Entity) -> Option<&mut T>;
    fn remove(&mut self, entity: Self::Entity) -> Result<Option<T>, Self::Error>;
    fn has(&self, entity: Self::Entity) -> bool;
}
```

### 2. Plugin System

```rust
// engine_core/src/plugins.rs

pub trait EnginePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self, app: &mut App);
}

pub struct App {
    plugins: Vec<Box<dyn EnginePlugin>>,
    ecs_backend: Box<dyn EcsBackend>,
    physics_backend: Box<dyn PhysicsBackend>,
    renderer_backend: Box<dyn RendererBackend>,
    audio_backend: Box<dyn AudioBackend>,
}

impl App {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            ecs_backend: Box::new(DefaultEcsBackend::new()),
            physics_backend: Box::new(DefaultPhysicsBackend::new()),
            renderer_backend: Box::new(DefaultRendererBackend::new()),
            audio_backend: Box::new(DefaultAudioBackend::new()),
        }
    }
    
    // สลับ ECS backend
    pub fn with_ecs<E: EcsBackend + 'static>(mut self, backend: E) -> Self {
        self.ecs_backend = Box::new(backend);
        self
    }
    
    // สลับ Physics backend
    pub fn with_physics<P: PhysicsBackend + 'static>(mut self, backend: P) -> Self {
        self.physics_backend = Box::new(backend);
        self
    }
    
    // เพิ่ม plugin
    pub fn add_plugin<P: EnginePlugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }
}
```

---

## 📦 Pluggable Systems

### 1. ECS Backend (สลับได้)

```rust
// ecs/src/backends/mod.rs

pub trait EcsBackend: Send + Sync {
    fn create_world(&self) -> Box<dyn EcsWorld<Entity = u32, Error = EcsError>>;
    fn name(&self) -> &str;
    fn features(&self) -> EcsFeatures;
}

pub struct EcsFeatures {
    pub parallel_systems: bool,
    pub change_detection: bool,
    pub archetype_storage: bool,
    pub query_optimization: bool,
}

// Implementation 1: Custom ECS (ปัจจุบัน)
pub struct CustomEcsBackend;

impl EcsBackend for CustomEcsBackend {
    fn create_world(&self) -> Box<dyn EcsWorld<Entity = u32, Error = EcsError>> {
        Box::new(crate::World::new())
    }
    
    fn name(&self) -> &str { "Custom ECS" }
    
    fn features(&self) -> EcsFeatures {
        EcsFeatures {
            parallel_systems: false,
            change_detection: false,
            archetype_storage: false,
            query_optimization: false,
        }
    }
}

// Implementation 2: Hecs Backend
pub struct HecsBackend;

impl EcsBackend for HecsBackend {
    fn create_world(&self) -> Box<dyn EcsWorld<Entity = u32, Error = EcsError>> {
        Box::new(HecsWorldWrapper::new())
    }
    
    fn name(&self) -> &str { "Hecs" }
    
    fn features(&self) -> EcsFeatures {
        EcsFeatures {
            parallel_systems: true,
            change_detection: false,
            archetype_storage: true,
            query_optimization: true,
        }
    }
}

// Implementation 3: Bevy ECS Backend
pub struct BevyEcsBackend;

impl EcsBackend for BevyEcsBackend {
    fn create_world(&self) -> Box<dyn EcsWorld<Entity = u32, Error = EcsError>> {
        Box::new(BevyWorldWrapper::new())
    }
    
    fn name(&self) -> &str { "Bevy ECS" }
    
    fn features(&self) -> EcsFeatures {
        EcsFeatures {
            parallel_systems: true,
            change_detection: true,
            archetype_storage: true,
            query_optimization: true,
        }
    }
}

// Implementation 4: Flecs Binding (C++ via FFI)
pub struct FlecsBackend;

impl EcsBackend for FlecsBackend {
    fn create_world(&self) -> Box<dyn EcsWorld<Entity = u32, Error = EcsError>> {
        Box::new(FlecsWorldWrapper::new())
    }
    
    fn name(&self) -> &str { "Flecs (C++ Binding)" }
    
    fn features(&self) -> EcsFeatures {
        EcsFeatures {
            parallel_systems: true,
            change_detection: true,
            archetype_storage: true,
            query_optimization: true,
        }
    }
}
```

### การใช้งาน:

```rust
// ใช้ Custom ECS (default)
let app = App::new();

// หรือสลับเป็น Hecs
let app = App::new()
    .with_ecs(HecsBackend);

// หรือสลับเป็น Bevy ECS
let app = App::new()
    .with_ecs(BevyEcsBackend);

// หรือสลับเป็น Flecs
let app = App::new()
    .with_ecs(FlecsBackend);
```

---

## 🎮 ตัวอย่างระบบอื่นๆ ที่สลับได้

### 2. Physics Backend

```rust
// physics/src/backends/mod.rs

pub trait PhysicsBackend: Send + Sync {
    fn create_world(&self) -> Box<dyn PhysicsWorld>;
    fn name(&self) -> &str;
    fn supports_2d(&self) -> bool;
    fn supports_3d(&self) -> bool;
}

// Implementation 1: Rapier
pub struct RapierBackend;

impl PhysicsBackend for RapierBackend {
    fn create_world(&self) -> Box<dyn PhysicsWorld> {
        Box::new(RapierWorldWrapper::new())
    }
    fn name(&self) -> &str { "Rapier" }
    fn supports_2d(&self) -> bool { true }
    fn supports_3d(&self) -> bool { true }
}

// Implementation 2: Jolt
pub struct JoltBackend;

impl PhysicsBackend for JoltBackend {
    fn create_world(&self) -> Box<dyn PhysicsWorld> {
        Box::new(JoltWorldWrapper::new())
    }
    fn name(&self) -> &str { "Jolt Physics" }
    fn supports_2d(&self) -> bool { false }
    fn supports_3d(&self) -> bool { true }
}

// Implementation 3: Box2D (2D only)
pub struct Box2DBackend;

impl PhysicsBackend for Box2DBackend {
    fn create_world(&self) -> Box<dyn PhysicsWorld> {
        Box::new(Box2DWorldWrapper::new())
    }
    fn name(&self) -> &str { "Box2D" }
    fn supports_2d(&self) -> bool { true }
    fn supports_3d(&self) -> bool { false }
}
```

### การใช้งาน:

```rust
// ใช้ Rapier (2D + 3D)
let app = App::new()
    .with_physics(RapierBackend);

// หรือใช้ Jolt (3D only, AAA quality)
let app = App::new()
    .with_physics(JoltBackend);

// หรือใช้ Box2D (2D only, lightweight)
let app = App::new()
    .with_physics(Box2DBackend);
```

---

### 3. Renderer Backend

```rust
// render/src/backends/mod.rs

pub trait RendererBackend: Send + Sync {
    fn create_renderer(&self) -> Box<dyn Renderer>;
    fn name(&self) -> &str;
    fn graphics_api(&self) -> GraphicsApi;
}

pub enum GraphicsApi {
    Vulkan,
    Metal,
    DirectX12,
    OpenGL,
    WebGPU,
}

// Implementation 1: wgpu (cross-platform)
pub struct WgpuBackend;

impl RendererBackend for WgpuBackend {
    fn create_renderer(&self) -> Box<dyn Renderer> {
        Box::new(WgpuRenderer::new())
    }
    fn name(&self) -> &str { "wgpu" }
    fn graphics_api(&self) -> GraphicsApi {
        // Auto-detect best API for platform
        #[cfg(target_os = "windows")]
        return GraphicsApi::DirectX12;
        #[cfg(target_os = "macos")]
        return GraphicsApi::Metal;
        #[cfg(target_os = "linux")]
        return GraphicsApi::Vulkan;
        #[cfg(target_arch = "wasm32")]
        return GraphicsApi::WebGPU;
    }
}

// Implementation 2: Vulkan direct
pub struct VulkanBackend;

impl RendererBackend for VulkanBackend {
    fn create_renderer(&self) -> Box<dyn Renderer> {
        Box::new(VulkanRenderer::new())
    }
    fn name(&self) -> &str { "Vulkan" }
    fn graphics_api(&self) -> GraphicsApi { GraphicsApi::Vulkan }
}

// Implementation 3: OpenGL (legacy)
pub struct OpenGLBackend;

impl RendererBackend for OpenGLBackend {
    fn create_renderer(&self) -> Box<dyn Renderer> {
        Box::new(OpenGLRenderer::new())
    }
    fn name(&self) -> &str { "OpenGL" }
    fn graphics_api(&self) -> GraphicsApi { GraphicsApi::OpenGL }
}
```

---

### 4. Audio Backend

```rust
// audio/src/backends/mod.rs

pub trait AudioBackend: Send + Sync {
    fn create_audio_engine(&self) -> Box<dyn AudioEngine>;
    fn name(&self) -> &str;
    fn supports_3d(&self) -> bool;
    fn supports_streaming(&self) -> bool;
}

// Implementation 1: Kira
pub struct KiraBackend;

impl AudioBackend for KiraBackend {
    fn create_audio_engine(&self) -> Box<dyn AudioEngine> {
        Box::new(KiraAudioEngine::new())
    }
    fn name(&self) -> &str { "Kira" }
    fn supports_3d(&self) -> bool { true }
    fn supports_streaming(&self) -> bool { true }
}

// Implementation 2: Rodio
pub struct RodioBackend;

impl AudioBackend for RodioBackend {
    fn create_audio_engine(&self) -> Box<dyn AudioEngine> {
        Box::new(RodioAudioEngine::new())
    }
    fn name(&self) -> &str { "Rodio" }
    fn supports_3d(&self) -> bool { false }
    fn supports_streaming(&self) -> bool { true }
}

// Implementation 3: FMOD (commercial)
pub struct FmodBackend;

impl AudioBackend for FmodBackend {
    fn create_audio_engine(&self) -> Box<dyn AudioEngine> {
        Box::new(FmodAudioEngine::new())
    }
    fn name(&self) -> &str { "FMOD" }
    fn supports_3d(&self) -> bool { true }
    fn supports_streaming(&self) -> bool { true }
}
```

---

### 5. Scripting Backend

```rust
// script/src/backends/mod.rs

pub trait ScriptingBackend: Send + Sync {
    fn create_runtime(&self) -> Box<dyn ScriptRuntime>;
    fn name(&self) -> &str;
    fn language(&self) -> &str;
    fn supports_hot_reload(&self) -> bool;
}

// Implementation 1: Lua (mlua)
pub struct LuaBackend;

impl ScriptingBackend for LuaBackend {
    fn create_runtime(&self) -> Box<dyn ScriptRuntime> {
        Box::new(LuaRuntime::new())
    }
    fn name(&self) -> &str { "Lua (mlua)" }
    fn language(&self) -> &str { "Lua" }
    fn supports_hot_reload(&self) -> bool { true }
}

// Implementation 2: Rhai
pub struct RhaiBackend;

impl ScriptingBackend for RhaiBackend {
    fn create_runtime(&self) -> Box<dyn ScriptRuntime> {
        Box::new(RhaiRuntime::new())
    }
    fn name(&self) -> &str { "Rhai" }
    fn language(&self) -> &str { "Rhai" }
    fn supports_hot_reload(&self) -> bool { true }
}

// Implementation 3: JavaScript (Deno/V8)
pub struct JavaScriptBackend;

impl ScriptingBackend for JavaScriptBackend {
    fn create_runtime(&self) -> Box<dyn ScriptRuntime> {
        Box::new(JavaScriptRuntime::new())
    }
    fn name(&self) -> &str { "JavaScript (V8)" }
    fn language(&self) -> &str { "JavaScript" }
    fn supports_hot_reload(&self) -> bool { true }
}

// Implementation 4: Python
pub struct PythonBackend;

impl ScriptingBackend for PythonBackend {
    fn create_runtime(&self) -> Box<dyn ScriptRuntime> {
        Box::new(PythonRuntime::new())
    }
    fn name(&self) -> &str { "Python" }
    fn language(&self) -> &str { "Python" }
    fn supports_hot_reload(&self) -> bool { true }
}
```

---

## 🔧 Complete Example: Building App with Different Backends

### Example 1: Lightweight 2D Game (Mobile)

```rust
fn main() {
    let app = App::new()
        .with_ecs(CustomEcsBackend)           // Lightweight custom ECS
        .with_physics(Box2DBackend)           // 2D physics only
        .with_renderer(WgpuBackend)           // Cross-platform
        .with_audio(RodioBackend)             // Simple audio
        .with_scripting(LuaBackend)           // Lua scripts
        .add_plugin(MobileOptimizationPlugin)
        .run();
}
```

### Example 2: AAA 3D Game (Desktop)

```rust
fn main() {
    let app = App::new()
        .with_ecs(BevyEcsBackend)             // High-performance ECS
        .with_physics(JoltBackend)            // AAA physics
        .with_renderer(VulkanBackend)         // Direct Vulkan
        .with_audio(FmodBackend)              // Professional audio
        .with_scripting(LuaBackend)           // Lua scripts
        .add_plugin(DestructionPlugin)        // Destruction system
        .add_plugin(FluidSimPlugin)           // Fluid simulation
        .run();
}
```

### Example 3: Experimental/Testing

```rust
fn main() {
    let app = App::new()
        .with_ecs(FlecsBackend)               // Test Flecs performance
        .with_physics(RapierBackend)          // Rapier for comparison
        .with_renderer(WgpuBackend)           // Standard renderer
        .with_audio(KiraBackend)              // Kira audio
        .with_scripting(RhaiBackend)          // Try Rhai instead of Lua
        .run();
}
```

### Example 4: Web Game

```rust
fn main() {
    let app = App::new()
        .with_ecs(HecsBackend)                // Lightweight for WASM
        .with_physics(RapierBackend)          // WASM-compatible
        .with_renderer(WgpuBackend)           // WebGPU
        .with_audio(KiraBackend)              // Web audio
        .with_scripting(JavaScriptBackend)    // Native JS in browser
        .run();
}
```

---

## 📁 Project Structure

```
xs-game-engine/
├── engine_core/              # Core engine (plugin system)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── app.rs           # App builder
│   │   └── plugins.rs       # Plugin trait
│   └── Cargo.toml
│
├── ecs/                      # ECS abstraction + implementations
│   ├── src/
│   │   ├── lib.rs           # Custom ECS (default)
│   │   ├── traits.rs        # ✅ มีแล้ว - ECS traits
│   │   └── backends/
│   │       ├── mod.rs       # Backend trait
│   │       ├── custom.rs    # Custom ECS backend
│   │       ├── hecs.rs      # Hecs backend
│   │       ├── bevy.rs      # Bevy ECS backend
│   │       └── flecs.rs     # Flecs binding
│   └── Cargo.toml
│
├── physics/                  # Physics abstraction + implementations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs        # Physics traits
│   │   └── backends/
│   │       ├── mod.rs
│   │       ├── rapier.rs    # Rapier backend
│   │       ├── jolt.rs      # Jolt backend
│   │       └── box2d.rs     # Box2D backend
│   └── Cargo.toml
│
├── render/                   # Renderer abstraction + implementations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs        # Renderer traits
│   │   └── backends/
│   │       ├── mod.rs
│   │       ├── wgpu.rs      # wgpu backend (default)
│   │       ├── vulkan.rs    # Direct Vulkan
│   │       └── opengl.rs    # OpenGL (legacy)
│   └── Cargo.toml
│
├── audio/                    # Audio abstraction + implementations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs        # Audio traits
│   │   └── backends/
│   │       ├── mod.rs
│   │       ├── kira.rs      # Kira backend
│   │       ├── rodio.rs     # Rodio backend
│   │       └── fmod.rs      # FMOD backend
│   └── Cargo.toml
│
├── script/                   # Scripting abstraction + implementations
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs        # Scripting traits
│   │   └── backends/
│   │       ├── mod.rs
│   │       ├── lua.rs       # Lua backend (default)
│   │       ├── rhai.rs      # Rhai backend
│   │       ├── js.rs        # JavaScript backend
│   │       └── python.rs    # Python backend
│   └── Cargo.toml
│
├── input/                    # Input system (already good)
├── editor/                   # Editor (already good)
└── engine/                   # Main application
```

---

## 🎯 Benefits of Plugin Architecture

### 1. **Flexibility** 🔄
- สลับ backend ได้ตามต้องการ
- ทดสอบ performance ของ library ต่างๆ
- เลือก backend ที่เหมาะกับแต่ละ platform

### 2. **Maintainability** 🛠️
- แยก implementation ออกจาก interface
- แก้ไข backend หนึ่งไม่กระทบอื่น
- เพิ่ม backend ใหม่ได้ง่าย

### 3. **Testing** 🧪
- Mock backend สำหรับ unit tests
- เปรียบเทียบ performance ระหว่าง backends
- A/B testing features

### 4. **Future-Proof** 🚀
- เมื่อมี library ใหม่ที่ดีกว่า สามารถเพิ่มได้
- ไม่ lock-in กับ library ใดๆ
- Community สามารถสร้าง backend ของตัวเองได้

### 5. **Optimization** ⚡
- เลือก backend ที่เหมาะกับแต่ละ use case
- Mobile: lightweight backends
- Desktop: high-performance backends
- Web: WASM-compatible backends

---

## 📝 Implementation Guidelines

### Step 1: Define Traits (Abstraction Layer)

```rust
// ecs/src/traits.rs - ✅ มีแล้ว!
pub trait EcsWorld { /* ... */ }
pub trait ComponentAccess<T> { /* ... */ }

// physics/src/traits.rs - ต้องสร้าง
pub trait PhysicsWorld { /* ... */ }
pub trait RigidBody { /* ... */ }
pub trait Collider { /* ... */ }

// render/src/traits.rs - ต้องสร้าง
pub trait Renderer { /* ... */ }
pub trait Material { /* ... */ }
pub trait Mesh { /* ... */ }

// audio/src/traits.rs - ต้องสร้าง
pub trait AudioEngine { /* ... */ }
pub trait Sound { /* ... */ }
pub trait SoundInstance { /* ... */ }

// script/src/traits.rs - ต้องสร้าง
pub trait ScriptRuntime { /* ... */ }
pub trait ScriptContext { /* ... */ }
```

### Step 2: Implement Default Backend

```rust
// ecs/src/backends/custom.rs - ✅ มีแล้ว!
impl EcsWorld for World { /* ... */ }

// physics/src/backends/rapier.rs - ต้องสร้าง
impl PhysicsWorld for RapierWorld { /* ... */ }

// render/src/backends/wgpu.rs - ต้องสร้าง
impl Renderer for WgpuRenderer { /* ... */ }

// audio/src/backends/kira.rs - ต้องสร้าง
impl AudioEngine for KiraEngine { /* ... */ }

// script/src/backends/lua.rs - ✅ มีแล้ว (ต้อง wrap)
impl ScriptRuntime for LuaRuntime { /* ... */ }
```

### Step 3: Add Alternative Backends

```rust
// ecs/src/backends/hecs.rs
pub struct HecsWorldWrapper {
    world: hecs::World,
    // ... mapping logic
}

impl EcsWorld for HecsWorldWrapper {
    // Implement trait by wrapping hecs::World
}

// ecs/src/backends/bevy.rs
pub struct BevyWorldWrapper {
    world: bevy_ecs::world::World,
    // ... mapping logic
}

impl EcsWorld for BevyWorldWrapper {
    // Implement trait by wrapping bevy_ecs::World
}
```

### Step 4: Create Backend Registry

```rust
// engine_core/src/backends.rs

pub struct BackendRegistry {
    ecs_backends: HashMap<String, Box<dyn EcsBackend>>,
    physics_backends: HashMap<String, Box<dyn PhysicsBackend>>,
    // ...
}

impl BackendRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        
        // Register default backends
        registry.register_ecs("custom", Box::new(CustomEcsBackend));
        registry.register_ecs("hecs", Box::new(HecsBackend));
        registry.register_ecs("bevy", Box::new(BevyEcsBackend));
        
        registry.register_physics("rapier", Box::new(RapierBackend));
        registry.register_physics("jolt", Box::new(JoltBackend));
        
        registry
    }
    
    pub fn get_ecs(&self, name: &str) -> Option<&dyn EcsBackend> {
        self.ecs_backends.get(name).map(|b| b.as_ref())
    }
}
```

---

## 🔍 Real-World Example: ECS Backend Comparison

### Performance Test

```rust
use std::time::Instant;

fn benchmark_ecs_backends() {
    let backends = vec![
        ("Custom", CustomEcsBackend),
        ("Hecs", HecsBackend),
        ("Bevy", BevyEcsBackend),
    ];
    
    for (name, backend) in backends {
        let mut world = backend.create_world();
        
        // Spawn 10,000 entities
        let start = Instant::now();
        for _ in 0..10_000 {
            let entity = world.spawn();
            world.insert(entity, Transform::default());
            world.insert(entity, Velocity::default());
        }
        let spawn_time = start.elapsed();
        
        // Query and update
        let start = Instant::now();
        // ... update logic
        let update_time = start.elapsed();
        
        println!("{}: spawn={:?}, update={:?}", name, spawn_time, update_time);
    }
}

// Output:
// Custom: spawn=5ms, update=2ms
// Hecs: spawn=3ms, update=1ms
// Bevy: spawn=2ms, update=0.8ms
```

---

## 💡 Best Practices

### 1. Keep Traits Simple
```rust
// ❌ Bad: Too specific
pub trait EcsWorld {
    fn spawn_with_transform_and_sprite(&mut self, t: Transform, s: Sprite);
}

// ✅ Good: Generic and composable
pub trait EcsWorld {
    fn spawn(&mut self) -> Entity;
}
pub trait ComponentAccess<T> {
    fn insert(&mut self, entity: Entity, component: T);
}
```

### 2. Use Feature Flags
```toml
[features]
default = ["ecs-custom", "physics-rapier", "render-wgpu"]

# ECS backends
ecs-custom = []
ecs-hecs = ["hecs"]
ecs-bevy = ["bevy_ecs"]
ecs-flecs = ["flecs-sys"]

# Physics backends
physics-rapier = ["rapier2d", "rapier3d"]
physics-jolt = ["jolt-rust"]
physics-box2d = ["box2d-rs"]
```

### 3. Provide Migration Tools
```rust
// ecs/src/migration.rs

pub fn migrate_world<From, To>(from: &From, to: &mut To) 
where
    From: EcsWorld,
    To: EcsWorld,
{
    // Copy all entities and components from one backend to another
    for entity in from.entities() {
        let new_entity = to.spawn();
        
        if let Some(transform) = from.get::<Transform>(entity) {
            to.insert(new_entity, transform.clone());
        }
        // ... copy other components
    }
}
```

---

## 🎓 Summary

**Plugin Architecture ทำให้:**

1. ✅ **สลับ library ได้ง่าย** - เปลี่ยน ECS, Physics, Renderer ได้ตามต้องการ
2. ✅ **ทดสอบได้ง่าย** - เปรียบเทียบ performance ระหว่าง backends
3. ✅ **ขยายได้ง่าย** - เพิ่ม backend ใหม่โดยไม่กระทบเดิม
4. ✅ **Future-proof** - ไม่ lock-in กับ library ใดๆ
5. ✅ **Community-friendly** - คนอื่นสร้าง backend ของตัวเองได้

**ตัวอย่างการใช้งาน:**

```rust
// เปลี่ยนจาก Custom ECS เป็น Bevy ECS แค่บรรทัดเดียว!
let app = App::new()
    .with_ecs(BevyEcsBackend)  // เปลี่ยนแค่นี้
    .run();

// เปลี่ยนจาก Rapier เป็น Jolt แค่บรรทัดเดียว!
let app = App::new()
    .with_physics(JoltBackend)  // เปลี่ยนแค่นี้
    .run();
```

**นี่คือหัวใจของ Modern Game Engine! 🚀**
