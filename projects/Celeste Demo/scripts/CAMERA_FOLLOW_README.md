# Camera Follow Scripts

มี 3 เวอร์ชันให้เลือกใช้ตามความต้องการ

## 1. Unity-Style Camera Follow ⭐ แนะนำ!

**ไฟล์**: `camera_follow_unity_style.lua`

### Features:
- ✅ ใช้ Entity Reference Parameter (เหมือน Unity's `public GameObject`)
- ✅ ไม่ต้องใช้ Tag หรือค้นหา entity
- ✅ Drag & Drop ใน Inspector
- ✅ Smooth movement
- ✅ Offset ได้

### Parameters:
- `playerTarget` (Entity) - ลาก player entity มาวางที่นี่ใน Inspector
- `smooth_speed` (5.0) - ความเร็วในการติดตาม
- `offset_x` (0.0) - ระยะห่างแนวนอน
- `offset_y` (0.0) - ระยะห่างแนวตั้ง

### การใช้งาน:
1. เลือก Camera entity
2. Add Script component
3. เลือก `camera_follow_unity_style.lua`
4. ใน Inspector ลาก Player entity มาวางที่ `playerTarget`
5. ปรับ parameters ตามต้องการ
6. กด Play

### ทำไมต้องใช้เวอร์ชันนี้?
- 🎯 ชัดเจนที่สุด - ไม่ต้องพึ่ง tag
- 🎯 ง่ายที่สุด - แค่ drag & drop
- 🎯 เหมือน Unity - คุ้นเคยสำหรับคนที่มาจาก Unity
- 🎯 เปลี่ยน target ได้ง่าย - แค่เลือกใหม่ใน Inspector

---

## 2. Simple Camera Follow

**ไฟล์**: `camera_follow_simple.lua`

### Features:
- ✅ ติดตาม player อย่างง่าย (ใช้ Tag)
- ✅ Smooth movement
- ✅ Offset ได้

### Parameters:
- `smooth_speed` (5.0) - ความเร็วในการติดตาม
- `offset_x` (0.0) - ระยะห่างแนวนอน
- `offset_y` (0.0) - ระยะห่างแนวตั้ง

### การใช้งาน:
1. เลือก Camera entity
2. Add Script component
3. เลือก `camera_follow_simple.lua`
4. ตรวจสอบว่า Player มี tag "Player"
5. กด Play

---

## 3. Advanced Camera Follow (แนะนำสำหรับ platformer)

**ไฟล์**: `camera_follow_advanced.lua`

### Features:
- ✅ Smooth movement
- ✅ Dead zone (พื้นที่ที่ camera ไม่เคลื่อนที่)
- ✅ Look-ahead (มองไปข้างหน้าตามทิศทางที่เคลื่อนที่)
- ✅ Camera bounds (จำกัดขอบเขต)

### Parameters:

#### Basic:
- `smooth_speed` (5.0) - ความเร็วติดตาม
- `offset_x` (0.0) - offset แนวนอน
- `offset_y` (1.0) - offset แนวตั้ง

#### Dead Zone:
- `dead_zone_x` (1.0) - พื้นที่แนวนอนที่ camera ไม่เคลื่อนที่
- `dead_zone_y` (0.5) - พื้นที่แนวตั้งที่ camera ไม่เคลื่อนที่

#### Look Ahead:
- `look_ahead_x` (1.5) - มองไปข้างหน้าแนวนอน
- `look_ahead_y` (0.5) - มองไปข้างหน้าแนวตั้ง
- `look_ahead_smooth` (3.0) - ความนุ่มนวลของ look ahead

#### Bounds:
- `use_bounds` (true) - เปิด/ปิด camera bounds
- `bound_min_x` (-15.0) - ขอบซ้าย
- `bound_max_x` (35.0) - ขอบขวา
- `bound_min_y` (-10.0) - ขอบล่าง
- `bound_max_y` (15.0) - ขอบบน

---

## 🆕 Entity Reference Parameters (Unity-Style)

Engine ตอนนี้รองรับ Entity reference parameters เหมือน Unity's `public GameObject` แล้ว!

### ใน Lua Script:
```lua
-- ประกาศ parameter ด้วย nil (จะแสดงเป็น Entity dropdown ใน Inspector)
playerTarget = nil

function on_update(dt)
    if playerTarget then
        local pos = get_position_of(playerTarget)
        -- ใช้ entity ได้เลย...
    end
end
```

### ใน Inspector:
1. Parameter จะแสดงเป็น dropdown
2. เลือก "None" หรือ entity ใดก็ได้จาก list
3. Entity จะแสดงชื่อและ ID
4. การเปลี่ยนแปลงจะถูกบันทึกกับ scene

### ข้อดี:
- **ชัดเจน**: ไม่ต้องค้นหาหรือพึ่ง tag
- **ยืดหยุ่น**: เปลี่ยน target ได้ง่าย
- **ปลอดภัย**: Type-safe entity references
- **คุ้นเคย**: เหมือน Unity workflow

---

## Parameter Types ที่รองรับ

- `Float` - ทศนิยม (เช่น `speed = 5.0`)
- `Int` - จำนวนเต็ม (เช่น `health = 100`)
- `String` - ข้อความ (เช่น `name = "Player"`)
- `Bool` - จริง/เท็จ (เช่น `enabled = true`)
- `Entity` - Entity references (เช่น `target = nil`) ⭐ ใหม่!

---

## Tips & Best Practices

### 1. Smooth Speed
```lua
smooth_speed = 0    -- Instant (hard follow)
smooth_speed = 3    -- Slow (cinematic)
smooth_speed = 5    -- Normal (recommended)
smooth_speed = 10   -- Fast (responsive)
```

### 2. Dead Zone
Dead zone ช่วยให้ camera ไม่สั่นเมื่อ player เคลื่อนที่เล็กน้อย

```lua
dead_zone_x = 0.5   -- Small (camera moves often)
dead_zone_x = 1.0   -- Medium (recommended)
dead_zone_x = 2.0   -- Large (camera moves less)
```

### 3. Look Ahead
Look ahead ทำให้เห็นพื้นที่ข้างหน้าที่ player กำลังจะไป

```lua
look_ahead_x = 0.0  -- No look ahead
look_ahead_x = 1.0  -- Subtle
look_ahead_x = 2.0  -- Noticeable (recommended for platformer)
```

### 4. Camera Bounds
ใช้ bounds เพื่อไม่ให้ camera แสดงพื้นที่นอกแผนที่

```lua
-- สำหรับแผนที่ 37x26 cells:
use_bounds = true
bound_min_x = 0
bound_max_x = 37
bound_min_y = -26
bound_max_y = 0
```

---

## Troubleshooting

### Camera ไม่ติดตาม player
- ✅ (Unity-style) ตรวจสอบว่าได้ลาก player entity มาวางที่ `playerTarget` แล้ว
- ✅ (Tag-based) ตรวจสอบว่า player มี tag "Player"
- ✅ ตรวจสอบ console log
- ✅ ตรวจสอบว่า script ถูก attach กับ Camera entity

### Camera เคลื่อนที่กระตุก
- ลด `smooth_speed` ลง (ลองใช้ 3-5)
- เพิ่ม `dead_zone` (ลองใช้ 1.0-2.0)

### Camera เคลื่อนที่ช้าเกินไป
- เพิ่ม `smooth_speed` ขึ้น (ลองใช้ 8-10)
- ลด `dead_zone` ลง

### Camera แสดงพื้นที่นอกแผนที่
- เปิด `use_bounds = true`
- ตั้งค่า bounds ให้ตรงกับขนาดแผนที่

---

## Example Setup

### Unity-Style (แนะนำ):
```lua
-- camera_follow_unity_style.lua
playerTarget = nil  -- ลาก player entity มาวางใน Inspector
smooth_speed = 5.0
offset_x = 0.0
offset_y = 0.0
```

### Celeste-style Camera:
```lua
-- camera_follow_advanced.lua
smooth_speed = 5.0
offset_x = 0.0
offset_y = 1.0
dead_zone_x = 1.5
dead_zone_y = 0.8
look_ahead_x = 2.0
look_ahead_y = 0.5
use_bounds = true
```

### Tight Follow Camera:
```lua
-- camera_follow_simple.lua
smooth_speed = 10.0
offset_x = 0.0
offset_y = 0.5
```

### Cinematic Camera:
```lua
-- camera_follow_advanced.lua
smooth_speed = 2.0
offset_x = 0.0
offset_y = 2.0
dead_zone_x = 2.0
dead_zone_y = 1.5
look_ahead_x = 3.0
look_ahead_y = 1.0
```
