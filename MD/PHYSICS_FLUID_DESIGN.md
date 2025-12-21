# Mobile Fluid Physics Design (SPH/PBF)

## Overview
This document outlines the design for a real-time **fluid simulation (Water/Lava)** optimized for **Mobile Devices** in a Rust/WGPU engine.
Simulating fluids is expensive ($O(N^2)$). To achieve "AAA" results on mobile, we must move the heavy lifting to the **GPU (Compute Shaders)** and use smart rendering tricks.

---

## 1. Core Architecture: GPU-Accelerated PBF
We recommend **Position Based Fluids (PBF)** over traditional SPH. PBF is more stable, allowing larger time steps (better FPS) and is easier to constrain (less "exploded" particles).

### 1.1 The Loop (Compute Shaders)
The entire simulation runs in a sequence of WGPU Compute Passes every frame:

1.  **Predict Position:** Apply gravity + velocity.
2.  **Spatial Hash (The Secret Sauce):**
    *   To find neighbors fast, we sort particles into a Grid.
    *   *Kernel 1:* Calculate Grid Index for each particle `(CellID = Pos / CellSize)`.
    *   *Kernel 2:* Sort particles by CellID (Bitonic Sort or Radix Sort on GPU).
    *   *Result:* Neighbors are now contiguous in memory. Neighbor search becomes $O(N)$ instead of $O(N^2)$.
3.  **Constraint Solve (Density):**
    *   Calculate density for each particle based on neighbors.
    *   Move particles apart to satisfy "Rest Density" (incompressibility).
4.  **Update Velocity:** Update real velocity based on position change.
5.  **Integration:** Commit new positions.

---

## 2. Mobile Optimizations (Critical)

### 2.1 Fixed Particle Budget
*   **Target:** 4,000 - 10,000 particles on High-End Mobile. 2,000 on Low-End.
*   **Static Buffers:** Pre-allocate all WGPU Buffers. No `malloc` during gameplay.

### 2.2 Shared Memory Optimization
*   In the Compute Shader, load neighbor data into `workgroup` (LDS) memory. This drastically reduces VRAM bandwidth usage, which is the #1 bottleneck on mobile.

### 2.3 One-Way Coupling (Solid Interaction)
Full two-way coupling (Water pushes Box) is hard.
*   **Mobile Trick:** Use **SDF (Signed Distance Fields)** or **Depth Buffer collisions**.
    *   Render the scene's depth map (Low Res).
    *   Bind Depth Map to Fluid Compute Shader.
    *   If `Particle.z > DepthMap(Particle.xy)`, the particle is inside a solid. Push it out.
    *   *Result:* Water flows around rocks cheaply. Rocks don't float (acceptable tradeoff).

---

## 3. Rendering Strategy: Screen Space Fluids
Don't use Marching Cubes (generating mesh is too heavy for mobile). Use **Screen Space Filtering**.

1.  **Draw Particles:** Draw every particle as a simple 2D Sphere Sprite (Billboards) into a format.
    *   Output: `Depth` and `Thickness` (how much water you look through).
2.  **Depth Smooth:** Blur the depth buffer using a "Bilateral Filter" to merge the spheres into a smooth surface.
3.  **Shading (Post-Process):**
    *   Reconstruct Normals from the smoothed depth.
    *   Calculate Reflection/Refraction.
    *   Apply Beer's Law (Absorption) based on Thickness.

---

## 4. Integration with Rust Engine

### 4.1 ECS Component
```rust
#[derive(Component)]
struct FluidEmitter {
    pub particle_count: u32,
    pub fluid_type: FluidType (Water, Lava, Slime),
    pub emission_rate: f32,
}
```

### 4.2 System `FluidSystem`
*   Responsible for dispatching the generated `wgpu::ComputePipeline`.
*   Does **NOT** read particle data back to CPU (too slow).
*   If gameplay needs to know "Is Player in Water?", use a simple `AABB` check for the fluid volume, not per-particle.

### 4.3 ECS Integration Strategy: The Hybrid Model

**Question:** "Does this work well with ECS?"
**Answer:** Yes, but ONLY if you use a **Hybrid Model**.

*   **The Trap (Don't do this):** Creating an ECS `Entity` for every single water particle.
    *   *Why:* Overhead. ECS is fast, but maintaining 10,000 entities with Transform/Velocity components is too slow for mobile, and transferring that data to GPU every frame is bandwidth suicide.
*   **The Solution (Hybrid):**
    *   **Emitter is an Entity:** The "Source" of the water (e.g., a broken pipe) is a normal ECS Entity with `Transform` and `FluidEmitter` components.
    *   **Particles are GPU Data:** The system reads the *Emitter's* position and updates a **Single Compute Buffer** on the GPU. The particles themselves do not exist in the ECS World.
    *   **Interaction:** If you need Gameplay Interaction (e.g., Fireball evaporates water), you use a "Influence Volume" (Sphere/Box). The ECS sends this Volume to the GPU, and the Compute Shader kills particles inside it.

## 5. Roadmap
1.  **Basic Compute:** Implement "Integartion Pass" (Gravity). Move dots on screen.
2.  **Spatial Hashing:** Implement Grid Sort.
3.  **PBF Solver:** Implement Density Constraints.
4.  **Renderer:** Implement Bilateral Blur and Normal reconstruction.
