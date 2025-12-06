-- World UI Example for Celeste Demo
-- Demonstrates how to spawn world-space UI elements

local WorldUIExample = {}

-- Spawn damage number at position
function WorldUIExample.spawn_damage_number(world, x, y, damage)
    -- This would be called from Rust side
    -- For now, this is a reference implementation
    
    print(string.format("Damage number: %d at (%.1f, %.1f)", damage, x, y))
    
    -- In Rust, you would do:
    -- let entity = world.spawn();
    -- world.transforms.insert(entity, Transform::with_position(x, y, 0.0));
    -- world.world_uis.insert(entity, WorldUI::damage_number(damage));
end

-- Spawn health bar above enemy
function WorldUIExample.spawn_enemy_with_healthbar(world, x, y)
    print(string.format("Enemy with health bar at (%.1f, %.1f)", x, y))
    
    -- In Rust:
    -- let enemy = world.spawn();
    -- world.transforms.insert(enemy, Transform::with_position(x, y, 0.0));
    -- world.sprites.insert(enemy, Sprite::new("enemy.png", 32.0, 32.0));
    -- world.world_uis.insert(enemy, WorldUI::health_bar(100.0, 100.0));
end

-- Spawn interaction prompt
function WorldUIExample.spawn_chest(world, x, y)
    print(string.format("Chest with prompt at (%.1f, %.1f)", x, y))
    
    -- In Rust:
    -- let chest = world.spawn();
    -- world.transforms.insert(chest, Transform::with_position(x, y, 0.0));
    -- world.sprites.insert(chest, Sprite::new("chest.png", 32.0, 32.0));
    -- world.world_uis.insert(chest, WorldUI::interaction_prompt("Open", "E"));
end

-- Test function to spawn various UI elements
function WorldUIExample.test_spawn_ui(world)
    print("=== Testing World UI ===")
    
    -- Spawn some damage numbers
    WorldUIExample.spawn_damage_number(world, 5.0, 0.0, 25)
    WorldUIExample.spawn_damage_number(world, 10.0, 0.0, 50)
    WorldUIExample.spawn_damage_number(world, 15.0, 0.0, 100)
    
    -- Spawn enemies with health bars
    WorldUIExample.spawn_enemy_with_healthbar(world, 20.0, 5.0)
    WorldUIExample.spawn_enemy_with_healthbar(world, 25.0, 5.0)
    
    -- Spawn interactive objects
    WorldUIExample.spawn_chest(world, 30.0, 0.0)
    
    print("=== World UI Test Complete ===")
end

return WorldUIExample
