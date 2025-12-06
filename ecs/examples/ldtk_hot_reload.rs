/// Example: LDtk Hot-Reload
/// 
/// This example demonstrates how to use the LDtk hot-reload system.
/// 
/// 1. Create an LDtk file using LDtk editor (https://ldtk.io/)
/// 2. Run this example with the path to your .ldtk file
/// 3. Edit the file in LDtk and save
/// 4. Watch the console for reload messages
/// 
/// Usage:
/// ```
/// cargo run --example ldtk_hot_reload -- path/to/your/level.ldtk
/// ```

use ecs::{World, loaders::LdtkHotReloader, traits::EcsWorld};
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logger (optional - comment out if env_logger not available)
    // env_logger::init();

    // Get file path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-ldtk-file>", args[0]);
        eprintln!("\nExample:");
        eprintln!("  cargo run --example ldtk_hot_reload -- levels/world.ldtk");
        std::process::exit(1);
    }

    let ldtk_path = &args[1];
    
    println!("=== LDtk Hot-Reload Example ===");
    println!("Watching: {}", ldtk_path);
    println!("\nEdit the file in LDtk editor and save to see hot-reload in action.");
    println!("Press Ctrl+C to exit.\n");

    // Create world and hot-reloader
    let mut world = World::new();
    let mut reloader = LdtkHotReloader::new();

    // Load and watch the file
    match reloader.watch(ldtk_path, &mut world) {
        Ok(entities) => {
            println!("✓ Loaded {} entities from {}", entities.len(), ldtk_path);
            print_world_stats(&world);
        }
        Err(e) => {
            eprintln!("✗ Failed to load file: {}", e);
            std::process::exit(1);
        }
    }

    // Main loop - check for updates every second
    let mut frame = 0;
    loop {
        thread::sleep(Duration::from_millis(500));
        frame += 1;

        // Check for updates
        if let Some(entities) = reloader.check_updates(&mut world) {
            println!("\n🔄 Hot-reload detected! (frame {})", frame);
            println!("   Reloaded {} entities", entities.len());
            print_world_stats(&world);
        }

        // Print heartbeat every 10 seconds
        if frame % 20 == 0 {
            println!("💓 Still watching... (frame {})", frame);
        }
    }
}

fn print_world_stats(world: &World) {
    println!("   World stats:");
    println!("     - Transforms: {}", world.transforms.len());
    println!("     - Tilemaps: {}", world.tilemaps.len());
    println!("     - Sprites: {}", world.sprites.len());
    println!("     - Total entities: {}", world.entity_count());
}
