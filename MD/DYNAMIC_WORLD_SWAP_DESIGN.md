# Dynamic World Environment Swapping Design ("Domain Expansion")

## Overview
This document outlines a high-performance system to dynamically swap the entire 3D scene's environment (visuals, physics, lighting) instantly—similar to a "Domain Expansion" in Anime or "Dimension Shift" in games like *Soul Reaver* or *Titanfall 2*.

**Goal:** Allow a Skill (e.g., "Lava World") to transform the entire map from "Green Forest" to "Hellscape" seamlessly.

---

## 1. Technical Approaches

There are three layers to this transformation, ordered by performance cost:

### 1.1 Level 1: Global Shader Parameters (Instant & Cheap)
Instead of swapping 10,000 textures, we use a **Global Material Parameter Collection (MPC)**.

*   **Implementation:**
    *   All PBR Shaders leverage a global Uniform Buffer: `WorldEnvironment { blend_factor, snow_amount, lava_amount }`.
    *   **Texture Blending:** In the pixel shader, we blend the base Albedo with a "World Texture" (e.g., Lava Noise) based on `lava_amount`.
    *   **Emission:** Ramping up `lava_amount` turns dark cracks into glowing magma.
    *   **Transition:** Lerp `lava_amount` from 0.0 to 1.0 over 2 seconds.
*   **Pros:** Zero CPU cost. Instant. No loading screens.
*   **Cons:** Shader complexity increases.

### 1.2 Level 2: Post-Processing & Lighting (Mood Change)
Changing the effective lighting without re-baking lightmaps.

*   **Implementation:**
    *   **LUT Swap:** Cross-fade between "Forest LUT" (Vibrant) and "Lava LUT" (Red/Contrast) in the Post-Process pass.
    *   **Fog:** Change Fog Color from Blue to Orange.
    *   **Skybox:** Cross-fade Skybox cubemaps.
*   **Result:** The entire color palette shifts instantly.

### 1.3 Level 3: Gameplay/Logic Swap (The "Real" Change)
Changing physics and game rules (e.g., Ground deals damage).

*   **ECS Tag Swapping:**
    *   We do NOT iterate 10,000 entities to add "DamageComponent".
    *   Instead, we use a global **Resource:** `CurrentWorldState`.
    *   **System Logic:**
        ```rust
        fn damage_on_ground_system(mut healths: Query<&mut Health>, world_state: Res<CurrentWorldState>) {
            if world_state.is_lava_active {
                // Apply damage to everyone touching 'Ground' tag
            }
        }
        ```
    *   **Physics:** If terrain geometry needs to change (e.g., lava rises), use a Vertex Displacement Shader + updating the Heightfield Collider (expensive, do sparingly).

---

## 2. Architecture Recommendation for XS Engine

Don't use "Load Level" for this. Loading levels is slow and disconnects state. Use a **State-Based Approach**.

### 2.1 The `WorldVariant` System

Create a new ECS Resource and System to manage these states.

```rust
#[derive(Resource)]
pub struct WorldEnvironmentSystem {
    pub current_variant: WorldVariant,
    pub transition_progress: f32, // 0.0 to 1.0
    pub target_variant: WorldVariant,
}

pub enum WorldVariant {
    Normal,
    Lava,
    Ice,
    Cyberpunk,
}
```

### 2.2 The "Uber-Shader" Strategy
Your Standard PBR Shader should support a "Biome Blend" feature.
*   **Input:** `Material.albedo` (Base).
*   **Input:** `World.SecondaryAlbedo` (Global).
*   **Logic:** `FinalColor = lerp(Material.albedo, World.SecondaryAlbedo, World.blend_factor * Material.influence_mask)`.
    *   *Note:* `Material.influence_mask` allows some objects (like metal crates) to RESIST the change, while rocks/grass accept it.

---

## 3. Technique Deep Dive: Sphere Mask Implementation (Unreal Style)

Implementing the "Sphere Mask" effect from Unreal in WGSL to create a localized or expanding transformation.

### 3.1 Global Params (Rust Side)
Update the `WorldUniform` buffer struct:
```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct WorldEffects {
    mask_center: [f32; 3], // The player's position (or skill center)
    mask_radius: f32,      // Expands from 0 to 1000
    mask_softness: f32,    // 0 to 1 for edge blur
    effect_strength: f32,  // 0 to 1 total opacity
    _padding: [f32; 2],
}
```

### 3.2 Shader Logic (WGSL)
In `pbr.wgsl`, inside the Fragment Shader:

```wgsl
struct WorldEffects {
    mask_center: vec3<f32>,
    mask_radius: f32,
    mask_softness: f32,
    effect_strength: f32,
}
@group(0) @binding(2) var<uniform> world_effects: WorldEffects;

fn get_sphere_mask(world_pos: vec3<f32>) -> f32 {
    let dist = distance(world_pos, world_effects.mask_center);
    // 1.0 inside sphere, 0.0 outside, with smooth falloff
    let mask = 1.0 - smoothstep(
        world_effects.mask_radius - world_effects.mask_softness, 
        world_effects.mask_radius, 
        dist
    );
    return clamp(mask * world_effects.effect_strength, 0.0, 1.0);
}

fn fragment_main(...) {
    // ... calculate base pbr ...
    let mask_alpha = get_sphere_mask(in.world_position);
    
    // Blend to "Lava" style
    let lava_color = vec3<f32>(1.0, 0.2, 0.0); // Or sample a Lava Texture
    let lava_emission = vec3<f32>(2.0, 0.5, 0.0);
    
    // Mix based on mask
    final_color = mix(final_color, lava_color, mask_alpha);
    final_emission = mix(final_emission, lava_emission, mask_alpha);
}
```

This effectively replicates the Unreal "SphereMask" node.

---

## 4. Advanced Gameplay: Vertex Animation & Physics Sync

To handle the "Growth Puzzle" scenario (Tree grows, Player jumps on it), visually enlarging the mesh in the shader is **not enough** because the Physics Collider won't update (Player falls through).

We must implement **"Synced Parametric Growth"**.

### 4.1 The Problem
*   **Vertex Shader (WPO):** Can scale vertices instantly on GPU.
*   **Physics Engine (Rapier):** Needs CPU update to `Collider` shape or `Transform.scale`.
*   **Mismatch:** If Shader scales up by 2x but Collider stays 1x, gameplay breaks.

### 4.2 The Solution: "Driven by ECS"
Instead of letting the shader run wild with `sin(time)`, the **ECS System** calculates the growth value and feeds it to **BOTH** the Shader (via Instance Data) and the Collider.

**ECS Data:**
```rust
#[derive(Component)]
struct GrowthObject {
    current_growth: f32, // 0.0 to 1.0
    max_scale: f32,      // e.g., 2.5
    growth_speed: f32,
}
```

**System Logic (CPU):**
```rust
fn update_growth_logic(
    time: Res<Time>,
    mut query: Query<(&mut GrowthObject, &mut Transform, &mut Collider)>,
    world_effects: Res<WorldEffects> // Checks if we are in "Green" world
) {
    for (growth, transform, collider) in query.iter_mut() {
        // 1. Calculate new growth based on World State
        let target = if world_effects.is_green_world { 1.0 } else { 0.0 };
        growth.current_growth = lerp(growth.current_growth, target, time.delta_seconds() * growth.growth_speed);
        
        // 2. Update Visual Transform (Passed to Shader as Model Matrix)
        let scale = 1.0 + (growth.max_scale - 1.0) * growth.current_growth;
        transform.scale = Vec3::splat(scale);
        
        // 3. Update Physics Collider (Crucial for Puzzle!)
        // Rapier handles scale updates automatically if we change the Transform
        // BUT if using custom shapes, might need collider.set_scale(scale);
    }
}
```

**Shader Logic (WGSL):**
*   The Shader actually does **NOT** need to do complex WPO (World Position Offset) if we just scale the whole object via `Transform`. This is cleaner.
*   *However*, if you want "Wobbly Growth" (vertex displacement), pass `growth.current_growth` as a custom instance attribute to the shader.
    ```wgsl
    // Vertex Shader
    let wobble = sin(in.position.y * 5.0 + time) * 0.1 * instance.growth_factor;
    out.position += normal * wobble; 
    ```

### 4.3 Conclusion for Puzzle Mechanic
*   **DO NOT** write complex growth logic purely in Shader (GLSL/WGSL) if physics interaction is required.
*   **DO** calculate the "Scale Factor" in Rust/ECS.
*   **DO** apply that Scale Factor to the Entity's `Transform`.
*   **DO** let the Renderer pass that Transform to the GPU, and let the Physics engine use it for Collision.
*   This ensures "What you see is what you stand on".
