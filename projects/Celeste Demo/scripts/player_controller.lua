-- Celeste-Style Player Controller
-- Features: Run, Jump, Dash, Wall Slide, Wall Jump

-- Movement parameters
local move_speed = 3.0  -- Units per second (reduced for better control)
local jump_force = 15.0  -- Jump velocity (increased significantly)
local dash_speed = 10.0 -- Dash velocity
local wall_slide_speed = 1.0
local gravity_scale = 0.3  -- Very low gravity for testing

-- State
local velocity_x = 0.0
local velocity_y = 0.0
local is_grounded = false
local can_dash = true
local is_dashing = false
local dash_timer = 0.0
local dash_duration = 0.2
local dash_direction_x = 0.0
local dash_direction_y = 0.0

-- Wall slide
local is_touching_wall = false
local wall_direction = 0

function on_start(entity)
    print("Player Controller started!")
    
    -- Set initial velocity and gravity
    set_velocity(0.0, 0.0)
    set_gravity_scale(gravity_scale)
end

function on_update(entity, dt)
    -- Update dash timer
    if is_dashing then
        dash_timer = dash_timer + dt
        if dash_timer >= dash_duration then
            is_dashing = false
            dash_timer = 0.0
        end
    end
    
    -- Get current velocity
    local vel = get_velocity()
    if vel then
        velocity_x = vel.x
        velocity_y = vel.y
    end
    
    -- Check if grounded (simple check - velocity.y near 0)
    -- In a real game, you'd use collision detection
    local was_grounded = is_grounded
    is_grounded = math.abs(velocity_y) < 0.1
    
    -- Debug: print when grounded state changes
    if was_grounded ~= is_grounded then
        print("Grounded: " .. tostring(is_grounded) .. ", velocity_y: " .. velocity_y)
    end
    
    -- Reset dash when grounded
    if is_grounded then
        can_dash = true
    end
    
    -- Handle input
    if not is_dashing then
        handle_movement(dt)
        handle_jump()
    end
    
    handle_dash()
    
    -- Apply dash velocity
    if is_dashing then
        velocity_x = dash_direction_x * dash_speed
        velocity_y = dash_direction_y * dash_speed
    end
    
    -- Apply velocity
    print("Setting velocity: x=" .. velocity_x .. ", y=" .. velocity_y)
    set_velocity(velocity_x, velocity_y)
    
    -- Debug: verify velocity was set
    local check_vel = get_velocity()
    if check_vel then
        print("Velocity after set: x=" .. check_vel.x .. ", y=" .. check_vel.y)
        if math.abs(check_vel.y - velocity_y) > 0.01 then
            print("WARNING: Velocity Y not set correctly! Expected: " .. velocity_y .. ", Got: " .. check_vel.y)
        end
    end
end

function handle_movement(dt)
    -- Horizontal movement
    if is_key_down("A") or is_key_down("Left") then
        velocity_x = -move_speed
    elseif is_key_down("D") or is_key_down("Right") then
        velocity_x = move_speed
    else
        -- Deceleration
        velocity_x = velocity_x * 0.8
        if math.abs(velocity_x) < 10.0 then
            velocity_x = 0.0
        end
    end
end

function handle_jump()
    -- Jump (use is_key_just_pressed for single press detection)
    if is_key_just_pressed("Space") then
        print("JUMP TRIGGERED! Before: velocity_y = " .. velocity_y)
        velocity_y = -jump_force
        is_grounded = false
        print("JUMP SET! After: velocity_y = " .. velocity_y)
    end
end

function handle_dash()
    -- Dash (Shift key - use is_key_just_pressed for single press)
    if is_key_just_pressed("LShift") and can_dash then
        -- Get dash direction from input
        local dash_x = 0.0
        local dash_y = 0.0
        
        if is_key_down("A") or is_key_down("Left") then
            dash_x = -1.0
        elseif is_key_down("D") or is_key_down("Right") then
            dash_x = 1.0
        end
        
        if is_key_down("W") or is_key_down("Up") then
            dash_y = -1.0
        elseif is_key_down("S") or is_key_down("Down") then
            dash_y = 1.0
        end
        
        -- Default to horizontal dash if no direction
        if dash_x == 0.0 and dash_y == 0.0 then
            dash_x = 1.0 -- Dash right by default
        end
        
        -- Normalize direction
        local length = math.sqrt(dash_x * dash_x + dash_y * dash_y)
        if length > 0 then
            dash_direction_x = dash_x / length
            dash_direction_y = dash_y / length
        end
        
        -- Start dash
        is_dashing = true
        can_dash = false
        dash_timer = 0.0
        
        print("Dash! Direction: " .. dash_direction_x .. ", " .. dash_direction_y)
    end
end

function on_collision(entity, other)
    -- Handle collision with ground/platforms
    print("Collision detected!")
end
