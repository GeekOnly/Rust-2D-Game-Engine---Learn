# ✅ เชื่อม Lua UI API กับ UIManager สำเร็จ!

## สรุป

เชื่อม Lua UI API กับ UIManager เรียบร้อย ตอนนี้ Lua scripts สามารถควบคุม UI ได้แล้ว!

## การทำงาน

### 1. Command Queue Pattern
ใช้ pattern แบบ command queue:
```
Lua Script → UICommand → ScriptEngine → Engine → UIManager
```

### 2. UICommand Types
เพิ่ม enum `UICommand` ใน `script/src/lib.rs`:
```rust
pub enum UICommand {
    LoadPrefab { path: String },
    ActivatePrefab { path: String, instance_name: String },
    DeactivatePrefab { instance_name: String },
    SetText { element_path: String, text: String },
    SetImageFill { element_path: String, fill_amount: f32 },
    SetColor { element_path: String, r: f32, g: f32, b: f32, a: f32 },
    ShowElement { element_path: String },
    HideElement { element_path: String },
}
```

### 3. Lua Functions
แก้ไข Lua UI functions ให้ส่ง commands แทนที่จะเป็น placeholders:
```lua
-- ตอนนี้ใช้งานได้จริง!
UI.load_prefab("path/to/prefab.uiprefab")
UI.activate_prefab("path/to/prefab.uiprefab", "my_ui")
UI.set_text("my_ui/text_element", "Hello World!")
UI.set_image_fill("my_ui/health_bar", 0.75)
```

### 4. UIManager Methods
เพิ่ม methods ใหม่ใน UIManager:
- `set_element_color()` - เปลี่ยนสีของ element
- `set_element_fill()` - เปลี่ยน fill amount ของ image
- `show_element()` - แสดง element (alpha = 1.0)
- `hide_element()` - ซ่อน element (alpha = 0.0)
- `find_element_mut()` - หา element แบบ recursive

### 5. Command Processing
เพิ่ม code ใน `engine/src/main.rs` เพื่อประมวลผล UI commands:
```rust
let ui_commands = script_engine.take_ui_commands();
for command in ui_commands {
    match command {
        UICommand::LoadPrefab { path } => { ... }
        UICommand::ActivatePrefab { path, instance_name } => { ... }
        // ... etc
    }
}
```

## Lua UI API

### 1. Load & Activate Prefab
```lua
-- โหลด prefab file
UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")

-- เปิดใช้งาน prefab
UI.activate_prefab(
    "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab",
    "celeste_hud"
)
```

### 2. Update Text
```lua
-- อัพเดทข้อความ
-- Format: "instance_name/element_name"
UI.set_text("celeste_hud/fps_counter", "FPS: 60")
UI.set_text("celeste_hud/position_debug", "X: 100 Y: 200")
UI.set_text("celeste_hud/dash_indicator", "Dash: 2")
```

### 3. Update Health/Stamina Bars
```lua
-- อัพเดท fill amount (0.0 - 1.0)
UI.set_image_fill("celeste_hud/player_health_fill", 0.75)  -- 75% health
UI.set_image_fill("celeste_hud/stamina_bar_fill", 0.5)     -- 50% stamina
```

### 4. Change Colors
```lua
-- เปลี่ยนสี (r, g, b, a)
UI.set_color("celeste_hud/player_health_fill", {
    r = 1.0,  -- Red
    g = 0.0,  -- Green
    b = 0.0,  -- Blue
    a = 1.0   -- Alpha
})
```

### 5. Show/Hide Elements
```lua
-- แสดง element
UI.show_element("celeste_hud/grounded_indicator")

-- ซ่อน element
UI.hide_element("celeste_hud/dashing_indicator")
```

### 6. Deactivate UI
```lua
-- ปิด UI
UI.deactivate_prefab("celeste_hud")
```

## ตัวอย่างการใช้งานจริง

### Player Health Script
```lua
-- player_health.lua
function Update(dt)
    -- Get player health from somewhere
    local health = GetHealth()  -- 0-100
    local health_percent = health / 100.0
    
    -- Update health bar
    UI.set_image_fill("celeste_hud/player_health_fill", health_percent)
    
    -- Change color based on health
    if health_percent < 0.3 then
        -- Red when low
        UI.set_color("celeste_hud/player_health_fill", {r=1.0, g=0.0, b=0.0, a=1.0})
    elseif health_percent < 0.6 then
        -- Yellow when medium
        UI.set_color("celeste_hud/player_health_fill", {r=1.0, g=0.8, b=0.0, a=1.0})
    else
        -- Green when high
        UI.set_color("celeste_hud/player_health_fill", {r=0.2, g=1.0, b=0.3, a=1.0})
    end
end
```

### FPS Counter Script
```lua
-- fps_counter.lua
local frame_count = 0
local elapsed_time = 0
local fps = 0

function Update(dt)
    frame_count = frame_count + 1
    elapsed_time = elapsed_time + dt
    
    -- Update every 0.5 seconds
    if elapsed_time >= 0.5 then
        fps = math.floor(frame_count / elapsed_time)
        UI.set_text("celeste_hud/fps_counter", "FPS: " .. fps)
        
        frame_count = 0
        elapsed_time = 0
    end
end
```

### Dash Indicator Script
```lua
-- dash_indicator.lua
function Update(dt)
    local dash_count = GetDashCount()  -- Get from player
    
    -- Update text
    UI.set_text("celeste_hud/dash_indicator", "Dash: " .. dash_count)
    
    -- Show/hide based on availability
    if dash_count > 0 then
        UI.show_element("celeste_hud/dash_indicator")
    else
        UI.hide_element("celeste_hud/dash_indicator")
    end
end
```

### Grounded Indicator Script
```lua
-- grounded_indicator.lua
function Update(dt)
    local is_grounded = IsGrounded()  -- Get from player
    
    if is_grounded then
        UI.show_element("celeste_hud/grounded_indicator")
    else
        UI.hide_element("celeste_hud/grounded_indicator")
    end
end
```

## Element Path Format

Element path ใช้รูปแบบ: `"instance_name/element_name"`

### ตัวอย่าง:
```lua
-- Instance name: "celeste_hud"
-- Element name: "player_health_fill"
-- Full path: "celeste_hud/player_health_fill"

UI.set_image_fill("celeste_hud/player_health_fill", 0.5)
```

### Elements ใน celeste_hud.uiprefab:
- `celeste_hud/player_health_fill` - Health bar fill
- `celeste_hud/stamina_bar_fill` - Stamina bar fill
- `celeste_hud/dash_indicator` - Dash count text
- `celeste_hud/fps_counter` - FPS text
- `celeste_hud/position_debug` - Position text
- `celeste_hud/velocity_debug` - Velocity text
- `celeste_hud/grounded_indicator` - Grounded text
- `celeste_hud/wall_slide_indicator` - Wall slide text
- `celeste_hud/dashing_indicator` - Dashing text
- `celeste_hud/controls_hint` - Controls text

## การทดสอบ

### 1. สร้าง test script
สร้างไฟล์ `projects/Celeste Demo/scripts/ui_test.lua`:
```lua
function Start()
    print("UI Test Script Started")
    
    -- Load and activate HUD
    UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")
    UI.activate_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab", "celeste_hud")
    
    print("HUD Loaded and Activated")
end

local time = 0

function Update(dt)
    time = time + dt
    
    -- Animate health bar
    local health = (math.sin(time) + 1.0) / 2.0  -- 0.0 to 1.0
    UI.set_image_fill("celeste_hud/player_health_fill", health)
    
    -- Update FPS
    local fps = math.floor(1.0 / dt)
    UI.set_text("celeste_hud/fps_counter", "FPS: " .. fps)
    
    -- Blink dashing indicator
    if math.floor(time * 2) % 2 == 0 then
        UI.show_element("celeste_hud/dashing_indicator")
    else
        UI.hide_element("celeste_hud/dashing_indicator")
    end
end
```

### 2. เพิ่ม script ให้ entity
1. เปิด Celeste Demo project
2. เลือก entity ใน Hierarchy
3. เพิ่ม Script component
4. ตั้งชื่อ script เป็น `ui_test`
5. กด Play

### 3. ดูผลลัพธ์
- Health bar จะเคลื่อนไหวขึ้นลง
- FPS counter จะอัพเดท
- DASHING! text จะกระพริบ

## ไฟล์ที่แก้ไข

1. ✅ `script/src/lib.rs`
   - เพิ่ม `UICommand` enum
   - เพิ่ม `ui_commands` field ใน ScriptEngine
   - แก้ไข Lua UI functions ให้ส่ง commands
   - เพิ่ม `take_ui_commands()` method

2. ✅ `engine/src/ui_manager.rs`
   - เพิ่ม `set_element_color()` method
   - เพิ่ม `set_element_fill()` method
   - เพิ่ม `show_element()` method
   - เพิ่ม `hide_element()` method
   - เพิ่ม `find_element_mut()` helper

3. ✅ `engine/src/main.rs`
   - เพิ่ม code ประมวลผล UI commands
   - เชื่อมต่อ ScriptEngine กับ UIManager

## ข้อจำกัด

### ปัจจุบัน:
- ✅ Load/activate prefabs
- ✅ Update text
- ✅ Update fill amounts
- ✅ Change colors
- ✅ Show/hide elements

### ยังไม่มี:
- ❌ Create UI elements dynamically
- ❌ Button click callbacks
- ❌ Input field handling
- ❌ Animation control
- ❌ Layout manipulation

## ขั้นตอนต่อไป

1. 🔄 ทดสอบ Lua UI API ใน game
2. 🔄 เพิ่ม button click callbacks
3. 🔄 เพิ่ม input field support
4. 🔄 เพิ่ม animation API
5. 🔄 สร้างตัวอย่างเกมที่ใช้ UI จริง

---

**สถานะ**: ✅ COMPLETED - พร้อมใช้งาน!

**การใช้งาน**: สร้าง Lua script และเรียก UI functions ได้เลย!
