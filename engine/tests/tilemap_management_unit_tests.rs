// Unit tests for Tilemap Management System
// These tests verify specific functionality of tilemap management components

mod common;

#[cfg(test)]
mod tilemap_unit_tests {
    use crate::common::tilemap_test_utils::*;
    
    #[test]
    fn test_mock_ldtk_creation() {
        // Test creating a basic mock LDtk project
        let project = create_mock_ldtk_project(20, 15, 16, None);
        
        assert_eq!(project["defaultGridSize"], 16);
        
        let levels = project["levels"].as_array().unwrap();
        assert_eq!(levels.len(), 1);
        
        let level = &levels[0];
        assert_eq!(level["identifier"], "Level_Test");
        assert_eq!(level["pxWid"], 20 * 16);
        assert_eq!(level["pxHei"], 15 * 16);
        
        let layers = level["layerInstances"].as_array().unwrap();
        assert_eq!(layers.len(), 1);
        
        let layer = &layers[0];
        assert_eq!(layer["__identifier"], "IntGrid_layer");
        assert_eq!(layer["__cWid"], 20);
        assert_eq!(layer["__cHei"], 15);
    }
    
    #[test]
    fn test_mock_ldtk_with_custom_intgrid() {
        // Test creating a mock LDtk project with custom IntGrid data
        let intgrid_data = vec![1, 0, 1, 0, 1, 0, 1, 0, 1];
        let project = create_mock_ldtk_project(3, 3, 8, Some(intgrid_data.clone()));
        
        let levels = project["levels"].as_array().unwrap();
        let level = &levels[0];
        let layers = level["layerInstances"].as_array().unwrap();
        let layer = &layers[0];
        
        let csv = layer["intGridCsv"].as_array().unwrap();
        assert_eq!(csv.len(), 9);
        
        for (i, value) in intgrid_data.iter().enumerate() {
            assert_eq!(csv[i].as_i64().unwrap(), *value);
        }
    }
    
    #[test]
    fn test_generate_pattern_checkerboard() {
        let data = generate_pattern_intgrid(4, 4, IntGridPattern::Checkerboard);
        
        // Verify checkerboard pattern
        assert_eq!(data.len(), 16);
        
        // (0,0) = 1, (1,0) = 0, (0,1) = 0, (1,1) = 1
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 0);
        assert_eq!(data[4], 0);
        assert_eq!(data[5], 1);
    }
    
    #[test]
    fn test_generate_pattern_horizontal_lines() {
        let data = generate_pattern_intgrid(4, 4, IntGridPattern::HorizontalLines);
        
        // Row 0: all 1s
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 1);
        assert_eq!(data[2], 1);
        assert_eq!(data[3], 1);
        
        // Row 1: all 0s
        assert_eq!(data[4], 0);
        assert_eq!(data[5], 0);
        assert_eq!(data[6], 0);
        assert_eq!(data[7], 0);
    }
    
    #[test]
    fn test_generate_pattern_border() {
        let data = generate_pattern_intgrid(5, 5, IntGridPattern::Border);
        
        // Top row: all 1s
        for x in 0..5 {
            assert_eq!(data[x], 1);
        }
        
        // Bottom row: all 1s
        for x in 0..5 {
            assert_eq!(data[20 + x], 1);
        }
        
        // Left column: all 1s
        for y in 0..5 {
            assert_eq!(data[y * 5], 1);
        }
        
        // Right column: all 1s
        for y in 0..5 {
            assert_eq!(data[y * 5 + 4], 1);
        }
        
        // Center: all 0s
        assert_eq!(data[6], 0);  // (1,1)
        assert_eq!(data[7], 0);  // (2,1)
        assert_eq!(data[8], 0);  // (3,1)
        assert_eq!(data[11], 0); // (1,2)
        assert_eq!(data[12], 0); // (2,2)
        assert_eq!(data[13], 0); // (3,2)
    }
    
    #[test]
    fn test_count_collision_tiles() {
        let data = vec![0, 1, 1, 0, 1, 0, 0, 1, 1, 1];
        
        assert_eq!(count_collision_tiles(&data, 1), 6);
        assert_eq!(count_collision_tiles(&data, 0), 4);
        assert_eq!(count_collision_tiles(&data, 2), 0);
    }
    
    #[test]
    fn test_find_rectangles_single_tile() {
        // Single collision tile
        let data = vec![
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ];
        
        let rectangles = find_rectangles_in_intgrid(&data, 3, 3, 1);
        
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].x, 1);
        assert_eq!(rectangles[0].y, 1);
        assert_eq!(rectangles[0].width, 1);
        assert_eq!(rectangles[0].height, 1);
    }
    
    #[test]
    fn test_find_rectangles_horizontal_line() {
        // Horizontal line of 3 tiles
        let data = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 0,
        ];
        
        let rectangles = find_rectangles_in_intgrid(&data, 3, 3, 1);
        
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 3);
        assert_eq!(rectangles[0].height, 1);
    }
    
    #[test]
    fn test_find_rectangles_vertical_line() {
        // Vertical line of 3 tiles
        let data = vec![
            0, 1, 0,
            0, 1, 0,
            0, 1, 0,
        ];
        
        let rectangles = find_rectangles_in_intgrid(&data, 3, 3, 1);
        
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].width, 1);
        assert_eq!(rectangles[0].height, 3);
    }
    
    #[test]
    fn test_find_rectangles_solid_block() {
        // 2x2 solid block
        let data = vec![
            0, 0, 0, 0,
            0, 1, 1, 0,
            0, 1, 1, 0,
            0, 0, 0, 0,
        ];
        
        let rectangles = find_rectangles_in_intgrid(&data, 4, 4, 1);
        
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].x, 1);
        assert_eq!(rectangles[0].y, 1);
        assert_eq!(rectangles[0].width, 2);
        assert_eq!(rectangles[0].height, 2);
        assert_eq!(rectangles[0].area(), 4);
    }
    
    #[test]
    fn test_find_rectangles_multiple_blocks() {
        // Two separate blocks
        let data = vec![
            1, 1, 0, 0,
            1, 1, 0, 0,
            0, 0, 1, 1,
            0, 0, 1, 1,
        ];
        
        let rectangles = find_rectangles_in_intgrid(&data, 4, 4, 1);
        
        assert_eq!(rectangles.len(), 2);
        
        // Both should be 2x2 blocks
        for rect in &rectangles {
            assert_eq!(rect.width, 2);
            assert_eq!(rect.height, 2);
            assert_eq!(rect.area(), 4);
        }
    }
    
    #[test]
    fn test_temp_file_creation() {
        let test_dir = create_test_dir();
        
        let project = create_mock_ldtk_project(5, 5, 8, None);
        let file_path = create_temp_ldtk_file(&test_dir, "test_map", &project);
        
        assert!(file_path.exists());
        assert!(file_path.to_str().unwrap().ends_with("test_map.ldtk"));
        
        // Verify file content
        let content = std::fs::read_to_string(&file_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["defaultGridSize"], 8);
        
        cleanup_test_dir(&test_dir);
        assert!(!test_dir.exists());
    }
    
    #[test]
    fn test_rectangle_area() {
        let rect = Rectangle {
            x: 0,
            y: 0,
            width: 5,
            height: 3,
        };
        
        assert_eq!(rect.area(), 15);
    }
    
    #[test]
    fn test_load_project_with_grid_creates_hierarchy() {
        use ecs::{World, loaders::LdtkLoader};
        use serde_json::json;
        
        let test_dir = create_test_dir();
        
        // Create a mock LDtk project with tile data so layers are actually created
        let project = json!({
            "defaultGridSize": 8,
            "levels": [
                {
                    "identifier": "Level_Test",
                    "worldX": 0,
                    "worldY": 0,
                    "pxWid": 80,
                    "pxHei": 80,
                    "layerInstances": [
                        {
                            "__identifier": "Tiles_layer",
                            "__type": "Tiles",
                            "__cWid": 10,
                            "__cHei": 10,
                            "__gridSize": 8,
                            "__pxTotalOffsetX": 0,
                            "__pxTotalOffsetY": 0,
                            "__tilesetDefUid": 1,
                            "layerDefUid": 1,
                            "intGridCsv": [],
                            "gridTiles": [
                                {
                                    "px": [0, 0],
                                    "t": 0,
                                    "f": 0
                                },
                                {
                                    "px": [8, 0],
                                    "t": 1,
                                    "f": 0
                                }
                            ],
                            "autoLayerTiles": []
                        }
                    ]
                }
            ],
            "defs": {
                "tilesets": [
                    {
                        "uid": 1,
                        "relPath": "../test_tileset.png",
                        "pxWid": 256,
                        "pxHei": 256
                    }
                ]
            }
        });
        
        let file_path = create_temp_ldtk_file(&test_dir, "test_hierarchy", &project);
        
        let mut world = World::new();
        
        // Load project with Grid
        let result = LdtkLoader::load_project_with_grid(&file_path, &mut world);
        
        assert!(result.is_ok(), "Failed to load project: {:?}", result.err());
        
        let (grid_entity, layer_entities) = result.unwrap();
        
        // Verify Grid entity exists
        assert!(world.transforms.contains_key(&grid_entity));
        assert!(world.grids.contains_key(&grid_entity));
        assert_eq!(world.names.get(&grid_entity).unwrap(), "LDtk Grid");
        
        // Verify Grid component properties
        let grid = world.grids.get(&grid_entity).unwrap();
        assert_eq!(grid.cell_size, (1.0, 1.0)); // 8 pixels / 8 pixels_per_unit = 1.0
        
        // Verify layers exist and are children of Grid
        assert_eq!(layer_entities.len(), 1);
        
        for &layer_entity in &layer_entities {
            // Check layer has parent set to Grid
            let parent = world.get_parent(layer_entity);
            assert_eq!(parent, Some(grid_entity), "Layer should have Grid as parent");
            
            // Check Grid has layer as child
            let children = world.get_children(grid_entity);
            assert!(children.contains(&layer_entity), "Grid should have layer as child");
        }
        
        cleanup_test_dir(&test_dir);
    }
    
    #[test]
    fn test_load_project_with_grid_and_colliders() {
        use ecs::{World, loaders::LdtkLoader};
        use serde_json::json;
        
        let test_dir = create_test_dir();
        
        // Create a mock LDtk project with IntGrid collision tiles
        let project = json!({
            "defaultGridSize": 8,
            "levels": [
                {
                    "identifier": "Level_Test",
                    "worldX": 0,
                    "worldY": 0,
                    "pxWid": 40,
                    "pxHei": 40,
                    "layerInstances": [
                        {
                            "__identifier": "IntGrid_layer",
                            "__type": "IntGrid",
                            "__cWid": 5,
                            "__cHei": 5,
                            "__gridSize": 8,
                            "__pxTotalOffsetX": 0,
                            "__pxTotalOffsetY": 0,
                            "layerDefUid": 1,
                            "intGridCsv": [
                                1, 1, 1, 0, 0,
                                1, 1, 1, 0, 0,
                                0, 0, 0, 0, 0,
                                0, 0, 1, 1, 1,
                                0, 0, 1, 1, 1
                            ],
                            "gridTiles": [],
                            "autoLayerTiles": []
                        }
                    ]
                }
            ],
            "defs": {
                "tilesets": []
            }
        });
        
        let file_path = create_temp_ldtk_file(&test_dir, "test_colliders", &project);
        
        let mut world = World::new();
        
        // Load project with Grid and auto-generate colliders
        let result = LdtkLoader::load_project_with_grid_and_colliders(
            &file_path,
            &mut world,
            true,  // auto_generate_colliders
            1,     // collision_value
        );
        
        assert!(result.is_ok(), "Failed to load project: {:?}", result.err());
        
        let (grid_entity, layer_entities, collider_entities) = result.unwrap();
        
        // Verify Grid entity exists
        assert!(world.grids.contains_key(&grid_entity));
        
        // Verify layers exist (IntGrid layers without tiles don't create layer entities)
        // So we expect 0 layer entities since IntGrid has no visual tiles
        assert_eq!(layer_entities.len(), 0);
        
        // Verify colliders were generated
        assert!(!collider_entities.is_empty(), "Should have generated colliders");
        
        // Verify all colliders are children of Grid
        for &collider_entity in &collider_entities {
            let parent = world.get_parent(collider_entity);
            assert_eq!(parent, Some(grid_entity), "Collider should have Grid as parent");
            
            // Verify collider has required components
            assert!(world.colliders.contains_key(&collider_entity), "Collider entity should have Collider component");
            assert!(world.rigidbodies.contains_key(&collider_entity), "Collider entity should have Rigidbody2D component");
            assert!(world.transforms.contains_key(&collider_entity), "Collider entity should have Transform component");
        }
        
        // Verify Grid has all children (layers + colliders)
        let children = world.get_children(grid_entity);
        assert_eq!(children.len(), layer_entities.len() + collider_entities.len());
        
        cleanup_test_dir(&test_dir);
    }
    
    #[test]
    fn test_unload_grid_cleans_up_hierarchy() {
        use ecs::{World, loaders::LdtkLoader};
        
        let test_dir = create_test_dir();
        
        // Create a mock LDtk project with collision tiles
        let intgrid_data = vec![1, 1, 0, 0, 1, 1, 0, 0, 1];
        let project = create_mock_ldtk_project(3, 3, 8, Some(intgrid_data));
        let file_path = create_temp_ldtk_file(&test_dir, "test_cleanup", &project);
        
        let mut world = World::new();
        
        // Load project
        let result = LdtkLoader::load_project_with_grid_and_colliders(
            &file_path,
            &mut world,
            true,
            1,
        );
        
        assert!(result.is_ok());
        let (grid_entity, layer_entities, collider_entities) = result.unwrap();
        
        // Verify entities exist
        assert!(world.transforms.contains_key(&grid_entity));
        for &entity in layer_entities.iter().chain(collider_entities.iter()) {
            assert!(world.transforms.contains_key(&entity));
        }
        
        // Despawn Grid entity (should clean up all children)
        world.despawn(grid_entity);
        
        // Verify Grid is gone
        assert!(!world.transforms.contains_key(&grid_entity));
        assert!(!world.grids.contains_key(&grid_entity));
        
        // Verify all children are gone
        for &entity in layer_entities.iter().chain(collider_entities.iter()) {
            assert!(!world.transforms.contains_key(&entity), "Child entity {} should be despawned", entity);
        }
        
        cleanup_test_dir(&test_dir);
    }
}
