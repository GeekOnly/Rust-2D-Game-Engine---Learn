# How to Use LdtkMap Component

The LdtkMap component integration is now complete and working! Here's how to use it:

## 🔧 **FIXED**: Entity หายไปหลังจากกด Load Map
**ปัญหาได้รับการแก้ไขแล้ว!** ตอนนี้เมื่อคุณกด Load Map:
- ✅ Entity เดิมจะไม่หายไป
- ✅ Grid component จะถูกเพิ่มให้ entity เดิม
- ✅ Tilemap layers และ colliders จะถูกสร้างเป็น children ของ entity เดิม
- ✅ ไม่มีการสร้าง entity ใหม่แทนที่

## ✅ Available LDTK Files in Project

The following LDTK files are available in your project:
- `levels/Level_01.ldtk`
- `levels/simple_level.ldtk` 
- `tilemaps/Level_01.ldtk`

## 🎯 How to Add and Use LdtkMap Component

### Step 1: Add LdtkMap Component
1. Select an entity in the hierarchy (or create a new empty entity)
2. In the Inspector, click **"➕ Add Component"**
3. Under **"🗺️ Tilemap"** section, click **"LDTK Map"**

### Step 2: Load an LDTK File
1. In the LdtkMap component UI:
   - **File Path**: Enter the relative path to your LDTK file (e.g., `levels/Level_01.ldtk`)
   - Or click the **📁 Browse** button to select a file
2. Click **"🔄 Load Map"** button

### Step 3: What Happens Automatically
When you load an LDTK map, the system automatically:
- ✅ Creates a **Grid** entity as the parent
- ✅ Creates **Tilemap** entities for each layer as children
- ✅ Generates **Collider** entities from IntGrid layers
- ✅ Sets up proper hierarchy: `Grid → Tilemap Layers + Colliders`

## 🔧 Component Features

### LdtkMap Component UI
- **File Path Input**: Enter or browse for LDTK files
- **Load Map Button**: Loads the LDTK file and creates tilemap hierarchy
- **Clear Button**: Resets the component data
- **Auto Reload**: Automatically reload when file changes
- **Data Display**: Shows loaded map info (identifier, size, levels, etc.)

### TilemapCollider Component
- **Mode**: Individual, Composite, Polygon, or None
- **Physics Properties**: Friction, Restitution
- **Options**: Use Composite, Is Trigger, Auto Update

### LdtkIntGridCollider Component  
- **Collision Value**: Which IntGrid value represents collision (default: 1)
- **Mode**: Collider generation mode
- **Physics Properties**: Friction, Restitution
- **Options**: Is Trigger, Auto Update

## 🚀 Quick Start Example

1. Create a new empty entity
2. Add **LdtkMap** component
3. Set file path to: `levels/simple_level.ldtk`
4. Click **"🔄 Load Map"**
5. The system will automatically create the complete tilemap hierarchy!

## ⚠️ Error Resolution

If you see the error: `"Failed to read LDTK file: The system cannot find the path specified"`

**Solution**: Make sure the file path is correct and relative to the project root:
- ✅ Correct: `levels/Level_01.ldtk`
- ❌ Wrong: `Level_01.ldtk` (missing directory)
- ❌ Wrong: Empty file path

## 🎮 Integration with Map Manager

The LdtkMap component uses the Map Manager system which provides:
- Hot-reload functionality (auto-reload when files change)
- Collider generation from IntGrid layers
- Proper Grid + Tilemap + Collider hierarchy creation
- Integration with the tilemap rendering system

## ✨ Component Categories in Add Component Menu

Components are organized in logical categories:
- **🗺️ Tilemap**: LdtkMap
- **⚙️ Physics**: TilemapCollider, LdtkIntGridCollider  
- **🎨 Rendering**: Sprite Renderer, Mesh Renderer
- **📜 Other**: Camera, Script, etc.

The LdtkMap integration is now complete and ready to use!