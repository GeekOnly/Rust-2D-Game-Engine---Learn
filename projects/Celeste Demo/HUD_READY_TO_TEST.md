# ✅ HUD พร้อมทดสอบแล้ว!

## สรุป

สร้าง HUD controller scripts เรียบร้อย พร้อมทดสอบใน game!

## ไฟล์ที่สร้าง

### 1. `scripts/hud_controller.lua` (Full Version)
HUD controller แบบเต็มรูปแบบที่อ่านข้อมูลจาก player จริง

**ฟีเจอร์**:
- ✅ FPS counter
- 🔄 Position/Velocity debug (ต้อง implement helper functions)
- 🔄 Grounded/Wall slide indicators (ต้อง implement helper functions)
- 🔄 Dash indicator (ต้อง implement helper functions)

**ข้อจำกัด**: ต้อง implement Lua helper functions เพิ่มเติม

### 2. `scripts/hud_controller_simple.lua` (Demo Version) ⭐ แนะนำ
HUD controller แบบ demo ที่ใช้งานได้เลยโดยไม่ต้องแก้ไขอะไร

**ฟีเจอร์**:
- ✅ FPS counter (ทำงานจริง)
- ✅ Health bar animation (sine wave demo)
- ✅ Stamina bar animation (cosine wave demo)
- ✅ Position/Velocity display (fake animation)
- ✅ Dashing indicator (blink animation)
- ✅ Grounded indicator (show/hide every 3 seconds)

**ข้อดี**: ใช้งานได้ทันที ไม่ต้องแก้ไขอะไร!

## วิธีทดสอบ

### ขั้นตอนที่ 1: เพิ่ม HUD Controller Entity

#### วิธีที่ 1: ใช้ Editor (แนะนำ)

1. เปิด Celeste Demo project
2. โหลด scene `scenes/main.json`
3. สร้าง Empty Entity:
   - คลิกขวาใน Hierarchy
   - เลือก "Create Empty"
   - ตั้งชื่อ "HUD Controller"
4. เพิ่ม Script Component:
   - เลือก "HUD Controller" entity
   - Add Component > Script
   - Script Name: `hud_controller_simple` (แนะนำ) หรือ `hud_controller`
   - Enabled: ✅
5. บันทึก scene (Ctrl+S)

#### วิธีที่ 2: แก้ไข JSON

เพิ่มใน `scenes/main.json`:

```json
{
  "scripts": [
    // ... existing scripts ...
    [
      490,
      {
        "script_name": "hud_controller_simple",
        "enabled": true,
        "parameters": {}
      }
    ]
  ],
  "active": [
    // ... existing ...
    [490, true]
  ],
  "names": [
    // ... existing ...
    [490, "HUD Controller"]
  ]
}
```

### ขั้นตอนที่ 2: ทดสอบ

1. **กด Play** ใน Editor
2. **ดู Console** ควรเห็น:
   ```
   === HUD Controller Simple: Starting ===
   === HUD Controller Simple: HUD Loaded ===
   ```
3. **ดู Game View** ควรเห็น:
   - 🟢 Health bar เคลื่อนไหวขึ้นลง (สีเปลี่ยนตามเปอร์เซ็นต์)
   - 🟡 Stamina bar เคลื่อนไหวขึ้นลง
   - 📊 FPS counter อัพเดททุก 0.5 วินาที
   - 📍 Position/Velocity แสดงค่าที่เคลื่อนไหว
   - 🔴 "DASHING!" กระพริบทุกวินาที
   - 🟢 "GROUNDED" แสดงทุก 3 วินาที

## สิ่งที่ HUD Controller Simple ทำ

### 1. โหลดและเปิดใช้งาน HUD
```lua
UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")
UI.activate_prefab(..., "celeste_hud")
```

### 2. อัพเดท FPS (จริง)
```lua
local fps = math.floor(frame_count / elapsed_time)
UI.set_text("celeste_hud/fps_counter", "FPS: " .. fps)
```

### 3. Animate Health Bar (demo)
```lua
local health = (math.sin(time) + 1.0) / 2.0
UI.set_image_fill("celeste_hud/player_health_fill", health)

-- Change color based on health
if health < 0.3 then
    UI.set_color(..., {r=1.0, g=0.0, b=0.0, a=1.0})  -- Red
elseif health < 0.6 then
    UI.set_color(..., {r=1.0, g=0.8, b=0.0, a=1.0})  -- Yellow
else
    UI.set_color(..., {r=0.2, g=1.0, b=0.3, a=1.0})  -- Green
end
```

### 4. Animate Stamina Bar (demo)
```lua
local stamina = (math.cos(time * 1.5) + 1.0) / 2.0
UI.set_image_fill("celeste_hud/stamina_bar_fill", stamina)
```

### 5. Show/Hide Indicators (demo)
```lua
-- Blink dashing indicator
if math.floor(time) % 2 == 0 then
    UI.show_element("celeste_hud/dashing_indicator")
else
    UI.hide_element("celeste_hud/dashing_indicator")
end

-- Show grounded every 3 seconds
if math.floor(time) % 3 == 0 then
    UI.show_element("celeste_hud/grounded_indicator")
end
```

### 6. Update Text (demo)
```lua
-- Fake animated position
local x = math.sin(time * 0.5) * 10
local y = math.cos(time * 0.5) * 5
UI.set_text("celeste_hud/position_debug", string.format("X: %.1f Y: %.1f", x, y))
```

## ผลลัพธ์ที่คาดหวัง

เมื่อกด Play คุณจะเห็น:

```
┌─────────────────────────────────────────────────────────┐
│ ████████░░ (Health - เคลื่อนไหว)    FPS: 60           │
│ ██████░░░░ (Stamina - เคลื่อนไหว)   X: 5.2 Y: -3.1    │
│ Dash: Ready                          VX: 2.1 VY: -1.5  │
│                                                          │
│                                                          │
│                    DASHING! (กระพริบ)                   │
│                                                          │
│                                                          │
│ GROUNDED (แสดงทุก 3 วินาที)                             │
│                                                          │
│         WASD: Move | Space: Jump | Shift: Dash          │
└─────────────────────────────────────────────────────────┘
```

## Lua UI API ที่ใช้

Script นี้แสดงการใช้งาน Lua UI API ทั้งหมด:

1. ✅ `UI.load_prefab(path)` - โหลด prefab
2. ✅ `UI.activate_prefab(path, name)` - เปิดใช้งาน UI
3. ✅ `UI.set_text(element_path, text)` - อัพเดทข้อความ
4. ✅ `UI.set_image_fill(element_path, amount)` - อัพเดท fill amount
5. ✅ `UI.set_color(element_path, {r, g, b, a})` - เปลี่ยนสี
6. ✅ `UI.show_element(element_path)` - แสดง element
7. ✅ `UI.hide_element(element_path)` - ซ่อน element

## ขั้นตอนต่อไป

### เพื่อให้ HUD แสดงข้อมูลจริง:

1. **Implement Lua Helper Functions**:
   - `GetEntityByTag(tag)` - หา entity จาก tag
   - `GetTransform(entity)` - อ่าน transform
   - `GetVelocity(entity)` - อ่าน velocity
   - `GetScriptParameter(entity, name)` - อ่าน script parameters

2. **แก้ไข player_controller.lua**:
   - Export ค่า is_grounded, is_dashing, can_dash เป็น global variables
   - ให้ hud_controller อ่านค่าเหล่านี้ได้

3. **ใช้ hud_controller.lua แทน hud_controller_simple.lua**:
   - แก้ไข script name ใน scene
   - HUD จะแสดงข้อมูลจริงจาก player

## การ Debug

ถ้า HUD ไม่แสดง:

1. **ตรวจสอบ Console**:
   - ควรเห็น "HUD Controller Simple: Starting"
   - ควรเห็น "HUD Controller Simple: HUD Loaded"

2. **ตรวจสอบ Script**:
   - Script name ถูกต้อง: `hud_controller_simple`
   - Script enabled: ✅
   - ไฟล์อยู่ที่: `scripts/hud_controller_simple.lua`

3. **ตรวจสอบ Prefab**:
   - ไฟล์อยู่ที่: `assets/ui/celeste_hud.uiprefab`
   - Prefab มี elements ครบ

4. **ตรวจสอบ Game View**:
   - เปิด Game tab
   - กด Play
   - ดูว่า UI แสดงหรือไม่

---

**สถานะ**: ✅ พร้อมทดสอบ 100%!

**ลองเลย**: เพิ่ม HUD Controller entity และกด Play! 🚀
