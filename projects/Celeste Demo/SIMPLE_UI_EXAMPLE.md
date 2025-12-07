# ตัวอย่าง UI แบบง่ายที่ใช้งานได้ทันที

## ปัญหาปัจจุบัน

UI System ใหม่ยังไม่ได้ integrate กับ engine rendering pipeline จริงๆ ดังนั้นจะใช้ egui (ที่ engine มีอยู่แล้ว) แทนชั่วคราว

## วิธีแสดง UI ในเกม (ใช้ egui)

### 1. แก้ไข player_controller.lua

เพิ่มโค้ดนี้ใน `scripts/player_controller.lua`:

```lua
-- ตัวแปรสำหรับ UI
local player_hp = 100
local max_hp = 100
local stamina = 100
local max_stamina = 100
local score = 0
local dash_count = 1

function on_start()
    print("Player controller started with UI!")
end

function on_update(dt)
    -- ตัวอย่าง: ลด HP เมื่อกด H
    if Input.is_key_pressed("H") then
        player_hp = math.max(0, player_hp - 10)
        print("HP: " .. player_hp)
    end
    
    -- ตัวอย่าง: เพิ่ม Score เมื่อกด S
    if Input.is_key_pressed("S") then
        score = score + 100
        print("Score: " .. score)
    end
    
    -- ตัวอย่าง: ลด Stamina เมื่อกด Shift
    if Input.is_key_down("LeftShift") then
        stamina = math.max(0, stamina - 50 * dt)
    else
        stamina = math.min(max_stamina, stamina + 30 * dt)
    end
end

-- ฟังก์ชันนี้จะถูกเรียกจาก engine เพื่อวาด UI
function on_draw_ui()
    -- วาด Health Bar
    draw_bar(20, 20, 200, 25, player_hp / max_hp, "HP: " .. math.floor(player_hp), {0.2, 1.0, 0.3})
    
    -- วาด Stamina Bar
    draw_bar(20, 50, 200, 20, stamina / max_stamina, "Stamina", {1.0, 0.8, 0.2})
    
    -- วาด Score
    draw_text(20, 80, "Score: " .. score, 20, {1.0, 1.0, 0.0})
    
    -- วาด Dash Count
    draw_text(20, 110, "Dash: " .. dash_count, 18, {0.3, 0.8, 1.0})
    
    -- วาด Controls Hint
    draw_text_centered("WASD: Move | Space: Jump | Shift: Dash | H: Damage | S: Score", 
                      screen_height - 30, 14, {0.8, 0.8, 0.8, 0.7})
end

-- Helper functions (จะต้อง implement ใน engine)
function draw_bar(x, y, width, height, fill_percent, label, color)
    -- Background
    UI.draw_rect(x, y, width, height, {0.15, 0.15, 0.15, 0.9})
    
    -- Fill
    local fill_width = width * fill_percent
    UI.draw_rect(x, y, fill_width, height, {color[1], color[2], color[3], 1.0})
    
    -- Label
    if label then
        UI.draw_text(x + width/2, y + height/2, label, 14, {1, 1, 1, 1}, "center")
    end
end

function draw_text(x, y, text, size, color)
    UI.draw_text(x, y, text, size, color, "left")
end

function draw_text_centered(text, y, size, color)
    UI.draw_text(screen_width / 2, y, text, size, color, "center")
end
```

## 2. วิธีใช้งานชั่วคราว (ด้วย Console)

เนื่องจาก UI system ยังไม่ได้ integrate เต็มรูปแบบ คุณสามารถใช้ Console แสดงข้อมูลได้:

```lua
function on_update(dt)
    -- แสดงข้อมูลใน console
    if frame_count % 60 == 0 then  -- ทุก 60 frames
        print(string.format("HP: %d/%d | Stamina: %.1f | Score: %d", 
              player_hp, max_hp, stamina, score))
    end
    
    frame_count = (frame_count or 0) + 1
end
```

## 3. ใช้ Debug Text Overlay

ถ้า engine รองรับ debug text:

```lua
function on_update(dt)
    -- แสดง debug text
    Debug.draw_text(10, 10, string.format("HP: %d/%d", player_hp, max_hp))
    Debug.draw_text(10, 30, string.format("Stamina: %.0f", stamina))
    Debug.draw_text(10, 50, string.format("Score: %d", score))
    Debug.draw_text(10, 70, string.format("FPS: %.1f", 1.0/dt))
end
```

## 4. ทางเลือก: ใช้ World UI (Text ในโลกเกม)

สร้าง text entities ในโลกเกม:

```lua
local ui_texts = {}

function on_start()
    -- สร้าง text entities
    ui_texts.hp = create_world_text("HP: 100", 50, 50)
    ui_texts.score = create_world_text("Score: 0", 50, 80)
end

function on_update(dt)
    -- อัพเดท text
    update_world_text(ui_texts.hp, "HP: " .. player_hp)
    update_world_text(ui_texts.score, "Score: " .. score)
end

function create_world_text(text, x, y)
    -- สร้าง entity ที่มี text component
    local entity = World.create_entity()
    World.add_component(entity, "Transform", {x = x, y = y})
    World.add_component(entity, "Text", {
        text = text,
        size = 16,
        color = {1, 1, 1, 1}
    })
    return entity
end

function update_world_text(entity, new_text)
    World.set_component(entity, "Text", {text = new_text})
end
```

## 5. ใช้ HUD Prefab ที่มีอยู่

ถ้า engine รองรับการโหลด prefab:

```lua
local hud_instance = nil

function on_start()
    -- โหลด HUD prefab
    hud_instance = load_prefab("assets/ui/celeste_hud.uiprefab")
    
    if hud_instance then
        print("HUD loaded successfully!")
    else
        print("Failed to load HUD")
    end
end

function on_update(dt)
    if hud_instance then
        -- อัพเดท HUD elements
        update_hud_element(hud_instance, "player_health_fill", {fill_amount = player_hp / max_hp})
        update_hud_element(hud_instance, "stamina_bar_fill", {fill_amount = stamina / max_stamina})
        update_hud_element(hud_instance, "dash_indicator", {text = "Dash: " .. dash_count})
    end
end

function load_prefab(path)
    -- ฟังก์ชันนี้ต้อง implement ใน engine
    return UI.load_prefab(path)
end

function update_hud_element(hud, element_name, properties)
    -- อัพเดท properties ของ element
    UI.update_element(hud, element_name, properties)
end
```

## สรุป

**ปัญหา:** UI System ใหม่ยังไม่ได้ integrate กับ engine rendering pipeline

**วิธีแก้ชั่วคราว:**
1. ✅ ใช้ Console แสดงข้อมูล (ใช้งานได้ทันที)
2. ✅ ใช้ Debug.draw_text (ถ้า engine รองรับ)
3. ✅ ใช้ World UI (text entities ในโลกเกม)
4. ⏳ รอ UI System integrate เสร็จ

**สิ่งที่ต้องทำต่อ:**
- Implement UI rendering ใน engine
- เชื่อม UI System กับ game loop
- เพิ่ม Lua bindings สำหรับ UI functions

## ตัวอย่างที่ใช้งานได้ทันที

```lua
-- simple_ui_test.lua
local hp = 100
local frame = 0

function on_update(dt)
    frame = frame + 1
    
    -- แสดงข้อมูลทุก 30 frames
    if frame % 30 == 0 then
        print("=== GAME STATUS ===")
        print("HP: " .. hp)
        print("Frame: " .. frame)
        print("FPS: " .. math.floor(1.0/dt))
        print("==================")
    end
    
    -- ทดสอบ: กด H เพื่อลด HP
    if Input.is_key_pressed("H") then
        hp = math.max(0, hp - 10)
        print(">>> HP decreased to: " .. hp)
    end
    
    -- ทดสอบ: กด R เพื่อ reset
    if Input.is_key_pressed("R") then
        hp = 100
        print(">>> HP reset to: " .. hp)
    end
end
```

**วิธีใช้:**
1. บันทึกเป็น `scripts/simple_ui_test.lua`
2. Attach script นี้กับ Player entity
3. กด Play
4. กด H เพื่อลด HP
5. กด R เพื่อ reset
6. ดูผลลัพธ์ใน Console

---

**หมายเหตุ:** เมื่อ UI System integrate เสร็จแล้ว จะสามารถใช้ตัวอย่างใน `INGAME_UI_EXAMPLE.md` ได้เต็มรูปแบบ
