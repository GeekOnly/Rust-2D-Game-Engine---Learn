-- Unity-Style Camera Follow Script
-- Uses Entity reference parameter (like Unity's public GameObject)

-- ============================================
-- PARAMETERS (Set in Inspector)
-- ============================================
playerTarget = nil        -- Entity reference (drag player entity here in inspector)
smooth_speed = 5.0        -- How fast camera moves (higher = faster, 0 = instant)
offset_x = 0.0            -- Camera offset from player
offset_y = 0.0

function on_start()
    print("Camera Follow (Unity Style): Started")
    
    if playerTarget then
        print("Camera Follow: Target entity is " .. playerTarget)
        
        -- Snap to player immediately
        local player_pos = get_position_of(playerTarget)
        if player_pos then
            local cam_pos = get_position()
            if cam_pos then
                set_position(player_pos.x + offset_x, player_pos.y + offset_y, cam_pos.z)
            end
        end
    else
        print("Camera Follow: Warning - No target entity assigned!")
        print("Please assign a target in the Inspector")
    end
end

function on_update(entity, dt)
    -- Check if target is assigned
    if not playerTarget then
        return
    end
    
    -- Get target position
    local target_pos = get_position_of(playerTarget)
    if not target_pos then
        return
    end
    
    -- Get current camera position
    local cam_pos = get_position()
    if not cam_pos then
        return
    end
    
    -- Calculate desired position
    local desired_x = target_pos.x + offset_x
    local desired_y = target_pos.y + offset_y
    
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
