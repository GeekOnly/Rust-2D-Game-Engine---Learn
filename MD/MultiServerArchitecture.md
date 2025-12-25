# Multi-Instance Game Server Architecture

To run multiple Game Server instances on a single physical machine (or VPS) efficiently, the Game Engine needs specific architectural changes. This ensures valid resource isolation, dynamic configuration, and "Headless" operation.

## Phase 1: Headless Mode (No-GPU Support)
*Objective: Run the engine without a window, graphics context, or audio device.*

### 1.1 `RunMode` Enumeration
Modify `engine/src/lib.rs` (App Builder) to accept a run mode.
```rust
pub enum RunMode {
    Client,       // Normal Window + GPU + Audio
    Server,       // Headless Loop + Networking + Physics only
    Editor,       // Editor UI + Tooling
}
```

### 1.2 The Server Loop
The standard `winit` event loop causes panic on headless Linux servers (no Display).
- **Implementation**: Create a conditional entry point.
- If `RunMode::Server`:
  - Skip `winit::event_loop`.
  - Use a simple `std::thread::sleep` loop (Fixed Time Step, e.g., 60Hz or 30Hz).
  - Disable all WGPU initialization.

---

## Phase 2: Dynamic Configuration (CLI & Env Vars)
*Objective: Tell "Instance A" to listen on Port 7000 and "Instance B" on Port 7001.*

### 2.1 Argument Parsing
Integrate `clap` crate to parse startup flags.
```bash
./game_server --port 7777 --world "world_1" --max-players 50
```

### 2.2 Docker-Friendly Environment Variables
Container orchestration relies on Env Vars. The engine should prioritize these:
- `SERVER_PORT`: Overrides default port.
- `SERVER_ID`: Unique ID for logging.
- `ASSET_PATH`: Absolute path to assets (Docker volumes).

### 2.3 Logic Implementation
```rust
// pseudo-code
let port = std::env::var("SERVER_PORT").unwrap_or("7777");
network_transport.bind(format!("0.0.0.0:{}", port));
```

---

## Phase 3: Resource Management & Isolation
*Objective: Prevent memory balloons when running 10+ instances.*

### 3.1 Shared Asset Loading (Advanced)
If running 10 instances of the same game, loading 10 copies of "map.json" into RAM is wasteful.
- **OS File Cache**: relies on Linux usually doing a good job.
- **Memory Mapping (mmap)**: For very large immutable assets (big terrain data), use `mmap` so the OS shares physical RAM pages across processes.

### 3.2 CPU Affinity (Optional)
Allows assigning specific cores to specific server instances to prevent one laggy server from killing others.

---

## Phase 4: Observability & Health Checks
*Objective: Let the Admin Panel know if a server crashes/freezes.*

### 4.1 Structured Logging
Log messages must include the Instance ID to separate them in centralized logs (Portainer/Grafana).
```text
[Instance-01] [INFO] Player joined.
[Instance-02] [INFO] Match started.
```

### 4.2 Health Check Endpoint
Expose a tiny HTTP endpoint (or file touch) for Docker to "Ping".
- If the Game Loop thread freezes (infinite loop), this endpoint stops responding.
- Docker detects this and restarts the container automatically.

---

## Example `main.rs` Adaptation

```rust
fn main() {
    let args = parse_args();

    if args.mode == RunMode::Server {
        log::info!("🚀 Starting Dedicated Server on Port {}", args.port);
        
        // Initialize ECS World (Physics only, No Rendering)
        let mut world = World::new();
        
        // Load Scene
        scene_loader::load_headless(&mut world, args.scene_path);

        // Server Loop
        let tick_rate = Duration::from_millis(16); // 60 TPS
        loop {
            let start = Instant::now();
            
            // 1. Network Pump (Receive Packets)
            network_manager.update(&mut world);
            
            // 2. Physics Step
            physics_system.step(&mut world);
            
            // 3. Game Logic
            script_system.update(&mut world);
            
            // 4. Replication (Send Packets)
            replication_manager.broadcast(&mut world);

            // Sleep remainder
            let elapsed = start.elapsed();
            if elapsed < tick_rate {
                std::thread::sleep(tick_rate - elapsed);
            }
        }
    } else {
        // Standard Client/Editor Startup
        run_client();
    }
}
```
