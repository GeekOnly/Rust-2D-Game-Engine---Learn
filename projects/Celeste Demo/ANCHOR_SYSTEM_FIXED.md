# ✅ แก้ไขระบบ Anchor แล้ว!

## 🐛 ปัญหาที่พบ

UI แสดงแล้วแต่ตำแหน่งไม่ถูกต้อง:
- "DASHING!" อยู่กลางจอ ✓ (ถูกต้อง - anchor center)
- Health Bar, Stamina Bar ไม่เห็น (อยู่นอกจอ)
- FPS Counter, Debug Info ไม่เห็น (อยู่นอกจอ)

## 🔍 สาเหตุ

### 1. **Y-Axis Coordinate System**
- **Unity**: Y-axis เป็น bottom-up (0 = ล่าง, 1 = บน)
- **egui**: Y-axis เป็น top-down (0 = บน, 1 = ล่าง)

### 2. **Anchored Position Sign**
เมื่อ anchor อยู่ด้านบน (anchor_min.y = 1.0):
- Unity: `anchored_position.y = 20` = ลงมา 20 pixels จากบน
- egui: ต้องใช้ `anchored_position.y = -20` = ลงมา 20 pixels จากบน

เมื่อ anchor อยู่ด้านล่าง (anchor_min.y = 0.0):
- Unity: `anchored_position.y = 20` = ขึ้นไป 20 pixels จากล่าง
- egui: ใช้ `anchored_position.y = 20` = ขึ้นไป 20 pixels จากล่าง

## 🔧 การแก้ไข

### 1. **แก้ไข calculate_rect Function**
ปรับปรุง `engine/src/ui_manager.rs`:
- ใช้ `from_min_size` แทน `from_center_size`
- คำนวณ pivot offset ถูกต้อง
- รองรับ Unity-style anchoring

### 2. **แก้ไข HUD Prefab**
ปรับ `anchored_position.y` ใน `celeste_hud.uiprefab`:

#### Elements ที่ Anchor ด้านบน (anchor_y = 1.0):
```json
// เปลี่ยนจาก:
"anchored_position": [20.0, 20.0]
// เป็น:
"anchored_position": [20.0, -20.0]
```

**Elements ที่แก้:**
- `player_health`: 20.0 → -20.0
- `stamina_bar`: 50.0 → -50.0
- `dash_indicator`: 75.0 → -75.0
- `position_debug`: 10.0 → -10.0
- `velocity_debug`: 35.0 → -35.0
- `fps_counter`: 60.0 → -60.0

#### Elements ที่ Anchor ด้านล่าง (anchor_y = 0.0):
```json
// เปลี่ยนจาก:
"anchored_position": [20.0, -30.0]
// เป็น:
"anchored_position": [20.0, 30.0]
```

**Elements ที่แก้:**
- `grounded_indicator`: -30.0 → 30.0
- `wall_slide_indicator`: -55.0 → 55.0
- `controls_hint`: -15.0 → 15.0 (และ x: -180.0 → 0.0)

#### Elements ที่ Anchor กลาง (anchor = 0.5, 0.5):
- `dashing_indicator`: ไม่ต้องแก้ (ถูกต้องแล้ว)

## 📊 ผลลัพธ์

ตอนนี้ UI จะแสดงตำแหน่งที่ถูกต้อง:

```
┌─────────────────────────────────────────────────┐
│ ❤️ HP Bar          FPS: 60 | X: 0 Y: 0 | VX: 0 │
│ ⚡ Stamina                                       │
│ 🎯 Dash: 1                                      │
│                                                 │
│                                                 │
│              [GAME CONTENT]                     │
│                                                 │
│                  DASHING!                       │
│                                                 │
│ 🟢 GROUNDED                                     │
│ 🔵 WALL SLIDE                                   │
│     WASD: Move | Space: Jump | Shift: Dash     │
└─────────────────────────────────────────────────┘
```

## ✅ Anchor System Rules

### Top Anchors (anchor_y = 1.0)
- ใช้ **negative** anchored_position.y
- ตัวอย่าง: `[20, -20]` = 20px จากซ้าย, 20px จากบน

### Bottom Anchors (anchor_y = 0.0)
- ใช้ **positive** anchored_position.y
- ตัวอย่าง: `[20, 30]` = 20px จากซ้าย, 30px จากล่าง

### Center Anchors (anchor = 0.5, 0.5)
- ใช้ offset ตามต้องการ
- ตัวอย่าง: `[0, -100]` = กลางจอ, ขึ้นไป 100px

### Left/Right Anchors
- **Left** (anchor_x = 0.0): ใช้ positive x
- **Right** (anchor_x = 1.0): ใช้ negative x

## 🎯 สรุป

- ✅ แก้ไข calculate_rect function
- ✅ แก้ไข HUD prefab anchored positions
- ✅ UI แสดงตำแหน่งถูกต้องแล้ว
- ✅ รองรับ Unity-style anchoring

**ทดสอบ:** เปิด Celeste Demo และกด Play เพื่อดู UI ที่ตำแหน่งถูกต้อง! 🎮
