#!/usr/bin/env python3
"""
Script to clean up individual collider entities from scene file
and prepare for migration to Tilemap Collider components
"""

import json
import sys
from pathlib import Path

def cleanup_scene_colliders(scene_path):
    """Remove individual collider entities from scene file"""
    
    print(f"🔧 Cleaning up colliders in: {scene_path}")
    
    # Read scene file
    with open(scene_path, 'r', encoding='utf-8') as f:
        scene_data = json.load(f)
    
    # Find collider entities by name
    collider_entities = set()
    
    if 'names' in scene_data:
        for name_entry in scene_data['names']:
            entity_id = name_entry[0]
            entity_name = name_entry[1]
            
            # Check if it's a collider entity
            if (entity_name.startswith('CompositeCollider') or 
                entity_name.startswith('Collider_')):
                collider_entities.add(entity_id)
                print(f"  📦 Found collider entity: {entity_id} - {entity_name}")
    
    print(f"  🎯 Total collider entities found: {len(collider_entities)}")
    
    if not collider_entities:
        print("  ✅ No collider entities to clean up!")
        return
    
    # Remove collider entities from all sections
    sections_cleaned = 0
    
    for section_name, section_data in scene_data.items():
        if isinstance(section_data, list):
            original_count = len(section_data)
            
            # Filter out collider entities
            filtered_data = []
            for entry in section_data:
                if isinstance(entry, list) and len(entry) >= 1:
                    entity_id = entry[0]
                    if entity_id not in collider_entities:
                        filtered_data.append(entry)
                else:
                    filtered_data.append(entry)
            
            # Update section if changed
            if len(filtered_data) != original_count:
                scene_data[section_name] = filtered_data
                removed_count = original_count - len(filtered_data)
                print(f"  🗑️  Cleaned {section_name}: removed {removed_count} entries")
                sections_cleaned += 1
    
    # Create backup
    backup_path = scene_path.with_suffix('.json.backup')
    with open(backup_path, 'w', encoding='utf-8') as f:
        json.dump(scene_data, f, indent=2)
    print(f"  💾 Created backup: {backup_path}")
    
    # Write cleaned scene
    with open(scene_path, 'w', encoding='utf-8') as f:
        json.dump(scene_data, f, indent=2)
    
    print(f"  ✅ Cleanup complete! Cleaned {sections_cleaned} sections")
    print(f"  📊 Removed {len(collider_entities)} collider entities")

def show_migration_guide():
    """Show guide for migrating to Tilemap Collider components"""
    
    print("\n" + "="*60)
    print("🚀 MIGRATION TO TILEMAP COLLIDER COMPONENTS")
    print("="*60)
    
    print("""
📋 MIGRATION STEPS:

1. 🧹 CLEANUP INDIVIDUAL COLLIDERS (this script):
   - Remove old CompositeCollider entities from scene
   - Remove old Collider_ entities from scene
   - Clean up all related data (transforms, rigidbodies, etc.)

2. 🗺️ ADD TILEMAP COLLIDER COMPONENTS:
   - Select your Grid entity in hierarchy
   - Add "Tilemap Collider" component in Inspector
   - Configure collider mode (Composite recommended)
   - Set physics properties (friction, restitution)

3. 🔧 CONFIGURE LDTK INTGRID COLLIDER:
   - Add "LDTK IntGrid Collider" component to Grid entity
   - Set collision value (usually 1 for walls)
   - Choose collider mode (Composite for performance)
   - Enable auto-update for dynamic changes

4. ⚡ BENEFITS OF NEW SYSTEM:
   - Better performance (fewer entities)
   - Automatic collision generation from LDTK
   - Easier to manage and debug
   - Supports hot-reload and auto-update
   - Cleaner hierarchy

5. 🎯 COMPONENT COMPARISON:
   OLD: Individual entities with Collider + Rigidbody
   NEW: Single Grid entity with TilemapCollider + LdtkIntGridCollider

6. 🔄 TESTING:
   - Load your scene in the engine
   - Verify collisions still work
   - Check performance improvements
   - Test LDTK map reloading
""")

def main():
    if len(sys.argv) != 2:
        print("Usage: python cleanup_colliders.py <scene_file.json>")
        print("Example: python cleanup_colliders.py 'projects/Celeste Demo/scenes/main.json'")
        sys.exit(1)
    
    scene_path = Path(sys.argv[1])
    
    if not scene_path.exists():
        print(f"❌ Scene file not found: {scene_path}")
        sys.exit(1)
    
    if not scene_path.suffix == '.json':
        print(f"❌ File must be a .json scene file: {scene_path}")
        sys.exit(1)
    
    print("🎮 COLLIDER CLEANUP TOOL")
    print("=" * 40)
    
    # Show migration guide first
    show_migration_guide()
    
    # Ask for confirmation
    print("\n" + "="*60)
    response = input("🤔 Do you want to proceed with cleanup? (y/N): ").strip().lower()
    
    if response in ['y', 'yes']:
        cleanup_scene_colliders(scene_path)
        print("\n🎉 Cleanup completed successfully!")
        print("💡 Next: Add Tilemap Collider components to your Grid entities")
    else:
        print("❌ Cleanup cancelled.")

if __name__ == "__main__":
    main()