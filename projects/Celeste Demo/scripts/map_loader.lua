-- Map Loader Script
-- โหลดและจัดการ LDtk map

local map_entity = nil
local ldtk_runtime = nil

function on_start()
    print("Map Loader: Starting...")
    
    -- สร้าง LDtk runtime สำหรับ hot-reload
    ldtk_runtime = LdtkRuntime.new()
    
    -- โหลด map
    local map_path = "levels/Level_01.ldtk"
    print("Loading map: " .. map_path)
    
    local success = ldtk_runtime:load(map_path)
    if success then
        print("✓ Map loaded successfully!")
    else
        print("✗ Failed to load map")
    end
end

function on_update(dt)
    -- ตรวจสอบ hot-reload
    if ldtk_runtime and ldtk_runtime:update() then
        print("🔄 Map hot-reloaded!")
        on_map_reloaded()
    end
end

function on_map_reloaded()
    -- เรียกเมื่อ map reload
    -- ใช้สำหรับ reset game state
    print("Map reloaded - resetting game state...")
    
    -- ตัวอย่าง: reset player position
    -- reset_player()
end

function on_destroy()
    print("Map Loader: Cleaning up...")
    if ldtk_runtime then
        ldtk_runtime = nil
    end
end
