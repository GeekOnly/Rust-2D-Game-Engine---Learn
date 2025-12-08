# วิธีเพิ่ม HUD Controller ใน Scene

## ขั้นตอนที่ 1: เปิด Scene ใน Editor

1. เปิด Celeste Demo project
2. โหลด scene `scenes/main.json`

## ขั้นตอนที่ 2: สร้าง HUD Controller Entity

### วิธีที่ 1: ใช้ Editor (แนะนำ)

1. **สร้าง Empty Entity**:
   - คลิกขวาใน Hierarchy panel
   - เลือก "Create Empty"
   - ตั้งชื่อเป็น "HUD Controller"

2. **เพิ่ม Script Component**:
   - เลือก "HUD Controller" entity
   - ใน Inspector panel คลิก "Add Component"
   - เลือก "Script"
   - ตั้งค่า Script Name เป็น `hud_controller`
   - เปิดใช้งาน (Enabled = true)

3. **บันทึก Scene**:
   - กด Ctrl+S หรือ File > Save Scene

### วิธีที่ 2: แก้ไข JSON โดยตรง

เพิ่ม entity ใหม่ใน `scenes/main.json`:

```json
{
  "scripts": [
    [
      11,
      {
        "script_name": "player_controller",
        "enabled": true,
        "parameters": { ... }
      }
    ],
    [
      0,
      {
        "script_name": "camera_follow_simple",
        "enabled": true,
        "parameters": { ... }
      }
    ],
    // เพิ่มตรงนี้ 👇
    [
      490,
      {
        "script_name": "hud_controller",
        "enabled": true,
        "parameters": {}
      }
    ]
  ],
  "active": [
    // ... existing entities ...
    [490, true]  // เพิ่มตรงนี้
  ],
  "names": [
    // ... existing names ...
    [490, "HUD Controller"]  // เพิ่มตรงนี้
  ]
}
```

**หมายเหตุ**: ใช้ entity ID ที่ไม่ซ้ำกับที่มีอยู่ (ดูจาก `next_entity` ใน scene file)

## ขั้นตอนที่ 3: ทดสอบ

1. **กด Play** ใน Editor
2. **ตรวจสอบ Console** ควรเห็นข้อความ:
   ```
   HUD Controller: Starting...
   HUD Controller: HUD loaded and activated
   ```
3. **ดู Game View** ควรเห็น HUD แสดงผล:
   - FPS counter (บนขวา)
   - Position/Velocity debug (บนขวา)
   - Grounded indicator (ล่างซ้าย)
   - Wall slide indicator (ล่างซ้าย)
   - Dashing indicator (กลางจอ)
   - Dash status (บนซ้าย)

## สิ่งที่ HUD Controller ทำ

### 1. โหลดและเปิดใช้งาน HUD
```lua
UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")
UI.activate_prefab(..., "celeste_hud")
```

### 2. อัพเดท FPS Counter
- คำนวณ FPS ทุก 0.5 วินาที
- แสดงผลที่ `fps_counter` element

### 3. อัพเดท Position/Velocity Debug
- อ่านตำแหน่งและความเร็วของ Player
- แสดงผลที่ `position_debug` และ `velocity_debug`

### 4. อัพเดท State Indicators
- **Grounded**: แสดงเมื่อ player อยู่บนพื้น
- **Wall Slide**: แสดงเมื่อ player ชิดกำแพง
- **Dashing**: แสดงเมื่อ player กำลัง dash

### 5. อัพเดท Dash Indicator
- แสดง "Dash: Ready" เมื่อใช้ dash ได้
- แสดง "Dash: Used" เมื่อใช้ dash ไปแล้ว
- เปลี่ยนสีตามสถานะ

## ข้อจำกัดปัจจุบัน

HUD controller script ใช้ helper functions ที่ยังเป็น placeholders:
- `GetEntityByTag()` - hardcoded เป็น entity 11
- `GetTransform()` - ยังไม่ได้ implement
- `GetVelocity()` - ยังไม่ได้ implement
- `GetScript()` - ยังไม่ได้ implement
- `GetScriptParameter()` - ยังไม่ได้ implement

**ผลลัพธ์**: 
- ✅ FPS counter ทำงานได้
- ❌ Position/Velocity debug ยังไม่ทำงาน (ต้อง implement helper functions)
- ❌ State indicators ยังไม่ทำงาน (ต้อง implement helper functions)
- ❌ Dash indicator ยังไม่ทำงาน (ต้อง implement helper functions)

## ขั้นตอนต่อไป

เพื่อให้ HUD ทำงานเต็มรูปแบบ ต้อง implement Lua helper functions:

1. **GetEntityByTag(tag)** - หา entity จาก tag
2. **GetTransform(entity)** - อ่าน transform component
3. **GetVelocity(entity)** - อ่าน velocity component
4. **GetScript(entity)** - ตรวจสอบว่ามี script component
5. **GetScriptParameter(entity, name)** - อ่านค่า parameter จาก script

หรือแก้ไข `hud_controller.lua` ให้ใช้ global variables ที่ player_controller ตั้งไว้แทน

## ตัวอย่าง: HUD Controller แบบง่าย (ใช้งานได้เลย)

สร้างไฟล์ `scripts/hud_controller_simple.lua`:

```lua
-- Simple HUD Controller (FPS only)
local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"

local frame_count = 0
local elapsed_time = 0
local hud_loaded = false

function Start()
    print("HUD Controller: Starting...")
    UI.load_prefab(hud_prefab_path)
    UI.activate_prefab(hud_prefab_path, hud_instance_name)
    hud_loaded = true
    print("HUD Controller: HUD loaded!")
end

function Update(dt)
    if not hud_loaded then return end
    
    -- Update FPS
    frame_count = frame_count + 1
    elapsed_time = elapsed_time + dt
    
    if elapsed_time >= 0.5 then
        local fps = math.floor(frame_count / elapsed_time)
        UI.set_text(hud_instance_name .. "/fps_counter", "FPS: " .. fps)
        frame_count = 0
        elapsed_time = 0
    end
    
    -- Animate dashing indicator (demo)
    local time = elapsed_time * 2
    if math.floor(time) % 2 == 0 then
        UI.show_element(hud_instance_name .. "/dashing_indicator")
    else
        UI.hide_element(hud_instance_name .. "/dashing_indicator")
    end
end
```

นี่จะแสดง FPS และทำให้ "DASHING!" กระพริบเป็นตัวอย่าง

---

**สถานะ**: ✅ Script พร้อมใช้งาน - เพิ่ม entity และทดสอบได้เลย!
