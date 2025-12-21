# Minecraft-Style Voxel System Design (Budget Multiplayer)

## Overview
This document outlines the design for a **Voxel Sandbox System** integrated into the XS Engine.
The goal is to replicate *Minecraft*'s core loop (Break/Place Blocks, Infinite World) with a specific focus on **"Most Economical Multiplayer"** (Zero Server Cost).

---

## 1. The "Budget" Multiplayer Architecture
**Goal:** 0$ Server Cost.

### 1.1 The "Listen Server" Model (Host-Client)
Instead of renting a Dedicated Server (AWS/DigitalOcean), we use the **Host-Client** model.
*   **Player A (Host):**
    *   Runs the Game Logic + Graphics.
    *   Acts as the "Server" for Player B, C, D.
    *   Saves the World data to their local disk.
*   **Player B/C/D (Clients):**
    *   Connect directly to Player A via IP.
    *   Send Input -> Receive Block Updates.

### 1.2 "Hole Punching" (No Port Forwarding)
To make this user-friendly (like Minecraft Bedrock friend join) without renting logic:
*   Use a **Free STUN/TURN** list or a very cheap "Signaling Server" (can run on f1-micro free tier) just to help players find each other.
*   Once connected, traffic is **P2P (Peer-to-Peer)** directly between Host and Client.

---

## 2. Voxel Engine Architecture

### 2.1 Data Structure: The Chunk
*   **Size:** 32x32x32 blocks (Standard is 16x256x16, but cubic chunks are better for infinite height).
*   **Storage:** `Flattened Array` (Fastest CPU cache) or `Palette Compression`.
    *   *Palette:* If a chunk is only Dirt and Air, we map `0=Air`, `1=Dirt` and use only 1 bit per block. Massive RAM saving.

### 2.2 Meshing Strategy (Optimization)
Generating meshes from blocks is CPU intensive.
1.  **Face Culling:** If Block A is next to Block B (and both are solid), don't draw the face between them.
2.  **Greedy Meshing:** Combine adjacent faces of the same type into one large Quad.
    *   *Result:* Reduces triangle count by 10x-50x. Critical for Mobile/Budget devices.
3.  **Threaded Generation:**
    *   Chunk Mesh Gen runs on a `TaskPool` background thread.
    *   Main Thread simply uploads the `VertexBuffer` to GPU.

---

## 3. Network Protocol (Bandwidth Saving)

Minecraft consumes bandwidth sending chunks. We optimize this.

### 3.1 Initial Join (Heavy)
*   Host sends existing chunks to Client.
*   **Compression:** Apply **LZ4** or **Zstd** compression to the Chunk Data before sending. Voxel data compresses *extremely* well (90%+ ratio).

### 3.2 Gameplay Sync (Light)
*   **Event-Based:** When Host breaks a block, do NOT resend the whole chunk.
*   **Packet:** `[BlockUpdate: (10, 5, 10) = Air]` (Approx 12 bytes).
*   **Entity Interpolation:**
    *   Sync Player Positions every 100ms.
    *   Clients interpolate between updates for smoothness.

---

## 4. Implementation Steps (Rust)

### Phase 1: The Core (Singleplayer)
1.  **`VoxelChunk` Struct:** Array `[BlockID; 32*32*32]`.
2.  **`MeshGen` System:** Iterate array -> Emit Vertices (Vertices/UVs).
3.  **`VoxelWorld` Resource:** HashMap `(IVec3, Chunk)`. Infinite scrolling logic.

### Phase 2: The Network (Multiplayer)
1.  **Transport Layer:** Use `Renet` or `Quinnet` (Rust UDP Libraries).
2.  **Host Logic:**
    *   `if is_host { server.broadcast(BlockEvent) }`.
3.  **Client Logic:**
    *   `client.on_receive(|packet| world.set_block(packet.pos, packet.id))`.

---

## 5. ECS Integration

```rust
#[derive(Component)]
struct ChunkMesh {
    pub chunk_coord: IVec3,
    pub is_dirty: bool, // Needs remeshing?
}

// System: Only mesh chunks that changed
fn voxel_mesher_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChunkMesh)>,
    world_data: Res<VoxelWorld>
) {
    query.par_iter_mut().for_each(|(entity, mesh)| {
        if mesh.is_dirty {
            let vertices = generate_greedy_mesh(mesh.chunk_coord, &world_data);
            // Send to Render Thread...
        }
    });
}
```

---

## 6. Comparison

| Feature | Standard Minecraft Server | **XS Engine Budget Design** |
| :--- | :--- | :--- |
| **Cost** | $5-$20 / month (Realms/VPS) | **$0 / month** (Host runs logic) |
| **Performance** | Java (Heavy RAM) | **Rust** (Native Speed) |
| **Mesh** | Simple Culling | **Greedy Meshing** (Mobile Friendly) |
| **Limit** | 256 Height | **Infinite Height** (Cubic Chunks) |

---

## 7. Scripting Architecture (Lua Integration)

**Question:** "Should we draw from Lua script or use a Rust module?"
**Answer:** **Use a Rust Module.** Lua is too slow for pixel-level loops.

### 7.1 Performance Gap
*   **Lua Logic:** "Place a Tree at (10, 10)" -> **Fast enough**.
*   **Lua Meshing:** "Loop 32x32x32 (32,000 blocks), check 6 faces each, merge quads" -> **Extremely Slow**. Lua cannot handle this tight loop in real-time without stuttering.

### 7.2 Communication API
We expose a **High-Level API** for Lua to control the Voxel World without touching vertices.

**Lua Script (`world_gen.lua`):**
```lua
-- Lua only manages logic
function on_player_click(x, y, z)
    -- 1. Logical Change
    VoxelSystem:set_block(x, y, z, BlockIDs.Dirt)
    
    -- 2. Audio/VFX (Handled by other systems)
    Audio:play("dig_sound")
    VFX:spawn("dirt_particles", x, y, z)
end

function generate_tree(x, y, z)
    -- Lua loops 5-6 times (Fast)
    for i = 0, 4 do
        VoxelSystem:set_block(x, y+i, z, BlockIDs.Log)
    end
end
```

**Rust Module (`voxel_module`):**
1.  Receives `set_block` command.
2.  Updates `[BlockID]` array (Nanoseconds).
3.  Marks Chunk as `Dirty`.
4.  **Background Thread** wakes up -> Runs **Greedy Meshing (Rust)** -> Generates Vertices -> Uploads to GPU.
5.  All this is hidden from Lua.

### 7.3 Conclusion for Scripting
*   **Keep Logic in Lua:** (Game Rules, Biome Generation settings, Item interactions).
*   **Keep Math in Rust:** (Meshing, Raycasting, Physics Collisions).
