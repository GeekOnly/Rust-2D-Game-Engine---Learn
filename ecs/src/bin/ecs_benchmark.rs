//! ECS Benchmark CLI Tool
//!
//! Command-line tool for running ECS benchmarks and comparing backends.

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use ecs::{
    EcsBackendType, DynamicWorld, BenchmarkRunner, BenchmarkSuite
};
use ecs::traits::EcsWorld;

#[derive(Parser)]
#[command(name = "ecs-benchmark")]
#[command(about = "ECS Backend Benchmark and Comparison Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available ECS backends
    List,
    /// Show detailed information about backends
    Info {
        /// Specific backend to show info for
        #[arg(short, long)]
        backend: Option<String>,
    },
    /// Run benchmarks
    Benchmark {
        /// Number of iterations per test
        #[arg(short, long, default_value = "1000")]
        iterations: usize,
        /// Number of warmup iterations
        #[arg(short, long, default_value = "100")]
        warmup: usize,
        /// Specific backend to benchmark (default: all)
        #[arg(short, long)]
        backend: Option<String>,
        /// Output file for results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compare benchmark results
    Compare {
        /// Input file with benchmark results
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Test basic functionality of backends
    Test {
        /// Specific backend to test (default: all)
        #[arg(short, long)]
        backend: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::List => list_backends(),
        Commands::Info { backend } => show_backend_info(backend),
        Commands::Benchmark { iterations, warmup, backend, output } => {
            run_benchmarks(iterations, warmup, backend, output)
        },
        Commands::Compare { input } => compare_results(input),
        Commands::Test { backend } => test_backends(backend),
    }
}

fn list_backends() -> Result<(), Box<dyn std::error::Error>> {
    println!("Available ECS Backends:");
    println!("======================");
    
    for (i, backend) in EcsBackendType::available_backends().iter().enumerate() {
        let default_marker = if *backend == EcsBackendType::default() { " (default)" } else { "" };
        println!("{}. {}{}", i + 1, backend, default_marker);
    }
    
    Ok(())
}

fn show_backend_info(backend_filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let backends = if let Some(filter) = backend_filter {
        let backend_type = parse_backend_type(&filter)?;
        vec![backend_type]
    } else {
        EcsBackendType::available_backends()
    };
    
    for backend in backends {
        println!("Backend: {}", backend);
        println!("Description: {}", backend.description());
        
        let perf_info = backend.performance_info();
        println!("Performance Characteristics:");
        println!("  Entity Spawn Speed: {}", perf_info.entity_spawn_speed);
        println!("  Component Access Speed: {}", perf_info.component_access_speed);
        println!("  Query Speed: {}", perf_info.query_speed);
        println!("  Memory Usage: {}", perf_info.memory_usage);
        println!("  Parallel Systems: {}", perf_info.parallel_systems);
        println!("  Archetype-based: {}", perf_info.archetype_based);
        println!();
    }
    
    Ok(())
}

fn run_benchmarks(
    iterations: usize,
    warmup: usize,
    backend_filter: Option<String>,
    output_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let runner = BenchmarkRunner::new(iterations, warmup);
    
    println!("Running ECS Benchmarks...");
    println!("Iterations: {}, Warmup: {}", iterations, warmup);
    println!();
    
    let suite = if let Some(filter) = backend_filter {
        let backend_type = parse_backend_type(&filter)?;
        println!("Benchmarking {} backend only", backend_type);
        run_single_backend_benchmark(&runner, backend_type)?
    } else {
        println!("Benchmarking all available backends");
        runner.run_all_benchmarks()
    };
    
    // Display results
    println!("\n{}", suite.generate_report());
    
    // Save to file if requested
    if let Some(output_path) = output_file {
        suite.save_to_file(output_path.to_str().unwrap())?;
        println!("Results saved to: {}", output_path.display());
    }
    
    Ok(())
}

fn run_single_backend_benchmark(
    runner: &BenchmarkRunner,
    backend_type: EcsBackendType,
) -> Result<BenchmarkSuite, Box<dyn std::error::Error>> {
    let mut suite = BenchmarkSuite::new();
    
    println!("Benchmarking {:?} backend...", backend_type);
    
    // Entity spawn benchmark
    print!("  Entity spawn... ");
    if let Ok(result) = runner.bench_entity_spawn(backend_type, 1000) {
        suite.add_result(result);
        println!("✓");
    } else {
        println!("✗");
    }
    
    // Entity despawn benchmark
    print!("  Entity despawn... ");
    if let Ok(result) = runner.bench_entity_despawn(backend_type, 1000) {
        suite.add_result(result);
        println!("✓");
    } else {
        println!("✗");
    }
    
    // World clear benchmark
    print!("  World clear... ");
    if let Ok(result) = runner.bench_world_clear(backend_type, 1000) {
        suite.add_result(result);
        println!("✓");
    } else {
        println!("✗");
    }
    
    // Hierarchy operations benchmark
    print!("  Hierarchy operations... ");
    if let Ok(result) = runner.bench_hierarchy_operations(backend_type, 100) {
        suite.add_result(result);
        println!("✓");
    } else {
        println!("✗");
    }
    
    // Mixed operations benchmark
    print!("  Mixed operations... ");
    if let Ok(result) = runner.bench_mixed_operations(backend_type) {
        suite.add_result(result);
        println!("✓");
    } else {
        println!("✗");
    }
    
    Ok(suite)
}

fn compare_results(input_file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let suite = BenchmarkSuite::load_from_file(input_file.to_str().unwrap())?;
    
    println!("Benchmark Results from: {}", input_file.display());
    println!("Timestamp: {}", suite.timestamp);
    println!();
    
    println!("{}", suite.generate_report());
    
    Ok(())
}

fn test_backends(backend_filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let backends = if let Some(filter) = backend_filter {
        let backend_type = parse_backend_type(&filter)?;
        vec![backend_type]
    } else {
        EcsBackendType::available_backends()
    };
    
    println!("Testing ECS Backend Functionality:");
    println!("=================================");
    
    for backend in backends {
        println!("\nTesting {} backend...", backend);
        
        match test_backend_functionality(backend) {
            Ok(_) => println!("  ✓ All tests passed"),
            Err(e) => println!("  ✗ Test failed: {}", e),
        }
    }
    
    Ok(())
}

fn test_backend_functionality(backend_type: EcsBackendType) -> Result<(), Box<dyn std::error::Error>> {
    let mut world = DynamicWorld::new(backend_type)?;
    
    // Test entity spawning
    let entity1 = world.spawn();
    let entity2 = world.spawn();
    assert!(world.is_alive(entity1));
    assert!(world.is_alive(entity2));
    assert_eq!(world.entity_count(), 2);
    
    // Test hierarchy
    world.set_parent(entity2, Some(entity1))?;
    assert_eq!(world.get_parent(entity2), Some(entity1));
    assert_eq!(world.get_children(entity1), vec![entity2]);
    
    // Test despawning
    world.despawn(entity1)?;
    assert!(!world.is_alive(entity1));
    assert!(!world.is_alive(entity2)); // Should be despawned with parent
    assert_eq!(world.entity_count(), 0);
    
    // Test world clear
    let _entity3 = world.spawn();
    let _entity4 = world.spawn();
    assert_eq!(world.entity_count(), 2);
    world.clear();
    assert_eq!(world.entity_count(), 0);
    
    Ok(())
}

fn parse_backend_type(backend_str: &str) -> Result<EcsBackendType, Box<dyn std::error::Error>> {
    match backend_str.to_lowercase().as_str() {
        "custom" => Ok(EcsBackendType::Custom),
        #[cfg(feature = "hecs")]
        "hecs" => Ok(EcsBackendType::Hecs),
        #[cfg(feature = "specs")]
        "specs" => Ok(EcsBackendType::Specs),
        #[cfg(feature = "bevy")]
        "bevy" => Ok(EcsBackendType::Bevy),
        _ => Err(format!("Unknown backend: {}", backend_str).into()),
    }
}