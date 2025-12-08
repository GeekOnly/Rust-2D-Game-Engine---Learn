-- HUD Update Test
-- Load UI in Update() instead of Start() to test if Start() is the problem

local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"  -- ใช้ instance ที่มีอยู่แล้วจาก Editor
local update_count = 0
local hud_loaded = false
local load_attempted = false

function Awake()
    log("=== HUD UPDATE TEST: Awake() called ===")
end

function Start()
    log("========================================")
    log("=== HUD UPDATE TEST: Start() called ===")
    log("========================================")
end

function Update(dt)
    update_count = update_count + 1
    
    -- Try to load UI on first update (frame 1)
    if not load_attempted then
        log("========================================")
        log("=== LOADING UI IN UPDATE (FRAME 1) ===")
        log("========================================")
        
        log("Loading prefab: " .. hud_prefab_path)
        UI.load_prefab(hud_prefab_path)
        
        log("Activating prefab as: " .. hud_instance_name)
        UI.activate_prefab(hud_prefab_path, hud_instance_name)
        
        log("Setting initial text...")
        UI.set_text(hud_instance_name .. "/fps_counter", "LOADED IN UPDATE!")
        UI.set_text(hud_instance_name .. "/position_debug", "Position: UPDATE TEST")
        UI.set_text(hud_instance_name .. "/velocity_debug", "Velocity: UPDATE TEST")
        
        hud_loaded = true
        load_attempted = true
        
        log("========================================")
        log("=== UI LOADED IN UPDATE! ===")
        log("========================================")
    end
    
    -- Update every 60 frames
    if update_count % 60 == 0 then
        log("=== UPDATE TEST: Update() called " .. update_count .. " times ===")
        
        if hud_loaded then
            UI.set_text(hud_instance_name .. "/fps_counter", "Updates: " .. update_count)
            log("UI.set_text called - count: " .. update_count)
        else
            log("WARNING: HUD not loaded!")
        end
    end
end
