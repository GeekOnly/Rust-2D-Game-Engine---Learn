#!/usr/bin/env python3
"""
Script to expand tilemap from 10x10 to 20x20 by duplicating existing tiles
"""

import json
import sys

def expand_tilemap(scene_file, old_width=10, old_height=10, new_width=20, new_height=20):
    """Expand tilemap by duplicating existing tile pattern"""
    
    # Load scene file
    with open(scene_file, 'r') as f:
        scene = json.load(f)
    
    # Find tilemap entity (ID 315)
    tilemap_entity = None
    for tilemap_data in scene.get('tilemaps', []):
        if tilemap_data[0] == 315:  # Entity ID 315
            tilemap_entity = tilemap_data[1]
            break
    
    if not tilemap_entity:
        print("Tilemap entity 315 not found!")
        return False
    
    # Get existing tiles
    existing_tiles = tilemap_entity.get('tiles', [])
    if len(existing_tiles) != old_width * old_height:
        print(f"Expected {old_width * old_height} tiles, found {len(existing_tiles)}")
        return False
    
    # Create new tile array
    new_tiles = []
    
    for y in range(new_height):
        for x in range(new_width):
            # Map to original tile position using modulo
            orig_x = x % old_width
            orig_y = y % old_height
            orig_index = orig_y * old_width + orig_x
            
            # Copy the original tile
            original_tile = existing_tiles[orig_index]
            new_tiles.append(original_tile.copy())
    
    # Update tilemap
    tilemap_entity['tiles'] = new_tiles
    tilemap_entity['width'] = new_width
    tilemap_entity['height'] = new_height
    
    print(f"Expanded tilemap from {old_width}x{old_height} to {new_width}x{new_height}")
    print(f"Tile count: {len(existing_tiles)} -> {len(new_tiles)}")
    
    # Save scene file
    with open(scene_file, 'w') as f:
        json.dump(scene, f, indent=2)
    
    return True

if __name__ == "__main__":
    scene_file = "projects/Celeste Demo/scenes/main.json"
    
    if expand_tilemap(scene_file):
        print("Successfully expanded tilemap!")
    else:
        print("Failed to expand tilemap!")
        sys.exit(1)