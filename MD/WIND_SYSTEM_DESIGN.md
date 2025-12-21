# specialized Wind System Design ("God of War" Style)

## Overview
This document outlines the design for a **High-Fidelity 3D Wind System** based on the *God of War (GDC 2019)* technique.
The system simulates air fluid dynamics (Advection/Diffusion) in a local 3D Volume around the player, allowing for realistic interactions (e.g., Kratos throwing an axe and the grass parting, or a spinning attack creating a vortex).

---

## 1. Feasibility Study & Architecture

### 1.1 Can we do this in our Rust Engine?
**YES.**
The technique relies on **Compute Shaders** and **3D Textures** (Volume Textures).
*   **WGPU:** Fully supports `texture_3d` and `storage_texture` (Read/Write).
*   **Math:** The Advection/Diffusion algorithms are standard Fluid Dynamics (Navier-Stokes simplified), which work perfectly in WGSL.

### 1.2 "Should we separate into a specific module?"
**YES (Highly Recommended).**
Physics/Fluid simulations are complex to debug.
*   **Module Name:** `crates/wind_sim` or `engine/src/physics/wind`.
*   **Testing Strategy:** 
    *   **Unit Test:** Create a "Headless" test that instantiates the `WindWorld`, runs 60 ticks of simulation, and inspects the 3D Texture data (downloading from GPU) to verify that a "Motor" actually pushes values.
    *   This is much harder to debug if mixed directly with the Rendering logic.

---

## 2. Core Implementation (God of War Approach)

### 2.1 The Data Structure (Double Buffering)
We need "Ping-Pong" buffers because we cannot read and write the same texture safely in parallel.
*   `WindVolume A` (Read)
*   `WindVolume B` (Write)
*   Format: `Rgba16Float` (High precision) or `R8G8B8A8Snorm` (Bandwidth efficient).

### 2.2 The Compute Pipeline (WGSL)
Executed every frame in this order:

1.  **Offset (Scroll):**
    *   When the player moves, we "shift" the 3D volume content so the wind stays relative to the world, not the camera.
2.  **Inject Motors:**
    *   Rasterize `WindMotor` components into the velocity grid.
    *   *Types:* `Directional`, `Omni` (Explosion), `Vortex` (Tornado).
3.  **Diffusion:**
    *   Blur the velocity to simulate air viscosity.
4.  **Advection:**
    *   Move velocity along itself (Self-transport). This creates the "swirly" fluid motion.
5.  **Output:**
    *   Generate a final Texture for the Grass/Particle shaders to sample.

### 2.3 ECS Integration (Wind Motors)

Instead of hardcoding, we use ECS Components.

```rust
#[derive(Component)]
pub struct WindMotor {
    pub motor_type: MotorType, // Directional, Omni, Vortex
    pub radius: f32,
    pub force: f32,
    pub decay: f32,
}

#[derive(Component)]
pub struct LocalWindVolume {
    // The player has this component. 
    // The system centers the simulation around this entity.
    pub resolution: UVec3, // e.g. 32x16x32
    pub world_size: Vec3,  // e.g. 20m x 10m x 20m
}
```

---

## 3. Shader Implementation Plan (WGSL)

We need to port the Unity HLSL code to WGSL.

**Example: Advection (WGSL)**
```wgsl
@group(0) @binding(0) var velocity_read: texture_3d<f32>;
@group(0) @binding(1) var velocity_write: texture_storage_3d<rgba16float, write>;
@group(0) @binding(2) var<uniform> dt: f32;

@compute @workgroup_size(8, 8, 8)
fn advect(@builtin(global_invocation_id) id: vec3<u32>) {
    let pos = vec3<f32>(id);
    // 1. Sample current velocity
    let vel = textureLoad(velocity_read, id, 0).xyz;
    
    // 2. Back-trace where this air came from
    let prev_pos = pos - vel * dt;
    
    // 3. Sample velocity at that previous position
    let new_vel = textureSampleLevel(velocity_read, sampler_linear, prev_pos / volume_size, 0).xyz;
    
    // 4. Write
    textureStore(velocity_write, id, vec4<f32>(new_vel, 0.0));
}
```

---

## 4. Integration with other Systems

1.  **Grass System (MD/TERRAIN_SYSTEM_DESIGN.md):**
    *   Grass Shader adds: `var wind = textureSample(WindVolume, world_pos);`
    *   This makes grass bend realistically to explosions/melee attacks.
2.  **Particle System (Niagara-Lite):**
    *   Particles read the Wind Volume to flow with the air.

## 5. Roadmap

1.  Create `wind_sim` module.
2.  Implement `WindVolume` struct (manages WGPU Textures).
3.  Write `wind_advect.wgsl` and `wind_motor.wgsl`.
4.  Create a "Debug Box" using Gizmos to visualize arrows for wind direction.
5.  Connect to Grass Shader.
