# Component Management System (Unity-like)

ระบบจัดการ Component แบบ Unity สำหรับ XS Game Engine

## 🎯 ภาพรวม

ระบบนี้ช่วยให้คุณสามารถจัดการ Component ของ Entity ได้แบบ Unity:
- **Add Component** - เพิ่ม Component ใหม่
- **Remove Component** - ลบ Component ที่ไม่ต้องการ
- **Get Component** - ดึงข้อมูล Component
- **Has Component** - ตรวจสอบว่ามี Component หรือไม่

## 📦 Component Types

### 1. Transform (จำเป็น - ไม่สามารถลบได้)
```rust
ComponentType::Transform
```
- Position (X, Y, Z)
- Rotation (X, Y, Z)
- Scale (X, Y, Z)

### 2. Sprite Renderer
```rust
ComponentType::Sprite
```
- Texture ID
- Width, Height
- Color (RGBA)
- Billboard (สำหรับ 3D mode)

### 3. Box Collider 2D
```rust
ComponentType::BoxCollider
```
- Width
- Height

### 4. Rigidbody 2D
```rust
ComponentType::Rigidbody
```
- Velocity X
- Velocity Y

### 5. Mesh Renderer (3D)
```rust
ComponentType::Mesh
```
- Mesh Type (Cube, Sphere, Cylinder, Plane, Capsule)
- Color (RGBA)

### 6. Camera
```rust
ComponentType::Camera
```
- Projection (Orthographic/Perspective)
- FOV / Orthographic Size
- Near/Far Clip
- Background Color

### 7. Script
```rust
ComponentType::Script
```
- Script Name
- Enabled
- Parameters (Float, Int, String, Bool)

### 8. Tag
```rust
ComponentType::Tag
```
- Player
- Item

## 💻 การใช้งานใน Code

### เพิ่ม Component
```rust
use ecs::{World, ComponentType, ComponentManager};

let mut world = World::new();
let entity = world.spawn();

// เพิ่ม Transform (จำเป็น)
world.add_component(entity, ComponentType::Transform)?;

// เพิ่ม Sprite
world.add_component(entity, ComponentType::Sprite)?;

// เพิ่ม Box Collider
world.add_component(entity, ComponentType::BoxCollider)?;

// เพิ่ม Rigidbody
world.add_component(entity, ComponentType::Rigidbody)?;
```

### ลบ Component
```rust
// ลบ Sprite
world.remove_component(entity, ComponentType::Sprite)?;

// ลบ Collider
world.remove_component(entity, ComponentType::BoxCollider)?;

// ⚠️ ไม่สามารถลบ Transform ได้ (จะ return Error)
world.remove_component(entity, ComponentType::Transform)?; // Error!
```

### ตรวจสอบ Component
```rust
// ตรวจสอบว่ามี Component หรือไม่
if world.has_component(entity, ComponentType::Sprite) {
    println!("Entity has Sprite!");
}

// ดึงรายการ Component ทั้งหมด
let components = world.get_components(entity);
for component_type in components {
    println!("Component: {:?}", component_type);
}

// ดึงรายการ Component ที่สามารถเพิ่มได้
let addable = world.get_addable_components(entity);
for component_type in addable {
    println!("Can add: {:?}", component_type);
}
```

## 🎨 การใช้งานใน Inspector UI

### 1. เพิ่ม Component
1. เลือก Entity ใน Hierarchy
2. คลิกปุ่ม **"➕ Add Component"** ใน Inspector
3. เลือก Component ที่ต้องการจากเมนู:
   - 🎨 **Rendering**: Sprite Renderer, Mesh Renderer
   - ⚙️ **Physics**: Box Collider 2D, Rigidbody 2D
   - 📜 **Other**: Camera, Script, Tag

### 2. แก้ไข Component
- คลิกที่ชื่อ Component เพื่อ Expand/Collapse
- แก้ไขค่าต่างๆ ใน Inspector
- การเปลี่ยนแปลงจะมีผลทันที

### 3. ลบ Component
1. คลิกปุ่ม **"❌ Remove Component"** ใต้ Component ที่ต้องการลบ
2. Component จะถูกลบทันที
3. ⚠️ Transform ไม่สามารถลบได้

## 🔧 ตัวอย่างการใช้งาน

### สร้าง Player Entity
```rust
let mut world = World::new();
let player = world.spawn();

// เพิ่ม Components
world.add_component(player, ComponentType::Transform)?;
world.add_component(player, ComponentType::Sprite)?;
world.add_component(player, ComponentType::BoxCollider)?;
world.add_component(player, ComponentType::Rigidbody)?;
world.add_component(player, ComponentType::Script)?;

// ตั้งค่า Sprite
if let Some(sprite) = world.sprites.get_mut(&player) {
    sprite.texture_id = "player".to_string();
    sprite.width = 40.0;
    sprite.height = 40.0;
    sprite.color = [0.2, 0.6, 1.0, 1.0];
}

// ตั้งค่า Script
if let Some(script) = world.scripts.get_mut(&player) {
    script.script_name = "PlayerController".to_string();
    script.enabled = true;
}
```

### สร้าง Camera Entity
```rust
let camera = world.spawn();

world.add_component(camera, ComponentType::Transform)?;
world.add_component(camera, ComponentType::Camera)?;

// ตั้งค่า Transform
if let Some(transform) = world.transforms.get_mut(&camera) {
    transform.position = [0.0, 0.0, -10.0];
}

// ตั้งค่า Camera
if let Some(cam) = world.cameras.get_mut(&camera) {
    cam.projection = ecs::CameraProjection::Orthographic;
    cam.orthographic_size = 5.0;
}
```

### สร้าง 3D Cube
```rust
let cube = world.spawn();

world.add_component(cube, ComponentType::Transform)?;
world.add_component(cube, ComponentType::Mesh)?;

// ตั้งค่า Mesh
if let Some(mesh) = world.meshes.get_mut(&cube) {
    mesh.mesh_type = ecs::MeshType::Cube;
    mesh.color = [1.0, 0.0, 0.0, 1.0]; // สีแดง
}
```

## 🎯 Best Practices

1. **Transform เป็น Component จำเป็น** - ทุก Entity ควรมี Transform
2. **ใช้ Component Manager** - ใช้ `add_component()` และ `remove_component()` แทนการเข้าถึง HashMap โดยตรง
3. **ตรวจสอบก่อนเพิ่ม** - ใช้ `has_component()` เพื่อตรวจสอบก่อนเพิ่ม Component
4. **จัดกลุ่ม Component** - จัดกลุ่ม Component ที่เกี่ยวข้องกัน (เช่น Sprite + Collider)
5. **ใช้ Prefab** - สร้าง Prefab สำหรับ Entity ที่ใช้บ่อย

## 🔍 Error Handling

```rust
// เพิ่ม Component ที่มีอยู่แล้ว
match world.add_component(entity, ComponentType::Sprite) {
    Ok(_) => println!("Component added!"),
    Err(e) => println!("Error: {}", e), // "Entity already has Sprite"
}

// ลบ Component ที่ไม่มี
match world.remove_component(entity, ComponentType::Sprite) {
    Ok(_) => println!("Component removed!"),
    Err(e) => println!("Error: {}", e), // "Entity does not have Sprite"
}

// ลบ Transform (ไม่อนุญาต)
match world.remove_component(entity, ComponentType::Transform) {
    Ok(_) => println!("Component removed!"),
    Err(e) => println!("Error: {}", e), // "Transform is required and cannot be removed"
}
```

## 📚 API Reference

### ComponentManager Trait

```rust
pub trait ComponentManager {
    /// เพิ่ม Component
    fn add_component(&mut self, entity: Entity, component_type: ComponentType) 
        -> Result<(), String>;
    
    /// ลบ Component
    fn remove_component(&mut self, entity: Entity, component_type: ComponentType) 
        -> Result<(), String>;
    
    /// ตรวจสอบว่ามี Component หรือไม่
    fn has_component(&self, entity: Entity, component_type: ComponentType) -> bool;
    
    /// ดึงรายการ Component ทั้งหมด
    fn get_components(&self, entity: Entity) -> Vec<ComponentType>;
    
    /// ดึงรายการ Component ที่สามารถเพิ่มได้
    fn get_addable_components(&self, entity: Entity) -> Vec<ComponentType>;
}
```

## 🚀 ความแตกต่างจาก Unity

| Feature | Unity | XS Game Engine |
|---------|-------|----------------|
| Add Component | `AddComponent<T>()` | `add_component(entity, ComponentType::T)` |
| Remove Component | `Destroy(component)` | `remove_component(entity, ComponentType::T)` |
| Get Component | `GetComponent<T>()` | `world.sprites.get(&entity)` |
| Has Component | `GetComponent<T>() != null` | `has_component(entity, ComponentType::T)` |
| Required Component | `[RequireComponent]` | Transform เท่านั้น |

## 🎓 ตัวอย่างเพิ่มเติม

ดูตัวอย่างเพิ่มเติมได้ที่:
- `ecs/src/component_manager.rs` - Implementation
- `engine/src/editor/ui/inspector.rs` - UI Integration
- `ecs/tests/` - Unit Tests
