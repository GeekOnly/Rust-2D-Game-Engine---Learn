# Multiplayer Architecture Roadmap (Roblox-Style)

This document outlines the architectural plan to evolve the Game Engine from a single-player architecture to a robust, server-authoritative multiplayer system similar to Roblox.

## Core Philosophy
- **Server Authority**: The server holds the "true" state of the game. Clients only predict and render.
- **Service-Based Hierarchy**: The world is divided into strict containers (Workspace, ServerStorage, etc.) defining visibility and execution context.
- **Replication Automagic**: Common tasks like moving objects should sync automatically without manual packet handling.
- **Hybrid Scripting**: Scripts declare their context (Server vs Client) explicitly.

---

## Phase 1: Hierarchy & Data Structure Refactor
*Objective: Restructure the World to support authoritative domains and strict visibility scopes.*

### 1.1 `SystemContainer` Component
Create a special marker component to identify the standard service containers.
```rust
// ecs/src/components/network.rs
pub enum ContainerType {
    Workspace,        // Replicated physics world (Shared)
    ServerStorage,    // Server-only assets/logic (Server Only)
    ReplicatedStorage,// Shared assets/data (Shared, Server Write/Client Read)
    StarterPlayer,    // Configs copied to new players
    Lighting,         // World rendering settings
    ServerScriptService, // Server-side logic entries
}

pub struct SystemContainer {
    pub container_type: ContainerType,
}
```

### 1.2 World Initialization
Modify `engine/src/lib.rs` (or `App::new`) to guarantee these entities exist on startup.
- These entities are **Immutable Roots**; they cannot be deleted.
- All dynamic entities must be children of `Workspace` to be rendered/simulated physically.

### 1.3 Editor Hierarchy Update
Modify `editor/src/ui/panels/hierarchy.rs`:
- Stop listing raw entities.
- Group entities under their specific `SystemContainer`.
- Visually distinguish containers (e.g., Lock icon for `ServerStorage` when debugging as Client).

---

## Phase 2: Networking Foundation
*Objective: Establish a stable UDP connection and loop structure.*

### 2.1 Dependencies
Add to `Cargo.toml`:
- `renet`: For reliable/unreliable UDP channels.
- `bincode`: For efficient serialization.
- `serde`: Ensure all replicated components are serializable.

### 2.2 Game Loop Splitting
Refactor `engine/src/runtime` to support two distinct modes:
- **ServerHost Mode**:
  - Runs Physics (Rapier)
  - Runs `ServerScriptService` scripts
  - Sends World Snapshots
- **Client Mode**:
  - Runs Rendering (WGPU)
  - Runs `StarterPlayer` (Local) scripts
  - Interpolates World Snapshots
  - Sends Inputs

### 2.3 Connection Handshake
- Implement a basic lobby/connection screen.
- Verify protocol versioning between Client and Server.

---

## Phase 3: Replication System
*Objective: Sync game state automatically.*

### 3.1 Network Components
```rust
pub struct NetworkIdentity {
    pub id: u64,           // Unique Network ID (separate from Entity ID)
    pub owner: Option<u64> // Client ID who has authority (for client-side prediction)
}

pub struct NetworkTransform {
    pub position: Vec3,
    pub rotation: Quat,
    // Compression logic (e.g., Half-Float)
}
```

### 3.2 Replication Manager
A new ECS System running on Server:
1. Iterate all entities in `Workspace` with `NetworkIdentity`.
2. check dirty flags (has it moved?).
3. Bundle changes into a Replication Packet.
4. Broadcast to relevant clients (Scope checking can be added later).

### 3.3 Snapshot Interpolation (Client)
- Client maintains a "State Buffer" (Past, Present, Future).
- Smoothly interpolate visuals between snapshots to mask network latency (Lag Compensation).

---

## Phase 3.5: "Better Than Roblox" Optimizations
*Objective: Achieve AAA-grade performance and cost-efficiency using Rust's low-level capabilities.*

### 3.5.1 Interest Management (Spatial Hashing)
**Problem**: Roblox often broadcasts too much data or streams chunks too slowly.
**Solution**:
- Implement a **Grid-based Culling System**.
- Server only sends Entity updates to clients within the same or adjacent grid cells.
- **Benefit**: Massive bandwidth saving. A player in Zone A doesn't need data from Zone Z.

### 3.5.2 Delta Compression & Quantization
**Problem**: Sending full 32-bit floats for every coordinate is wasteful.
**Solution**:
- **Bit-packing**: Compress rotation (Quat) from 128 bits to 29-32 bits (Smallest Three method).
- **Quantization**: Map world positions to 16-bit integers for network transmission (lossy but precise enough).
- **Delta Encoding**: Only send the diff from the last acknowledged snapshot.

### 3.5.3 ECS-Native Burst Replication
**Problem**: Object-based reflection (like Roblox) is CPU heavy.
**Solution**:
- Serialize specific **Component Arrays** (SoA) directly from memory.
- Avoid overhead of checking individual objects; batch process all `NetworkTransform` components in one go.
- **Benefit**: Server can handle thousands of moving entities with minimal CPU usage.

---

## Phase 4: Scripting Evolution
*Objective: Enable gameplay logic to interact with the network transparently.*

### 4.1 Script Types
Add metadata to `Script` component or file extension convention:
- `.server.lua` -> Runs in `ServerScriptService` or `Workspace` (Server Side).
- `.client.lua` -> Runs in `StarterPlayer` or Character (Client Side).

### 4.2 Remote Events (The Bridge)
Implement a binding in `ScriptEngine` to allow cross-boundary calls.
- **Lua API**:
  ```lua
  -- Server
  local remote = Instance.new("RemoteEvent")
  remote.OnServerEvent:Connect(function(player, data) ... end)
  
  -- Client
  remote:FireServer({ x = 10, y = 20 })
  ```

### 4.3 Network Variables (SyncVars)
Allow scripts to set variables that sync automatically.
- `entity.Network.HP = 50` (Server sets, Clients see update event).

---

## Phase 5: Editor Simulation tools
*Objective: Test multiplayer without leaving the editor.*

### 5.1 Play Mode Options
- **Play Solo**: Current behavior (Standalone).
- **Start Server + Client**: Spawns a headless server thread and a client window.
- **Start Server + 2 Clients**: Stress test replication and interaction.

### 5.2 Packet Simulator
- Add an Editor Debug panel to simulate Latency (Ping) and Packet Loss to ensure gameplay code is robust.
