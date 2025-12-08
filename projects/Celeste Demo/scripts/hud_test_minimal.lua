-- Minimal HUD Test
-- Test if UI system works at all

local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"
local update_count = 0
local hud_loaded = false

function Awake()
    log("=== MINIMAL HUD TEST: Awake() called ===")
end

function Start()
    log("=== MINIMAL HUD TEST: Start() called ===")
    
    -- Load and activate HUD
    log("Loading prefab: " .. hud_prefab_path)
    UI.load_prefab(hud_prefab_path)
    
    log("Activating prefab as: " .. hud_instance_name)
    UI.activate_prefab(hud_prefab_path, hud_instance_name)
    
    log("Setting initial text...")
    UI.set_text(hud_instance_name .. "/fps_counter", "TEST: Start() called")
    UI.set_text(hud_instance_name .. "/position_debug", "Position: TEST")
    UI.set_text(hud_instance_name .. "/velocity_debug", "Velocity: TEST")
    
    hud_loaded = true
    log("=== MINIMAL HUD TEST: Start() complete ===")
end

function Update(dt)
    update_count = update_count + 1
    
    -- Log every 60 frames (about 1 second)
    if update_count % 60 == 0 then
        log("=== MINIMAL HUD TEST: Update() called " .. update_count .. " times ===")
        
        if hud_loaded then
            -- Update FPS counter with update count
            UI.set_text(hud_instance_name .. "/fps_counter", "Updates: " .. update_count)
            log("UI.set_text called for fps_counter")
        else
            log("WARNING: HUD not loaded yet!")
        end
    end
end
