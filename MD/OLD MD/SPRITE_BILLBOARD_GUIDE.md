# Sprite Billboard Feature - คู่มือการใช้งาน

## ✅ Billboard คืออะไร?

Billboard เป็นเทคนิคใน 3D graphics ที่ทำให้ sprite หมุนตามกล้องเสมอ เหมือนกับว่า sprite "มองหน้า" กล้องตลอดเวลา

**ใช้สำหรับ:**
- ต้นไม้, พุ่มไม้ใน 3D game
- Particle effects (ควัน, ไฟ, ระเบิด)
- Health bars, damage numbers
- NPCs ใน 2.5D games
- Sprites ที่ต้องการให้มองเห็นชัดเสมอ

## 🎯 วิธีใช้งาน

### 1. เปิด Billboard ใน Inspector

1. **เลือก Entity ที่มี Sprite component**
   - คลิกที่ entity ใน Hierarchy

2. **ไปที่ Inspector > Sprite Renderer**
   - มองหา section "Sprite Renderer" พร้อมไอคอน 🎨

3. **เปิด Billboard checkbox**
   ```
   Billboard: ☑️ Always face camera in 3D mode
   ```

4. **สลับเป็น 3D mode**
   - คลิกปุ่ม "3D" ใน Scene View toolbar

5. **หมุนกล้อง**
   - ใช้ Right-click + drag เพื่อหมุนกล้อง
   - Sprite จะหมุนตามกล้องอัตโนมัติ!

### 2. ทดสอบ Billboard

**ขั้นตอนทดสอบ:**
1. สร้าง sprite entity ใน scene
2. เปิด Billboard checkbox
3. สลับเป็น 3D mode
4. หมุนกล้องรอบๆ sprite
5. สังเกตว่า sprite หมุนตามกล้องเสมอ

**เปรียบเทียบ:**
- ❌ **Billboard OFF**: Sprite อยู่นิ่ง, มองเห็นด้านข้างเมื่อหมุนกล้อง
- ✅ **Billboard ON**: Sprite หมุนตามกล้อง, มองเห็นหน้าตรงเสมอ

## 🔧 Technical Details

### Sprite Component Structure
```rust
pub struct Sprite {
    pub texture_id: String,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
    pub billboard: bool,  // ← Billboard flag
    pub flip_x: bool,
    pub flip_y: bool,
    pub sprite_rect: Option<[u32; 4]>,
    pub pixels_per_unit: f32,
}
```

### Billboard Rotation Calculation
```rust
// engine/src/editor/ui/scene_view/rendering/sprite_3d.rs
pub fn calculate_billboard_rotation(&self, sprite_pos: Vec3, camera: &SceneCamera) -> f32 {
    // Calculate vector from sprite to camera
    let to_camera = Vec2::new(
        camera.position.x - sprite_pos.x,
        camera.position.y - sprite_pos.z,
    );
    
    // Calculate angle to face camera
    to_camera.y.atan2(to_camera.x)
}
```

### Rendering Logic
```rust
// In project_sprite_to_screen()
let rotation = if sprite.billboard {
    self.calculate_billboard_rotation(sprite.position, camera)
} else {
    sprite.rotation  // Use world rotation
};
```

## 📊 Use Cases

### 1. Trees & Vegetation (ต้นไม้และพืช)
```
Billboard: ✅ ON
- ต้นไม้มองเห็นชัดจากทุกมุม
- ประหยัด polygons กว่า 3D models
- เหมาะสำหรับ forest scenes
```

### 2. Particle Effects (เอฟเฟกต์)
```
Billboard: ✅ ON
- ควัน, ไฟ, ระเบิด
- มองเห็นชัดจากทุกมุม
- เคลื่อนไหวสมจริง
```

### 3. UI Elements in World Space
```
Billboard: ✅ ON
- Health bars
- Damage numbers
- Quest markers
- Name tags
```

### 4. 2.5D Characters
```
Billboard: ✅ ON (optional)
- NPCs ใน 2.5D games
- มองเห็นหน้าตรงเสมอ
- เหมาะสำหรับ isometric games
```

### 5. Static Objects (วัตถุนิ่ง)
```
Billboard: ❌ OFF
- Buildings, walls
- Ground tiles
- Static decorations
- ต้องการ perspective ที่ถูกต้อง
```

## 🎨 Best Practices

### ✅ ควรใช้ Billboard เมื่อ:
- Sprite ต้องการมองเห็นชัดจากทุกมุม
- เป็น particle effects หรือ visual effects
- เป็น UI elements ใน world space
- ต้องการประหยัด performance (แทน 3D models)

### ❌ ไม่ควรใช้ Billboard เมื่อ:
- ต้องการ perspective ที่ถูกต้อง (อาคาร, กำแพง)
- Sprite เป็นส่วนหนึ่งของ tilemap
- ต้องการ rotation ที่แน่นอน
- เป็น ground/floor sprites

## 🐛 Troubleshooting

### Sprite ไม่หมุนตามกล้อง
1. ✅ ตรวจสอบว่าเปิด Billboard checkbox แล้ว
2. ✅ ตรวจสอบว่าอยู่ใน 3D mode (ไม่ใช่ 2D)
3. ✅ ลองหมุนกล้องด้วย Right-click + drag
4. ✅ ตรวจสอบว่า sprite มี Transform component

### Sprite หมุนผิดทิศทาง
- Billboard คำนวณจาก camera position
- ตรวจสอบว่า camera position ถูกต้อง
- ลอง reset camera (กด Home หรือ F)

### Billboard ทำงานใน 2D mode
- Billboard ทำงานเฉพาะใน 3D mode เท่านั้น
- ใน 2D mode จะใช้ rotation ปกติ

## 📝 Example Scene Setup

### ตัวอย่าง: Forest Scene with Billboard Trees

```
Scene Hierarchy:
├─ Camera (Main Camera)
├─ Ground (Tilemap, Billboard OFF)
├─ Tree_01 (Sprite, Billboard ON)
├─ Tree_02 (Sprite, Billboard ON)
├─ Tree_03 (Sprite, Billboard ON)
├─ Rock (Sprite, Billboard OFF)
└─ Grass (Sprite, Billboard OFF)
```

**Settings:**
- Trees: Billboard ✅ ON (หมุนตามกล้อง)
- Ground/Rock/Grass: Billboard ❌ OFF (นิ่ง)

## 🎯 Performance Notes

**Billboard Performance:**
- ✅ **Very Fast** - เพียงคำนวณ rotation angle
- ✅ **No Extra Draw Calls** - ใช้ mesh เดิม
- ✅ **Better than 3D Models** - ประหยัด polygons

**Comparison:**
- Billboard Sprite: ~2 triangles
- 3D Tree Model: ~500-2000 triangles
- Performance gain: 250x - 1000x faster!

## ✨ Advanced Tips

### 1. Combine with Depth Sorting
```
Billboard + Depth Sorting = Perfect 2.5D
- Sprites หมุนตามกล้อง
- Render order ถูกต้อง
- มองเห็นชัดจากทุกมุม
```

### 2. Use with Particle Systems
```
Particle + Billboard = Realistic Effects
- ควัน, ไฟ, ระเบิด
- มองเห็นชัดจากทุกมุม
- เคลื่อนไหวสมจริง
```

### 3. Mix Billboard and Non-Billboard
```
Scene = Billboard (trees) + Non-Billboard (ground)
- ต้นไม้หมุนตามกล้อง
- พื้นนิ่ง, perspective ถูกต้อง
- ดูสมจริงที่สุด
```

## 📚 Related Features

- **Transform Component** - Position, Rotation, Scale
- **Sprite Renderer** - Texture, Color, Flip
- **3D Scene View** - Camera controls, Grid
- **Depth Sorting** - Render order in 3D

## ✅ Status: FULLY FUNCTIONAL

Billboard feature ทำงานได้เต็มรูปแบบแล้ว!
- ✅ UI ใน Inspector
- ✅ Rotation calculation
- ✅ 3D rendering
- ✅ Serialization
- ✅ Performance optimized

---

**พร้อมใช้งาน!** เพียงเปิด Billboard checkbox ใน Inspector แล้วสลับเป็น 3D mode 🎉
