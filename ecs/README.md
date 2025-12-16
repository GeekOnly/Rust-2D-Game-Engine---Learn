# ECS Module - Multi-Backend Entity Component System

A flexible Entity Component System (ECS) implementation that supports multiple backends including custom HashMap-based, Hecs, Specs, and Bevy ECS.

## Features

- **Multiple ECS Backends**: Choose between different ECS implementations based on your needs
- **Runtime Backend Switching**: Switch between backends at runtime (clears world)
- **Comprehensive Benchmarking**: Built-in benchmark suite to compare backend performance
- **Unified API**: Same API across all backends for easy switching
- **Performance Analysis**: Detailed performance characteristics for each backend

## Available Backends

### 1. Custom HashMap Backend (Always Available)
- **Description**: Simple HashMap-based ECS implementation
- **Best For**: Prototyping and small games
- **Performance**: Medium entity spawn, medium component access, low query speed
- **Features**: Basic functionality, easy to understand

### 2. Hecs Backend (Feature: `hecs`)
- **Description**: Fast, minimal, and flexible ECS library
- **Best For**: Balance of performance and simplicity
- **Performance**: High entity spawn, high component access, high query speed
- **Features**: Archetype-based, excellent performance, minimal API

### 3. Specs Backend (Feature: `specs`)
- **Description**: Mature, parallel ECS library
- **Best For**: Complex games with many systems
- **Performance**: Medium entity spawn, high component access, high query speed
- **Features**: Parallel systems, mature ecosystem, flexible

### 4. Bevy ECS Backend (Feature: `bevy`)
- **Description**: Modern, high-performance ECS with excellent ergonomics
- **Best For**: Modern game development with advanced scheduling
- **Performance**: High entity spawn, high component access, very high query speed
- **Features**: Archetype-based, parallel systems, advanced scheduling

## Usage

### Basic Usage

```rust
use ecs::{EcsBackendType, DynamicWorld, Transform, Sprite};
use ecs::traits::{EcsWorld, ComponentAccess};

// Create a world with the default backend
let mut world = DynamicWorld::new(EcsBackendType::default())?;

// Spawn entities
let player = world.spawn();
let enemy = world.spawn();

// Add components (simplified - actual implementation varies by backend)
// world.insert_component(player, Transform::default())?;
// world.insert_component(player, Sprite::default())?;

// Set up hierarchy
world.set_parent(enemy, Some(player))?;

// Query entities
println!("Player children: {:?}", world.get_children(player));
```

### Backend Selection

```rust
use ecs::{EcsBackendType, DynamicWorld};

// List available backends
for backend in EcsBackendType::available_backends() {
    println!("{}: {}", backend, backend.description());
}

// Create world with specific backend
let mut world = DynamicWorld::new(EcsBackendType::Hecs)?;

// Switch backend at runtime (clears world)
world.switch_backend(EcsBackendType::Specs)?;
```

### Performance Analysis

```rust
use ecs::{EcsBackendType, BenchmarkRunner};

// Get performance characteristics
let backend = EcsBackendType::Hecs;
let perf_info = backend.performance_info();
println!("Entity spawn speed: {}", perf_info.entity_spawn_speed);
println!("Supports parallel systems: {}", perf_info.parallel_systems);

// Run benchmarks
let runner = BenchmarkRunner::new(1000, 100);
let suite = runner.run_all_benchmarks();
println!("{}", suite.generate_report());
```

## Cargo Features

Enable specific backends by adding features to your `Cargo.toml`:

```toml
[dependencies]
ecs = { path = "../ecs", features = ["hecs", "specs", "bevy"] }

# Or enable all backends
ecs = { path = "../ecs", features = ["all_backends"] }
```

Available features:
- `hecs` - Enable Hecs backend
- `specs` - Enable Specs backend  
- `bevy` - Enable Bevy ECS backend
- `all_backends` - Enable all backends

## Command Line Tools

### ECS Benchmark CLI

Run benchmarks from the command line:

```bash
# List available backends
cargo run --bin ecs_benchmark list

# Show backend information
cargo run --bin ecs_benchmark info

# Run benchmarks for all backends
cargo run --bin ecs_benchmark benchmark --iterations 1000 --output results.json

# Run benchmarks for specific backend
cargo run --bin ecs_benchmark benchmark --backend hecs

# Compare results
cargo run --bin ecs_benchmark compare --input results.json

# Test backend functionality
cargo run --bin ecs_benchmark test
```

### Example Usage

```bash
# Run with all features enabled
cargo run --features all_backends --bin ecs_benchmark benchmark

# Run with specific backend
cargo run --features hecs --bin ecs_benchmark benchmark --backend hecs
```

## Examples

Run the backend chooser example:

```bash
cargo run --example backend_chooser --features all_backends
```

## Benchmarking

The module includes comprehensive benchmarking tools:

### Built-in Benchmarks

- **Entity Spawn**: Measures entity creation speed
- **Entity Despawn**: Measures entity destruction speed
- **Component Insert**: Measures component addition speed
- **World Clear**: Measures world clearing speed
- **Hierarchy Operations**: Measures parent-child relationship operations
- **Mixed Operations**: Simulates typical game frame operations

### Running Criterion Benchmarks

```bash
# Run all benchmarks with Criterion
cargo bench

# Run specific benchmark
cargo bench entity_spawn
```

### Custom Benchmarks

```rust
use ecs::{BenchmarkRunner, EcsBackendType};

let runner = BenchmarkRunner::new(1000, 100);

// Benchmark specific operation
let result = runner.bench_entity_spawn(EcsBackendType::Hecs, 10000)?;
println!("Hecs: {:.0} entities/sec", result.operations_per_second);

// Run full benchmark suite
let suite = runner.run_all_benchmarks();
suite.save_to_file("benchmark_results.json")?;
```

## Performance Comparison

Based on typical benchmarks (results may vary):

| Backend | Entity Spawn | Component Access | Query Speed | Memory Usage | Parallel |
|---------|-------------|------------------|-------------|--------------|----------|
| Custom  | Medium      | Medium           | Low         | Medium       | No       |
| Hecs    | High        | High             | High        | High         | No       |
| Specs   | Medium      | High             | High        | Medium       | Yes      |
| Bevy    | High        | High             | Very High   | High         | Yes      |

## Architecture

The ECS module uses a trait-based abstraction layer:

- `EcsWorld` - Core world operations (spawn, despawn, hierarchy)
- `ComponentAccess<T>` - Type-safe component operations
- `Serializable` - World persistence
- `DynamicWorld` - Runtime backend switching

Each backend implements these traits, providing a unified API while leveraging the strengths of different ECS libraries.

## Testing

Run tests for all available backends:

```bash
# Run unit tests
cargo test

# Run tests with all features
cargo test --features all_backends

# Run property-based tests
cargo test property_tests
```

## Contributing

When adding new backends:

1. Implement the core traits (`EcsWorld`, `ComponentAccess<T>`, `Serializable`)
2. Add feature flag to `Cargo.toml`
3. Update `EcsBackendType` enum
4. Add backend to `DynamicWorld`
5. Add benchmarks and tests
6. Update documentation

## License

This module is part of the Rust 2D Game Engine project.