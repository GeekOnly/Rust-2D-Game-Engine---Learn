# UGC & Multiplayer System Design (Roblox-like Feature)

## 1. Overview
This document outlines the architecture for a "Create, Share, Play" system similar to Roblox, enabling players to create multiplayer maps with custom logic and share them with others.

## 2. Core Architecture

### 2.1 The "Place" Format (UGC Package)
To share a game, we define a standard package format, e.g., `.game` or `.xsg_pkg`.
This package is a compressed archive containing:
- **Scene Data** (`scene.json`): The ECS snapshot of the world (entities, components).
- **Scripts** (`*.lua`): The logic files referenced by entities.
- **Assets** (`*.png`, `*.gltf`): Custom resources.

### 2.2 Networking Topology
To support multiplayer logic, we need a **Client-Server** model.
- **Host (Server)**: One player (or a dedicated server) runs the simulation. It owns the "Truth".
- **Client**: Connects to Host, sends Input, receives World State.

### 2.3 Scripting Sandbox (Lua Security)
Since generic players will write scripts, we must ensure safety:
- **Restriction**: Lua scripts cannot access OS files, network sockets, or unsafe system calls.
- **API**: Only expose the `Engine` API (move, spawn, UI).
- **Networking API**: Expose functions to sync data.

---

## 3. Implementation Plan

### Phase 1: Networking Layer (The "Plumbing")
We need a transport layer to send packets between players.
*Recommended Library*: `renet` (UDP/Reliable/Unreliable channels) or `matchbox` (WebRTC for browser support).

**New Components**:
```rust
struct NetworkIdentity {
    pub net_id: u64, // Unique ID across network (stable unlike ECS Entity ID)
    pub owner_id: u64, // Who controls this object? (Server or specific Player)
}

struct NetworkTransform {
    pub interp_target: Vec3, // For client-side smoothing
}
```

### Phase 2: State Replication (The "Space")
The Host needs to serialize the world state and send it to clients.
1.  **Snapshotting**: Every N milliseconds, Serialize all entities with `NetworkIdentity`.
2.  **Delta Compression**: Only send what changed to save bandwidth.
3.  **Client Application**:
    *   If `NetworkIdentity` exists locally → Update position/state.
    *   If missing → Spawn new entity prefabs.

### Phase 3: Script Networking (The "Logic")
Roblox allows scripts to talk across the network (`RemoteEvent`, `RemoteFunction`). We need equivalent Lua bindings.

**Lua API Extensions**:
```lua
-- Host Code
Network.OnClientEvent("FireGun", function(player, dir)
    -- Verify logic (Anti-cheat)
    local bullet = World.Spawn("Bullet")
    bullet.SetVelocity(dir * 100)
    Network.Broadcast("PlaySound", "shoot.wav")
end)

-- Client Code
if Input.IsKeyPressed("Fire") then
    Network.FireServer("FireGun", camera.Forward)
end
```

### Phase 4: UGC Web Platform (The "Store")
A simple web backend (API) is needed to:
1.  **Upload**: Authenticated users upload their `.game` packages.
2.  **Listing**: Browser for "Popular", "New", "Featured".
3.  **Download**: The Game Engine downloads the package to a temp folder, loads `scene.json`, and starts the `NetworkClient`.

---

## 4. Current Engine Check
| Feature | Status | Action Needed |
| :--- | :--- | :--- |
| **ECS** | ✅ Ready | Add `NetworkIdentity` component. |
| **Scripting** | ✅ Ready (`mlua`) | Add `Network` namespace bindings. |
| **Asset Loading** | ✅ Ready | Add "Package" loader (Zip/Tar support). |
| **Networking** | ❌ Missing | **Integrate `renet` or `matchbox`.** |
| **UI** | ✅ Ready | Build a "Server Browser" UI. |

## 5. Feasibility Conclusion
**Yes, it is practically possible.**
The limitation is **Security**. Running untrusted code is dangerous.
*   **Solution**: Strict Lua sandboxing (block `io`, `os` libraries). `mlua` allows this by default if we don't inject them.

## 6. Example Workflow
1.  **Creator**: Opens Editor -> Places blocks -> Writes Lua `OnTriggerEnter { KillPlayer() }` -> Clicks "Publish".
2.  **Server**: Receives `.game` file -> Stores in S3/Database.
3.  **Player**: Opens Game -> Clicks "Play" -> Engine downloads `.game` -> Starts Client -> Connects to Host.
