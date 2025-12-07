-- UI Test Script for Celeste Demo
-- This script demonstrates the UI system

-- Player stats
local hp = 100
local max_hp = 100
local stamina = 100
local max_stamina = 100
local dash_count = 1
local fps = 60
local frame_count = 0

-- State
local is_grounded = false
local is_dashing = false

function on_start()
    print("=== UI Test Script Started ===")
    print("Press H to damage")
    print("Press R to restore")
    print("Press Shift to use stamina")
    print("==============================")
end

function on_update(entity, dt)
    frame_count = frame_count + 1
    fps = math.floor(1.0 / dt)
    
    -- Get player position
    local pos = get_position()
    local vel = get_velocity()
    
    -- Update stamina
    if is_key_down("LeftShift") then
        stamina = math.max(0, stamina - 50 * dt)
        is_dashing = true
    else
        stamina = math.min(max_stamina, stamina + 30 * dt)
        is_dashing = false
    end
    
    -- Test controls
    if is_key_just_pressed("H") then
        hp = math.max(0, hp - 10)
        print("HP: " .. hp .. "/" .. max_hp)
    end
    
    if is_key_just_pressed("R") then
        hp = max_hp
        stamina = max_stamina
        dash_count = 1
        print("Stats restored!")
    end
    
    -- Display info every 60 frames
    if frame_count % 60 == 0 then
        print("=== GAME STATUS ===")
        print("HP: " .. hp .. "/" .. max_hp .. " (" .. math.floor((hp/max_hp)*100) .. "%)")
        print("Stamina: " .. math.floor(stamina) .. "/" .. max_stamina)
        print("Dash: " .. dash_count)
        print("FPS: " .. fps)
        if pos then
            print("Position: X=" .. string.format("%.1f", pos.x) .. " Y=" .. string.format("%.1f", pos.y))
        end
        if vel then
            print("Velocity: VX=" .. string.format("%.1f", vel.x) .. " VY=" .. string.format("%.1f", vel.y))
        end
        print("Dashing: " .. tostring(is_dashing))
        print("==================")
    end
end
