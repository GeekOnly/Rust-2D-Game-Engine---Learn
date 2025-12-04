/// Example: Load LDtk Map
/// 
/// แสดงวิธีการ load ไฟล์ .ldtk เข้า game engine
/// 
/// Usage:
/// ```
/// cargo run --example load_ldtk_map
/// ```

use ecs::{World, loaders::LdtkLoader};

fn main() {
    env_logger::init();
    
    println!("=== Load LDtk Map Example ===\n");
    
    // สร้าง world
    let mut world = World::new();
    
    // Path ไปยังไฟล์ LDtk
    let ldtk_path = "projects/Celeste Demo/levels/world.ldtk";
    
    println!("Loading LDtk file: {}", ldtk_path);
    
    // Load LDtk file
    match LdtkLoader::load_project(ldtk_path, &mut world) {
        Ok(entities) => {
            println!("✓ Successfully loaded {} entities", entities.len());
            println!("\nWorld contents:");
            println!("  - Transforms: {}", world.transforms.len());
            println!("  - Tilemaps: {}", world.tilemaps.len());
            println!("  - Names: {}", world.names.len());
            
            // แสดงรายละเอียด entities
            println!("\nEntities:");
            for entity in entities {
                if let Some(name) = world.names.get(&entity) {
                    println!("  - Entity {}: {}", entity, name);
                }
                
                if let Some(transform) = world.transforms.get(&entity) {
                    println!("    Position: [{}, {}, {}]", 
                        transform.position[0],
                        transform.position[1],
                        transform.position[2]
                    );
                }
                
                if let Some(tilemap) = world.tilemaps.get(&entity) {
                    println!("    Tilemap: {}x{} ({})", 
                        tilemap.width,
                        tilemap.height,
                        tilemap.name
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load LDtk file: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. The file exists at: {}", ldtk_path);
            eprintln!("  2. The file is a valid LDtk project");
            eprintln!("  3. You have created the file in LDtk editor");
            std::process::exit(1);
        }
    }
    
    println!("\n=== Example Complete ===");
}
