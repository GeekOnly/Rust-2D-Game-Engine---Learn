-- Simple HUD Update Test
-- Assumes UI is already loaded by Editor
-- Just updates the text values

local update_count = 0

function Awake()
    log("=== SIMPLE HUD TEST: Awake() ===")
end

function Start()
    log("=== SIMPLE HUD TEST: Start() ===")
end

function Update(dt)
    update_count = update_count + 1
    
    -- Update every 60 frames (1 second at 60 FPS)
    if update_count % 60 == 0 then
        log("=== SIMPLE HUD TEST: Update " .. update_count .. " ===")
        
        -- Check if UI table exists
        if UI == nil then
            log("ERROR: UI table is NIL!")
        elseif type(UI) ~= "table" then
            log("ERROR: UI is not a table, type=" .. type(UI))
        elseif UI.set_text == nil then
            log("ERROR: UI.set_text is NIL!")
        else
            log("UI table exists, calling set_text...")
            UI.set_text("celeste_hud/fps_counter", "Updates: " .. update_count)
            log("UI.set_text called for fps_counter")
        end
    end
end
