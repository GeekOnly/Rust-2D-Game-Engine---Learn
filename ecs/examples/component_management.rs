/// ตัวอย่างการใช้งาน Component Management System
/// 
/// รันด้วยคำสั่ง: cargo run --example component_management

use ecs::{World, ComponentType, ComponentManager};
use ecs::traits::EcsWorld;

fn main() {
    println!("=== Component Management System Demo ===\n");

    let mut world = World::new();

    // 1. สร้าง Player Entity
    println!("1. Creating Player Entity...");
    let player = world.spawn();
    world.names.insert(player, "Player".to_string());

    // เพิ่ม Components
    world.add_component(player, ComponentType::Transform).unwrap();
    world.add_component(player, ComponentType::Sprite).unwrap();
    world.add_component(player, ComponentType::BoxCollider).unwrap();
    world.add_component(player, ComponentType::Rigidbody).unwrap();

    println!("   Player Entity ID: {}", player);
    println!("   Components added: Transform, Sprite, BoxCollider, Rigidbody\n");

    // 2. แสดง Components ทั้งหมด
    println!("2. Player Components:");
    let components = world.get_components(player);
    for component_type in &components {
        println!("   - {}", component_type.display_name());
    }
    println!();

    // 3. ตั้งค่า Sprite
    println!("3. Configuring Sprite...");
    if let Some(sprite) = world.sprites.get_mut(&player) {
        sprite.texture_id = "player_sprite".to_string();
        sprite.width = 40.0;
        sprite.height = 40.0;
        sprite.color = [0.2, 0.6, 1.0, 1.0]; // สีฟ้า
        println!("   Texture: {}", sprite.texture_id);
        println!("   Size: {}x{}", sprite.width, sprite.height);
        println!("   Color: {:?}\n", sprite.color);
    }

    // 4. ตรวจสอบ Component
    println!("4. Checking Components:");
    println!("   Has Sprite? {}", world.has_component(player, ComponentType::Sprite));
    println!("   Has Camera? {}", world.has_component(player, ComponentType::Camera));
    println!();

    // 5. ดูรายการ Component ที่สามารถเพิ่มได้
    println!("5. Available Components to Add:");
    let addable = world.get_addable_components(player);
    for component_type in &addable {
        println!("   - {}", component_type.display_name());
    }
    println!();

    // 6. เพิ่ม Script Component
    println!("6. Adding Script Component...");
    world.add_component(player, ComponentType::Script).unwrap();
    if let Some(script) = world.scripts.get_mut(&player) {
        script.script_name = "PlayerController".to_string();
        script.enabled = true;
        println!("   Script: {}", script.script_name);
        println!("   Enabled: {}\n", script.enabled);
    }

    // 7. ลบ Rigidbody
    println!("7. Removing Rigidbody Component...");
    match world.remove_component(player, ComponentType::Rigidbody) {
        Ok(_) => println!("   Rigidbody removed successfully\n"),
        Err(e) => println!("   Error: {}\n", e),
    }

    // 8. แสดง Components หลังจากลบ
    println!("8. Player Components After Removal:");
    let components = world.get_components(player);
    for component_type in &components {
        println!("   - {}", component_type.display_name());
    }
    println!();

    // 9. พยายามลบ Transform (ไม่สำเร็จ)
    println!("9. Trying to Remove Transform (should fail)...");
    match world.remove_component(player, ComponentType::Transform) {
        Ok(_) => println!("   Transform removed (unexpected!)\n"),
        Err(e) => println!("   Error: {} ✓\n", e),
    }

    // 10. สร้าง Camera Entity
    println!("10. Creating Camera Entity...");
    let camera = world.spawn();
    world.names.insert(camera, "Main Camera".to_string());
    
    world.add_component(camera, ComponentType::Transform).unwrap();
    world.add_component(camera, ComponentType::Camera).unwrap();

    if let Some(transform) = world.transforms.get_mut(&camera) {
        transform.position = [0.0, 0.0, -10.0];
    }

    if let Some(cam) = world.cameras.get_mut(&camera) {
        cam.projection = ecs::CameraProjection::Orthographic;
        cam.orthographic_size = 5.0;
    }

    println!("   Camera Entity ID: {}", camera);
    println!("   Position: {:?}", world.transforms.get(&camera).unwrap().position);
    println!("   Projection: Orthographic\n");

    // 11. สร้าง 3D Cube
    println!("11. Creating 3D Cube Entity...");
    let cube = world.spawn();
    world.names.insert(cube, "Cube".to_string());
    
    world.add_component(cube, ComponentType::Transform).unwrap();
    world.add_component(cube, ComponentType::Mesh).unwrap();

    if let Some(mesh) = world.meshes.get_mut(&cube) {
        mesh.mesh_type = ecs::MeshType::Cube;
        mesh.color = [1.0, 0.0, 0.0, 1.0]; // สีแดง
    }

    println!("   Cube Entity ID: {}", cube);
    println!("   Mesh Type: {:?}", world.meshes.get(&cube).unwrap().mesh_type);
    println!("   Color: {:?}\n", world.meshes.get(&cube).unwrap().color);

    // 12. สรุป
    println!("=== Summary ===");
    println!("Total Entities: {}", world.entity_count());
    println!("\nEntities:");
    
    for entity in [player, camera, cube] {
        if let Some(name) = world.names.get(&entity) {
            println!("\n{} (ID: {})", name, entity);
            let components = world.get_components(entity);
            println!("  Components ({}):", components.len());
            for component_type in components {
                println!("    - {}", component_type.display_name());
            }
        }
    }

    println!("\n=== Demo Complete ===");
}
