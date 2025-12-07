# คู่มือการใช้งาน UI System ในเกม Celeste Demo

## ✅ UI System พร้อมใช้งานแล้ว!

UI System ได้ถูก integrate กับ engine เรียบร้อยแล้ว คุณสามารถแสดง HUD และ UI อื่นๆ ในเกมได้

---

## 🎮 วิธีใช้งาน UI ใน Lua Script

### 1. โหลดและแสดง HUD Prefab

```lua
-- player_controller.lua

-- ตัวแปร UI
local hud_loaded = false
local player_hp = 100
local max_hp = 100
local stamina = 100
local max_stamina = 100
local dash_count = 1
local fps = 60

function on_start()
    print("=== Celeste Demo Started ===")
    
    -- โหลด HUD prefab
    local success = UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")
    if success then
        -- เปิดใช้งาน HUD
        UI.activate_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab", "main_hud")
        hud_loaded = true
        print("✓ HUD loaded successfully!")
    else
        print("✗ Failed to load HUD")
    end
end

function on_update(dt)
    if not hud_loaded then
        return
    end
    
    -- คำนวณ FPS
    fps = math.floor(1.0 / dt)
    
    -- อัพเดท UI elements
    update_health_bar()
    update_stamina_bar()
    update_dash_indicator()
    update_debug_info()
    update_fps_counter()
    
    -- ตัวอย่าง: ลด HP เมื่อกด H
    if Input.is_key_pressed("H") then
        player_hp = math.max(0, player_hp - 10)
        print("HP: " .. player_hp)
    end
    
    -- ตัวอย่าง: ฟื้น HP เมื่อกด R
    if Input.is_key_pressed("R") then
        player_hp = max_hp
        stamina = max_stamina
        print("HP and Stamina restored!")
    end
    
    -- ลด Stamina เมื่อกด Shift
    if Input.is_key_down("LeftShift") then
        stamina = math.max(0, stamina - 50 * dt)
    else
        stamina = math.min(max_stamina, stamina + 30 * dt)
    end
end

-- อัพเดท Health Bar
function update_health_bar()
    local fill_percent = player_hp / max_hp
    UI.set_image_fill("main_hud/player_health_fill", fill_percent)
    
    -- เปลี่ยนสีตาม HP
    if fill_percent > 0.5 then
        UI.set_color("main_hud/player_health_fill", {0.2, 1.0, 0.3, 1.0}) -- เขียว
    elseif fill_percent > 0.25 then
        UI.set_color("main_hud/player_health_fill", {1.0, 0.8, 0.2, 1.0}) -- เหลือง
    else
        UI.set_color("main_hud/player_health_fill", {1.0, 0.2, 0.2, 1.0}) -- แดง
    end
end

-- อัพเดท Stamina Bar
function update_stamina_bar()
    local fill_percent = stamina / max_stamina
    UI.set_image_fill("main_hud/stamina_bar_fill", fill_percent)
end

-- อัพเดท Dash Indicator
function update_dash_indicator()
    UI.set_text("main_hud/dash_indicator", "Dash: " .. dash_count)
end

-- อัพเดท Debug Info
function update_debug_info()
    -- ดึงตำแหน่งของ player
    local pos = Transform.get_position(entity)
    local vel = Physics.get_velocity(entity) or {x = 0, y = 0}
    
    UI.set_text("main_hud/position_debug", 
        string.format("X: %.1f Y: %.1f", pos.x, pos.y))
    
    UI.set_text("main_hud/velocity_debug", 
        string.format("VX: %.1f VY: %.1f", vel.x, vel.y))
end

-- อัพเดท FPS Counter
function update_fps_counter()
    UI.set_text("main_hud/fps_counter", "FPS: " .. fps)
end
```

---

## 📋 UI API Reference

### โหลดและจัดการ Prefab

```lua
-- โหลด prefab จากไฟล์
UI.load_prefab(path)
-- Returns: boolean (success)

-- เปิดใช้งาน prefab (แสดง UI)
UI.activate_prefab(path, instance_name)
-- Returns: boolean (success)

-- ปิด prefab (ซ่อน UI)
UI.deactivate_prefab(instance_name)
```

### อัพเดท Text Elements

```lua
-- เปลี่ยนข้อความ
UI.set_text(element_path, text)
-- element_path format: "instance_name/element_name"
-- Example: UI.set_text("main_hud/fps_counter", "FPS: 60")
```

### อัพเดท Image Elements

```lua
-- เปลี่ยน fill amount (สำหรับ health bars, progress bars)
UI.set_image_fill(element_path, fill_amount)
-- fill_amount: 0.0 to 1.0
-- Example: UI.set_image_fill("main_hud/health_bar", 0.75)

-- เปลี่ยนสี
UI.set_color(element_path, {r, g, b, a})
-- Example: UI.set_color("main_hud/health_bar", {1.0, 0.0, 0.0, 1.0})
```

### แสดง/ซ่อน Elements

```lua
-- แสดง element
UI.show_element(element_path)

-- ซ่อน element
UI.hide_element(element_path)

-- Toggle visibility
UI.toggle_element(element_path)
```

---

## 🎨 ตัวอย่างการใช้งานขั้นสูง

### 1. สร้าง Health Bar แบบเต็มรูปแบบ

```lua
local HealthBar = {}

function HealthBar:new(instance_name, element_name)
    local obj = {
        instance = instance_name,
        element = element_name,
        current = 100,
        max = 100,
        regen_rate = 10, -- HP/sec
    }
    setmetatable(obj, self)
    self.__index = self
    return obj
end

function HealthBar:update(dt)
    -- Auto regeneration
    if self.current < self.max then
        self.current = math.min(self.max, self.current + self.regen_rate * dt)
        self:refresh()
    end
end

function HealthBar:damage(amount)
    self.current = math.max(0, self.current - amount)
    self:refresh()
    
    -- Flash effect
    UI.set_color(self.instance .. "/" .. self.element, {1.0, 0.5, 0.5, 1.0})
    -- TODO: Add timer to reset color
end

function HealthBar:heal(amount)
    self.current = math.min(self.max, self.current + amount)
    self:refresh()
end

function HealthBar:refresh()
    local fill = self.current / self.max
    UI.set_image_fill(self.instance .. "/" .. self.element, fill)
    
    -- Color based on health
    if fill > 0.6 then
        UI.set_color(self.instance .. "/" .. self.element, {0.2, 1.0, 0.3, 1.0})
    elseif fill > 0.3 then
        UI.set_color(self.instance .. "/" .. self.element, {1.0, 0.8, 0.2, 1.0})
    else
        UI.set_color(self.instance .. "/" .. self.element, {1.0, 0.2, 0.2, 1.0})
    end
end

-- การใช้งาน
local health_bar = HealthBar:new("main_hud", "player_health_fill")

function on_update(dt)
    health_bar:update(dt)
end

function on_damage(amount)
    health_bar:damage(amount)
end
```

### 2. สร้าง Score Display พร้อม Animation

```lua
local ScoreDisplay = {}

function ScoreDisplay:new(instance_name, element_name)
    local obj = {
        instance = instance_name,
        element = element_name,
        score = 0,
        display_score = 0,
        animation_speed = 500, -- points/sec
    }
    setmetatable(obj, self)
    self.__index = self
    return obj
end

function ScoreDisplay:add_score(points)
    self.score = self.score + points
    print("Score: " .. self.score)
end

function ScoreDisplay:update(dt)
    -- Animate score counting up
    if self.display_score < self.score then
        self.display_score = math.min(
            self.score,
            self.display_score + self.animation_speed * dt
        )
        self:refresh()
    end
end

function ScoreDisplay:refresh()
    UI.set_text(
        self.instance .. "/" .. self.element,
        "Score: " .. math.floor(self.display_score)
    )
end

-- การใช้งาน
local score = ScoreDisplay:new("main_hud", "score_text")

function on_update(dt)
    score:update(dt)
end

function on_collect_coin()
    score:add_score(100)
end
```

### 3. สร้าง Pause Menu

```lua
local PauseMenu = {}

function PauseMenu:new()
    local obj = {
        is_paused = false,
        menu_instance = "pause_menu",
    }
    setmetatable(obj, self)
    self.__index = self
    return obj
end

function PauseMenu:toggle()
    self.is_paused = not self.is_paused
    
    if self.is_paused then
        self:show()
        Time.set_time_scale(0) -- หยุดเกม
    else
        self:hide()
        Time.set_time_scale(1) -- เล่นต่อ
    end
end

function PauseMenu:show()
    UI.activate_prefab("assets/ui/pause_menu.uiprefab", self.menu_instance)
    print("Game Paused")
end

function PauseMenu:hide()
    UI.deactivate_prefab(self.menu_instance)
    print("Game Resumed")
end

-- การใช้งาน
local pause_menu = PauseMenu:new()

function on_update(dt)
    if Input.is_key_pressed("Escape") then
        pause_menu:toggle()
    end
end
```

---

## 🔧 การ Debug UI

### แสดงข้อมูล Debug

```lua
function on_update(dt)
    -- แสดงข้อมูล debug ทุก 30 frames
    if frame_count % 30 == 0 then
        print("=== UI DEBUG ===")
        print("HUD Active: " .. tostring(hud_loaded))
        print("HP: " .. player_hp .. "/" .. max_hp)
        print("Stamina: " .. math.floor(stamina))
        print("FPS: " .. fps)
    end
    
    frame_count = (frame_count or 0) + 1
end
```

### ตรวจสอบว่า UI โหลดสำเร็จ

```lua
function on_start()
    local success = UI.load_prefab("path/to/prefab.uiprefab")
    if success then
        print("✓ Prefab loaded")
        UI.activate_prefab("path/to/prefab.uiprefab", "my_ui")
    else
        print("✗ Failed to load prefab")
        print("Check file path and JSON format")
    end
end
```

---

## 📝 HUD Elements ที่มีใน celeste_hud.uiprefab

| Element Name | Type | Description |
|--------------|------|-------------|
| `player_health` | Container | กรอบ Health Bar |
| `player_health_background` | Image | พื้นหลัง Health Bar (สีเทาเข้ม) |
| `player_health_fill` | Image | แถบ HP (สีเขียว, Filled) |
| `stamina_bar` | Container | กรอบ Stamina Bar |
| `stamina_bar_background` | Image | พื้นหลัง Stamina Bar |
| `stamina_bar_fill` | Image | แถบ Stamina (สีเหลือง, Filled) |
| `dash_indicator` | Text | แสดงจำนวน Dash |
| `position_debug` | Text | แสดงตำแหน่ง X, Y |
| `velocity_debug` | Text | แสดงความเร็ว VX, VY |
| `fps_counter` | Text | แสดง FPS |
| `grounded_indicator` | Text | แสดงสถานะ "GROUNDED" |
| `wall_slide_indicator` | Text | แสดงสถานะ "WALL SLIDE" |
| `dashing_indicator` | Text | แสดงสถานะ "DASHING!" |
| `controls_hint` | Text | แสดงคำแนะนำการควบคุม |

---

## 🎯 ตัวอย่างโค้ดที่พร้อมใช้งาน

บันทึกเป็น `scripts/celeste_ui_controller.lua`:

```lua
-- Celeste UI Controller
-- จัดการ HUD และ UI ทั้งหมดของเกม

local CelesteUI = {
    hud_loaded = false,
    
    -- Player stats
    hp = 100,
    max_hp = 100,
    stamina = 100,
    max_stamina = 100,
    dash_count = 1,
    
    -- State
    is_grounded = false,
    is_wall_sliding = false,
    is_dashing = false,
    
    -- Performance
    fps = 60,
    frame_count = 0,
}

function CelesteUI:init()
    print("=== Initializing Celeste UI ===")
    
    local success = UI.load_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab")
    if success then
        UI.activate_prefab("projects/Celeste Demo/assets/ui/celeste_hud.uiprefab", "hud")
        self.hud_loaded = true
        print("✓ HUD initialized successfully")
    else
        print("✗ Failed to initialize HUD")
    end
end

function CelesteUI:update(dt)
    if not self.hud_loaded then
        return
    end
    
    self.frame_count = self.frame_count + 1
    self.fps = math.floor(1.0 / dt)
    
    -- Update all UI elements
    self:update_health()
    self:update_stamina(dt)
    self:update_dash()
    self:update_fps()
    self:update_state_indicators()
end

function CelesteUI:update_health()
    local fill = self.hp / self.max_hp
    UI.set_image_fill("hud/player_health_fill", fill)
    
    -- Color coding
    if fill > 0.5 then
        UI.set_color("hud/player_health_fill", {0.2, 1.0, 0.3, 1.0})
    elseif fill > 0.25 then
        UI.set_color("hud/player_health_fill", {1.0, 0.8, 0.2, 1.0})
    else
        UI.set_color("hud/player_health_fill", {1.0, 0.2, 0.2, 1.0})
    end
end

function CelesteUI:update_stamina(dt)
    -- Auto regen
    if self.stamina < self.max_stamina and not self.is_dashing then
        self.stamina = math.min(self.max_stamina, self.stamina + 30 * dt)
    end
    
    local fill = self.stamina / self.max_stamina
    UI.set_image_fill("hud/stamina_bar_fill", fill)
end

function CelesteUI:update_dash()
    UI.set_text("hud/dash_indicator", "Dash: " .. self.dash_count)
end

function CelesteUI:update_fps()
    UI.set_text("hud/fps_counter", "FPS: " .. self.fps)
end

function CelesteUI:update_state_indicators()
    -- Show/hide state indicators based on player state
    if self.is_grounded then
        UI.show_element("hud/grounded_indicator")
    else
        UI.hide_element("hud/grounded_indicator")
    end
    
    if self.is_wall_sliding then
        UI.show_element("hud/wall_slide_indicator")
    else
        UI.hide_element("hud/wall_slide_indicator")
    end
    
    if self.is_dashing then
        UI.show_element("hud/dashing_indicator")
    else
        UI.hide_element("hud/dashing_indicator")
    end
end

function CelesteUI:damage(amount)
    self.hp = math.max(0, self.hp - amount)
    print("Damaged! HP: " .. self.hp)
end

function CelesteUI:use_stamina(amount)
    self.stamina = math.max(0, self.stamina - amount)
end

function CelesteUI:use_dash()
    if self.dash_count > 0 then
        self.dash_count = self.dash_count - 1
        self.is_dashing = true
        return true
    end
    return false
end

function CelesteUI:restore_dash()
    self.dash_count = 1
    self.is_dashing = false
end

-- Global instance
celeste_ui = CelesteUI

-- Lifecycle hooks
function on_start()
    celeste_ui:init()
end

function on_update(dt)
    celeste_ui:update(dt)
    
    -- Test controls
    if Input.is_key_pressed("H") then
        celeste_ui:damage(10)
    end
    
    if Input.is_key_pressed("R") then
        celeste_ui.hp = celeste_ui.max_hp
        celeste_ui.stamina = celeste_ui.max_stamina
        celeste_ui:restore_dash()
    end
end
```

---

## ✅ สรุป

1. **UI System พร้อมใช้งาน** - ไม่ต้องรอ integration แล้ว
2. **HUD Prefab พร้อมแล้ว** - `celeste_hud.uiprefab` มีครบทุกอย่าง
3. **API ง่ายต่อการใช้** - `UI.set_text()`, `UI.set_image_fill()`, etc.
4. **รองรับ Dynamic Updates** - อัพเดท UI แบบ real-time ได้
5. **ตัวอย่างโค้ดพร้อมใช้** - Copy-paste ได้เลย

**เริ่มต้นใช้งาน:**
1. Copy โค้ดจาก `celeste_ui_controller.lua` ไปใส่ใน script ของคุณ
2. Attach script กับ Player entity
3. กด Play
4. เห็น HUD แสดงบนหน้าจอ!

**ทดสอบ:**
- กด `H` เพื่อลด HP
- กด `R` เพื่อ restore HP และ Stamina
- ดู FPS counter มุมขวาบน
- ดู debug info (position, velocity)

🎉 **สนุกกับการพัฒนาเกม!**
