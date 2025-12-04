-- Simple Camera Follow Script
-- Basic smooth camera that follows the player

-- Parameters (can be set in inspector)
target_entity = nil       -- Entity reference (drag player here, or leave nil to auto-find by tag)
smooth_speed = 5.0        -- How fast camera moves (higher = faster, 0 = instant)
offset_x = 0.0            -- Camera offset from player
offset_y = 0.0

function on_start()
    print("Camera Follow: Started")
    
    -- If no target assigned, try to find player by tag
    if not target_entity then
        print("Camera Follow: No target assigned, searching for Player tag...")
        local all_entities = get_all_entities()
        for _, ent in ipairs(all_entities) do
            local tags = get_tags(ent)
            if tags then
                for _, tag in ipairs(tags) do
                    if tag == "Player" then
                        target_entity = ent
                        print("Camera Follow: Found player entity " .. ent)
                        break
                    end
                end
            end
            if target_entity then break end
        end
    else
        print("Camera Follow: Using assigned target entity " .. target_entity)
    end
    
    if not target_entity then
        print("Camera Follow: Warning - No player found!")
        return
    end
    
    -- Snap to player immediately
    local player_pos = get_position_of(target_entity)
    if player_pos then
        local cam_pos = get_position()
        if cam_pos then
            set_position(player_pos.x + offset_x, player_pos.y + offset_y, cam_pos.z)
        end
    end
end

function on_update(entity, dt)
    -- Note: entity parameter is passed by engine but we don't need it
    -- All API functions already know which entity they're operating on
    if not target_entity then
        return
    end
    
    -- Get player position
    local player_pos = get_position_of(target_entity)
    if not player_pos then
        return
    end
    
    -- Get current camera position
    local cam_pos = get_position()
    if not cam_pos then
        return
    end
    
    -- Calculate desired position
    local desired_x = player_pos.x + offset_x
    local desired_y = player_pos.y + offset_y
    
    -- Smooth movement
    local new_x = cam_pos.x
    local new_y = cam_pos.y
    
    if smooth_speed > 0 then
        -- Lerp towards target
        local t = math.min(smooth_speed * dt, 1.0)
        new_x = cam_pos.x + (desired_x - cam_pos.x) * t
        new_y = cam_pos.y + (desired_y - cam_pos.y) * t
    else
        -- Instant follow
        new_x = desired_x
        new_y = desired_y
    end
    
    -- Update camera position (keep Z unchanged)
    local z = cam_pos.z or -10.0
    set_position(new_x, new_y, z)
end
