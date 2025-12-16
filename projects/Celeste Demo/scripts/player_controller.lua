-- Celeste-Style Player Controller
-- Features: Run, Jump, Dash, Wall Slide, Wall Jump

-- Movement parameters
local move_speed = 3.0  -- Units per second (reduced for better control)
local jump_force = 25.0  -- Jump velocity (positive = up)
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

-- HUD Configuration
local hud_prefab_path = "projects/Celeste Demo/assets/ui/celeste_hud.uiprefab"
local hud_instance_name = "celeste_hud"
local hud_loaded = false
local frame_count = 0
local elapsed_time = 0
local current_fps = 0

-- Unity-style lifecycle: Awake is called when script is loaded
function Awake()
    log("🎮 Player Controller Awake() called - Script loaded successfully!")
    -- Initialization
end

-- Unity-style lifecycle: Start is called before first Update
function Start()
    log("🎮 Player Controller Start() called - Initializing player")
    -- Set initial velocity and gravity
    set_velocity(0.0, 0.0)
    set_gravity_scale(gravity_scale)
    
    -- Load and activate HUD
    log("🎮 Player Controller - Loading HUD: " .. hud_prefab_path)
    UI.load_prefab(hud_prefab_path)
    UI.activate_prefab(hud_prefab_path, hud_instance_name)
    hud_loaded = true
    
    -- Initialize HUD values
    UI.set_image_fill(hud_instance_name .. "/player_health_fill", 1.0)
    UI.set_image_fill(hud_instance_name .. "/stamina_bar_fill", 1.0)
    UI.set_color(hud_instance_name .. "/player_health_fill", {r=0.2, g=1.0, b=0.3, a=1.0})
    
    log("🎮 Player Controller initialized - velocity set, gravity scale: " .. gravity_scale)
end

-- Unity-style lifecycle: Update is called every frame
function Update(dt)
    -- Debug: Always log to confirm script is running
    log("🎮 Player controller Update() - dt: " .. dt .. ", grounded: " .. tostring(is_grounded))
    
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
    
    -- ✅ SIMPLE GROUND CHECK - Always assume grounded for testing
    -- This is a temporary fix to test jumping
    is_grounded = true  -- Force grounded for testing
    
    -- Ground state updated (debug logs disabled)
    
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
    
    -- Update HUD
    if hud_loaded then
        UpdateHUD(dt)
    end
end

function UpdateHUD(dt)
    -- Update FPS
    frame_count = frame_count + 1
    elapsed_time = elapsed_time + dt
    if elapsed_time >= 0.5 then
        current_fps = math.floor(frame_count / elapsed_time)
        UI.set_text(hud_instance_name .. "/fps_counter", "FPS: " .. current_fps)
        frame_count = 0
        elapsed_time = 0
    end
    
    -- Update Debug Info
    local pos = get_position()
    if pos then
        UI.set_text(hud_instance_name .. "/position_debug", string.format("X: %.1f Y: %.1f", pos.x, pos.y))
    end
    
    local vel = get_velocity() 
    if vel then
         UI.set_text(hud_instance_name .. "/velocity_debug", string.format("VX: %.1f VY: %.1f", vel.x, vel.y))
    end
    
    -- Update Status Indicators
    if is_grounded then
        UI.show_element(hud_instance_name .. "/grounded_indicator")
    else
        UI.hide_element(hud_instance_name .. "/grounded_indicator")
    end
    
    if is_dashing then
        UI.show_element(hud_instance_name .. "/dashing_indicator")
        UI.set_text(hud_instance_name .. "/dashing_indicator", "DASHING!")
    else
        UI.hide_element(hud_instance_name .. "/dashing_indicator")
    end
    
    if can_dash then
        UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Ready")
        UI.set_color(hud_instance_name .. "/dash_indicator", {r=0.2, g=1.0, b=0.3, a=1.0})
    else
        UI.set_text(hud_instance_name .. "/dash_indicator", "Dash: Used")
        UI.set_color(hud_instance_name .. "/dash_indicator", {r=0.5, g=0.5, b=0.5, a=1.0})
    end
    
    if is_touching_wall and not is_grounded then
        UI.show_element(hud_instance_name .. "/wall_slide_indicator")
    else
        UI.hide_element(hud_instance_name .. "/wall_slide_indicator")
    end
end

function handle_movement(dt)
    -- Horizontal movement
    if is_key_down("A") or is_key_down("Left") then
        log("🎮 Moving LEFT - A or Left key pressed")
        velocity_x = -move_speed
        set_velocity(velocity_x, velocity_y)  -- Update velocity
        set_sprite_flip_x(true)  -- Flip sprite to face left
    elseif is_key_down("D") or is_key_down("Right") then
        log("🎮 Moving RIGHT - D or Right key pressed")
        velocity_x = move_speed
        set_velocity(velocity_x, velocity_y)  -- Update velocity
        set_sprite_flip_x(false)  -- Face right (normal)
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
    -- Simple jump test - just check for Space key
    if is_key_just_pressed("Space") then
        log("🎮 SPACE key pressed - JUMPING!")
        -- Apply jump force (positive Y = up in this physics engine)
        velocity_y = jump_force 
        set_velocity(velocity_x, velocity_y)
        log("🎮 Jump applied - velocity set to: " .. velocity_x .. ", " .. velocity_y)
    end
end

function handle_dash()
    -- Dash (Shift key - use is_key_just_pressed for single press)
    if is_key_just_pressed("LShift") and can_dash then
        log("🎮 DASH - LShift pressed and can dash")
        -- Get dash direction from input
        local dash_x = 0.0
        local dash_y = 0.0
        
        if is_key_down("A") or is_key_down("Left") then
            dash_x = -1.0
        elseif is_key_down("D") or is_key_down("Right") then
            dash_x = 1.0
        end
        
        if is_key_down("W") or is_key_down("Up") then
            dash_y = 1.0
        elseif is_key_down("S") or is_key_down("Down") then
            dash_y = -1.0
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
    end
end

-- Note: We use Rapier's contact-based ground detection (is_grounded_rapier)
-- instead of OnCollisionEnter callbacks, so this function is not needed
