-- Simple HUD Controller
-- Shows FPS and demonstrates UI API usage

local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"

local frame_count = 0
local elapsed_time = 0
local total_time = 0
local hud_loaded = false

function Start()
    print("=== HUD Controller Simple: Starting ===")
    
    -- Load and activate HUD
    UI.load_prefab(hud_prefab_path)
    UI.activate_prefab(hud_prefab_path, hud_instance_name)
    
    hud_loaded = true
    print("=== HUD Controller Simple: HUD Loaded ===")
    
    -- Set initial text
    UI.set_text(hud_instance_name .. "/position_debug", "X: 0.0 Y: 0.0")
    UI.set_text(hud_instance_name .. "/velocity_debug", "VX: 0.0 VY: 0.0")
    UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Ready")
    
    -- Hide some indicators initially
    UI.hide_element(hud_instance_name .. "/grounded_indicator")
    UI.hide_element(hud_instance_name .. "/wall_slide_indicator")
    UI.hide_element(hud_instance_name .. "/dashing_indicator")
end

function Update(dt)
    if not hud_loaded then
        return
    end
    
    total_time = total_time + dt
    
    -- Update FPS counter
    UpdateFPS(dt)
    
    -- Demo: Animate dashing indicator (blink every second)
    if math.floor(total_time) % 2 == 0 then
        UI.show_element(hud_instance_name .. "/dashing_indicator")
    else
        UI.hide_element(hud_instance_name .. "/dashing_indicator")
    end
    
    -- Demo: Animate health bar (sine wave)
    local health_percent = (math.sin(total_time) + 1.0) / 2.0
    UI.set_image_fill(hud_instance_name .. "/player_health_fill", health_percent)
    
    -- Change health bar color based on health
    if health_percent < 0.3 then
        -- Red when low
        UI.set_color(hud_instance_name .. "/player_health_fill", {r=1.0, g=0.0, b=0.0, a=1.0})
    elseif health_percent < 0.6 then
        -- Yellow when medium
        UI.set_color(hud_instance_name .. "/player_health_fill", {r=1.0, g=0.8, b=0.0, a=1.0})
    else
        -- Green when high
        UI.set_color(hud_instance_name .. "/player_health_fill", {r=0.2, g=1.0, b=0.3, a=1.0})
    end
    
    -- Demo: Animate stamina bar (cosine wave)
    local stamina_percent = (math.cos(total_time * 1.5) + 1.0) / 2.0
    UI.set_image_fill(hud_instance_name .. "/stamina_bar_fill", stamina_percent)
    
    -- Demo: Show grounded indicator every 3 seconds
    if math.floor(total_time) % 3 == 0 then
        UI.show_element(hud_instance_name .. "/grounded_indicator")
    else
        UI.hide_element(hud_instance_name .. "/grounded_indicator")
    end
    
    -- Demo: Update position (fake animation)
    local fake_x = math.sin(total_time * 0.5) * 10
    local fake_y = math.cos(total_time * 0.5) * 5
    UI.set_text(hud_instance_name .. "/position_debug", string.format("X: %.1f Y: %.1f", fake_x, fake_y))
    
    -- Demo: Update velocity (fake animation)
    local fake_vx = math.cos(total_time) * 5
    local fake_vy = math.sin(total_time) * 3
    UI.set_text(hud_instance_name .. "/velocity_debug", string.format("VX: %.1f VY: %.1f", fake_vx, fake_vy))
end

function UpdateFPS(dt)
    frame_count = frame_count + 1
    elapsed_time = elapsed_time + dt
    
    -- Update every 0.5 seconds
    if elapsed_time >= 0.5 then
        local fps = math.floor(frame_count / elapsed_time)
        UI.set_text(hud_instance_name .. "/fps_counter", "FPS: " .. fps)
        
        frame_count = 0
        elapsed_time = 0
    end
end
