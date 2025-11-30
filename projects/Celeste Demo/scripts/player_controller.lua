-- Celeste-Style Player Controller
-- Features: Run, Jump, Dash, Wall Slide, Wall Jump

-- Movement parameters
local move_speed = 3.0  -- Units per second (reduced for better control)
local jump_force = 25.0  -- Jump velocity (high enough to separate from ground in 1 frame)
local max_jump_height = 2.0  -- Maximum jump height in units (reduced to compensate for higher jump force)
local dash_speed = 10.0 -- Dash velocity
local wall_slide_speed = 1.0
local gravity_scale = 1.0  -- Normal gravity
local jump_start_y = 0.0  -- Track where jump started

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
local just_jumped = false  -- Flag to prevent collision reset on jump frame

-- Wall slide
local is_touching_wall = false
local wall_direction = 0

-- Unity-style lifecycle: Awake is called when script is loaded
function Awake()
    -- Initialization
end

-- Unity-style lifecycle: Start is called before first Update
function Start()
    -- Set initial velocity and gravity
    set_velocity(0.0, 0.0)
    set_gravity_scale(gravity_scale)
end

-- Unity-style lifecycle: Update is called every frame
function Update(dt)
    -- Update dash timer
    if is_dashing then
        dash_timer = dash_timer + dt
        if dash_timer >= dash_duration then
            is_dashing = false
            dash_timer = 0.0
        end
    end
    
    -- Get current velocity from physics (includes gravity)
    local vel = get_velocity()
    if vel then
        velocity_x = vel.x
        velocity_y = vel.y
    end
    
    -- ✅ RAPIER GROUND CHECK - แม่นยำ 100%
    -- ใช้ contact normals จาก Rapier แทนการเช็คตำแหน่ง
    is_grounded = is_grounded_rapier
    
    -- Debug: log ground state
    if is_grounded then
        log("🟢 GROUNDED (can jump)")
    else
        log("🔴 IN AIR (cannot jump)")
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
        set_velocity(velocity_x, velocity_y)  -- Apply dash velocity
    end
    
    -- Note: We only call set_velocity when we actually want to change it
    -- This allows physics (gravity) to work naturally
end

function handle_movement(dt)
    -- Horizontal movement
    if is_key_down("A") or is_key_down("Left") then
        velocity_x = -move_speed
        set_velocity(velocity_x, velocity_y)  -- Update velocity
    elseif is_key_down("D") or is_key_down("Right") then
        velocity_x = move_speed
        set_velocity(velocity_x, velocity_y)  -- Update velocity
    else
        -- Deceleration
        local new_vel_x = velocity_x * 0.8
        if math.abs(new_vel_x) < 0.1 then
            new_vel_x = 0.0
        end
        if new_vel_x ~= velocity_x then
            velocity_x = new_vel_x
            set_velocity(velocity_x, velocity_y)  -- Update velocity
        end
    end
end

function handle_jump()
    -- Jump (use is_key_just_pressed for single press detection)
    if is_key_just_pressed("Space") then
        log(string.format("🎮 SPACE PRESSED! is_grounded=%s", tostring(is_grounded)))
        if is_grounded then
            -- Record jump start position
            local pos = get_position()
            if pos then
                jump_start_y = pos.y
            end
            
            -- Apply jump force (positive Y = up)
            velocity_y = jump_force
            is_grounded = false  -- Immediately set to false to prevent double jump
            set_velocity(velocity_x, velocity_y)  -- Apply jump velocity
            log(string.format("✅ JUMPED! velocity_y = %.1f", velocity_y))
        else
            log("❌ CANNOT JUMP - not grounded")
        end
    end
    
    -- Variable jump height: if player releases Space while going up, reduce velocity
    -- This gives more control over jump height (Celeste-style)
    if not is_key_down("Space") and velocity_y > 0.0 then
        -- Player released jump button while going up - cut velocity for shorter jump
        velocity_y = velocity_y * 0.5
        set_velocity(velocity_x, velocity_y)  -- Apply modified velocity
    end
    
    -- Limit max jump height
    local pos = get_position()
    if pos and velocity_y > 0.0 then
        local jump_height = jump_start_y - pos.y  -- How high we've jumped
        if jump_height > max_jump_height then
            -- Reached max height - stop upward movement
            velocity_y = 0.0
            set_velocity(velocity_x, velocity_y)  -- Stop upward movement
        end
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
        log("DASH!")
    end
end

-- Note: We use Rapier's contact-based ground detection (is_grounded_rapier)
-- instead of OnCollisionEnter callbacks, so this function is not needed
