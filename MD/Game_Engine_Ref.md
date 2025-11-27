# XS GAME Engine Design - Mobile First with AI-Powered Development

## 🎯 Engine Vision & Goals

### Target Games & Genres

**2D/2.5D Games:**
- **Celeste-style Platformer** - Precise physics, pixel-perfect movement
- **Katana Zero** - Fast-paced action, time manipulation, cinematic effects
- **Dead Cells** - Roguelike, procedural generation, fluid combat

**3D Games:**
- **The Witcher 3 Style RPG** - Open world, complex quests, rich storytelling
- **Action RPG** - Third-person combat, inventory systems, character progression

**Core Philosophy:**
- **Mobile-First Design** - Optimized for mobile devices, scales up to desktop
- **Modular ECS Architecture** - Clean separation, easy to extend
- **AI-Assisted Development** - LLM integration for rapid prototyping
- **AAA-Quality Systems** - Destruction (Battlefield-level), Fluid simulation

---

# 🏗️ Modular Architecture Overview

## Core Engine Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    XS GAME ENGINE                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   AI Layer   │  │  Editor UI   │  │  Dev Tools   │    │
│  │              │  │              │  │              │    │
│  │ • LLM API    │  │ • Scene View │  │ • Profiler   │    │
│  │ • Script Gen │  │ • Inspector  │  │ • Debugger   │    │
│  │ • Level Gen  │  │ • Asset Mgr  │  │ • Console    │    │
│  │ • Bug Detect │  │ • Play Mode  │  │ • Analytics  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    GAME SYSTEMS LAYER                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Gameplay    │  │  Animation   │  │   Audio      │    │
│  │              │  │              │  │              │    │
│  │ • Character  │  │ • Skeletal   │  │ • 3D Sound   │    │
│  │ • Inventory  │  │ • Blend Tree │  │ • Music Sys  │    │
│  │ • Quest Sys  │  │ • IK System  │  │ • DSP/FX     │    │
│  │ • Dialogue   │  │ • Ragdoll    │  │ • Streaming  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Advanced    │  │  Procedural  │  │   Network    │    │
│  │  Physics     │  │  Generation  │  │              │    │
│  │              │  │              │  │ • Multiplayer│    │
│  │ • Destruction│  │ • Terrain    │  │ • Sync       │    │
│  │ • Cloth Sim  │  │ • Dungeons   │  │ • Lobby      │    │
│  │ • Fluid/SPH  │  │ • Quests     │  │ • P2P/Server │    │
│  │ • Soft Body  │  │ • Loot       │  │              │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    CORE ENGINE LAYER                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │     ECS      │  │   Renderer   │  │   Physics    │    │
│  │              │  │              │  │              │    │
│  │ • Entities   │  │ • Forward+   │  │ • Jolt/Rapier│    │
│  │ • Components │  │ • Deferred   │  │ • Collision  │    │
│  │ • Systems    │  │ • PBR/Toon   │  │ • Triggers   │    │
│  │ • Queries    │  │ • 2D/3D      │  │ • Raycasts   │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Scripting  │  │    Input     │  │   Assets     │    │
│  │              │  │              │  │              │    │
│  │ • Lua/Rhai   │  │ • Keyboard   │  │ • Loader     │    │
│  │ • Hot Reload │  │ • Mouse      │  │ • Streaming  │    │
│  │ • Debugging  │  │ • Gamepad    │  │ • Bundles    │    │
│  │ • Profiling  │  │ • Touch      │  │ • Hot Reload │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                   PLATFORM ABSTRACTION                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   Graphics   │  │   Platform   │  │   Memory     │    │
│  │              │  │              │  │              │    │
│  │ • Vulkan     │  │ • Windows    │  │ • Allocators │    │
│  │ • Metal      │  │ • Linux      │  │ • Pools      │    │
│  │ • DX12       │  │ • macOS      │  │ • Tracking   │    │
│  │ • OpenGL ES  │  │ • Android    │  │ • Profiling  │    │
│  │ • WebGPU     │  │ • iOS/Web    │  │              │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

# 🎮 Feature Roadmap by Game Type

## 1. 2D Platformer Features (Celeste-like)

### Core Systems
- **Precise Physics Engine**
  - Sub-pixel positioning (fixed-point math)
  - Custom collision detection (AABB, slopes, one-way platforms)
  - Coyote time, jump buffering, variable jump height
  - Wall slide, wall jump, dash mechanics
  
- **Pixel-Perfect Rendering**
  - Integer scaling, no texture filtering
  - Sprite batching, atlas packing
  - Parallax backgrounds (multiple layers)
  - Screen shake, freeze frames
  
- **Tilemap System**
  - Autotiling with rule tiles
  - Animated tiles
  - Collision layer separation
  - Tile-based triggers
  
- **Camera System**
  - Smooth follow with deadzone
  - Look-ahead prediction
  - Room-based transitions
  - Zoom effects

### Advanced Features
- **Time Manipulation** (Katana Zero)
  - Rewind system (record/playback)
  - Slow-motion effects
  - Frame-perfect input replay
  
- **Procedural Generation** (Dead Cells)
  - Room templates
  - Graph-based level generation
  - Loot tables, enemy spawning
  - Seed-based randomization

---

## 2. 3D RPG Features (Witcher 3-like)

### Core Systems
- **Open World Rendering**
  - Terrain system (heightmap, splatmap)
  - LOD system (mesh, texture)
  - Occlusion culling (frustum, portal)
  - Streaming (async asset loading)
  
- **Character System**
  - Skeletal animation (GPU skinning)
  - Animation blending (state machine)
  - Inverse Kinematics (foot placement)
  - Facial animation, lip sync
  
- **Combat System**
  - Lock-on targeting
  - Combo system
  - Hitbox/hurtbox detection
  - Damage calculation, stats
  
- **Quest & Dialogue**
  - Quest graph (objectives, branches)
  - Dialogue trees (choices, conditions)
  - Journal system
  - NPC AI (behavior trees)

### Advanced Features
- **Weather & Time of Day**
  - Dynamic sky, sun/moon
  - Weather transitions (rain, fog, snow)
  - Lighting changes
  
- **AI Systems**
  - Pathfinding (A*, navmesh)
  - Behavior trees
  - Perception system (sight, sound)
  - Group tactics

---

## 3. Destruction System (Battlefield-level)

### Architecture

```rust
// Destruction ECS Components
struct Destructible {
    health: f32,
    fracture_threshold: f32,
    debris_prefab: AssetHandle,
}

struct FractureData {
    voronoi_cells: Vec<ConvexHull>,
    connection_graph: Graph<usize>,
    break_force: f32,
}

struct DebrisParticle {
    lifetime: f32,
    velocity: Vec3,
    angular_velocity: Vec3,
}
```

### Implementation Strategy

**Level 1: Simple Destruction (Mobile-Friendly)**
- Pre-fractured meshes (swap intact → broken)
- Particle effects for debris
- Decals for damage
- ~5-10ms per destruction event

**Level 2: Dynamic Fracturing (Mid-Range)**
- Runtime Voronoi fracturing (cached)
- Physics simulation for chunks (10-20 pieces)
- Constraint solver for connections
- ~10-20ms per event

**Level 3: Advanced Destruction (High-End)**
- Real-time fracturing (GPU compute)
- Hundreds of debris pieces
- Soft-body deformation
- Fluid simulation integration
- ~20-50ms per event

### Mobile Optimization
```rust
struct DestructionBudget {
    max_active_debris: usize,      // 50-100 on mobile
    max_fractures_per_frame: usize, // 2-3 on mobile
    debris_lifetime: f32,           // 5-10 seconds
    lod_distance: f32,              // Simplify far away
}
```

---

## 4. Fluid & Water VFX System (Mobile-First)

### Approach: Hybrid System

**GPU Particle-Based Fluid (SPH - Smoothed Particle Hydrodynamics)**

```rust
// Compute Shader (WGSL/GLSL)
struct FluidParticle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    density: f32,
    pressure: f32,
}

// SPH Algorithm (simplified for mobile)
fn compute_density(particles: &[FluidParticle]) {
    // Neighbor search (spatial hash grid)
    // Density calculation (kernel function)
    // Pressure calculation (equation of state)
}

fn compute_forces(particles: &mut [FluidParticle]) {
    // Pressure force
    // Viscosity force
    // External forces (gravity, wind)
}

fn integrate(particles: &mut [FluidParticle], dt: f32) {
    // Velocity Verlet integration
    // Collision detection (SDF)
    // Boundary handling
}
```

**Mobile Optimization Techniques:**

1. **Reduced Particle Count**
   - Desktop: 100k-500k particles
   - Mobile: 5k-20k particles
   - Use larger particle radius

2. **Spatial Hashing**
   - Grid-based neighbor search
   - O(n) instead of O(n²)
   - GPU-friendly parallel algorithm

3. **Screen-Space Rendering**
   - Render particles as point sprites
   - Depth-based smoothing (bilateral filter)
   - Normal reconstruction from depth
   - Cheap reflections/refractions

4. **LOD System**
   - Far particles: billboards
   - Mid-range: low-res simulation
   - Close-up: full simulation

**Water Surface Rendering:**

```rust
// Water shader features (mobile-friendly)
struct WaterMaterial {
    // Base
    color: Color,
    transparency: f32,
    
    // Waves (Gerstner waves)
    wave_amplitude: f32,
    wave_frequency: f32,
    wave_speed: f32,
    
    // Rendering
    fresnel_strength: f32,
    refraction_distortion: f32,
    foam_texture: AssetHandle,
    
    // Performance
    use_planar_reflection: bool,  // Cheap
    use_ssr: bool,                // Expensive
    use_caustics: bool,           // Medium
}
```

**Performance Targets:**
- Mobile: 60 FPS with 5k-10k particles
- Desktop: 60 FPS with 50k-100k particles
- Fluid simulation: 2-5ms per frame (mobile)

---

## 5. AI/LLM Integration for Development

### Architecture

```rust
// AI Service Layer
struct AIAssistant {
    llm_client: LLMClient,
    engine_context: EngineDataset,
    code_generator: CodeGenerator,
    level_designer: LevelDesigner,
}

impl AIAssistant {
    // Script Generation
    async fn generate_script(&self, prompt: &str) -> Result<String> {
        let context = self.engine_context.get_api_docs();
        let response = self.llm_client.complete(prompt, context).await?;
        Ok(response)
    }
    
    // Scene Creation
    async fn create_demo_scene(&self, description: &str) -> Result<Scene> {
        // Parse description
        // Generate entity hierarchy
        // Add components
        // Return scene
    }
    
    // Level Design
    async fn generate_level(&self, params: LevelParams) -> Result<Level> {
        // Procedural generation with AI guidance
        // Ensure playability
        // Balance difficulty
    }
    
    // Bug Detection
    async fn analyze_code(&self, code: &str) -> Vec<Issue> {
        // Static analysis
        // Pattern matching
        // Suggest fixes
    }
}
```

### Use Cases

**1. Script Generation**
```
User: "Create a player controller with WASD movement and jump"

AI Output:
```lua
-- Generated Player Controller
local Player = {}

function Player:new()
    local obj = {
        speed = 200,
        jump_force = 500,
        is_grounded = false
    }
    setmetatable(obj, self)
    self.__index = self
    return obj
end

function Player:update(dt)
    local input = Input.get_axis("horizontal")
    self.velocity.x = input * self.speed
    
    if Input.is_key_pressed("space") and self.is_grounded then
        self.velocity.y = -self.jump_force
    end
    
    -- Apply gravity
    self.velocity.y = self.velocity.y + 980 * dt
end

return Player
```
```

**2. Scene Demo Creation**
```
User: "Create a forest scene with trees, rocks, and a campfire"

AI: Generates scene with:
- Terrain entity (grass texture)
- 10-15 tree entities (random positions)
- 5-8 rock entities (scattered)
- Campfire entity (particle system, light)
- Ambient sound (birds, wind)
```

**3. Level Design Assistance**
```
User: "Design a platformer level with increasing difficulty"

AI: Generates:
- Start area (safe, tutorial)
- 3-4 challenge sections (jumps, enemies)
- Secret areas (optional collectibles)
- Boss arena (final challenge)
- Ensures proper pacing and flow
```

**4. Bug Detection**
```
AI analyzes code and finds:
- Memory leaks (unreleased resources)
- Logic errors (null pointer access)
- Performance issues (nested loops)
- Best practice violations
```

### Dataset Structure

```
engine_dataset/
├── api_reference/
│   ├── ecs_api.md
│   ├── rendering_api.md
│   ├── physics_api.md
│   └── scripting_api.md
├── examples/
│   ├── player_controller.lua
│   ├── enemy_ai.lua
│   ├── level_generation.lua
│   └── ui_system.lua
├── patterns/
│   ├── component_patterns.md
│   ├── system_patterns.md
│   └── optimization_patterns.md
└── common_issues/
    ├── crash_patterns.md
    ├── performance_issues.md
    └── solutions.md
```

### LLM API Integration

```rust
// Example: OpenAI API
struct LLMClient {
    api_key: String,
    model: String, // "gpt-4", "claude-3", etc.
}

impl LLMClient {
    async fn complete(&self, prompt: &str, context: &str) -> Result<String> {
        let request = CompletionRequest {
            model: &self.model,
            messages: vec![
                Message::system(context),
                Message::user(prompt),
            ],
            temperature: 0.7,
            max_tokens: 2000,
        };
        
        let response = self.send_request(request).await?;
        Ok(response.choices[0].message.content.clone())
    }
}
```

---

# 🛠️ Recommended Technology Stack

## Core Engine

### ECS Framework
**Option 1: Bevy ECS** (Recommended)
- Mature, well-tested
- Excellent performance
- Great documentation
- Active community

**Option 2: Hecs**
- Lightweight
- Flexible
- Good for custom engines

**Option 3: Custom ECS**
- Full control
- Optimized for specific needs
- More work upfront

### Physics

**2D Physics:**
- **Rapier2D** - Rust-native, excellent performance
- Features: Rigid bodies, colliders, joints, raycasts

**3D Physics:**
- **Jolt Physics** (via jolt-rust) - AAA-quality, used in Horizon games
- **Rapier3D** - Good alternative, pure Rust
- Features: Rigid bodies, soft bodies, cloth, vehicles

**Destruction Physics:**
- Custom Voronoi fracturing
- Integration with Jolt/Rapier
- GPU compute for large-scale destruction

### Rendering

**Graphics API:**
- **wgpu** - Cross-platform (Vulkan, Metal, DX12, WebGPU)
- Modern, safe, well-maintained

**Rendering Pipeline:**
```rust
// Forward+ for 3D (high-quality)
// Forward for 2D (simple, fast)

struct RenderPipeline {
    // Passes
    shadow_pass: ShadowPass,
    depth_prepass: DepthPrepass,
    opaque_pass: OpaquePass,
    transparent_pass: TransparentPass,
    post_process: PostProcessPass,
    
    // Features
    pbr_materials: bool,
    dynamic_lighting: bool,
    shadows: ShadowQuality,
    anti_aliasing: AAMethod, // FXAA, TAA, MSAA
}
```

### Scripting

**Lua (mlua)**
- Fast, lightweight
- Easy to learn
- Great for gameplay logic

**Rhai (Alternative)**
- Rust-native
- Type-safe
- Good IDE support

### Audio

**Kira**
- Game-focused
- Streaming support
- 3D audio
- Effects (reverb, delay, etc.)

### UI

**Editor UI: egui**
- Immediate mode
- Fast iteration
- Good for tools

**Game UI: Custom System**
- Retained mode
- Optimized for runtime
- Skinnable, themeable

---

# 📊 Performance Targets

## Mobile (Mid-Range Device)

| System | Target | Budget |
|--------|--------|--------|
| **Frame Time** | 16.6ms (60 FPS) | Total |
| Rendering | 8-10ms | 50-60% |
| Physics | 2-3ms | 12-18% |
| Gameplay Logic | 2-3ms | 12-18% |
| Scripting | 1-2ms | 6-12% |
| Audio | 0.5-1ms | 3-6% |
| **Destruction** | 5-10ms (spike) | Amortized |
| **Fluid Sim** | 2-5ms | 12-30% |

## Desktop (High-End)

| System | Target | Budget |
|--------|--------|--------|
| **Frame Time** | 16.6ms (60 FPS) | Total |
| Rendering | 10-12ms | 60-72% |
| Physics | 2-3ms | 12-18% |
| Gameplay Logic | 1-2ms | 6-12% |
| **Destruction** | 20-50ms (spike) | Amortized |
| **Fluid Sim** | 5-10ms | 30-60% |

## Memory Budget

**Mobile:**
- Total: 2-4 GB
- Textures: 500 MB - 1 GB
- Meshes: 200-500 MB
- Audio: 100-200 MB
- Code/Scripts: 50-100 MB
- Runtime: 500 MB - 1 GB

**Desktop:**
- Total: 4-8 GB
- Textures: 2-4 GB
- Meshes: 1-2 GB
- Audio: 500 MB - 1 GB
- Runtime: 1-2 GB

---

# 🚀 Implementation Roadmap

## Phase 1: Foundation (Months 1-3)

### Month 1: Core ECS & Rendering
- [ ] ECS architecture (Bevy/Hecs)
- [ ] Basic 2D renderer (sprites, batching)
- [ ] Basic 3D renderer (meshes, PBR)
- [ ] Asset loading system
- [ ] Input system (keyboard, mouse, gamepad)

### Month 2: Physics & Scripting
- [ ] 2D physics (Rapier2D)
- [ ] 3D physics (Jolt/Rapier3D)
- [ ] Lua scripting integration
- [ ] Hot reload system
- [ ] Basic editor (scene view, inspector)

### Month 3: Core Gameplay
- [ ] Character controller (2D & 3D)
- [ ] Camera system
- [ ] Animation system (sprite & skeletal)
- [ ] Audio system (Kira)
- [ ] UI system (basic)

## Phase 2: Advanced Features (Months 4-6)

### Month 4: Destruction System
- [ ] Voronoi fracturing algorithm
- [ ] Physics integration
- [ ] Debris management
- [ ] Mobile optimization
- [ ] LOD system

### Month 5: Fluid Simulation
- [ ] SPH implementation (GPU)
- [ ] Spatial hashing
- [ ] Screen-space rendering
- [ ] Water surface shader
- [ ] Performance optimization

### Month 6: AI Integration
- [ ] LLM API client
- [ ] Engine dataset creation
- [ ] Script generation
- [ ] Scene generation
- [ ] Bug detection

## Phase 3: Game-Specific Features (Months 7-9)

### Month 7: 2D Platformer
- [ ] Precise physics (sub-pixel)
- [ ] Tilemap system
- [ ] Time manipulation (rewind)
- [ ] Procedural generation

### Month 8: 3D RPG
- [ ] Open world rendering
- [ ] Quest system
- [ ] Dialogue system
- [ ] AI behaviors

### Month 9: Polish & Optimization
- [ ] Performance profiling
- [ ] Memory optimization
- [ ] Mobile testing
- [ ] Documentation

## Phase 4: Production (Months 10-12)

### Month 10: Tools & Pipeline
- [ ] Asset pipeline
- [ ] Build system
- [ ] Export (Windows, Linux, Android)
- [ ] Debugging tools

### Month 11: Advanced Editor
- [ ] Visual scripting
- [ ] Terrain editor
- [ ] Particle editor
- [ ] Material editor

### Month 12: Release Preparation
- [ ] Example projects
- [ ] Tutorials
- [ ] API documentation
- [ ] Community setup

---

# XS GAME Engine Design - Mobile First

# Runtime Gameplay

- Components of the Gameplay Foundation System
- Runtime Object Model Architectures
- World Chunk Data Formats
- Loading and Streaming Game Worlds
- Object References and World Queries
- Updating Game Objects in Real Time
- Applying Concurrency to Game Object Updates
- Events and Message-Passing
- Scripting
- High-Level Game Flow

# Gameplay Systems

- Anatomy of a Game World
- Implementing Dynamic Elements: Game Objects
- Data-Driven Game Engines
- The Game World Editor

# Audio

- The Physics of Sound
- The Mathematics of Sound
- The Technology of Sound
- Rendering Audio in 3D
- Audio Engine Architecture
- Game-Specific Audio Features

# Collision and RigidBody Dynamics

- Do You Want Physics in Your Game?
- Collision/Physics Middleware
- The Collision Detection System
- Rigid Body Dynamics
- Integrating a Physics Engine into Your Game
- Advanced Physics Features

# Animation Systems

- Types of Character Animation
- Skeletons
- Poses
- Clips
- Skinning and Matrix Palette Generation
- Animation Blending
- Post-Processing
- Compression Techniques
- The Animation Pipeline
- Action State Machines
- Constraints

# The Rendering Engine

- Foundations of Depth-Buffered Triangle Rasterization
- The Rendering Pipeline
- Advanced Lighting and Global Illumination
- Visual Effects and Overlays
- Further Reading

# Tools for Debugging and Development

- Logging and Tracing
- Debug Drawing Facilities
- In-Game Menus
- In-Game Console
- Debug Cameras and Pausing the Game
- Cheats
- Screenshots and Movie Capture
- In-Game Profiling
- In-Game Memory Stats and Leak Detection

# Human Interface Devices

- Types of Human Interface Devices
- Interfacing with a HID
- Types of Inputs
- Types of Outputs
- Game Engine HID Systems
- Human Interface Devices in Practice

# The Game Loop and Real-Time Simulation

- The Rendering Loop
- The Game Loop
- Game Loop Architectural Styles
- Abstract Timelines
- Measuring and Dealing with Time
- Multiprocessor Game Loops

# Resources and the File System

- File System
- The Resource Manager

# Engine Support Systems

- Subsystem Start-Up and Shut-Down
- Memory Management
- Containers
- Strings
- Engine Configuration

# 3D Math for Games

- Solving 3D Problems in 2D
- Points and Vectors
- Matrices
- Quaternions
- Comparison of Rotational Representations
- Other Useful Mathematical Objects
- Random Number Generation

# Parallelism & Concurrent

- Defining Concurrency and Parallelism
- Implicit Parallelism
- Explicit Parallelism
- Operating System Fundamentals
- Introduction to Concurrent Programming
- Thread Synchronization Primitives
- Problems with Lock-Based Concurrency
- Some Rules of thumb for Concurrency
- Lock-Free Concurrency
- SIMD/Vector Processing
- Introduction to GPGPU Programming

# TOOLS

- Version Control
- CI CD
- Complier, Linkers and IDEs
- Profile Tools
- Memory Leak and Corruption Detection

---

# Physics

- **jolt-rust**

**การเรนเดอร์:**

- เรนเดอร์ DirectX 12
- เรนเดอร์ Vulkan
- การเรนเดอร์ภาพ
- การเรนเดอร์ฟอนต์ (True Type)
- การเรนเดอร์ 3D Mesh
- การเรนเดอร์แสงสว่าง: จุด, สปอตไลท์, ดวงดาว
- การสร้างภาพสะท้อนและการหักเห: Planar reflections, Cube map reflections (static และ real time), Refractions
- แสงสะท้อนแบบ real-time และ chromatic aberration
- การเรนเดอร์ที่มีคุณภาพสูง: Bloom, Edge outline, Motion Blur, Lens Flare, Light shafts, Bokeh Depth of Field
- Normal mapping, Displacement mapping, Parallax occlusion mapping
- การสร้างและจัดการพาร์ทิเคิล: GPU-based particles, Hair particle systems, Soft particles
- การเรนเดอร์ที่ใช้เทคนิคใหม่: Tessellation, Multithreaded rendering, Variable Rate Shading, Supersampling, MSAA, FXAA, TAA (Temporal Antialiasing)
- Shadow mapping: Cascaded shadow maps, Shadow cubemaps, Soft shadows (PCF)

**ฟิสิกส์และการจำลอง:**

- ฟิสิกส์: rigid body, soft body, ragdoll
- การจำลองน้ำ: Interactive Water, Ocean simulation (FFT), Smooth Particle Hydrodynamics (SPH)
- การจำลองแรง: Force Fields GPU simulation
- การจำลองฟลูอิด: Particle - Depth Buffer collisions
- การจำลองการเคลื่อนที่ของสิ่งมีชีวิต: Springs, Colliders

**เสียง:**

- 3D Audio (Xaudio2)
- การตอบสนองของคอนโทรลเลอร์: Vibration, LED

**สคริปต์และการจัดการข้อมูล:**

- Lua Scripting
- ระบบการจัดการข้อมูล: Resource Manager, Job system
- ระบบ Entity-Component System (Data oriented design)

**การจัดการพื้นผิวและแสง:**

- Color Grading
- Gamma correct, HDR rendering
- Screen Space Ambient Occlusion (SSAO, HBAO, MSAO)
- Screen Space Contact Shadows
- Dynamic environment mapping
- Texture atlas packing
- Tiled decals
- Texture streaming
- Lightmap baking (with GPU path tracing)
- Dynamic Diffuse Global Illumination (DDGI)
- Voxel Global Illumination
- Surfel GI

**การสร้างและการควบคุม:**

- Procedural terrain generator
- Animation retargeting
- Humanoid rig
- Inverse Kinematics
- Path finding 3D
- Virtual textures
- Expressions
- Video decoding: H264

**การเรนเดอร์แบบใหม่:**

- Ray tracing, path tracing (on GPU)
- Real time ray tracing: ambient occlusion, shadows, reflections (DXR and Vulkan raytracing)
- Stochastic Screen Space Reflections
- Parallax-corrected environment maps
- Volumetric light scattering
- Stochastic alphatest transparency

**การสนับสนุน GLTF 2.0:**

- KHR extensions: materials_unlit, materials_transmission, materials_pbrSpecularGlossiness, materials_sheen, materials_clearcoat, materials_specular, materials_anisotropy, materials_ior, materials_emissive_strength, texture_basisu, lights_punctual, lights_image_based
- VRM 0.0 extensions: secondaryAnimation, blendShapeMaster, humanoid
- VRM 1.0 extensions: springBone, vrm_expressions, vrm_humanoid

# **Features of the engine**

**General**

**Create 3D Games**

Create first class 3D games. The engine provides almost everything you need to create 3D games.

**Create 2D Games**

Want something simpler, then create first class 2D games!

**Mix 2D with 3D**

Want to create something else? Then try mixing 2D with 3D, everything is limited by your imagination.

**Multiple Scenes**

Create and manage multiple scenes for various game parts, for example one for game menu and one per game level.

**Scripting**

Write your game entirely in Rust using powerful scripting system of the engine.

**Multi-mode**

Use the engine either as Framework or as Full-Featured Game Engine. The first option allows you to use the engine in a editor-less mode, it is possible to build your own tooling using this mode. The latter option releases the full power of the engine, allowing you to run your game in the editor and use full power of the editor.

**Editor**

**Full-Featured Editor**

Use the editor to create levels for your games. It handles 2D as well as 3D, providing enough flexibility to mix both.

**World Viewer**

World viewer allows you to see objects hierarchy in the world, search objects, attach/detach objects, etc.

**Asset Browser**

Manage your game assets in easy way. The asset browser allows you to preview your assets, change their properies, etc.

**Animation Blending State Machine Editor**

Use animation blending state machine editor to create complex animations using simple ones.

**Curve Editor**

Use curve editor to create complex behavior for numeric parameters.

**Navigation Mesh Editor**

Create and edit navigational meshes for path finding.

**Property Inspector**

Edit properties of your game objects in unified way.

**Material Editor**

Edit properties of your materials.

**Lightmapper**

Bake static lighting into a texture to speed up rendering.

**Scene Graph**

**Wide Variety of Built-in Nodes**

By default the engine offers Camera, Collider, Decal, Joint, Pivot, RigidBody, Sprite, Terrain, 2D Rigid Body, 2D Collider, 2D Joint, Rectangle (2D Sprite), Point Light, Spot Light, Directional Light, Mesh, Particle System, Sound Source.

**Property Inheritance**

Object properties are inheritable, this means that you can create a prefab, put its instances on a scene and every change to prefab properties will reflect to its instances (if there was no manual change to a property).

**Rendering**

**Advanced Lighting System**

Use built-in advanced lighting system, that can handle tons of light sources with soft shadows.

**Custom Materials and Shaders**

Create your own materials and shaders.

**Render To Texture**

Render scene to a texture and use it later, it could be useful for offscreen rendering.

**Various Built-in Postprocessing Effects**

Use built-in postprocessing effects, such as SSAO, FXAA, Color Grading, Tone Mapping, etc.

**Physically-based Rendering (PBR)**

Standard material provides easy-to-use physically-based rendering with metallic workflow.

**Skybox**

Use skybox to add details to background of your scenes.

**Asset Management**

**Wide Variety of Supported Formats**

The engine supports import of following formats: 3D models - FBX; Textures - DDS, PNG, TGA, JPG, TIFF; Sound - OGG, WAV. There are also few native engine formats for: shaders, curves, animation blending state machines.

**Fully Asynchronous Asset Import**

Import tons of assets in parallel and fully utilize the available power of your CPU to speed up asset loading.

**Sound**

**Sound Sources**

Use powerful sound system to create rich sound environment. Mix spatial sound sources with 2D sources using spatial blend parameter, tweak various parameters (distance falloff, gain, etc.) to get best results.

**Head-Related Transfer Function (HRTF)**

HRTF dramatically increases perception of sound, it makes it sound natural as if it'd be in real world. The engine offers 40+ individual 'head models' to allow you to pick one that best suits you or your players.

**Sound Effects**

Use sound effects (such as reverb) to improve sound quality and make it sound more natural. Digital sound processing (DSP) module will help you to create your own, custom effects.

**Physics**

**Rigid Body Dynamics**

Rigid body dynamics provides you with everything you need for physics simulation, including joints.

**Various Collider Shapes**

The engine offers a lot of colliders shapes, the list includes: ball, capsule, cube, cone, cylinder, triangle mesh, convex hull, etc.

**Powered by Rapier**

The engine offers powerful physics system that is powered by Rapier physics engine.

**User Interface**

**Animation**

**Skeletal Animation**

Use GPU-powered skeletal animation to create quality animations for your game characters. GPU acceleration means that you can have tons of objects with skeletal animation with very little performance overhead.

**Animation Blending State Machine**

Animation blending state machine (ABSM) allows you to create state graphs to mix multiple animation into one, creating complex animations from simple ones.

**Events**

Add custom events to a animation timeline and be notified when they'll happen. It could be useful to add foostep sound effect for walking animation, and so on.

**Plugins & Scripting**

**Scripting**

Write your game entirely in Rust, while being able to run it in the editor. Scripts are statically linked, so there is no performance loss.

**Static Plugins**

Create static plugins, that can used in multiple projects.

**Multiplatform**

**PC**

Create your games for Windows, Linux, macOS

**WebAssembly**

Create your games for Web using WebAssembly. WebAssembly allows you extending your game audience to more users, especially for cases when you need to run your game on a mobile device (since there is no Android support yet).

# 2D and 3D game engine

At its core, Heaps is built to support both 2D and 3D environments.

![](https://heaps.io/img/feather/shuffle.svg)

# Cross platform compilation

One source code that compiles natively to the platform you want!

![](https://heaps.io/img/feather/heart.svg)

# Easy to get started

Still entirely customizable for high end graphics.

![](https://heaps.io/img/feather/fast-forward.svg)

# Fast compile-and-run cycle

Never wait more than a few seconds to build your project.

![](https://heaps.io/img/feather/cpu.svg)

# GPU accelerated

Of course! And it works for both 2D and 3D.

![](https://heaps.io/img/feather/layers.svg)

# Cross platform GPU Shader system

Advanced effects made simple & globally compatible.

![](https://heaps.io/img/feather/corner-down-right.svg)

# Full controller support

That includes mouse, keyboard and gamepad support.

![](https://heaps.io/img/feather/image.svg)

# File formats support

PNG, JPG, FBX, OGG, etc.

![](https://heaps.io/img/feather/dollar-sign.svg)

# It's free

Like really free: no revenue share and no hidden fee.

## **raylib features**

- **NO external dependencies**, all required libraries included with raylib
- Multiplatform: **Windows, Linux, MacOS, RPI, Android, HTML5... and more!**
- Written in plain C code (C99) using PascalCase/camelCase notation
- Hardware accelerated with OpenGL (**1.1, 2.1, 3.3, 4.3 or ES 2.0**)
- **Unique OpenGL abstraction** layer: [**rlgl**](https://github.com/raysan5/raylib/blob/master/src/rlgl.h)
- Powerful **Fonts** module (SpriteFonts, BMfonts, TTF, SDF)
- Multiple texture formats support, including compressed formats (DXT, ETC, ASTC)
- **Full 3d support** for 3d Shapes, Models, Billboards, Heightmaps and more!
- Flexible Materials system, supporting classic maps and **PBR maps**
- **Animated 3d models** supported (skeletal bones animation)
- Shaders support, including **Model shaders** and **Postprocessing shaders**
- **Powerful math module** for Vector, Matrix and Quaternion operations: [**raymath**](https://github.com/raysan5/raylib/blob/master/src/raymath.h)
- Audio loading and playing with streaming support (WAV, OGG, MP3, FLAC, XM, MOD)
- **VR stereo rendering** support with configurable HMD device parameters
- Huge examples collection with [**+120 code examples**](https://www.raylib.com/examples.html)!
- Bindings to [**+60 programming languages**](https://github.com/raysan5/raylib/blob/master/BINDINGS.md)!
- Free and open source. Check [[**LICENSE**](https://www.raylib.com/license.html)].

## **raylib architecture**

raylib is a highly modular library. Everything is contained within a small number of well defined, specific and self-contained modules, named accordingly to its primary functionality. Note that some of those modules can be used in **standalone mode**, independently of raylib library.

## **What does it offer?**

Panda3D strives to be the world's most flexible and capable game engine. Here are some examples of how it achieves that:

### Platform Portability

The Panda3D core is written in portable C++. When combined with appropriate platform support code, Panda3D will run anywhere!

### Flexible Asset Handling

Panda3D includes command-line tools for processing and optimizing source assets, allowing you to automate and script your content production pipeline to fit your exact needs.

### Library Bindings

Panda3D comes with out-of-the-box support for many popular third-party libraries, such as the Bullet physics engine, Assimp model loader, OpenAL and FMOD sound libraries, and more.

### Extensibility

Panda3D exposes all of its low-level graphics primitives to the application. Invent your own graphics techniques and rendering pipelines!

### Performance Profiling

Panda3D includes pstats — an over-the-network profiling system designed to help you understand where every single millisecond of your frame time goes.

### Rapid Prototyping

Panda3D requires no boilerplate and no complicated initialization code. What you see here is a complete Panda3D app written in Python!

jMonkeyEngine is a feature-rich engine capabale of creating both beautiful

and complex games, single-player or networked, on a wide variety of platforms.

---

### Platforms

- Windows
- Linux
- Mac OSX
- Raspberry Pi 3 (OpenGL ES 2.0)
- Raspberry Pi 4 (OpenGL ES 3.2)
- Android

## Supported Model Formats

- GLTF
- OBJ

### Audio

- Support for WAV, MP3 and OGG file formats.
- Buffered and Streaming support.
- Global, directional and positional sounds.

### Input

- Mouse and Keyboard
- Touchscreen
- Joystick/Joypad/Wheel

### SceneGraph

- Batching
- Instancing
- 2D and 3D scene support
- Level of Detail
- Light Culling
- Single Pass Lighting

### Animation

- Tween API with out of the box support for spatial, bone and morph animations
- Stock Tweens availble:
    - Sequence tween: a tween that plays tweens in sequence.
    - Parallel tween: a tween that plays tweens in parallel.
    - Delay tween : a tween that just waits…
    - Stretch tween: a tween that wraps another tween and change its duration.
    - Camera tween: moves the camera…
    - CallMethod: calling a method on an object …
- Animation Blending
- Animation interpolation (interpolors for rotation, position, scale and time)
- Hardware Skinning

### Graphics

- OpenGL support up to OpenGL 4.5
- OpenGL ES support up to 3.0
- LWJGL2 and 3
- Post Processing
- Stock Post Processors
    - Water
    - Screen Space Ambient Occlusion
        - Supports Approximate Normals (50% faster)
    - Bloom
    - Cartoon Edge
    - Color Overlay
    - Cross-Hatch
    - Depth Of Field
    - Fast Approximate Anti Aliasing
    - Fog
    - Light Scattering
    - Posterization
    - Radial Blur
    - ToneMap
- Unshaded Materials
- Phong Lighting Materials
- PBR Materials
    - Sphere and OrientedBox Probe areas
    - Light Probe blending (up to 3 light probes)
    - Supports both Roughness/Metallic & Roughness/SpecularGloss workflow
- Vertex, Fragment and Geometry shader support
- Texture Atlas support
- Particles

### Language

- Support for Java 1.8+
- Use Kotlin, Groovy or any combination all in one project.

### Physics

- Bullet Physics
- [**Minie Physics**](https://github.com/stephengold/Minie) - A high-powered improved and up-to-date binding around Bullet with “soft body” support.

### Networking

- Networking API supporting UDP/TCP either with low-level Messaging or high-level RMI.
- [**SimEthereal**](https://github.com/Simsilica/SimEthereal) - A high performance library for real-time networked object synching

### GUI

- [**Lemur**](http://jmonkeyengine-contributions.github.io/Lemur/) - a fast and efficient Jme-Native 2D and 3D GUI Toolkit.
- [**JME-JFX-11**](https://github.com/jayfella/jme-jfx-11) - A bridge to create a 2D GUI in JME using JavaFX 11.

### Entity System

- [**Zay-ES**](https://github.com/jMonkeyEngine-Contributions/zay-es) - A high-performance entity-component-system

### Profiling

- DetailedProfiler - Displays timing information for various areas of your game to determine bottlenecks

# **FeaturesCreate captivating experiences with O3DE using a suite of tools developed for cutting-edge, real-time graphics, and complex interactions.**

# **Familiar Build System**

# **Physically-based Photorealistic Renderer**

# **Flexible Runtime Scripting**

# **Real-time Physics Simulations**

# **High-performance Math**

# **Extensible, Visual 3D Content Editor & Scripting Tools**

# **Robust Networking**

# **Terrain Performance**

# **Data-driven Asset Workflows & Handling**

# **Prefab Support**

# **Simplified Project Management**

# **Flexible Code & Data Templates**

# **White Box Tool**

# **List of features[](https://docs.godotengine.org/en/stable/about/list_of_features.html#list-of-features)**

This page aims to list **all** features currently supported by Godot.

**Note**

This page lists features supported by the current stable version of Godot. Some of these features are not available in the [3.x release series](https://docs.godotengine.org/en/3.6/about/list_of_features.html).

# **Platforms[](https://docs.godotengine.org/en/stable/about/list_of_features.html#platforms)**

**See also**

See [System requirements](https://docs.godotengine.org/en/stable/about/system_requirements.html#doc-system-requirements) for hardware and software version requirements.

**Can run both the editor and exported projects:**

- Windows (x86 and ARM, 64-bit and 32-bit).
- macOS (x86 and ARM, 64-bit only).
- Linux (x86 and ARM, 64-bit and 32-bit).
    
    > Binaries are statically linked and can run on any distribution if compiled on an old enough base distribution.Official binaries are compiled using the Godot Engine buildroot, allowing for binaries that work across common Linux distributions.
    > 
- Android (editor support is experimental).
- [Web browsers](https://docs.godotengine.org/en/stable/tutorials/editor/using_the_web_editor.html#doc-using-the-web-editor). Experimental in 4.0, using Godot 3.x is recommended instead when targeting HTML5.

**Note**

Linux supports rv64 (RISC-V), ppc64 & ppc32 (PowerPC), and loongarch64. However you must compile the editor for that platform (as well as export templates) yourself, no official downloads are currently provided. RISC-V compiling instructions can be found on the [Compiling for Linux, *BSD](https://docs.godotengine.org/en/stable/engine_details/development/compiling/compiling_for_linuxbsd.html#doc-compiling-for-linuxbsd) page.

**Runs exported projects:**

- iOS.
- [Consoles](https://docs.godotengine.org/en/stable/tutorials/platform/consoles.html#doc-consoles).

Godot aims to be as platform-independent as possible and can be [ported to new platforms](https://docs.godotengine.org/en/stable/engine_details/architecture/custom_platform_ports.html#doc-custom-platform-ports) with relative ease.

**Note**

Projects written in C# using Godot 4 currently cannot be exported to the web platform. To use C# on that platform, consider Godot 3 instead. Android and iOS platform support is available as of Godot 4.2, but is experimental and [some limitations apply](https://docs.godotengine.org/en/stable/tutorials/scripting/c_sharp/index.html#doc-c-sharp-platforms).

# **Editor[](https://docs.godotengine.org/en/stable/about/list_of_features.html#editor)**

**Features:**

- Scene tree editor.
- Built-in script editor.
- Support for [external script editors](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html#doc-external-editor) such as Visual Studio Code or Vim.
- GDScript [debugger](https://docs.godotengine.org/en/stable/tutorials/scripting/debug/debugger_panel.html#doc-debugger-panel).
    
    > Support for debugging in threads is available since 4.2.
    > 
- Visual profiler with CPU and GPU time indications for each step of the rendering pipeline.
- Performance monitoring tools, including [custom performance monitors](https://docs.godotengine.org/en/stable/tutorials/scripting/debug/custom_performance_monitors.html#doc-custom-performance-monitors).
- Live script reloading.
- Live scene editing.
    
    > Changes will reflect in the editor and will be kept after closing the running project.
    > 
- Remote inspector.
    
    > Changes won't reflect in the editor and won't be kept after closing the running project.
    > 
- Live camera replication.
    
    > Move the in-editor camera and see the result in the running project.
    > 
- Built-in offline class reference documentation.
- Use the editor in dozens of languages contributed by the community.

**Plugins:**

- Editor plugins can be downloaded from the [asset library](https://docs.godotengine.org/en/stable/community/asset_library/what_is_assetlib.html#doc-what-is-assetlib) to extend editor functionality.
- [Create your own plugins](https://docs.godotengine.org/en/stable/tutorials/plugins/editor/making_plugins.html#doc-making-plugins) using GDScript to add new features or speed up your workflow.
- [Download projects from the asset library](https://docs.godotengine.org/en/stable/community/asset_library/using_assetlib.html#doc-using-assetlib-editor) in the Project Manager and import them directly.

# **Rendering[](https://docs.godotengine.org/en/stable/about/list_of_features.html#rendering)**

Godot 4 includes three renderers:

- **Forward+**. The most advanced renderer, suited for desktop platforms only. Used by default on desktop platforms. This renderer uses **Vulkan**, **Direct3D 12**, or **Metal** as the rendering driver, and it uses the **RenderingDevice** backend.
- **Mobile**. Fewer features, but renders simple scenes faster. Suited for mobile and desktop platforms. Used by default on mobile platforms. This renderer uses **Vulkan**, **Direct3D 12**, or **Metal** as the rendering driver, and it uses the **RenderingDevice** backend.
- **Compatibility**, sometimes called **GL Compatibility**. The least advanced renderer, suited for low-end desktop and mobile platforms. Used by default on the web platform. This renderer uses **OpenGL** as the rendering driver.

See [Overview of renderers](https://docs.godotengine.org/en/stable/tutorials/rendering/renderers.html#doc-renderers) for a detailed comparison of the rendering methods.

# **2D graphics[](https://docs.godotengine.org/en/stable/about/list_of_features.html#d-graphics)**

- Sprite, polygon and line rendering.
    
    > High-level tools to draw lines and polygons such as Polygon2D and Line2D, with support for texturing.
    > 
- AnimatedSprite2D as a helper for creating animated sprites.
- Parallax layers.
    
    > Pseudo-3D support including preview in the editor.
    > 
- [2D lighting](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html#doc-2d-lights-and-shadows) with normal maps and specular maps.
    
    > Point (omni/spot) and directional 2D lights.Hard or soft shadows (adjustable on a per-light basis).Custom shaders can access a real-time SDF representation of the 2D scene based on LightOccluder2D nodes, which can be used for improved 2D lighting effects including 2D global illumination.
    > 
- [Font rendering](https://docs.godotengine.org/en/stable/tutorials/ui/gui_using_fonts.html#doc-gui-using-fonts) using bitmaps, rasterization using FreeType or multi-channel signed distance fields (MSDF).
    
    > Bitmap fonts can be exported using tools like BMFont, or imported from images (for fixed-width fonts only).Dynamic fonts support monochrome fonts as well as colored fonts (e.g. for emoji). Supported formats are TTF, OTF, WOFF1 and WOFF2.Dynamic fonts support optional font outlines with adjustable width and color.Dynamic fonts support variable fonts and OpenType features including ligatures.Dynamic fonts support simulated bold and italic when the font file lacks those styles.Dynamic fonts support oversampling to keep fonts sharp at higher resolutions.Dynamic fonts support subpixel positioning to make fonts crisper at low sizes.Dynamic fonts support LCD subpixel optimizations to make fonts even crisper at low sizes.Signed distance field fonts can be scaled at any resolution without requiring re-rasterization. Multi-channel usage makes SDF fonts scale down to lower sizes better compared to monochrome SDF fonts.
    > 
- GPU-based [particles](https://docs.godotengine.org/en/stable/tutorials/2d/particle_systems_2d.html#doc-particle-systems-2d) with support for [custom particle shaders](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/particle_shader.html#doc-particle-shader).
- CPU-based particles.
- Optional [2D HDR rendering](https://docs.godotengine.org/en/stable/tutorials/3d/environment_and_post_processing.html#doc-environment-and-post-processing-using-glow-in-2d) for better glow capabilities.

# **2D tools[](https://docs.godotengine.org/en/stable/about/list_of_features.html#d-tools)**

- [TileMaps](https://docs.godotengine.org/en/stable/tutorials/2d/using_tilemaps.html#doc-using-tilemaps) for 2D tile-based level design.
- 2D camera with built-in smoothing and drag margins.
- Path2D node to represent a path in 2D space.
    
    > Can be drawn in the editor or generated procedurally.PathFollow2D node to make nodes follow a Path2D.
    > 
- [2D geometry helper class](https://docs.godotengine.org/en/stable/classes/class_geometry2d.html#class-geometry2d).

# **2D physics[](https://docs.godotengine.org/en/stable/about/list_of_features.html#d-physics)**

**Physics bodies:**

- Static bodies.
- Animatable bodies (for objects moving only by script or animation, such as doors and platforms).
- Rigid bodies.
- Character bodies.
- Joints.
- Areas to detect bodies entering or leaving it.

**Collision detection:**

- Built-in shapes: line, box, circle, capsule, world boundary (infinite plane).
- Collision polygons (can be drawn manually or generated from a sprite in the editor).

# **3D graphics[](https://docs.godotengine.org/en/stable/about/list_of_features.html#id1)**

- HDR rendering with sRGB.
- Perspective, orthographic and frustum-offset cameras.
- When using the Forward+ renderer, a depth prepass is used to improve performance in complex scenes by reducing the cost of overdraw.
- [Variable rate shading](https://docs.godotengine.org/en/stable/tutorials/3d/variable_rate_shading.html#doc-variable-rate-shading) on supported GPUs in Forward+ and Mobile.

**Physically-based rendering (built-in material features):**

- Follows the Disney PBR model.
- Supports Burley, Lambert, Lambert Wrap (half-Lambert) and Toon diffuse shading modes.
- Supports Schlick-GGX, Toon and Disabled specular shading modes.
- Uses a roughness-metallic workflow with support for ORM textures.
- Uses horizon specular occlusion (Filament model) to improve material appearance.
- Normal mapping.
- Parallax/relief mapping with automatic level of detail based on distance.
- Detail mapping for the albedo and normal maps.
- Sub-surface scattering and transmittance.
- Screen-space refraction with support for material roughness (resulting in blurry refraction).
- Proximity fade (soft particles) and distance fade.
- Distance fade can use alpha blending or dithering to avoid going through the transparent pipeline.
- Dithering can be determined on a per-pixel or per-object basis.

**Real-time lighting:**

- Directional lights (sun/moon). Up to 4 per scene.
- Omnidirectional lights.
- Spot lights with adjustable cone angle and attenuation.
- Specular, indirect light, and volumetric fog energy can be adjusted on a per-light basis.
- Adjustable light "size" for fake area lights (will also make shadows blurrier).
- Optional distance fade system to fade distant lights and their shadows, improving performance.
- When using the Forward+ renderer (default on desktop), lights are rendered with clustered forward optimizations to decrease their individual cost. Clustered rendering also lifts any limits on the number of lights that can be used on a mesh.
- When using the Mobile renderer, up to 8 omni lights and 8 spot lights can be displayed per mesh resource. Baked lighting can be used to overcome this limit if needed.

**Shadow mapping:**

- *DirectionalLight:* Orthogonal (fastest), PSSM 2-split and 4-split. Supports blending between splits.
- *OmniLight:* Dual paraboloid (fast) or cubemap (slower but more accurate). Supports colored projector textures in the form of panoramas.
- *SpotLight:* Single texture. Supports colored projector textures.
- Shadow normal offset bias and shadow pancaking to decrease the amount of visible shadow acne and peter-panning.
- PCSSlike shadow blur based on the light size and distance from the surface the shadow is cast on.
- Adjustable shadow blur on a per-light basis.

**Global illumination with indirect lighting:**

- [Baked lightmaps](https://docs.godotengine.org/en/stable/tutorials/3d/global_illumination/using_lightmap_gi.html#doc-using-lightmap-gi) (fast, but can't be updated at runtime).
    
    > Supports baking indirect light only or baking both direct and indirect lighting. The bake mode can be adjusted on a per-light basis to allow for hybrid light baking setups.Supports lighting dynamic objects using automatic and manually placed probes.Optionally supports directional lighting and rough reflections based on spherical harmonics.Lightmaps are baked on the GPU using compute shaders (much faster compared to CPU lightmapping). Baking can only be performed from the editor, not in exported projects.Supports GPU-based denoising with JNLM, or CPU/GPU-based denoising with OIDN.
    > 
- [Voxel-based GI probes](https://docs.godotengine.org/en/stable/tutorials/3d/global_illumination/using_voxel_gi.html#doc-using-voxel-gi). Supports dynamic lights *and* dynamic occluders, while also supporting reflections. Requires a fast baking step which can be performed in the editor or at runtime (including from an exported project).
- [Signed-distance field GI](https://docs.godotengine.org/en/stable/tutorials/3d/global_illumination/using_sdfgi.html#doc-using-sdfgi) designed for large open worlds. Supports dynamic lights, but not dynamic occluders. Supports reflections. No baking required.
- [Screen-space indirect lighting (SSIL)](https://docs.godotengine.org/en/stable/tutorials/3d/environment_and_post_processing.html#doc-environment-and-post-processing-ssil) at half or full resolution. Fully real-time and supports any kind of emissive light source (including decals).
- VoxelGI and SDFGI use a deferred pass to allow for rendering GI at half resolution to improve performance (while still having functional MSAA support).

**Reflections:**

- Voxel-based reflections (when using GI probes) and SDF-based reflections (when using signed distance field GI). Voxel-based reflections are visible on transparent surfaces, while rough SDF-based reflections are visible on transparent surfaces.
- Fast baked reflections or slow real-time reflections using ReflectionProbe. Parallax box correction can optionally be enabled.
- Screen-space reflections with support for material roughness.
- Reflection techniques can be mixed together for greater accuracy or scalability.
- When using the Forward+ renderer (default on desktop), reflection probes are rendered with clustered forward optimizations to decrease their individual cost. Clustered rendering also lifts any limits on the number of reflection probes that can be used on a mesh.
- When using the Mobile renderer, up to 8 reflection probes can be displayed per mesh resource. When using the Compatibility renderer, up to 2 reflection probes can be displayed per mesh resource.

**Decals:**

- [Supports albedo](https://docs.godotengine.org/en/stable/tutorials/3d/using_decals.html#doc-using-decals), emissive, ORM, and normal mapping.
- Texture channels are smoothly overlaid on top of the underlying material, with support for normal/ORM-only decals.
- Support for normal fade to fade the decal depending on its incidence angle.
- Does not rely on runtime mesh generation. This means decals can be used on complex skinned meshes with no performance penalty, even if the decal moves every frame.
- Support for nearest, bilinear, trilinear or anisotropic texture filtering (configured globally).
- Optional distance fade system to fade distant decals, improving performance.
- When using the Forward+ renderer (default on desktop), decals are rendered with clustered forward optimizations to decrease their individual cost. Clustered rendering also lifts any limits on the number of decals that can be used on a mesh.
- When using the Mobile renderer, up to 8 decals can be displayed per mesh resource.

**Sky:**

- Panorama sky (using an HDRI).
- Procedural sky and Physically-based sky that respond to the DirectionalLights in the scene.
- Support for [custom sky shaders](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/sky_shader.html#doc-sky-shader), which can be animated.
- The radiance map used for ambient and specular light can be updated in real-time depending on the quality settings chosen.

**Fog:**

- Exponential depth fog.
- Exponential height fog.
- Support for automatic fog color depending on the sky color (aerial perspective).
- Support for sun scattering in the fog.
- Support for controlling how much fog rendering should affect the sky, with separate controls for traditional and volumetric fog.
- Support for making specific materials ignore fog.

**Volumetric fog:**

- Global [volumetric fog](https://docs.godotengine.org/en/stable/tutorials/3d/volumetric_fog.html#doc-volumetric-fog) that reacts to lights and shadows.
- Volumetric fog can take indirect light into account when using VoxelGI or SDFGI.
- Fog volume nodes that can be placed to add fog to specific areas (or remove fog from specific areas). Supported shapes include box, ellipse, cone, cylinder, and 3D texture-based density maps.
- Each fog volume can have its own custom shader.
- Can be used together with traditional fog.

**Particles:**

- GPU-based particles with support for subemitters (2D + 3D), trails (2D + 3D), attractors (3D only) and collision (2D + 3D).
    - 3D particle attractor shapes supported: box, sphere and 3D vector fields.
    - 3D particle collision shapes supported: box, sphere, baked signed distance field and real-time heightmap (suited for open world weather effects).
    - 2D particle collision is handled using a signed distance field generated in real-time based on [LightOccluder2D](https://docs.godotengine.org/en/stable/classes/class_lightoccluder2d.html#class-lightoccluder2d) nodes in the scene.
    - Trails can use the built-in ribbon trail and tube trail meshes, or custom meshes with skeletons.
    - Support for custom particle shaders with manual emission.
- CPU-based particles.

**Post-processing:**

- Tonemapping (Linear, Reinhard, Filmic, ACES, AgX).
- Automatic exposure adjustments based on viewport brightness (and manual exposure override).
- Near and far depth of field with adjustable bokeh simulation (box, hexagon, circle).
- Screen-space ambient occlusion (SSAO) at half or full resolution.
- Glow/bloom with optional bicubic upscaling and several blend modes available: Screen, Soft Light, Add, Replace, Mix.
- Glow can have a colored dirt map texture, acting as a lens dirt effect.
- Glow can be [used as a screen-space blur effect](https://docs.godotengine.org/en/stable/tutorials/3d/environment_and_post_processing.html#doc-environment-and-post-processing-using-glow-to-blur-the-screen).
- Color correction using a one-dimensional ramp or a 3D LUT texture.
- Roughness limiter to reduce the impact of specular aliasing.
- Brightness, contrast and saturation adjustments.

**Texture filtering:**

- Nearest, bilinear, trilinear or anisotropic filtering.
- Filtering options are defined on a per-use basis, not a per-texture basis.

**Texture compression:**

- Basis Universal (slow, but results in smaller files).
- BPTC for high-quality compression (not supported on macOS).
- ETC2 (not supported on macOS).
- S3TC (not supported on mobile/Web platforms).

**Antialiasing:**

- Temporal [antialiasing](https://docs.godotengine.org/en/stable/tutorials/3d/3d_antialiasing.html#doc-3d-antialiasing) (TAA).
- AMD FidelityFX Super Resolution 2.2 [antialiasing](https://docs.godotengine.org/en/stable/tutorials/3d/3d_antialiasing.html#doc-3d-antialiasing) (FSR2), which can be used at native resolution as a form of high-quality temporal antialiasing.
- Multi-sample antialiasing (MSAA), for both [2D antialiasing](https://docs.godotengine.org/en/stable/tutorials/2d/2d_antialiasing.html#doc-2d-antialiasing) and [3D antialiasing](https://docs.godotengine.org/en/stable/tutorials/3d/3d_antialiasing.html#doc-3d-antialiasing).
- Fast approximate antialiasing (FXAA).
- Super-sample antialiasing (SSAA) using bilinear 3D scaling and a 3D resolution scale above 1.0.
- Alpha antialiasing, MSAA alpha to coverage and alpha hashing on a per-material basis.

**Resolution scaling:**

- Support for [rendering 3D at a lower resolution](https://docs.godotengine.org/en/stable/tutorials/3d/resolution_scaling.html#doc-resolution-scaling) while keeping 2D rendering at the original scale. This can be used to improve performance on low-end systems or improve visuals on high-end systems.
- Resolution scaling uses bilinear filtering, AMD FidelityFX Super Resolution 1.0 (FSR1) or AMD FidelityFX Super Resolution 2.2 (FSR2).
- Texture mipmap LOD bias is adjusted automatically to improve quality at lower resolution scales. It can also be modified with a manual offset.

Most effects listed above can be adjusted for better performance or to further improve quality. This can be helpful when [using Godot for offline rendering](https://docs.godotengine.org/en/stable/tutorials/animation/creating_movies.html#doc-creating-movies).

# **3D tools[](https://docs.godotengine.org/en/stable/about/list_of_features.html#id2)**

- Built-in meshes: cube, cylinder/cone, (hemi)sphere, prism, plane, quad, torus, ribbon, tube.
- [GridMaps](https://docs.godotengine.org/en/stable/tutorials/3d/using_gridmaps.html#doc-using-gridmaps) for 3D tile-based level design.
- [Constructive solid geometry](https://docs.godotengine.org/en/stable/tutorials/3d/csg_tools.html#doc-csg-tools) (intended for prototyping).
- Tools for [procedural geometry generation](https://docs.godotengine.org/en/stable/tutorials/3d/procedural_geometry/index.html#doc-procedural-geometry).
- Path3D node to represent a path in 3D space.
    
    > Can be drawn in the editor or generated procedurally.PathFollow3D node to make nodes follow a Path3D.
    > 
- [3D geometry helper class](https://docs.godotengine.org/en/stable/classes/class_geometry3d.html#class-geometry3d).
- Support for exporting the current scene as a glTF 2.0 file, both from the editor and at runtime from an exported project.

# **3D physics[](https://docs.godotengine.org/en/stable/about/list_of_features.html#id3)**

**Physics bodies:**

- Static bodies.
- Animatable bodies (for objects moving only by script or animation, such as doors and platforms).
- Rigid bodies.
- Character bodies.
- Vehicle bodies (intended for arcade physics, not simulation).
- Joints.
- Soft bodies.
- Ragdolls.
- Areas to detect bodies entering or leaving it.

**Collision detection:**

- Built-in shapes: cuboid, sphere, capsule, cylinder, world boundary (infinite plane).
- Generate triangle collision shapes for any mesh from the editor.
- Generate one or several convex collision shapes for any mesh from the editor.

# **Shaders[](https://docs.godotengine.org/en/stable/about/list_of_features.html#shaders)**

- *2D:* Custom vertex, fragment, and light shaders.
- *3D:* Custom vertex, fragment, light, and sky shaders.
- Text-based shaders using a [shader language inspired by GLSL](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/shading_language.html#doc-shading-language).
- Visual shader editor.
    
    > Support for visual shader plugins.
    > 

# **Scripting[](https://docs.godotengine.org/en/stable/about/list_of_features.html#scripting)**

**General:**

- Object-oriented design pattern with scripts extending nodes.
- Signals and groups for communicating between scripts.
- Support for [cross-language scripting](https://docs.godotengine.org/en/stable/tutorials/scripting/cross_language_scripting.html#doc-cross-language-scripting).
- Many 2D, 3D and 4D linear algebra data types such as vectors and transforms.

[GDScript:](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/index.html#doc-gdscript)

- [High-level interpreted language](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_basics.html#doc-gdscript-reference) with [optional static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html#doc-gdscript-static-typing).
- Syntax inspired by Python. However, GDScript is **not** based on Python.
- Syntax highlighting is provided on GitHub.
- [Use threads](https://docs.godotengine.org/en/stable/tutorials/performance/using_multiple_threads.html#doc-using-multiple-threads) to perform asynchronous actions or make use of multiple processor cores.

[C#:](https://docs.godotengine.org/en/stable/tutorials/scripting/c_sharp/index.html#doc-c-sharp)

- Packaged in a separate binary to keep file sizes and dependencies down.
- Supports .NET 8 and higher.
    
    > Full support for the C# 12.0 syntax and features.
    > 
- Supports Windows, Linux, and macOS. Since Godot 4.2, experimental support for Android and iOS is also available.
    
    > On the iOS platform only some architectures are supported: arm64.The web platform is currently unsupported. To use C# on that platform, consider Godot 3 instead.
    > 
- Using an external editor is recommended to benefit from IDE functionality.

**GDExtension (C, C++, Rust, D, ...):**

- When you need it, link to native libraries for higher performance and third-party integrations.
    
    > For scripting game logic, GDScript or C# are recommended if their performance is suitable.
    > 
- Official GDExtension bindings for [C](https://github.com/godotengine/godot-headers) and [C++](https://github.com/godotengine/godot-cpp).
    
    > Use any build system and language features you wish.
    > 
- Actively developed GDExtension bindings for [D](https://github.com/godot-dlang/godot-dlang), [Swift](https://github.com/migueldeicaza/SwiftGodot), and [Rust](https://github.com/godot-rust/gdextension) bindings provided by the community. (Some of these bindings may be experimental and not production-ready).

# **Audio[](https://docs.godotengine.org/en/stable/about/list_of_features.html#audio)**

**Features:**

- Mono, stereo, 5.1 and 7.1 output.
- Non-positional and positional playback in 2D and 3D.
    
    > Optional Doppler effect in 2D and 3D.
    > 
- Support for re-routable [audio buses](https://docs.godotengine.org/en/stable/tutorials/audio/audio_buses.html#doc-audio-buses) and effects with dozens of effects included.
- Support for polyphony (playing several sounds from a single AudioStreamPlayer node).
- Support for random volume and pitch.
- Support for real-time pitch scaling.
- Support for sequential/random sample selection, including repetition prevention when using random sample selection.
- Listener2D and Listener3D nodes to listen from a position different than the camera.
- Support for [procedural audio generation](https://docs.godotengine.org/en/stable/classes/class_audiostreamgenerator.html#class-audiostreamgenerator).
- Audio input to record microphones.
- MIDI input.
    
    > No support for MIDI output yet.
    > 

**APIs used:**

- *Windows:* WASAPI.
- *macOS:* CoreAudio.
- *Linux:* PulseAudio or ALSA.

# **Import[](https://docs.godotengine.org/en/stable/about/list_of_features.html#import)**

- Support for [custom import plugins](https://docs.godotengine.org/en/stable/tutorials/plugins/editor/import_plugins.html#doc-import-plugins).

**Formats:**

- *Images:* See [Importing images](https://docs.godotengine.org/en/stable/tutorials/assets_pipeline/importing_images.html#doc-importing-images).
- *Audio:*
    
    > WAV with optional IMA-ADPCM compression.Ogg Vorbis.MP3.
    > 
- *3D scenes:* See [Importing 3D scenes](https://docs.godotengine.org/en/stable/tutorials/assets_pipeline/importing_3d_scenes/index.html#doc-importing-3d-scenes).
    
    > glTF 2.0 (recommended)..blend (by calling Blender's glTF export functionality transparently).FBX (by calling FBX2glTF transparently).Collada (.dae).Wavefront OBJ (static scenes only, can be loaded directly as a mesh or imported as a 3D scene).
    > 
- Support for loading glTF 2.0 scenes at runtime, including from an exported project.
- 3D meshes use [Mikktspace](http://www.mikktspace.com/) to generate tangents on import, which ensures consistency with other 3D applications such as Blender.

# **Input[](https://docs.godotengine.org/en/stable/about/list_of_features.html#input)**

- [Input mapping system](https://docs.godotengine.org/en/stable/tutorials/inputs/input_examples.html#doc-input-examples) using hardcoded input events or remappable input actions.
    
    > Axis values can be mapped to two different actions with a configurable deadzone.Use the same code to support both keyboards and gamepads.
    > 
- Keyboard input.
    
    > Keys can be mapped in "physical" mode to be independent of the keyboard layout.
    > 
- Mouse input.
    
    > The mouse cursor can be visible, hidden, captured or confined within the window.When captured, raw input will be used on Windows and Linux to sidestep the OS' mouse acceleration settings.
    > 
- Gamepad input (up to 8 simultaneous controllers).
- Pen/tablet input with pressure support.

# **Navigation[](https://docs.godotengine.org/en/stable/about/list_of_features.html#navigation)**

- A* algorithm in [2D](https://docs.godotengine.org/en/stable/classes/class_astar2d.html#class-astar2d) and [3D](https://docs.godotengine.org/en/stable/classes/class_astar3d.html#class-astar3d).
- Navigation meshes with dynamic obstacle avoidance in [2D](https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_introduction_2d.html#doc-navigation-overview-2d) and [3D](https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_introduction_3d.html#doc-navigation-overview-3d).
- Generate navigation meshes from the editor or at runtime (including from an exported project).

# **Networking[](https://docs.godotengine.org/en/stable/about/list_of_features.html#networking)**

- Low-level TCP networking using [StreamPeer](https://docs.godotengine.org/en/stable/classes/class_streampeer.html#class-streampeer) and [TCPServer](https://docs.godotengine.org/en/stable/classes/class_tcpserver.html#class-tcpserver).
- Low-level UDP networking using [PacketPeer](https://docs.godotengine.org/en/stable/classes/class_packetpeer.html#class-packetpeer) and [UDPServer](https://docs.godotengine.org/en/stable/classes/class_udpserver.html#class-udpserver).
- Low-level HTTP requests using [HTTPClient](https://docs.godotengine.org/en/stable/classes/class_httpclient.html#class-httpclient).
- High-level HTTP requests using [HTTPRequest](https://docs.godotengine.org/en/stable/classes/class_httprequest.html#class-httprequest).
    
    > Supports HTTPS out of the box using bundled certificates.
    > 
- [High-level multiplayer](https://docs.godotengine.org/en/stable/tutorials/networking/high_level_multiplayer.html#doc-high-level-multiplayer) API using UDP and ENet.
    
    > Automatic replication using remote procedure calls (RPCs).Supports unreliable, reliable and ordered transfers.
    > 
- [WebSocket](https://docs.godotengine.org/en/stable/tutorials/networking/websocket.html#doc-websocket) client and server, available on all platforms.
- [WebRTC](https://docs.godotengine.org/en/stable/tutorials/networking/webrtc.html#doc-webrtc) client and server, available on all platforms.
- Support for [UPnP](https://docs.godotengine.org/en/stable/classes/class_upnp.html#class-upnp) to sidestep the requirement to forward ports when hosting a server behind a NAT.

# **Internationalization[](https://docs.godotengine.org/en/stable/about/list_of_features.html#internationalization)**

- Full support for Unicode including emoji.
- Store localization strings using [CSV](https://docs.godotengine.org/en/stable/tutorials/i18n/internationalizing_games.html#doc-internationalizing-games) or [gettext](https://docs.godotengine.org/en/stable/tutorials/i18n/localization_using_gettext.html#doc-localization-using-gettext).
    - Support for generating gettext POT and PO files from the editor.
- Use localized strings in your project automatically in GUI elements or by using the **`tr()`** function.
- Support for pluralization and translation contexts when using gettext translations.
- Support for [bidirectional typesetting](https://docs.godotengine.org/en/stable/tutorials/i18n/internationalizing_games.html#doc-internationalizing-games-bidi), text shaping and OpenType localized forms.
- Automatic UI mirroring for right-to-left locales.
- Support for pseudolocalization to test your project for i18n-friendliness.

# **Windowing and OS integration[](https://docs.godotengine.org/en/stable/about/list_of_features.html#windowing-and-os-integration)**

- Spawn multiple independent windows within a single process.
- Move, resize, minimize, and maximize windows spawned by the project.
- Change the window title and icon.
- Request attention (will cause the title bar to blink on most platforms).
- Fullscreen mode.
    
    > Uses borderless fullscreen by default on Windows for fast alt-tabbing, but can optionally use exclusive fullscreen to reduce input lag.
    > 
- Borderless windows (fullscreen or non-fullscreen).
- Ability to keep a window always on top.
- Global menu integration on macOS.
- Execute commands in a blocking or non-blocking manner (including running multiple instances of the same project).
- Open file paths and URLs using default or custom protocol handlers (if registered on the system).
- Parse custom command line arguments.
- Any Godot binary (editor or exported project) can be [used as a headless server](https://docs.godotengine.org/en/stable/tutorials/export/exporting_for_dedicated_servers.html#doc-exporting-for-dedicated-servers) by starting it with the **`-headless`** command line argument. This allows running the engine without a GPU or display server.

# **Mobile[](https://docs.godotengine.org/en/stable/about/list_of_features.html#mobile)**

- In-app purchases on [Android](https://docs.godotengine.org/en/stable/tutorials/platform/android/android_in_app_purchases.html#doc-android-in-app-purchases) and [iOS](https://docs.godotengine.org/en/stable/tutorials/platform/ios/plugins_for_ios.html#doc-plugins-for-ios).
- Support for advertisements using third-party modules.

# **XR support (AR and VR)[](https://docs.godotengine.org/en/stable/about/list_of_features.html#xr-support-ar-and-vr)**

- Out of the box [support for OpenXR](https://docs.godotengine.org/en/stable/tutorials/xr/setting_up_xr.html#doc-setting-up-xr).
    
    > Including support for popular desktop headsets like the Valve Index, WMR headsets, and Quest over Link.
    > 
- Support for [Android-based headsets](https://docs.godotengine.org/en/stable/tutorials/xr/deploying_to_android.html#doc-deploying-to-android) using OpenXR through a plugin.
    - Including support for popular stand alone headsets like the Meta Quest 1/2/3 and Pro, Pico 4, Magic Leap 2, and Lynx R1.
- Out of the box limited support for visionOS Apple headsets.
    - Currently only exporting an application for use on a flat plane within the headset is supported. Immersive experiences are not supported.
- Other devices supported through an XR plugin structure.
- Various advanced toolkits are available that implement common features required by XR applications.

# **GUI system[](https://docs.godotengine.org/en/stable/about/list_of_features.html#gui-system)**

Godot's GUI is built using the same Control nodes used to make games in Godot. The editor UI can easily be extended in many ways using add-ons.

**Nodes:**

- Buttons.
- Checkboxes, check buttons, radio buttons.
- Text entry using [LineEdit](https://docs.godotengine.org/en/stable/classes/class_lineedit.html#class-lineedit) (single line) and [TextEdit](https://docs.godotengine.org/en/stable/classes/class_textedit.html#class-textedit) (multiple lines). TextEdit also supports code editing features such as displaying line numbers and syntax highlighting.
- Dropdown menus using [PopupMenu](https://docs.godotengine.org/en/stable/classes/class_popupmenu.html#class-popupmenu) and [OptionButton](https://docs.godotengine.org/en/stable/classes/class_optionbutton.html#class-optionbutton).
- Scrollbars.
- Labels.
- RichTextLabel for [text formatted using BBCode](https://docs.godotengine.org/en/stable/tutorials/ui/bbcode_in_richtextlabel.html#doc-bbcode-in-richtextlabel), with support for animated custom effects.
- Trees (can also be used to represent tables).
- Color picker with RGB and HSV modes.
- Controls can be rotated and scaled.

**Sizing:**

- Anchors to keep GUI elements in a specific corner, edge or centered.
- Containers to place GUI elements automatically following certain rules.
    
    > Stack layouts.Grid layouts.Flow layouts (similar to autowrapping text).Margin, centered and aspect ratio layouts.Draggable splitter layouts.
    > 
- Scale to [multiple resolutions](https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html#doc-multiple-resolutions) using the **`canvas_items`** or **`viewport`** stretch modes.
- Support any aspect ratio using anchors and the **`expand`** stretch aspect.

**Theming:**

- Built-in theme editor.
    
    > Generate a theme based on the current editor theme settings.
    > 
- Procedural vector-based theming using [StyleBoxFlat](https://docs.godotengine.org/en/stable/classes/class_styleboxflat.html#class-styleboxflat).
    
    > Supports rounded/beveled corners, drop shadows, per-border widths and antialiasing.
    > 
- Texture-based theming using [StyleBoxTexture](https://docs.godotengine.org/en/stable/classes/class_styleboxtexture.html#class-styleboxtexture).

Godot's small distribution size can make it a suitable alternative to frameworks like Electron or Qt.

# **Animation[](https://docs.godotengine.org/en/stable/about/list_of_features.html#animation)**

- Direct kinematics and inverse kinematics.
- Support for animating any property with customizable interpolation.
- Support for calling methods in animation tracks.
- Support for playing sounds in animation tracks.
- Support for Bézier curves in animation.

# **File formats[](https://docs.godotengine.org/en/stable/about/list_of_features.html#file-formats)**

- Scenes and resources can be saved in [text-based](https://docs.godotengine.org/en/stable/engine_details/file_formats/tscn.html#doc-tscn-file-format) or binary formats.
    
    > Text-based formats are human-readable and more friendly to version control.Binary formats are faster to save/load for large scenes/resources.
    > 
- Read and write text or binary files using [FileAccess](https://docs.godotengine.org/en/stable/classes/class_fileaccess.html#class-fileaccess).
    
    > Can optionally be compressed or encrypted.
    > 
- Read and write [JSON](https://docs.godotengine.org/en/stable/classes/class_json.html#class-json) files.
- Read and write INI-style configuration files using [ConfigFile](https://docs.godotengine.org/en/stable/classes/class_configfile.html#class-configfile).
    
    > Can (de)serialize any Godot datatype, including Vector2/3, Color, ...
    > 
- Read XML files using [XMLParser](https://docs.godotengine.org/en/stable/classes/class_xmlparser.html#class-xmlparser).
- [Load and save images, audio/video, fonts and ZIP archives](https://docs.godotengine.org/en/stable/tutorials/io/runtime_file_loading_and_saving.html#doc-runtime-loading-and-saving) in an exported project without having to go through Godot's import system.
- Pack game data into a PCK file (custom format optimized for fast seeking), into a ZIP archive, or directly into the executable for single-file distribution.
- [Export additional PCK files](https://docs.godotengine.org/en/stable/tutorials/export/exporting_pcks.html#doc-exporting-pcks) that can be read by the engine to support mods and DLCs.

# **Miscellaneous[](https://docs.godotengine.org/en/stable/about/list_of_features.html#miscellaneous)**

- [Video playback](https://docs.godotengine.org/en/stable/tutorials/animation/playing_videos.html#doc-playing-videos) with built-in support for Ogg Theora.
- [Movie Maker mode](https://docs.godotengine.org/en/stable/tutorials/animation/creating_movies.html#doc-creating-movies) to record videos from a running project with synchronized audio and perfect frame pacing.
- [Low-level access to servers](https://docs.godotengine.org/en/stable/tutorials/performance/using_servers.html#doc-using-servers) which allows bypassing the scene tree's overhead when needed.
- [Command line interface](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html#doc-command-line-tutorial) for automation.
    
    > Export and deploy projects using continuous integration platforms.Shell completion scripts are available for Bash, zsh and fish.Print colored text to standard output on all platforms using print_rich.
    > 
- The editor can [detect features used in a project and create a compilation profile](https://docs.godotengine.org/en/stable/tutorials/editor/using_engine_compilation_configuration_editor.html#doc-engine-compilation-configuration-editor), which can be used to create smaller export template binaries with unneeded features disabled.
- Support for [C++ modules](https://docs.godotengine.org/en/stable/engine_details/architecture/custom_modules_in_cpp.html#doc-custom-modules-in-cpp) statically linked into the engine binary.
    - Most built-in modules can be disabled at compile-time to reduce binary size in custom builds. See [Optimizing a build for size](https://docs.godotengine.org/en/stable/engine_details/development/compiling/optimizing_for_size.html#doc-optimizing-for-size) for details.
- Engine and editor written in C++17.
    
    > Can be compiled using GCC, Clang and MSVC. MinGW is also supported.Friendly towards packagers. In most cases, system libraries can be used instead of the ones provided by Godot. The build system doesn't download anything. Builds can be fully reproducible.
    > 
- Licensed under the permissive MIT license.
    
    > Open development process with contributions welcome.
    > 

Wicked Engine
Home
Forum
Downloads
Devblog
Discord
GitHub
Patreon
About
Privacy Policy
Wicked Engine’s graphics in 2024
Wicked Engine’s graphics in 2024
We are near the end of 2024 and I wanted to write up a long post about the current state of the rendering in Wicked Engine. If you are interested in graphics programming, then strap yourself in for a long read through some coarse brain dump, without going in too deep to any of the

Read article →

Texture Streaming
Texture Streaming
Texture streaming is an important feature of modern 3D engines as it is can be the largest contributor to reducing loading times and memory usage. Wicked Engine just got the first implementation of this system, and here you can read about the details in depth. Overview There are many various forms of texture streaming, here

Read article →

Dynamic vertex formats
Dynamic vertex formats
There are a variety of ways to send vertex data to the GPU. Since DX12 and Vulkan, we can choose to use the old-school input layouts definitions in the pipeline state, or using descriptors, which became much more flexible since the DX11-era limitations. Wicked Engine has been using descriptors with bindless manual fetching for a

Read article →

Vulkan Video Decoding
Vulkan Video Decoding
Recently the Vulkan API received an exciting new feature, which is video decoding, utilizing the built-in fixed function video unit found in many GPUs. This allows to the writing of super fast cross-platform video applications while freeing up the CPU from expensive decoding tasks.

Read article →

Graphics API secrets: format casting
Graphics API secrets: format casting
If you spend a long enough time in graphics development, the time will come eventually when you want to cast between different formats of a GPU resource. The problem is that information about how to do this was a bit hard to come by – until now.

Read article →

Animation Retargeting
Animation Retargeting
I recently implemented animation retargeting, something that’s not frequently discussed on the internet in detail. My goal was specifically to copy animations between similar types of skeletons: humanoid to humanoid. A more complicated solution for example, that can retarget humanoid animation to a frog was not my intention.

Read article →

Game dev journey: 10 years
Game dev journey: 10 years
10 years ago, I created my first game and became a game developer. Remembering the journey:

Read article →

Derivatives in compute shader
Derivatives in compute shader
This post shows a way to compute derivatives for texture filtering in compute shaders (for visibility buffer shading). I was missing a step-by-step explanation of how to do this, but after some trial and error, the following method turned out to work well.

Read article →

Shader compiler tools
Shader compiler tools
Wicked Engine used Visual Studio to compile all its shaders for a long time, but that changed around a year ago (in 2021) when custom shader compiling tools were implemented. This blog highlights the benefits of this and may provide some new ideas if you are developing graphics programs or tools.

Read article →

Future geometry pipeline
Future geometry pipeline
Lately I’ve been interested in modernizing the geometry pipeline in the rendering engine, for example reducing the vertex shaders. It’s a well known fact that geometry shaders are not making efficient use of the GPU hardware, but similar issues could apply for vertex and tessellation shaders too, just because there is a need for pushing

Read article →

Graphics API abstraction
Graphics API abstraction
Wicked Engine can handle today’s advanced rendering effects, with multiple graphics APIs (DX11, DX12 and Vulkan at the time of writing this). The key to enable this is to use a good graphics abstraction, so these complicated algorithms only need to be written once.

Read article →

Bindless Descriptors
Bindless Descriptors
The Vulkan and DX12 graphics devices now support bindless descriptors in Wicked Engine. Earlier and in DX11, it was only possible to access textures, buffers (resource descriptors) and samplers in the shaders by binding them to specific slots. First, the binding model limitations will be described briefly, then the bindless model will be discussed.

Read article →

Variable Rate Shading: first impressions
Variable Rate Shading: first impressions
Variable Rate Shading (VRS) is a new DX12 feature introduced recently, that can be used to control shading rate. To be more precise, it is used to reduce shading rate, as opposed to the Multi Sampling Anti Aliasing (MSAA) technique which is used to increase it.

Read article →

Capsule Collision Detection
Capsule Collision Detection
Capsule shapes are useful tools for handling simple game physics. Here you will find out why and how to detect and handle collisions between capsule and triangle mesh, as well as with other capsules, without using a physics engine.

Read article →

Tile-based optimization for post processing
Tile-based optimization for post processing
One way to optimize heavy post processing shaders is to determine which parts of the screen could use a simpler version. The simplest form of this is use branching in the shader code to early exit or switch to a variant with reduced sample count or computations. This comes with a downside that even the

Read article →

Entity-component system
Entity-component system
Here goes my idea of an entity-component system written in C++. I’ve been using this in my home-made game engine, Wicked Engine for exactly a year now and I am still very happy with it. The focus is on simplicity and performance, not adding many features.

Read article →

Improved normal reconstruction from depth
Improved normal reconstruction from depth
In a 3D renderer, we might want to read the scene normal vectors at some point, for example post processes. We can write them out using MRT – multiple render target outputs from the object rendering shaders and write the surface normals to a texture. But that normal map texture usually contains normals that have

Read article →

Thoughts on light culling: stream compaction vs flat bit arrays
Thoughts on light culling: stream compaction vs flat bit arrays
I had my eyes set on the light culling using flat bit arrays technique for a long time now and finally decided to give it a try. Let me share my notes on why it seemed so interesting and why I replaced the stream compaction technique with this. I will describe both techniques and make

Read article →

Simple job system using standard C++
Simple job system using standard C++
After experimenting with the entity-component system this fall, I wanted to see how difficult it would be to put my other unused CPU cores to good use. I never really got into CPU multithreading seriously, so this is something new for me. The idea behind the entity-component system is both to make more efficient use

Read article →

GPU Fluid Simulation
Let’s take a look at how to efficiently implement a particle based fluid simulation for real time rendering. We will be running a Smooth Particle Hydrodynamics (SPH) simulation on the GPU. This post is intended for experienced developers and provide the general steps of implementation. It is not a step-by step tutorial, but rather introducing

Read article →

Thoughts on Skinning and LDS
I’m letting out some thoughts on using LDS memory as a means to optimize a skinning compute shader. Consider the following workload: each thread is responsible for animating a single vertex, so it loads the vertex position, normal, bone indices and bone weights from a vertex buffer. After this, it starts doing the skinning: for

Read article →

Easy Transparent Shadow Maps
Supporting transparencies with traditional shadow mapping is straight forward and allows for nice effects but as with anything related to rendering transparents with rasterization, there are corner cases. Little sneak peak of what you can achieve with this:

Read article →

Optimizing tile-based light culling
Optimizing tile-based light culling
Tile-based lighting techniques like Forward+ and Tiled Deferred rendering are widely used these days. With the help of such technique we can efficiently query every light affecting any surface. But a trivial implementation has many ways to improve. The biggest goal is to refine the culling results as much as possible to help reduce the

Read article →

Next power of two in HLSL
There are many occasions when a programmer would want to calculate the next power of two for a given number. For me it was a bitonic sorting algorithm operating in a compute shader and I had this piece of code be responsible for calculating the next power of two of a number: uint myNumberPowerOfTwo =

Read article →

GPU-based particle simulation
GPU-based particle simulation
I finally took the leap and threw out my old CPU-based particle simulation code and ventured to GPU realms with it. The old system could spawn particles on the surface on a mesh with a starting velocity of each particle modulated by the surface normal. It kept a copy of each particle on CPU, updated

Read article →

Which blend state for me?
If you are familiar with creating graphics applications, you are probably somewhat familiar with different blending states. If you are like me, then you were not overly confident in using them, and got some basics ones copy-pasted from the web. Maybe got away with simpe alpha blending and additive states, and heard of premultiplied alpha

Read article →

Forward+ decal rendering
Drawing decals in deferred renderers is quite simple, straight forward and efficient: Just render boxes like you render the lights, read the gbuffer in the pixel shader, project onto the surface, then sample and blend the decal texture. The light evaluation then already computes lighting for the decaled surfaces. In traditional forward rendering pipelines, this

Read article →

Skinning in a Compute Shader
Recently I have moved my mesh skinning implementation from a streamout geometry shader to compute shader. One reason for this was the ugly API for the streamout which I wanted to leave behind, but the more important reason was that this could come with several benefits.

Read article →

Area Lights
I am trying to get back into blogging. I thought writing about implementing area light rendering might help me with that.

Read article →

Voxel-based Global Illumination
Voxel-based Global Illumination
There are several use cases of a voxel data structure. One interesting application is using it to calculate global illumination. There are a couple of techniques for that, too. I have chosen the voxel cone tracing approach, because I found it the most flexible one for dynamic scenes, but CryEngine for example, uses Light propagation

Read article →

Should we get rid of Vertex Buffers?
TLDR: If your only platform to support is a recent AMD GPU, or console, then yes. 🙂 I am working on a “game engine” nowadays but mainly focusing on the rendering aspect. I wanted to get rid of some APIs lately in my graphics wrapper to be more easier to use, because I just hate

Read article →

How to Resolve an MSAA DepthBuffer
If you want to implement MSAA (multisampled antialiasing) rendering, you need to render into multismpled render targets. When you want to read an anti aliased rendertarget as a shader resource, first you need to resolve it. Resolving means copying it to a non multisampled texture and averaging the subsamples (in D3D11 it is performed by

Read article →

Abuse the immediate constant buffer!
Very often, I need to draw simple geometries, like cubes, and I want to do the minimal amount of graphics state setup. With this technique, you don’t have to set up a vertex buffer or input layout, which means, we don’t have to write the boilerplate resource creation code for them, and don’t have to call the

Read article →

Smooth Lens Flare in the Geometry Shader
This is a historical feature from the Wicked Engine, meaning it was implemented a few years ago, but at the time it was a big step for me. I wanted to implement simple textured lens flares but at the time all I could find was by using occlusion queries to determine if a lens flare

Read article →

Welcome brave developer!
This is a blog containing development insight to my game engine, Wicked Engine. Feel free to rip off any code, example, techinque from here, as you could also do it from the open source engine itself: https://github.com/turanszkij/WickedEngine I want to post info from historical features as well as new ones. I try to select the ones

Read article →

Home
Forum
Downloads
Devblog
Discord
GitHub
Patreon
About
Privacy Policy

information, please [create an issue](https://github.com/redot-engine/redot-docs).

# **List of features[](https://docs.redotengine.org/en/stable/about/list_of_features#list-of-features)**

This page aims to list **all** features currently supported by Redot.

**Note**

This page lists features supported by the current stable version of Redot. Some of these features are not available in the [3.x release series](https://docs.redotengine.org/en/3.6/about/list_of_features.html).

# **Platforms[](https://docs.redotengine.org/en/stable/about/list_of_features#platforms)**

**See also**

See [System requirements](https://docs.redotengine.org/en/stable/about/system_requirements.html#doc-system-requirements) for hardware and software version requirements.

**Can run both the editor and exported projects:**

- Windows (x86 and ARM, 64-bit and 32-bit).
- macOS (x86 and ARM, 64-bit only).
- Linux (x86 and ARM, 64-bit and 32-bit).
    
    > Binaries are statically linked and can run on any distribution if compiled on an old enough base distribution.Official binaries are compiled using the Redot Engine buildroot, allowing for binaries that work across common Linux distributions.
    > 
- Android (editor support is experimental).
- [Web browsers](https://docs.redotengine.org/en/stable/tutorials/editor/using_the_web_editor.html#doc-using-the-web-editor). Experimental in 4.0, using Redot 3.x is recommended instead when targeting HTML5.

**Runs exported projects:**

- iOS.
- [Consoles](https://docs.redotengine.org/en/stable/tutorials/platform/consoles.html#doc-consoles).

Redot aims to be as platform-independent as possible and can be [ported to new platforms](https://docs.redotengine.org/en/stable/contributing/development/core_and_modules/custom_platform_ports.html#doc-custom-platform-ports) with relative ease.

**Note**

Projects written in C# using Redot 4 currently cannot be exported to the web platform. To use C# on that platform, consider Redot 3 instead. Android and iOS platform support is available as of Redot 4.2, but is experimental and [some limitations apply](https://docs.redotengine.org/en/stable/tutorials/scripting/c_sharp/index.html#doc-c-sharp-platforms).

# **Editor[](https://docs.redotengine.org/en/stable/about/list_of_features#editor)**

**Features:**

- Scene tree editor.
- Built-in script editor.
- Support for [external script editors](https://docs.redotengine.org/en/stable/tutorials/editor/external_editor.html#doc-external-editor) such as Visual Studio Code or Vim.
- GDScript [debugger](https://docs.redotengine.org/en/stable/tutorials/scripting/debug/debugger_panel.html#doc-debugger-panel).
    
    > Support for debugging in threads is available since 4.2.
    > 
- Visual profiler with CPU and GPU time indications for each step of the rendering pipeline.
- Performance monitoring tools, including [custom performance monitors](https://docs.redotengine.org/en/stable/tutorials/scripting/debug/custom_performance_monitors.html#doc-custom-performance-monitors).
- Live script reloading.
- Live scene editing.
    
    > Changes will reflect in the editor and will be kept after closing the running project.
    > 
- Remote inspector.
    
    > Changes won't reflect in the editor and won't be kept after closing the running project.
    > 
- Live camera replication.
    
    > Move the in-editor camera and see the result in the running project.
    > 
- Built-in offline class reference documentation.
- Use the editor in dozens of languages contributed by the community.

**Plugins:**

- Editor plugins can be downloaded from the [asset library](https://docs.redotengine.org/en/stable/community/asset_library/what_is_assetlib.html#doc-what-is-assetlib) to extend editor functionality.
- [Create your own plugins](https://docs.redotengine.org/en/stable/tutorials/plugins/editor/making_plugins.html#doc-making-plugins) using GDScript to add new features or speed up your workflow.
- [Download projects from the asset library](https://docs.redotengine.org/en/stable/community/asset_library/using_assetlib.html#doc-using-assetlib-editor) in the Project Manager and import them directly.

# **Rendering[](https://docs.redotengine.org/en/stable/about/list_of_features#rendering)**

3 rendering *methods* (running over 2 rendering *drivers*) are available:

- **Forward+**, running over Vulkan 1.0 (with optional Vulkan 1.1 and 1.2 features). The most advanced graphics backend, suited for desktop platforms only. Used by default on desktop platforms.
- **Forward Mobile**, running over Vulkan 1.0 (with optional Vulkan 1.1 and 1.2 features). Less features, but renders simple scenes faster. Suited for mobile and desktop platforms. Used by default on mobile platforms.
- **Compatibility**, running over OpenGL 3.3 / OpenGL ES 3.0 / WebGL 2.0. The least advanced graphics backend, suited for low-end desktop and mobile platforms. Used by default on the web platform.

# **2D graphics[](https://docs.redotengine.org/en/stable/about/list_of_features#d-graphics)**

- Sprite, polygon and line rendering.
    
    > High-level tools to draw lines and polygons such as Polygon2D and Line2D, with support for texturing.
    > 
- AnimatedSprite2D as a helper for creating animated sprites.
- Parallax layers.
    
    > Pseudo-3D support including preview in the editor.
    > 
- [2D lighting](https://docs.redotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html#doc-2d-lights-and-shadows) with normal maps and specular maps.
    
    > Point (omni/spot) and directional 2D lights.Hard or soft shadows (adjustable on a per-light basis).Custom shaders can access a real-time SDF representation of the 2D scene based on LightOccluder2D nodes, which can be used for improved 2D lighting effects including 2D global illumination.
    > 
- [Font rendering](https://docs.redotengine.org/en/stable/tutorials/ui/gui_using_fonts.html#doc-gui-using-fonts) using bitmaps, rasterization using FreeType or multi-channel signed distance fields (MSDF).
    
    > Bitmap fonts can be exported using tools like BMFont, or imported from images (for fixed-width fonts only).Dynamic fonts support monochrome fonts as well as colored fonts (e.g. for emoji). Supported formats are TTF, OTF, WOFF1 and WOFF2.Dynamic fonts support optional font outlines with adjustable width and color.Dynamic fonts support variable fonts and OpenType features including ligatures.Dynamic fonts support simulated bold and italic when the font file lacks those styles.Dynamic fonts support oversampling to keep fonts sharp at higher resolutions.Dynamic fonts support subpixel positioning to make fonts crisper at low sizes.Dynamic fonts support LCD subpixel optimizations to make fonts even crisper at low sizes.Signed distance field fonts can be scaled at any resolution without requiring re-rasterization. Multi-channel usage makes SDF fonts scale down to lower sizes better compared to monochrome SDF fonts.
    > 
- GPU-based [particles](https://docs.redotengine.org/en/stable/tutorials/2d/particle_systems_2d.html#doc-particle-systems-2d) with support for [custom particle shaders](https://docs.redotengine.org/en/stable/tutorials/shaders/shader_reference/particle_shader.html#doc-particle-shader).
- CPU-based particles.
- Optional [2D HDR rendering](https://docs.redotengine.org/en/stable/tutorials/3d/environment_and_post_processing.html#doc-environment-and-post-processing-using-glow-in-2d) for better glow capabilities.

# **2D tools[](https://docs.redotengine.org/en/stable/about/list_of_features#d-tools)**

- [TileMaps](https://docs.redotengine.org/en/stable/tutorials/2d/using_tilemaps.html#doc-using-tilemaps) for 2D tile-based level design.
- 2D camera with built-in smoothing and drag margins.
- Path2D node to represent a path in 2D space.
    
    > Can be drawn in the editor or generated procedurally.PathFollow2D node to make nodes follow a Path2D.
    > 
- [2D geometry helper class](https://docs.redotengine.org/en/stable/classes/class_geometry2d.html#class-geometry2d).

# **2D physics[](https://docs.redotengine.org/en/stable/about/list_of_features#d-physics)**

**Physics bodies:**

- Static bodies.
- Animatable bodies (for objects moving only by script or animation, such as doors and platforms).
- Rigid bodies.
- Character bodies.
- Joints.
- Areas to detect bodies entering or leaving it.

**Collision detection:**

- Built-in shapes: line, box, circle, capsule, world boundary (infinite plane).
- Collision polygons (can be drawn manually or generated from a sprite in the editor).

# **3D graphics[](https://docs.redotengine.org/en/stable/about/list_of_features#id1)**

- HDR rendering with sRGB.
- Perspective, orthographic and frustum-offset cameras.
- When using the Forward+ backend, a depth prepass is used to improve performance in complex scenes by reducing the cost of overdraw.
- [Variable rate shading](https://docs.redotengine.org/en/stable/tutorials/3d/variable_rate_shading.html#doc-variable-rate-shading) on supported GPUs in Forward+ and Forward Mobile.

**Physically-based rendering (built-in material features):**

- Follows the Disney PBR model.
- Supports Burley, Lambert, Lambert Wrap (half-Lambert) and Toon diffuse shading modes.
- Supports Schlick-GGX, Toon and Disabled specular shading modes.
- Uses a roughness-metallic workflow with support for ORM textures.
- Uses horizon specular occlusion (Filament model) to improve material appearance.
- Normal mapping.
- Parallax/relief mapping with automatic level of detail based on distance.
- Detail mapping for the albedo and normal maps.
- Sub-surface scattering and transmittance.
- Screen-space refraction with support for material roughness (resulting in blurry refraction).
- Proximity fade (soft particles) and distance fade.
- Distance fade can use alpha blending or dithering to avoid going through the transparent pipeline.
- Dithering can be determined on a per-pixel or per-object basis.

**Real-time lighting:**

- Directional lights (sun/moon). Up to 4 per scene.
- Omnidirectional lights.
- Spot lights with adjustable cone angle and attenuation.
- Specular, indirect light, and volumetric fog energy can be adjusted on a per-light basis.
- Adjustable light "size" for fake area lights (will also make shadows blurrier).
- Optional distance fade system to fade distant lights and their shadows, improving performance.
- When using the Forward+ backend (default on desktop), lights are rendered with clustered forward optimizations to decrease their individual cost. Clustered rendering also lifts any limits on the number of lights that can be used on a mesh.
- When using the Forward Mobile backend, up to 8 omni lights and 8 spot lights can be displayed per mesh resource. Baked lighting can be used to overcome this limit if needed.

**Shadow mapping:**

- *DirectionalLight:* Orthogonal (fastest), PSSM 2-split and 4-split. Supports blending between splits.
- *OmniLight:* Dual paraboloid (fast) or cubemap (slower but more accurate). Supports colored projector textures in the form of panoramas.
- *SpotLight:* Single texture. Supports colored projector textures.
- Shadow normal offset bias and shadow pancaking to decrease the amount of visible shadow acne and peter-panning.
- PCSSlike shadow blur based on the light size and distance from the surface the shadow is cast on.
- Adjustable shadow blur on a per-light basis.

**Global illumination with indirect lighting:**

- [Baked lightmaps](https://docs.redotengine.org/en/stable/tutorials/3d/global_illumination/using_lightmap_gi.html#doc-using-lightmap-gi) (fast, but can't be updated at run-time).
    
    > Supports baking indirect light only or baking both direct and indirect lighting. The bake mode can be adjusted on a per-light basis to allow for hybrid light baking setups.Supports lighting dynamic objects using automatic and manually placed probes.Optionally supports directional lighting and rough reflections based on spherical harmonics.Lightmaps are baked on the GPU using compute shaders (much faster compared to CPU lightmapping). Baking can only be performed from the editor, not in exported projects.Supports GPU-based denoising with JNLM, or CPU/GPU-based denoising with OIDN.
    > 
- [Voxel-based GI probes](https://docs.redotengine.org/en/stable/tutorials/3d/global_illumination/using_voxel_gi.html#doc-using-voxel-gi). Supports dynamic lights *and* dynamic occluders, while also supporting reflections. Requires a fast baking step which can be performed in the editor or at run-time (including from an exported project).
- [Signed-distance field GI](https://docs.redotengine.org/en/stable/tutorials/3d/global_illumination/using_sdfgi.html#doc-using-sdfgi) designed for large open worlds. Supports dynamic lights, but not dynamic occluders. Supports reflections. No baking required.
- [Screen-space indirect lighting (SSIL)](https://docs.redotengine.org/en/stable/tutorials/3d/environment_and_post_processing.html#doc-environment-and-post-processing-ssil) at half or full resolution. Fully real-time and supports any kind of emissive light source (including decals).
- VoxelGI and SDFGI use a deferred pass to allow for rendering GI at half resolution to improve performance (while still having functional MSAA support).

**Reflections:**

- Voxel-based reflections (when using GI probes) and SDF-based reflections (when using signed distance field GI). Voxel-based reflections are visible on transparent surfaces, while rough SDF-based reflections are visible on transparent surfaces.
- Fast baked reflections or slow real-time reflections using ReflectionProbe. Parallax box correction can optionally be enabled.
- Screen-space reflections with support for material roughness.
- Reflection techniques can be mixed together for greater accuracy or scalability.
- When using the Forward+ backend (default on desktop), reflection probes are rendered with clustered forward optimizations to decrease their individual cost. Clustered rendering also lifts any limits on the number of reflection probes that can be used on a mesh.
- When using the Forward Mobile backend, up to 8 reflection probes can be displayed per mesh resource.

**Decals:**

- [Supports albedo](https://docs.redotengine.org/en/stable/tutorials/3d/using_decals.html#doc-using-decals), emissive, ORM, and normal mapping.
- Texture channels are smoothly overlaid on top of the underlying material, with support for normal/ORM-only decals.
- Support for normal fade to fade the decal depending on its incidence angle.
- Does not rely on run-time mesh generation. This means decals can be used on complex skinned meshes with no performance penalty, even if the decal moves every frame.
- Support for nearest, bilinear, trilinear or anisotropic texture filtering (configured globally).
- Optional distance fade system to fade distant decals, improving performance.
- When using the Forward+ backend (default on desktop), decals are rendered with clustered forward optimizations to decrease their individual cost. Clustered rendering also lifts any limits on the number of decals that can be used on a mesh.
- When using the Forward Mobile backend, up to 8 decals can be displayed per mesh resource.

**Sky:**

- Panorama sky (using an HDRI).
- Procedural sky and Physically-based sky that respond to the DirectionalLights in the scene.
- Support for [custom sky shaders](https://docs.redotengine.org/en/stable/tutorials/shaders/shader_reference/sky_shader.html#doc-sky-shader), which can be animated.
- The radiance map used for ambient and specular light can be updated in real-time depending on the quality settings chosen.

**Fog:**

- Exponential depth fog.
- Exponential height fog.
- Support for automatic fog color depending on the sky color (aerial perspective).
- Support for sun scattering in the fog.
- Support for controlling how much fog rendering should affect the sky, with separate controls for traditional and volumetric fog.
- Support for making specific materials ignore fog.

**Volumetric fog:**

- Global [volumetric fog](https://docs.redotengine.org/en/stable/tutorials/3d/volumetric_fog.html#doc-volumetric-fog) that reacts to lights and shadows.
- Volumetric fog can take indirect light into account when using VoxelGI or SDFGI.
- Fog volume nodes that can be placed to add fog to specific areas (or remove fog from specific areas). Supported shapes include box, ellipse, cone, cylinder, and 3D texture-based density maps.
- Each fog volume can have its own custom shader.
- Can be used together with traditional fog.

**Particles:**

- GPU-based particles with support for subemitters (2D + 3D), trails (2D + 3D), attractors (3D only) and collision (2D + 3D).
    - 3D particle attractor shapes supported: box, sphere and 3D vector fields.
    - 3D particle collision shapes supported: box, sphere, baked signed distance field and real-time heightmap (suited for open world weather effects).
    - 2D particle collision is handled using a signed distance field generated in real-time based on [LightOccluder2D](https://docs.redotengine.org/en/stable/classes/class_lightoccluder2d.html#class-lightoccluder2d) nodes in the scene.
    - Trails can use the built-in ribbon trail and tube trail meshes, or custom meshes with skeletons.
    - Support for custom particle shaders with manual emission.
- CPU-based particles.

**Post-processing:**

- Tonemapping (Linear, Reinhard, Filmic, ACES).
- Automatic exposure adjustments based on viewport brightness (and manual exposure override).
- Near and far depth of field with adjustable bokeh simulation (box, hexagon, circle).
- Screen-space ambient occlusion (SSAO) at half or full resolution.
- Glow/bloom with optional bicubic upscaling and several blend modes available: Screen, Soft Light, Add, Replace, Mix.
- Glow can have a colored dirt map texture, acting as a lens dirt effect.
- Glow can be [used as a screen-space blur effect](https://docs.redotengine.org/en/stable/tutorials/3d/environment_and_post_processing.html#doc-environment-and-post-processing-using-glow-to-blur-the-screen).
- Color correction using a one-dimensional ramp or a 3D LUT texture.
- Roughness limiter to reduce the impact of specular aliasing.
- Brightness, contrast and saturation adjustments.

**Texture filtering:**

- Nearest, bilinear, trilinear or anisotropic filtering.
- Filtering options are defined on a per-use basis, not a per-texture basis.

**Texture compression:**

- Basis Universal (slow, but results in smaller files).
- BPTC for high-quality compression (not supported on macOS).
- ETC2 (not supported on macOS).
- S3TC (not supported on mobile/Web platforms).

**Antialiasing:**

- Temporal [antialiasing](https://docs.redotengine.org/en/stable/tutorials/3d/3d_antialiasing.html#doc-3d-antialiasing) (TAA).
- AMD FidelityFX Super Resolution 2.2 [antialiasing](https://docs.redotengine.org/en/stable/tutorials/3d/3d_antialiasing.html#doc-3d-antialiasing) (FSR2), which can be used at native resolution as a form of high-quality temporal antialiasing.
- Multi-sample antialiasing (MSAA), for both [2D antialiasing](https://docs.redotengine.org/en/stable/tutorials/2d/2d_antialiasing.html#doc-2d-antialiasing) and [3D antialiasing](https://docs.redotengine.org/en/stable/tutorials/3d/3d_antialiasing.html#doc-3d-antialiasing).
- Fast approximate antialiasing (FXAA).
- Super-sample antialiasing (SSAA) using bilinear 3D scaling and a 3D resolution scale above 1.0.
- Alpha antialiasing, MSAA alpha to coverage and alpha hashing on a per-material basis.

**Resolution scaling:**

- Support for [rendering 3D at a lower resolution](https://docs.redotengine.org/en/stable/tutorials/3d/resolution_scaling.html#doc-resolution-scaling) while keeping 2D rendering at the original scale. This can be used to improve performance on low-end systems or improve visuals on high-end systems.
- Resolution scaling uses bilinear filtering, AMD FidelityFX Super Resolution 1.0 (FSR1) or AMD FidelityFX Super Resolution 2.2 (FSR2).
- Texture mipmap LOD bias is adjusted automatically to improve quality at lower resolution scales. It can also be modified with a manual offset.

Most effects listed above can be adjusted for better performance or to further improve quality. This can be helpful when [using Redot for offline rendering](https://docs.redotengine.org/en/stable/tutorials/animation/creating_movies.html#doc-creating-movies).

# **3D tools[](https://docs.redotengine.org/en/stable/about/list_of_features#id2)**

- Built-in meshes: cube, cylinder/cone, (hemi)sphere, prism, plane, quad, torus, ribbon, tube.
- [GridMaps](https://docs.redotengine.org/en/stable/tutorials/3d/using_gridmaps.html#doc-using-gridmaps) for 3D tile-based level design.
- [Constructive solid geometry](https://docs.redotengine.org/en/stable/tutorials/3d/csg_tools.html#doc-csg-tools) (intended for prototyping).
- Tools for [procedural geometry generation](https://docs.redotengine.org/en/stable/tutorials/3d/procedural_geometry/index.html#doc-procedural-geometry).
- Path3D node to represent a path in 3D space.
    
    > Can be drawn in the editor or generated procedurally.PathFollow3D node to make nodes follow a Path3D.
    > 
- [3D geometry helper class](https://docs.redotengine.org/en/stable/classes/class_geometry3d.html#class-geometry3d).
- Support for exporting the current scene as a glTF 2.0 file, both from the editor and at run-time from an exported project.

# **3D physics[](https://docs.redotengine.org/en/stable/about/list_of_features#id3)**

**Physics bodies:**

- Static bodies.
- Animatable bodies (for objects moving only by script or animation, such as doors and platforms).
- Rigid bodies.
- Character bodies.
- Vehicle bodies (intended for arcade physics, not simulation).
- Joints.
- Soft bodies.
- Ragdolls.
- Areas to detect bodies entering or leaving it.

**Collision detection:**

- Built-in shapes: cuboid, sphere, capsule, cylinder, world boundary (infinite plane).
- Generate triangle collision shapes for any mesh from the editor.
- Generate one or several convex collision shapes for any mesh from the editor.

# **Shaders[](https://docs.redotengine.org/en/stable/about/list_of_features#shaders)**

- *2D:* Custom vertex, fragment, and light shaders.
- *3D:* Custom vertex, fragment, light, and sky shaders.
- Text-based shaders using a [shader language inspired by GLSL](https://docs.redotengine.org/en/stable/tutorials/shaders/shader_reference/shading_language.html#doc-shading-language).
- Visual shader editor.
    
    > Support for visual shader plugins.
    > 

# **Scripting[](https://docs.redotengine.org/en/stable/about/list_of_features#scripting)**

**General:**

- Object-oriented design pattern with scripts extending nodes.
- Signals and groups for communicating between scripts.
- Support for [cross-language scripting](https://docs.redotengine.org/en/stable/tutorials/scripting/cross_language_scripting.html#doc-cross-language-scripting).
- Many 2D, 3D and 4D linear algebra data types such as vectors and transforms.

[GDScript:](https://docs.redotengine.org/en/stable/tutorials/scripting/gdscript/index.html#toc-learn-scripting-gdscript)

- [High-level interpreted language](https://docs.redotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_basics.html#doc-gdscript) with [optional static typing](https://docs.redotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html#doc-gdscript-static-typing).
- Syntax inspired by Python. However, GDScript is **not** based on Python.
- Syntax highlighting is provided on GitHub.
- [Use threads](https://docs.redotengine.org/en/stable/tutorials/performance/using_multiple_threads.html#doc-using-multiple-threads) to perform asynchronous actions or make use of multiple processor cores.

[C#:](https://docs.redotengine.org/en/stable/tutorials/scripting/c_sharp/index.html#toc-learn-scripting-c)

- Packaged in a separate binary to keep file sizes and dependencies down.
- Supports .NET 6 and higher.
    
    > Full support for the C# 10.0 syntax and features.
    > 
- Supports Windows, Linux, and macOS. As of 4.2 experimental support for Android and iOS is also available (requires a .NET 7.0 project for Android and 8.0 for iOS).
    
    > On the Android platform only some architectures are supported: arm64 and x64.On the iOS platform only some architectures are supported: arm64.The web platform is currently unsupported. To use C# on that platform, consider Redot 3 instead.
    > 
- Using an external editor is recommended to benefit from IDE functionality.

**GDExtension (C, C++, Rust, D, ...):**

- When you need it, link to native libraries for higher performance and third-party integrations.
    
    > For scripting game logic, GDScript or C# are recommended if their performance is suitable.
    > 
- Official GDExtension bindings for [C](https://github.com/redot-engine/redot-headers) and [C++](https://github.com/redot-engine/redot-cpp).
    
    > Use any build system and language features you wish.
    > 
- Actively developed GDExtension bindings for [D](https://github.com/redot-dlang/redot-dlang), [Swift](https://github.com/migueldeicaza/SwiftGodot), and [Rust](https://github.com/redot-rust/gdextension) bindings provided by the community. (Some of these bindings may be experimental and not production-ready).

# **Audio[](https://docs.redotengine.org/en/stable/about/list_of_features#audio)**

**Features:**

- Mono, stereo, 5.1 and 7.1 output.
- Non-positional and positional playback in 2D and 3D.
    
    > Optional Doppler effect in 2D and 3D.
    > 
- Support for re-routable [audio buses](https://docs.redotengine.org/en/stable/tutorials/audio/audio_buses.html#doc-audio-buses) and effects with dozens of effects included.
- Support for polyphony (playing several sounds from a single AudioStreamPlayer node).
- Support for random volume and pitch.
- Support for real-time pitch scaling.
- Support for sequential/random sample selection, including repetition prevention when using random sample selection.
- Listener2D and Listener3D nodes to listen from a position different than the camera.
- Support for [procedural audio generation](https://docs.redotengine.org/en/stable/classes/class_audiostreamgenerator.html#class-audiostreamgenerator).
- Audio input to record microphones.
- MIDI input.
    
    > No support for MIDI output yet.
    > 

**APIs used:**

- *Windows:* WASAPI.
- *macOS:* CoreAudio.
- *Linux:* PulseAudio or ALSA.

# **Import[](https://docs.redotengine.org/en/stable/about/list_of_features#import)**

- Support for [custom import plugins](https://docs.redotengine.org/en/stable/tutorials/plugins/editor/import_plugins.html#doc-import-plugins).

**Formats:**

- *Images:* See [Importing images](https://docs.redotengine.org/en/stable/tutorials/assets_pipeline/importing_images.html#doc-importing-images).
- *Audio:*
    
    > WAV with optional IMA-ADPCM compression.Ogg Vorbis.MP3.
    > 
- *3D scenes:* See [Importing 3D scenes](https://docs.redotengine.org/en/stable/tutorials/assets_pipeline/importing_3d_scenes/index.html#doc-importing-3d-scenes).
    
    > glTF 2.0 (recommended)..blend (by calling Blender's glTF export functionality transparently).FBX (by calling FBX2glTF transparently).Collada (.dae).Wavefront OBJ (static scenes only, can be loaded directly as a mesh or imported as a 3D scene).
    > 
- Support for loading glTF 2.0 scenes at run-time, including from an exported project.
- 3D meshes use [Mikktspace](http://www.mikktspace.com/) to generate tangents on import, which ensures consistency with other 3D applications such as Blender.

# **Input[](https://docs.redotengine.org/en/stable/about/list_of_features#input)**

- [Input mapping system](https://docs.redotengine.org/en/stable/tutorials/inputs/input_examples.html#doc-input-examples) using hardcoded input events or remappable input actions.
    
    > Axis values can be mapped to two different actions with a configurable deadzone.Use the same code to support both keyboards and gamepads.
    > 
- Keyboard input.
    
    > Keys can be mapped in "physical" mode to be independent of the keyboard layout.
    > 
- Mouse input.
    
    > The mouse cursor can be visible, hidden, captured or confined within the window.When captured, raw input will be used on Windows and Linux to sidestep the OS' mouse acceleration settings.
    > 
- Gamepad input (up to 8 simultaneous controllers).
- Pen/tablet input with pressure support.

# **Navigation[](https://docs.redotengine.org/en/stable/about/list_of_features#navigation)**

- A* algorithm in [2D](https://docs.redotengine.org/en/stable/classes/class_astar2d.html#class-astar2d) and [3D](https://docs.redotengine.org/en/stable/classes/class_astar3d.html#class-astar3d).
- Navigation meshes with dynamic obstacle avoidance in [2D](https://docs.redotengine.org/en/stable/tutorials/navigation/navigation_introduction_2d.html#doc-navigation-overview-2d) and [3D](https://docs.redotengine.org/en/stable/tutorials/navigation/navigation_introduction_3d.html#doc-navigation-overview-3d).
- Generate navigation meshes from the editor or at run-time (including from an exported project).

# **Networking[](https://docs.redotengine.org/en/stable/about/list_of_features#networking)**

- Low-level TCP networking using [StreamPeer](https://docs.redotengine.org/en/stable/classes/class_streampeer.html#class-streampeer) and [TCPServer](https://docs.redotengine.org/en/stable/classes/class_tcpserver.html#class-tcpserver).
- Low-level UDP networking using [PacketPeer](https://docs.redotengine.org/en/stable/classes/class_packetpeer.html#class-packetpeer) and [UDPServer](https://docs.redotengine.org/en/stable/classes/class_udpserver.html#class-udpserver).
- Low-level HTTP requests using [HTTPClient](https://docs.redotengine.org/en/stable/classes/class_httpclient.html#class-httpclient).
- High-level HTTP requests using [HTTPRequest](https://docs.redotengine.org/en/stable/classes/class_httprequest.html#class-httprequest).
    
    > Supports HTTPS out of the box using bundled certificates.
    > 
- [High-level multiplayer](https://docs.redotengine.org/en/stable/tutorials/networking/high_level_multiplayer.html#doc-high-level-multiplayer) API using UDP and ENet.
    
    > Automatic replication using remote procedure calls (RPCs).Supports unreliable, reliable and ordered transfers.
    > 
- [WebSocket](https://docs.redotengine.org/en/stable/tutorials/networking/websocket.html#doc-websocket) client and server, available on all platforms.
- [WebRTC](https://docs.redotengine.org/en/stable/tutorials/networking/webrtc.html#doc-webrtc) client and server, available on all platforms.
- Support for [UPnP](https://docs.redotengine.org/en/stable/classes/class_upnp.html#class-upnp) to sidestep the requirement to forward ports when hosting a server behind a NAT.

# **Internationalization[](https://docs.redotengine.org/en/stable/about/list_of_features#internationalization)**

- Full support for Unicode including emoji.
- Store localization strings using [CSV](https://docs.redotengine.org/en/stable/tutorials/i18n/internationalizing_games.html#doc-internationalizing-games) or [gettext](https://docs.redotengine.org/en/stable/tutorials/i18n/localization_using_gettext.html#doc-localization-using-gettext).
    - Support for generating gettext POT and PO files from the editor.
- Use localized strings in your project automatically in GUI elements or by using the **`tr()`** function.
- Support for pluralization and translation contexts when using gettext translations.
- Support for [bidirectional typesetting](https://docs.redotengine.org/en/stable/tutorials/i18n/internationalizing_games.html#doc-internationalizing-games-bidi), text shaping and OpenType localized forms.
- Automatic UI mirroring for right-to-left locales.
- Support for pseudolocalization to test your project for i18n-friendliness.

# **Windowing and OS integration[](https://docs.redotengine.org/en/stable/about/list_of_features#windowing-and-os-integration)**

- Spawn multiple independent windows within a single process.
- Move, resize, minimize, and maximize windows spawned by the project.
- Change the window title and icon.
- Request attention (will cause the title bar to blink on most platforms).
- Fullscreen mode.
    
    > Uses borderless fullscreen by default on Windows for fast alt-tabbing, but can optionally use exclusive fullscreen to reduce input lag.
    > 
- Borderless windows (fullscreen or non-fullscreen).
- Ability to keep a window always on top.
- Global menu integration on macOS.
- Execute commands in a blocking or non-blocking manner (including running multiple instances of the same project).
- Open file paths and URLs using default or custom protocol handlers (if registered on the system).
- Parse custom command line arguments.
- Any Redot binary (editor or exported project) can be [used as a headless server](https://docs.redotengine.org/en/stable/tutorials/export/exporting_for_dedicated_servers.html#doc-exporting-for-dedicated-servers) by starting it with the **`-headless`** command line argument. This allows running the engine without a GPU or display server.

# **Mobile[](https://docs.redotengine.org/en/stable/about/list_of_features#mobile)**

- In-app purchases on [Android](https://docs.redotengine.org/en/stable/tutorials/platform/android/android_in_app_purchases.html#doc-android-in-app-purchases) and [iOS](https://docs.redotengine.org/en/stable/tutorials/platform/ios/plugins_for_ios.html#doc-plugins-for-ios).
- Support for advertisements using third-party modules.

# **XR support (AR and VR)[](https://docs.redotengine.org/en/stable/about/list_of_features#xr-support-ar-and-vr)**

- Out of the box [support for OpenXR](https://docs.redotengine.org/en/stable/tutorials/xr/setting_up_xr.html#doc-setting-up-xr).
    
    > Including support for popular desktop headsets like the Valve Index, WMR headsets, and Quest over Link.
    > 
- Support for [Android based headsets](https://docs.redotengine.org/en/stable/tutorials/xr/deploying_to_android.html#doc-deploying-to-android) using OpenXR through a plugin.
    - Including support for popular stand alone headsets like the Meta Quest 1/2/3 and Pro, Pico 4, Magic Leap 2, and Lynx R1.
- Other devices supported through an XR plugin structure.
- Various advanced toolkits are available that implement common features required by XR applications.

# **GUI system[](https://docs.redotengine.org/en/stable/about/list_of_features#gui-system)**

Redot's GUI is built using the same Control nodes used to make games in Redot. The editor UI can easily be extended in many ways using add-ons.

**Nodes:**

- Buttons.
- Checkboxes, check buttons, radio buttons.
- Text entry using [LineEdit](https://docs.redotengine.org/en/stable/classes/class_lineedit.html#class-lineedit) (single line) and [TextEdit](https://docs.redotengine.org/en/stable/classes/class_textedit.html#class-textedit) (multiple lines). TextEdit also supports code editing features such as displaying line numbers and syntax highlighting.
- Dropdown menus using [PopupMenu](https://docs.redotengine.org/en/stable/classes/class_popupmenu.html#class-popupmenu) and [OptionButton](https://docs.redotengine.org/en/stable/classes/class_optionbutton.html#class-optionbutton).
- Scrollbars.
- Labels.
- RichTextLabel for [text formatted using BBCode](https://docs.redotengine.org/en/stable/tutorials/ui/bbcode_in_richtextlabel.html#doc-bbcode-in-richtextlabel), with support for animated custom effects.
- Trees (can also be used to represent tables).
- Color picker with RGB and HSV modes.
- Controls can be rotated and scaled.

**Sizing:**

- Anchors to keep GUI elements in a specific corner, edge or centered.
- Containers to place GUI elements automatically following certain rules.
    
    > Stack layouts.Grid layouts.Flow layouts (similar to autowrapping text).Margin, centered and aspect ratio layouts.Draggable splitter layouts.
    > 
- Scale to [multiple resolutions](https://docs.redotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html#doc-multiple-resolutions) using the **`canvas_items`** or **`viewport`** stretch modes.
- Support any aspect ratio using anchors and the **`expand`** stretch aspect.

**Theming:**

- Built-in theme editor.
    
    > Generate a theme based on the current editor theme settings.
    > 
- Procedural vector-based theming using [StyleBoxFlat](https://docs.redotengine.org/en/stable/classes/class_styleboxflat.html#class-styleboxflat).
    
    > Supports rounded/beveled corners, drop shadows, per-border widths and antialiasing.
    > 
- Texture-based theming using [StyleBoxTexture](https://docs.redotengine.org/en/stable/classes/class_styleboxtexture.html#class-styleboxtexture).

Redot's small distribution size can make it a suitable alternative to frameworks like Electron or Qt.

# **Animation[](https://docs.redotengine.org/en/stable/about/list_of_features#animation)**

- Direct kinematics and inverse kinematics.
- Support for animating any property with customizable interpolation.
- Support for calling methods in animation tracks.
- Support for playing sounds in animation tracks.
- Support for Bézier curves in animation.

# **File formats[](https://docs.redotengine.org/en/stable/about/list_of_features#file-formats)**

- Scenes and resources can be saved in [text-based](https://docs.redotengine.org/en/stable/contributing/development/file_formats/tscn.html#doc-tscn-file-format) or binary formats.
    
    > Text-based formats are human-readable and more friendly to version control.Binary formats are faster to save/load for large scenes/resources.
    > 
- Read and write text or binary files using [FileAccess](https://docs.redotengine.org/en/stable/classes/class_fileaccess.html#class-fileaccess).
    
    > Can optionally be compressed or encrypted.
    > 
- Read and write [JSON](https://docs.redotengine.org/en/stable/classes/class_json.html#class-json) files.
- Read and write INI-style configuration files using [ConfigFile](https://docs.redotengine.org/en/stable/classes/class_configfile.html#class-configfile).
    
    > Can (de)serialize any Redot datatype, including Vector2/3, Color, ...
    > 
- Read XML files using [XMLParser](https://docs.redotengine.org/en/stable/classes/class_xmlparser.html#class-xmlparser).
- [Load and save images, audio/video, fonts and ZIP archives](https://docs.redotengine.org/en/stable/tutorials/io/runtime_file_loading_and_saving.html#doc-runtime-loading-and-saving) in an exported project without having to go through Redot's import system.
- Pack game data into a PCK file (custom format optimized for fast seeking), into a ZIP archive, or directly into the executable for single-file distribution.
- [Export additional PCK files](https://docs.redotengine.org/en/stable/tutorials/export/exporting_pcks.html#doc-exporting-pcks) that can be read by the engine to support mods and DLCs.

# **Miscellaneous[](https://docs.redotengine.org/en/stable/about/list_of_features#miscellaneous)**

- [Video playback](https://docs.redotengine.org/en/stable/tutorials/animation/playing_videos.html#doc-playing-videos) with built-in support for Ogg Theora.
- [Movie Maker mode](https://docs.redotengine.org/en/stable/tutorials/animation/creating_movies.html#doc-creating-movies) to record videos from a running project with synchronized audio and perfect frame pacing.
- [Low-level access to servers](https://docs.redotengine.org/en/stable/tutorials/performance/using_servers.html#doc-using-servers) which allows bypassing the scene tree's overhead when needed.
- [Command line interface](https://docs.redotengine.org/en/stable/tutorials/editor/command_line_tutorial.html#doc-command-line-tutorial) for automation.
    
    > Export and deploy projects using continuous integration platforms.Shell completion scripts are available for Bash, zsh and fish.Print colored text to standard output on all platforms using print_rich.
    > 
- Support for [C++ modules](https://docs.redotengine.org/en/stable/contributing/development/core_and_modules/custom_modules_in_cpp.html#doc-custom-modules-in-cpp) statically linked into the engine binary.
- Engine and editor written in C++17.
    
    > Can be compiled using GCC, Clang and MSVC. MinGW is also supported.Friendly towards packagers. In most cases, system libraries can be used instead of the ones provided by Redot. The build system doesn't download anything. Builds can be fully reproducible.
    > 
- Licensed under the permissive MIT license.
    
    > Open development process with contributions welcome.
    > 

Introduction
Architecture
Getting Started
Hello World
Bang
Components
GeneratesAttribute
IComponent
IMessage
IModifiableComponent
IParentRelativeComponent
ITransformComponent
KeepOnReplaceAttribute
RequiresAttribute
UniqueAttribute
Contexts
Context
ContextAccessorFilter
ContextAccessorKind
Observer
WatcherNotificationKind
Diagnostics
Assert
SmoothCounter
Entities
BangComponentTypes
Entity
Extensions
MurderComponentTypes
MurderEntityExtensions
MurderMessageTypes
Interactions
IInteraction
IInteractiveComponent
InteractiveComponent<T>
InteractorMessage
StateMachines
IStateMachineComponent
StateMachine
StateMachineComponent<T>
Wait
WaitKind
Systems
DoNotPauseAttribute
FilterAttribute
IExitSystem
IFixedUpdateSystem
IMessagerSystem
IncludeOnPauseAttribute
IReactiveSystem
IRenderSystem
IStartupSystem
ISystem
IUpdateSystem
MessagerAttribute
OnPauseAttribute
WatchAttribute
ComponentsLookup
MurderComponentsLookup
MurderTransformComponentsLookup
MurderWorldExtensions
SerializeAttribute
World
Murder
Assets
Graphics
FloorAsset
FontAsset
Kerning
ParticleSystemAsset
SpriteAsset
TilesetAsset
Localization
LanguageId
LanguageIdData
Languages
LocalizationAsset
LocalizedStringData
ResourceDataForAsset
Save
PackedSaveAssetsData
PackedSaveData
SaveDataInfo
SaveDataTracker
CharacterAsset
DynamicAsset
EditorAssets
Exploration
FeatureAsset
GameAsset
GameProfile
IPreview
IWorldAsset
LocalizedString
PrefabAsset
SaveData
SavedWorld
SmartFloatAsset
SmartIntAsset
SpeakerAsset
Theme
WorldAsset
WorldEventsAsset
Attributes
AngleAttribute
AtlasCoordinatesAttribute
DefaultAttribute
DefaultEditorSystemAttribute
DoNotPersistEntityOnSaveAttribute
DoNotPersistOnSaveAttribute
EditorFieldFlags
EditorFieldPropertiesAttribute
EditorLabelAttribute
EditorTupleTooltipAttribute
GameAssetDictionaryIdAttribute
GameAssetIdAttribute<T>
GameAssetIdAttribute
GameAssetIdInfo
HideInEditorAttribute
InstanceIdAttribute
IntrinsicAttribute
MultilineAttribute
NoLabelAttribute
OnlyPersistThisComponentForEntityOnSaveAttribute
PersistOnSaveAttribute
ShowInEditorAttribute
SimpleTextureAttribute
SliderAttribute
TileEditorAttribute
TooltipAttribute
TypeOfAttribute
Components
Agents
Cutscenes
Effects
Graphics
Serialization
Sound
Utilities
AdvancedCollisionComponent
AfterInteractRule
AgentComponent
AgentImpulseComponent
AgentSpeedMultiplierComponent
AgentSpriteComponent
AlphaComponent
AlphaSources
AnimationCompleteComponent
AnimationEventBroadcasterComponent
AnimationOverloadComponent
AnimationSpeedOverload
AreaInfo
AutomaticNextDialogueComponent
BounceAmountComponent
CameraFollowComponent
CameraStyle
CarveComponent
CellProperties
ChildTargetComponent
ChoiceComponent
ClippingStyle
ColliderComponent
CollisionCacheComponent
CreatedAtComponent
CustomCollisionMask
CustomDrawComponent
CustomTargetSpriteBatchComponent
DestroyAtTimeComponent
DestroyOnAnimationCompleteComponent
DestroyOnBlackboardConditionComponent
DestroyOnCollisionComponent
DisableAgentComponent
DisableParticleSystemComponent
DisableSceneTransitionEffectsComponent
DoNotPauseComponent
DoNotPersistEntityOnSaveComponent
DrawRectangleComponent
EntityTrackerComponent
EventListenerComponent
EventListenerEditorComponent
FacingComponent
FacingInfo
FadeScreenComponent
FadeScreenWithSolidColorComponent
FadeTransitionComponent
FadeType
FadeWhenInAreaComponent
FadeWhenInAreaStyle
FadeWhenInCutsceneComponent
FlashSpriteComponent
FreeMovementComponent
FreezeWorldComponent
FrictionComponent
GlobalShaderComponent
GuidId
GuidToIdTargetCollectionComponent
GuidToIdTargetComponent
HAAStarPathfindComponent
HasVisionComponent
HighlightOnChildrenComponent
HighlightSpriteComponent
IdTargetCollectionComponent
IdTargetComponent
IgnoreTriggersUntilComponent
IgnoreUntilComponent
IMurderTransformComponent
InCameraComponent
IndestructibleComponent
InsideMovementModAreaComponent
InstanceToEntityLookupComponent
InteractOn
InteractOnCollisionComponent
InteractOnRuleMatchCollectionComponent
InteractOnRuleMatchComponent
InteractOnStartComponent
IntRange
InvisibleComponent
LineComponent
MapComponent
MovementModAreaComponent
MoveToComponent
MoveToPerfectComponent
MoveToTargetComponent
MurderTransformExtensions
MusicComponent
MuteEventsComponent
NineSliceComponent
ParticleSystemComponent
ParticleSystemWorldTrackerComponent
PathfindComponent
PathfindGridComponent
PauseAnimationComponent
PersistPathfindComponent
PickEntityToAddOnStartComponent
PolygonSpriteComponent
PositionComponent
PrefabRefComponent
PushAwayComponent
QuadtreeComponent
QuestStage
QuestStageRuntime
QuestTrackerComponent
QuestTrackerRuntimeComponent
RandomizeSpriteComponent
RectPositionComponent
RemoveColliderWhenStoppedComponent
RemoveEntityOnRuleMatchAtLoadComponent
RemoveStyle
RequiresVisionComponent
RoomComponent
RotationComponent
RouteComponent
RuleWatcherComponent
SituationComponent
SoundComponent
SoundParameterComponent
SoundWatcherComponent
SpeakerComponent
SpriteClippingRectComponent
SpriteComponent
SpriteEventInfo
SpriteFacingComponent
SpriteOffsetComponent
SquishComponent
StateWatcherComponent
StaticComponent
StrafingComponent
TetheredComponent
TetherPoint
TextureComponent
TileGridComponent
TilesetComponent
TimeScaleComponent
TweenComponent
UnscaledDeltaTimeComponent
Vector2FromTo
VelocityComponent
VelocityTowardsFacingComponent
VerticalPositionComponent
WaitForVacancyComponent
WindowRefreshTrackerComponent
Core
Ai
Cutscenes
Dialogs
Geometry
Graphics
Input
MurderActions
Particles
Physics
Smart
Sounds
Ui
FloatRange
FrameInfo
GameScene
Grid
GridConfiguration
GridNumberExtensions
ITileProperties
Map
MapTile
Mask2D
MonoWorld
MurderTagsBase
Orientation
OrientationHelper
Portrait
RequirementsCollection
RoundingMode
Scene
SceneLoader
Tags
TileDimensions
TileGrid
TileKind
TriggerEventOn
Viewport
Data
AtlasId
BlackboardInfo
GameDataManager
PackedGameData
PackedSoundData
PreloadPackedGameData
Diagnostics
Command
CommandAttribute
CommandServices
GameLogger
GraphLogger
ICommands
LogLine
PerfTimeRecorder
SmoothFpsCounter
UpdateTimeTracker
Editor
Assets
Attributes
Helpers
Direction
DirectionHelper
Interaction
InteractWithDelayInteraction
Interactions
AddChildOnInteraction
AddChildProperties
AddComponentOnInteraction
AddEntityOnInteraction
AdvancedBlackboardInteraction
BlackboardAction
BlackboardActionInteraction
DebugInteraction
DestroyWho
EnableChildrenInteraction
InteractChildOnInteraction
InteractionCollection
InteractorComponent
PlayMusicInteraction
PlaySoundInteraction
RemoveEntityOnInteraction
SendInteractMessageInteraction
SendMessageInteraction
SendToOtherInteraction
SendToParentInteraction
SetPositionInteraction
SetSoundOnInteraction
StopMusicInteraction
TalkToInteraction
TargetedInteractionCollection
TargetedInteractionCollectionItem
Messages
Physics
AnimationCompleteMessage
CollidedWithMessage
FatalDamageMessage
HighlightMessage
InteractMessage
IsInsideOfMessage
NextDialogMessage
OnInteractExitMessage
PathNotPossibleMessage
PickChoiceMessage
ThetherSnapMessage
TouchedGroundMessage
Prefabs
EntityBuilder
EntityInstance
EntityModifier
IEntity
PrefabEntityInstance
PrefabReference
Save
BlackboardTracker
GamePreferences
Serialization
ComplexDictionary<TKey, TValue>
ComplexDictionaryConverter<T, V>
FileHelper
FileManager
IMurderSerializer
JsonTypeConverter
MurderSerializerOptionsExtensions
MurderSourceGenerationContext
Services
Info
ButtonStyle
CameraServices
ColliderServices
CoroutineServices
DebugServices
DialogueServices
DrawMenuStyle
EffectsServices
EntityServices
Feedback
FeedbackServices
FileWrapper
GeometryServices
LevelServices
LocalizationServices
MenuOption
MurderFonts
MurderFontServices
MurderSaveServices
MurderUiServices
NextAvailablePositionFlags
PhysicsServices
RaycastHit
RenderServices
SoundServices
TextureServices
WorldServices
StateMachines
Coroutine
DialogStateMachine
Systems
Agents
Effects
Graphics
Physics
Utilities
AgentMovementModifierSystem
AgentSpriteSystem
AnimationEventBroadcastSystem
AnimationOnPauseSystem
CalculatePathfindSystem
CameraShakeSystem
ConsoleSystem
DestroyAtTimeSystem
DynamicInCameraSystem
FadeScreenWithSolidColorSystem
FadeTransitionSystem
FloorWithBatchOptimizationRenderSystem
GridCacheRenderSystem
IgnoreUntilSystem
InteractOnCollisionSystem
InteractOnRuleMatchSystem
MapCarveCollisionSystem
MapInitializerSystem
MapPathfindInitializerSystem
MoveToPerfectSystem
ParticleAlphaTrackerSystem
ParticleDisableTrackerSystem
ParticleRendererSystem
ParticleTrackerSystem
PathfindRouteSystem
PolygonSpriteRenderSystem
QuadtreeCalculatorSystem
StateMachineOnPauseSystem
StateMachineSystem
TextureRenderSystem
TimeScaleSystem
TweenSystem
VelocityTowardsFacingSystem
Utilities
Attributes
AsepriteFileInfo
AssetRef<T>
BlackboardHelpers
CacheDictionary<TKey, TValue>
Calculator
CameraHelper
CollectionHelper
CollisionDirection
ColorHelper
Ease
EaseKind
GridHelper
Icons
InputHelpers
MatrixHelper
MurderAssetHelpers
NoiseHelper
NoiseType
PerlinNoise
PositionExtensions
RandomExtensions
SerializationHelper
ShaderHelper
StringHelper
TargetEntity
ThreeSlice
ThreeSliceInfo
Vector2Extensions
Vector2Helper
WorldHelper
XnaExtensions
Game
IMurderGame
IShaderProvider

**Defold**

Repository for the Defold engine, editor and command line tools.

**Supported by**

[](https://camo.githubusercontent.com/5dcb9dbfea17417c897e0e8c238172cf72a845e202c79c4a1b38380783d2e608/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f6c6f676f2f6f74686572732f6d656c736f66742d626c61636b2e706e67)

[](https://camo.githubusercontent.com/8511e0c13d2a0d3cecaf52f9c43217ba2c8579ede3ad837333f429b61dabf1a5/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f73706163657233322e706e67)

[](https://camo.githubusercontent.com/89a699aeac3afca9d5822818a41ecca067fe5c55e218612237e429be37f26e0d/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f6c6f676f2f6f74686572732f726976652d626c61636b2e706e67)

[](https://camo.githubusercontent.com/8511e0c13d2a0d3cecaf52f9c43217ba2c8579ede3ad837333f429b61dabf1a5/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f73706163657233322e706e67)

[](https://camo.githubusercontent.com/51df960abb1e918f2806f4b27d314fab05e5581fb68a1955cbbc2de598593027/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f6c6f676f2f6f74686572732f706f6b692d626c61636b2e706e67)

[](https://camo.githubusercontent.com/8511e0c13d2a0d3cecaf52f9c43217ba2c8579ede3ad837333f429b61dabf1a5/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f73706163657233322e706e67)

[](https://camo.githubusercontent.com/d078c51da13e118aed29fd73f9b00bca5ec3d7915daa9e1784845eb1d0785942/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f6c6f676f2f6f74686572732f6f702d67616d65732d636f6c6f722e706e67)

[](https://camo.githubusercontent.com/8511e0c13d2a0d3cecaf52f9c43217ba2c8579ede3ad837333f429b61dabf1a5/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f73706163657233322e706e67)

[](https://camo.githubusercontent.com/00053eda348bd7b9f533eb2a7f521b597b7ecfbacb5dd5221e028b29f1d75c1c/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f6c6f676f2f6f74686572732f6865726f69636c6162732d626c75652e706e67)

[](https://camo.githubusercontent.com/8511e0c13d2a0d3cecaf52f9c43217ba2c8579ede3ad837333f429b61dabf1a5/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f73706163657233322e706e67)

[](https://camo.githubusercontent.com/097d3b2bf0cd35e4e06f9f6292a63829fd096fa4c495b63855010cd766526a44/68747470733a2f2f6465666f6c642e636f6d2f696d616765732f6c6f676f2f6f74686572732f6b696e672d636f6c6f722e706e67)

**Folder Structure**

- **build_tools** - Build configuration and build tools used by build scripts
- **ci** - Continuous integration files for GitHub CI ([more info](https://github.com/defold/defold/blob/dev/README_CI.md))
- **com.dynamo.cr** - Bob
- **engine** - Engine
- **editor** - Editor
- **external** - External libraries that can be rebuilt using our build system
- **packages** - Prebuilt external packages
- **scripts** - Build and utility scripts
- **share** - Misc shared stuff used by other tools. Waf build-scripts, valgrind suppression files, etc.
- **share/ext** - External libraries that are built using custom build steps

**Setup and Build**

**Setup Engine**

Follow the [setup guide](https://github.com/defold/defold/blob/dev/README_SETUP.md) to install all of the tools needed to build the Defold engine.

**Build Engine**

Follow the [build instructions](https://github.com/defold/defold/blob/dev/README_BUILD.md) to build the engine and command line tools.

**Setup, Build and Run Editor**

Follow the [instructions](https://github.com/defold/defold/blob/dev/editor/README.md) in the editor folder.

**Engine Overview**

An overview of the engine architecture and additional engine information can be [viewed here](https://github.com/defold/defold/blob/dev/engine/docs/README.md).

**Platform Specifics**

- [iOS](https://github.com/defold/defold/blob/dev/README_IOS.md)
- [Android](https://github.com/defold/defold/blob/dev/README_ANDROID.md)
- [HTML5/Emscripten](https://github.com/defold/defold/blob/dev/README_EMSCRIPTEN.md)

**Releasing a new version**

The release process is documented [here](https://github.com/defold/defold/blob/dev/RELEASE.md).

**Complying with licenses**

A full list of third party software licenses along with information on how to give attribution and include the licenses in your game can be found in the [COMPLYING WITH LICENSES](https://github.com/defold/defold/blob/dev/COMPLYING_WITH_LICENSES.md) document in the Defold repository on GitHub.

Acid is an open-source, cross-platform game engine written in modern C++17 and structured to be fast, simple, and extremely modular.

Vulkan is the sole graphics API, Vulkan can be accessed in apps with the provided Acid rendering pipeline. Metal is supported through [MoltenVK](https://github.com/KhronosGroup/MoltenVK); eventually, DirectX will be supported in a similar way.

This project is being worked on part-time by a single developer, this is under heavy development, expect bugs, API changes, and plenty of missing features.

**Features**

- Multiplatform (Windows, Linux, MacOS, 32bit and 64bit)
- Multithreaded command buffers and thread safety
- On the fly GLSL to SPIR-V compilation and reflection
- Deferred physically based rendering (PBR)
- Networking (HTTP, FTP, UDP, TCP)
- Object serialization (JSON, XML)
- Resource management using serialization
- Event delegate callbacks with scoped functions
- Bullet physics
- Entity component system
- Particle effect systems
- File multi-path searching, and packaging
- UI constraints system, and MSDF font rendering
- Audio systems (flac, mp3, ogg, opus, wave)
- Shadow mapping
- Post effects pipeline (lensflare, glow, blur, SSAO, ...)
- Model file loading (obj, glTF 2.0)
- Animations loading (Collada)
- Image file loading (png, jpeg, dng, tiff, OpenEXR, bmp, dds, ppm, tga)

**Dependencies**

- [imgui](https://github.com/ocornut/imgui) : Dear ImGui: Bloat-free Immediate Mode Graphical User interface for C++ with minimal dependencies.
- [imguizmo](https://github.com/CedricGuillemet/ImGuizmo) : Immediate mode 3D gizmo for scene editing and other controls based on Dear Imgui.
- [entt](https://github.com/skypjack/entt) : Fast and reliable entity-component system (ECS)
- [glfw](https://github.com/glfw/glfw) : A multi-platform library for OpenGL, OpenGL ES, Vulkan, window and input.
- [stb](https://github.com/nothings/stb) : Single-file public domain (or MIT licensed) libraries for C/C++.
- [tinygltf](https://github.com/syoyo/tinygltf) : Header only C++11 tiny glTF 2.0 library
- [tinyobjloader](https://github.com/syoyo/tinyobjloader) : Tiny but powerful single file wavefront obj loader
- [volk](https://github.com/zeux/volk) : Meta loader for Vulkan API.
- [Box2D](https://github.com/erincatto/Box2D) : 2D physics engine.
- [sol2](https://github.com/ThePhD/sol2) : C++ <-> Lua API wrapper
- [cereal](https://github.com/USCiLab/cereal) : A C++11 library for serialization
- [meshoptimizer](https://github.com/zeux/meshoptimizer) : Mesh optimization library that makes meshes smaller and faster to render