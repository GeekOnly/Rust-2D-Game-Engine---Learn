# 🎮 คู่มือ UI System สำหรับ Celeste Demo

## 📋 สรุปสถานะ

UI System ได้ถูก integrate กับ engine เรียบร้อยแล้ว! คุณสามารถแสดง HUD และ UI elements ในเกมได้

### ✅ สิ่งที่ทำงานแล้ว
- ✅ UI Rendering System - แสดง UI บนหน้าจอเกมได้
- ✅ RectTransform - ระบบ anchoring แบบ Unity
- ✅ Image Components - รวม filled images สำหรับ health bars
- ✅ Text Components - แสดงข้อความพร้อม alignment
- ✅ HUD Prefab - `celeste_hud.uiprefab` พร้อมใช้งาน

### ⚠️ สิ่งที่ยังไม่เสร็จ
- ⚠️ Lua API Integration - ยังไม่สามารถควบคุม UI จาก Lua ได้

---

## 🎯 วิธีใช้งานในขณะนี้

### 1. ใช้ Console Output (แนะนำ)

สร้างไฟล์ `scripts/player_ui.lua`:

```lua
-- Player UI Controller
local hp = 100
local max_hp = 100
local stamina = 100
local max_stamina = 100
local frame_count = 0

function on_start()
    print("=== Celeste Demo Started ===")
    print("Controls:")
    print("  H - Take damage")
    print("  R - Restore health")
    print("  Shift - Use stamina")
    print("============================")
end

function on_update(entity, dt)
    frame_count = frame_count + 1
    
    -- Update stamina
    if is_key_down("LeftShift") then
        stamina = math.max(0, stamina - 50 * dt)
    else
        stamina = math.min(max_stamina, stamina + 30 * dt)
    end
    
    -- Test controls
    if is_key_just_pressed("H") then
        hp = math.max(0, hp - 10)
        print("💔 HP: " .. hp .. "/" .. max_hp)
    end
    
    if is_key_just_pressed("R") then
        hp = max_hp
        stamina = max_stamina
        print("💚 Health restored!")
    end
    
    -- Display status every 60 frames (1 second at 60 FPS)
    if frame_count % 60 == 0 then
        local hp_percent = math.floor((hp / max_hp) * 100)
        local stamina_percent = math.floor((stamina / max_stamina) * 100)
        
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━")
        print("❤️  HP: " .. hp .. "/" .. max_hp .. " (" .. hp_percent .. "%)")
        print("⚡ Stamina: " .. math.floor(stamina) .. "/" .. max_stamina .. " (" .. stamina_percent .. "%)")
        print("🎮 FPS: " .. math.floor(1.0 / dt))
        
        local pos = get_position()
        if pos then
            print("📍 Position: X=" .. string.format("%.1f", pos.x) .. " Y=" .. string.format("%.1f", pos.y))
        end
        
        local vel = get_velocity()
        if vel then
            print("💨 Velocity: VX=" .. string.format("%.1f", vel.x) .. " VY=" .. string.format("%.1f", vel.y))
        end
        print("━━━━━━━━━━━━━━━━━━━━━━━━━━")
    end
end
```

### 2. วิธีใช้งาน

1. **Attach Script** - เพิ่ม `player_ui.lua` ให้กับ Player entity
2. **Run Game** - กด Play button
3. **ดูผลลัพธ์** - เปิด Console window เพื่อดูข้อมูล
4. **ทดสอบ**:
   - กด `H` เพื่อลด HP
   - กด `R` เพื่อฟื้น HP
   - กด `Shift` เพื่อใช้ Stamina
   - ดูข้อมูลอัพเดททุก 1 วินาที

---

## 📁 ไฟล์ที่เกี่ยวข้อง

### HUD Prefab
- `assets/ui/celeste_hud.uiprefab` - HUD definition (JSON)

### Scripts
- `scripts/ui_test.lua` - ตัวอย่าง script พื้นฐาน
- `scripts/player_ui.lua` - Script ที่แนะนำให้ใช้

### Documentation
- `UI_USAGE_GUIDE.md` - คู่มือการใช้งานแบบละเอียด
- `UI_INTEGRATION_STATUS.md` - สถานะการ integrate
- `SIMPLE_UI_EXAMPLE.md` - ตัวอย่างแบบง่าย
- `INGAME_UI_EXAMPLE.md` - ตัวอย่างแบบครบถ้วน (สำหรับอนาคต)

---

## 🔮 อนาคต: เมื่อ Lua API เสร็จ

เมื่อ Lua integration เสร็จสมบูรณ์ คุณจะสามารถใช้:

```lua
function on_start()
    -- โหลดและแสดง HUD
    UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")
    UI.activate_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab", "hud")
end

function on_update(entity, dt)
    -- อัพเดท UI elements
    UI.set_text("hud/fps_counter", "FPS: " .. math.floor(1.0/dt))
    UI.set_image_fill("hud/player_health_fill", hp / max_hp)
    UI.set_image_fill("hud/stamina_bar_fill", stamina / max_stamina)
    UI.set_text("hud/dash_indicator", "Dash: " .. dash_count)
    
    -- แสดง/ซ่อน indicators
    if is_dashing then
        UI.show_element("hud/dashing_indicator")
    else
        UI.hide_element("hud/dashing_indicator")
    end
end
```

---

## 🎨 HUD Elements ที่มีใน celeste_hud.uiprefab

| Element | Type | Description |
|---------|------|-------------|
| `player_health` | Container | กรอบ Health Bar |
| `player_health_fill` | Image (Filled) | แถบ HP สีเขียว |
| `stamina_bar_fill` | Image (Filled) | แถบ Stamina สีเหลือง |
| `dash_indicator` | Text | "Dash: {count}" |
| `fps_counter` | Text | "FPS: {fps}" |
| `position_debug` | Text | "X: {x} Y: {y}" |
| `velocity_debug` | Text | "VX: {vx} VY: {vy}" |
| `grounded_indicator` | Text | "GROUNDED" |
| `wall_slide_indicator` | Text | "WALL SLIDE" |
| `dashing_indicator` | Text | "DASHING!" |
| `controls_hint` | Text | คำแนะนำการควบคุม |

---

## 🐛 Troubleshooting

### ไม่เห็นข้อมูลใน Console
- ✅ ตรวจสอบว่า script ถูก attach กับ entity แล้ว
- ✅ ตรวจสอบว่าเปิด Console window แล้ว
- ✅ ตรวจสอบว่า `on_update` ถูกเรียกโดยดู log

### Script ไม่ทำงาน
- ✅ ตรวจสอบ syntax error ใน Console
- ✅ ตรวจสอบชื่อ function: `on_start`, `on_update`
- ✅ ตรวจสอบว่า entity มี Transform component

### ต้องการแสดง UI จริงๆ
- ⏳ รอ Lua API integration เสร็จ
- 📖 อ่าน `UI_INTEGRATION_STATUS.md` สำหรับรายละเอียด
- 🔧 หรือช่วย implement Lua bindings (ดู Option 1 ใน status doc)

---

## 💡 Tips

### แสดงข้อมูลแบบสวยงาม

```lua
-- ใช้ emoji และ formatting
print("❤️  HP: " .. hp .. "/" .. max_hp)
print("⚡ Stamina: " .. math.floor(stamina))
print("🎯 Score: " .. score)
print("💨 Speed: " .. string.format("%.2f", speed))

-- สร้าง progress bar ใน console
function create_bar(value, max_value, length)
    local filled = math.floor((value / max_value) * length)
    local empty = length - filled
    return "[" .. string.rep("█", filled) .. string.rep("░", empty) .. "]"
end

print("HP: " .. create_bar(hp, max_hp, 20))
-- Output: HP: [████████████░░░░░░░░]
```

### จัดกลุ่มข้อมูล

```lua
-- สร้าง table สำหรับ stats
local stats = {
    hp = 100,
    max_hp = 100,
    stamina = 100,
    max_stamina = 100,
    score = 0,
    level = 1
}

function print_stats()
    print("━━━━━━━━━━━━━━━━━━━━━━")
    print("📊 PLAYER STATS")
    print("━━━━━━━━━━━━━━━━━━━━━━")
    for key, value in pairs(stats) do
        print("  " .. key .. ": " .. value)
    end
    print("━━━━━━━━━━━━━━━━━━━━━━")
end
```

### Debug Mode

```lua
local debug_mode = true

function debug_print(msg)
    if debug_mode then
        print("[DEBUG] " .. msg)
    end
end

function on_update(entity, dt)
    debug_print("Frame: " .. frame_count)
    debug_print("DT: " .. dt)
end
```

---

## 📚 เอกสารเพิ่มเติม

- **UI_USAGE_GUIDE.md** - คู่มือการใช้งาน UI API (เมื่อพร้อมใช้งาน)
- **UI_INTEGRATION_STATUS.md** - สถานะการพัฒนาและสิ่งที่ต้องทำต่อ
- **SIMPLE_UI_EXAMPLE.md** - ตัวอย่างการใช้งานแบบง่าย
- **INGAME_UI_EXAMPLE.md** - ตัวอย่างการใช้งานแบบครบถ้วน

---

## ✅ สรุป

**ปัจจุบัน:**
- ใช้ `print()` แสดงข้อมูลใน Console
- ใช้ `debug_draw_*` functions (ถ้ามี)
- Script ตัวอย่างพร้อมใช้งาน

**อนาคตใกล้:**
- Lua API จะสามารถควบคุม UI ได้
- HUD จะแสดงบนหน้าจอจริง
- ข้อมูลจะอัพเดท real-time

**เริ่มต้นใช้งาน:**
1. Copy script จาก section "วิธีใช้งานในขณะนี้"
2. Attach กับ Player entity
3. กด Play
4. เปิด Console
5. ทดสอบด้วย H, R, Shift

🎉 **Happy Coding!**
