# ตัวอย่างการใช้งาน In-Game UI System

## ภาพรวม

เอกสารนี้แสดงวิธีการใช้งาน UI System ใหม่ในเกม Celeste Demo โดยจะครอบคลุมการสร้าง UI แบบต่างๆ ทั้งจาก Lua และการใช้ Prefab

## 1. การสร้าง UI พื้นฐานด้วย Lua

### ตัวอย่าง: Health Bar และ Score Display

สร้างไฟล์ `scripts/game_ui.lua`:

```lua
-- game_ui.lua - ระบบ UI สำหรับเกม

local GameUI = {}

-- สร้าง Canvas หลัก
function GameUI.create_main_canvas()
    local canvas = UI.create_canvas({
        name = "MainGameCanvas",
        render_mode = "ScreenSpaceOverlay",
        sort_order = 0
    })
    
    return canvas
end

-- สร้าง Health Bar
function GameUI.create_health_bar(parent_canvas)
    -- Background
    local bg = UI.create_image({
        name = "HealthBarBG",
        parent = parent_canvas,
        anchor_min = {x = 0.02, y = 0.95},  -- บนซ้าย
        anchor_max = {x = 0.02, y = 0.95},
        pivot = {x = 0, y = 1},
        position = {x = 0, y = 0},
        size = {x = 200, y = 30},
        color = {r = 0.2, g = 0.2, b = 0.2, a = 0.8},
        sprite = nil  -- สีพื้นฐาน
    })
    
    -- Fill (แถบเลือด)
    local fill = UI.create_image({
        name = "HealthBarFill",
        parent = bg,
        anchor_min = {x = 0, y = 0},
        anchor_max = {x = 1, y = 1},  -- เต็มพื้นที่
        pivot = {x = 0, y = 0.5},
        position = {x = 2, y = 0},
        size = {x = -4, y = -4},  -- margin 2px
        color = {r = 1.0, g = 0.2, b = 0.2, a = 1.0},  -- สีแดง
        sprite = nil
    })
    
    -- Text แสดงค่า HP
    local text = UI.create_text({
        name = "HealthText",
        parent = bg,
        anchor_min = {x = 0.5, y = 0.5},
        anchor_max = {x = 0.5, y = 0.5},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 180, y = 25},
        text = "HP: 100/100",
        font_size = 16,
        color = {r = 1, g = 1, b = 1, a = 1},
        alignment = "MiddleCenter"
    })
    
    return {
        background = bg,
        fill = fill,
        text = text
    }
end

-- อัพเดท Health Bar
function GameUI.update_health_bar(health_bar, current_hp, max_hp)
    local percentage = current_hp / max_hp
    
    -- อัพเดทขนาดของ fill bar
    UI.set_size(health_bar.fill, {
        x = (200 - 4) * percentage,
        y = -4
    })
    
    -- อัพเดทข้อความ
    UI.set_text(health_bar.text, string.format("HP: %d/%d", current_hp, max_hp))
    
    -- เปลี่ยนสีตามเปอร์เซ็นต์
    if percentage > 0.5 then
        UI.set_color(health_bar.fill, {r = 0.2, g = 1.0, b = 0.2, a = 1.0})  -- เขียว
    elseif percentage > 0.25 then
        UI.set_color(health_bar.fill, {r = 1.0, g = 0.8, b = 0.0, a = 1.0})  -- เหลือง
    else
        UI.set_color(health_bar.fill, {r = 1.0, g = 0.2, b = 0.2, a = 1.0})  -- แดง
    end
end

-- สร้าง Score Display
function GameUI.create_score_display(parent_canvas)
    local score_text = UI.create_text({
        name = "ScoreText",
        parent = parent_canvas,
        anchor_min = {x = 0.98, y = 0.95},  -- บนขวา
        anchor_max = {x = 0.98, y = 0.95},
        pivot = {x = 1, y = 1},
        position = {x = 0, y = 0},
        size = {x = 200, y = 40},
        text = "Score: 0",
        font_size = 24,
        color = {r = 1, g = 1, b = 0, a = 1},  -- สีเหลือง
        alignment = "TopRight"
    })
    
    return score_text
end

-- อัพเดท Score
function GameUI.update_score(score_text, score)
    UI.set_text(score_text, string.format("Score: %d", score))
end

-- สร้าง Pause Menu
function GameUI.create_pause_menu(parent_canvas)
    -- Background overlay
    local overlay = UI.create_panel({
        name = "PauseOverlay",
        parent = parent_canvas,
        anchor_min = {x = 0, y = 0},
        anchor_max = {x = 1, y = 1},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 0, y = 0},
        color = {r = 0, g = 0, b = 0, a = 0.7},
        visible = false  -- ซ่อนไว้ตอนเริ่มต้น
    })
    
    -- Menu Panel
    local menu = UI.create_panel({
        name = "PauseMenu",
        parent = overlay,
        anchor_min = {x = 0.5, y = 0.5},
        anchor_max = {x = 0.5, y = 0.5},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 300, y = 400},
        color = {r = 0.1, g = 0.1, b = 0.1, a = 0.95}
    })
    
    -- Title
    local title = UI.create_text({
        name = "PauseTitle",
        parent = menu,
        anchor_min = {x = 0.5, y = 0.9},
        anchor_max = {x = 0.5, y = 0.9},
        pivot = {x = 0.5, y = 1},
        position = {x = 0, y = -20},
        size = {x = 260, y = 50},
        text = "PAUSED",
        font_size = 32,
        color = {r = 1, g = 1, b = 1, a = 1},
        alignment = "MiddleCenter"
    })
    
    -- Resume Button
    local resume_btn = UI.create_button({
        name = "ResumeButton",
        parent = menu,
        anchor_min = {x = 0.5, y = 0.6},
        anchor_max = {x = 0.5, y = 0.6},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 200, y = 50},
        text = "Resume",
        font_size = 20,
        normal_color = {r = 0.3, g = 0.3, b = 0.3, a = 1},
        highlighted_color = {r = 0.5, g = 0.5, b = 0.5, a = 1},
        pressed_color = {r = 0.2, g = 0.2, b = 0.2, a = 1},
        on_click = "on_resume_clicked"
    })
    
    -- Restart Button
    local restart_btn = UI.create_button({
        name = "RestartButton",
        parent = menu,
        anchor_min = {x = 0.5, y = 0.45},
        anchor_max = {x = 0.5, y = 0.45},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 200, y = 50},
        text = "Restart",
        font_size = 20,
        normal_color = {r = 0.3, g = 0.3, b = 0.3, a = 1},
        highlighted_color = {r = 0.5, g = 0.5, b = 0.5, a = 1},
        pressed_color = {r = 0.2, g = 0.2, b = 0.2, a = 1},
        on_click = "on_restart_clicked"
    })
    
    -- Quit Button
    local quit_btn = UI.create_button({
        name = "QuitButton",
        parent = menu,
        anchor_min = {x = 0.5, y = 0.3},
        anchor_max = {x = 0.5, y = 0.3},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 200, y = 50},
        text = "Quit to Menu",
        font_size = 20,
        normal_color = {r = 0.3, g = 0.3, b = 0.3, a = 1},
        highlighted_color = {r = 0.5, g = 0.5, b = 0.5, a = 1},
        pressed_color = {r = 0.2, g = 0.2, b = 0.2, a = 1},
        on_click = "on_quit_clicked"
    })
    
    return {
        overlay = overlay,
        menu = menu,
        resume_button = resume_btn,
        restart_button = restart_btn,
        quit_button = quit_btn
    }
end

-- แสดง/ซ่อน Pause Menu
function GameUI.toggle_pause_menu(pause_menu, show)
    UI.set_visible(pause_menu.overlay, show)
end

return GameUI
```

## 2. การใช้งานใน Player Controller

แก้ไขไฟล์ `scripts/player_controller.lua`:

```lua
-- เพิ่มที่ด้านบน
local GameUI = require("scripts/game_ui")

-- ตัวแปร UI
local game_canvas = nil
local health_bar = nil
local score_display = nil
local pause_menu = nil
local is_paused = false

-- ตัวแปรเกม
local current_hp = 100
local max_hp = 100
local score = 0

function on_start()
    -- สร้าง UI
    game_canvas = GameUI.create_main_canvas()
    health_bar = GameUI.create_health_bar(game_canvas)
    score_display = GameUI.create_score_display(game_canvas)
    pause_menu = GameUI.create_pause_menu(game_canvas)
    
    -- อัพเดท UI ครั้งแรก
    GameUI.update_health_bar(health_bar, current_hp, max_hp)
    GameUI.update_score(score_display, score)
    
    print("Game UI initialized!")
end

function on_update(dt)
    if not is_paused then
        -- เกมปกติ
        
        -- ตัวอย่าง: ลด HP เมื่อกด H
        if Input.is_key_pressed("H") then
            current_hp = math.max(0, current_hp - 10)
            GameUI.update_health_bar(health_bar, current_hp, max_hp)
        end
        
        -- ตัวอย่าง: เพิ่ม Score เมื่อกด S
        if Input.is_key_pressed("S") then
            score = score + 100
            GameUI.update_score(score_display, score)
        end
    end
    
    -- Toggle Pause Menu ด้วย ESC
    if Input.is_key_pressed("Escape") then
        is_paused = not is_paused
        GameUI.toggle_pause_menu(pause_menu, is_paused)
    end
end

-- Callback functions สำหรับปุ่ม
function on_resume_clicked()
    is_paused = false
    GameUI.toggle_pause_menu(pause_menu, false)
    print("Game resumed")
end

function on_restart_clicked()
    -- รีเซ็ตเกม
    current_hp = max_hp
    score = 0
    GameUI.update_health_bar(health_bar, current_hp, max_hp)
    GameUI.update_score(score_display, score)
    is_paused = false
    GameUI.toggle_pause_menu(pause_menu, false)
    print("Game restarted")
end

function on_quit_clicked()
    print("Quit to menu")
    -- โหลด main menu scene
end
```

## 3. การใช้ UI Prefab

### สร้าง Prefab ด้วย Widget Editor

1. เปิด Widget Editor ใน engine
2. สร้าง UI elements ตามต้องการ
3. บันทึกเป็น `.uiprefab` file

### โหลด Prefab ใน Lua

```lua
-- โหลด prefab
local hud_prefab = UI.load_prefab("assets/ui/game_hud.uiprefab")

-- Instantiate prefab
local hud_instance = UI.instantiate_prefab(hud_prefab, {
    parent = game_canvas,
    -- Override properties
    position = {x = 0, y = 0}
})

-- เข้าถึง elements ใน prefab
local health_text = UI.find_child(hud_instance, "HealthText")
UI.set_text(health_text, "HP: 100")
```

## 4. ตัวอย่าง: Floating Damage Numbers

```lua
-- floating_damage.lua
local FloatingDamage = {}

function FloatingDamage.show_damage(canvas, position, damage)
    -- สร้าง text แสดงความเสียหาย
    local damage_text = UI.create_text({
        name = "DamageNumber",
        parent = canvas,
        anchor_min = {x = 0.5, y = 0.5},
        anchor_max = {x = 0.5, y = 0.5},
        pivot = {x = 0.5, y = 0.5},
        position = position,
        size = {x = 100, y = 50},
        text = string.format("-%d", damage),
        font_size = 24,
        color = {r = 1, g = 0, b = 0, a = 1},
        alignment = "MiddleCenter"
    })
    
    -- Animate ขึ้นไปข้างบนและค่อยๆ จาง
    UI.animate_position(damage_text, {
        to_x = position.x,
        to_y = position.y + 50,
        duration = 1.0,
        easing = "EaseOutQuad"
    })
    
    UI.animate_alpha(damage_text, {
        to = 0.0,
        duration = 1.0,
        easing = "Linear",
        on_complete = function()
            UI.destroy(damage_text)
        end
    })
end

return FloatingDamage
```

## 5. ตัวอย่าง: Inventory UI

```lua
-- inventory_ui.lua
local InventoryUI = {}

function InventoryUI.create_inventory(parent_canvas)
    -- Background panel
    local panel = UI.create_panel({
        name = "InventoryPanel",
        parent = parent_canvas,
        anchor_min = {x = 0.5, y = 0.5},
        anchor_max = {x = 0.5, y = 0.5},
        pivot = {x = 0.5, y = 0.5},
        position = {x = 0, y = 0},
        size = {x = 600, y = 400},
        color = {r = 0.1, g = 0.1, b = 0.1, a = 0.9},
        visible = false
    })
    
    -- Grid layout สำหรับ items
    local grid = UI.create_grid_layout({
        parent = panel,
        anchor_min = {x = 0.05, y = 0.05},
        anchor_max = {x = 0.95, y = 0.85},
        cell_size = {x = 80, y = 80},
        spacing = {x = 10, y = 10},
        padding = {left = 10, right = 10, top = 10, bottom = 10}
    })
    
    -- สร้าง item slots
    local slots = {}
    for i = 1, 20 do
        local slot = UI.create_panel({
            name = "ItemSlot" .. i,
            parent = grid,
            size = {x = 80, y = 80},
            color = {r = 0.2, g = 0.2, b = 0.2, a = 1}
        })
        
        -- Item icon (ถ้ามี)
        local icon = UI.create_image({
            name = "ItemIcon",
            parent = slot,
            anchor_min = {x = 0, y = 0},
            anchor_max = {x = 1, y = 1},
            size = {x = -10, y = -10},
            visible = false
        })
        
        table.insert(slots, {
            slot = slot,
            icon = icon,
            item = nil
        })
    end
    
    return {
        panel = panel,
        grid = grid,
        slots = slots
    }
end

function InventoryUI.add_item(inventory, item_data)
    -- หา slot ว่าง
    for i, slot_data in ipairs(inventory.slots) do
        if slot_data.item == nil then
            slot_data.item = item_data
            UI.set_sprite(slot_data.icon, item_data.icon_path)
            UI.set_visible(slot_data.icon, true)
            return true
        end
    end
    return false  -- inventory เต็ม
end

function InventoryUI.toggle(inventory, show)
    UI.set_visible(inventory.panel, show)
end

return InventoryUI
```

## 6. การใช้ Animations

```lua
-- ตัวอย่าง: Pulse animation สำหรับ button
function create_pulsing_button(parent)
    local button = UI.create_button({
        name = "PulsingButton",
        parent = parent,
        -- ... properties ...
    })
    
    -- สร้าง loop animation
    local function pulse()
        UI.animate_scale(button, {
            to_x = 1.1,
            to_y = 1.1,
            duration = 0.5,
            easing = "EaseInOutSine",
            on_complete = function()
                UI.animate_scale(button, {
                    to_x = 1.0,
                    to_y = 1.0,
                    duration = 0.5,
                    easing = "EaseInOutSine",
                    on_complete = pulse  -- วนซ้ำ
                })
            end
        })
    end
    
    pulse()  -- เริ่ม animation
    
    return button
end
```

## 7. Tips และ Best Practices

### Performance

1. **ใช้ Object Pooling**: สำหรับ UI ที่สร้าง/ทำลายบ่อยๆ (เช่น damage numbers)
2. **ปิด UI ที่ไม่ใช้**: ใช้ `set_visible(false)` แทนการทำลาย
3. **จำกัดจำนวน UI elements**: อย่าสร้างมากเกินไป
4. **ใช้ Batching**: จัดกลุ่ม UI ที่ใช้ texture เดียวกัน

### Layout

1. **ใช้ Anchors**: สำหรับ responsive UI
2. **ใช้ Layout Groups**: สำหรับการจัดเรียงอัตโนมัติ
3. **ทดสอบหลายความละเอียด**: 1080p, 720p, 4K

### Code Organization

1. **แยก UI logic**: สร้างไฟล์แยกสำหรับแต่ละ UI system
2. **ใช้ Modules**: `require()` สำหรับ code reuse
3. **Comment ภาษาไทย**: ถ้าทำงานเป็นทีม

## 8. การ Debug UI

```lua
-- แสดงข้อมูล debug
function show_debug_info(canvas)
    local debug_text = UI.create_text({
        name = "DebugInfo",
        parent = canvas,
        anchor_min = {x = 0, y = 0},
        anchor_max = {x = 0, y = 0},
        pivot = {x = 0, y = 0},
        position = {x = 10, y = 10},
        size = {x = 300, y = 200},
        text = "",
        font_size = 14,
        color = {r = 0, g = 1, b = 0, a = 1},
        alignment = "TopLeft"
    })
    
    return debug_text
end

function update_debug_info(debug_text, fps, entity_count)
    local info = string.format(
        "FPS: %.1f\nEntities: %d\nMemory: %.2f MB",
        fps,
        entity_count,
        collectgarbage("count") / 1024
    )
    UI.set_text(debug_text, info)
end
```

## สรุป

ระบบ UI ใหม่มีความสามารถครบถ้วนสำหรับการสร้าง in-game UI ที่ซับซ้อน:

- ✅ สร้าง UI ด้วย Lua ได้ง่าย
- ✅ รองรับ Prefab สำหรับ reusability
- ✅ มี Animation system ในตัว
- ✅ Layout system อัตโนมัติ
- ✅ Event handling ที่สมบูรณ์
- ✅ Performance ดี

สามารถดูตัวอย่างเพิ่มเติมได้ที่:
- `ui/examples/` - ตัวอย่าง Rust
- `ui/examples/lua_ui_example.lua` - ตัวอย่าง Lua
- `ui/README.md` - เอกสารฉบับเต็ม
