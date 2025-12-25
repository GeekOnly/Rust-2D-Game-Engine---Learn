-- Simple Input Test Script
print("🔥 test_input.lua FILE IS BEING LOADED!")

function Awake()
    log("🔧 Test Input Script - Awake called")
end

function Start()
    log("🔧 Test Input Script - Start called")
end

function Update(dt)
    -- Test A key
    if is_key_down("A") then
        log("✅ A key is DOWN")
    end

    -- Test D key
    if is_key_down("D") then
        log("✅ D key is DOWN")
    end

    -- Test Space key
    if is_key_down("Space") then
        log("✅ Space key is DOWN")
    end

    -- Test just pressed
    if is_key_just_pressed("A") then
        log("🔵 A key JUST PRESSED")
    end

    if is_key_just_pressed("D") then
        log("🔵 D key JUST PRESSED")
    end

    -- Try to get and set velocity
    local vel = get_velocity()
    if vel then
        log("📍 Current velocity: X=" .. vel.x .. ", Y=" .. vel.y)
    else
        log("❌ get_velocity() returned nil")
    end

    -- Try to set velocity when A is pressed
    if is_key_down("A") then
        log("🎯 Trying to set velocity to LEFT (-5, 0)")
        set_velocity(-5.0, 0.0)
    elseif is_key_down("D") then
        log("🎯 Trying to set velocity to RIGHT (5, 0)")
        set_velocity(5.0, 0.0)
    end
end
