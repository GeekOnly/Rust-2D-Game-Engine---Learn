-- Lua UI Example Script
-- This script demonstrates how to build UI dynamically using the Lua API

-- ============================================================================
-- Example 1: Creating a Simple Menu
-- ============================================================================

function create_main_menu()
    print("=== Creating Main Menu ===")
    
    -- Create canvas
    local canvas = ui_create_canvas({
        render_mode = "ScreenSpaceOverlay",
        sort_order = 0
    })
    print("Created canvas: " .. canvas)
    
    -- Create background panel
    local panel = ui_create_panel({
        parent = canvas,
        background = "panel_bg"
    })
    ui_set_position(panel, 0, 0)
    ui_set_size(panel, 400, 500)
    ui_set_anchor_min(panel, 0.5, 0.5)
    ui_set_anchor_max(panel, 0.5, 0.5)
    ui_set_pivot(panel, 0.5, 0.5)
    ui_set_color({entity = panel, r = 0.2, g = 0.2, b = 0.3, a = 0.95})
    print("Created panel: " .. panel)
    
    -- Create title text
    local title = ui_create_text({
        parent = panel,
        text = "Main Menu",
        font_size = 36,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_position(title, 0, 180)
    ui_set_anchor_min(title, 0.5, 0.5)
    ui_set_anchor_max(title, 0.5, 0.5)
    ui_set_text_alignment(title, "MiddleCenter")
    print("Created title: " .. title)
    
    -- Create buttons
    local button_names = {"Start Game", "Options", "Credits", "Quit"}
    local button_y_positions = {80, 20, -40, -100}
    
    for i, name in ipairs(button_names) do
        local button = create_menu_button(panel, name, button_y_positions[i])
        print("Created button '" .. name .. "': " .. button)
    end
    
    print("Main menu created successfully!")
    return canvas
end

function create_menu_button(parent, text, y_pos)
    -- Create button background
    local button = ui_create_button({
        parent = parent,
        on_click = "on_menu_button_click"
    })
    ui_set_position(button, 0, y_pos)
    ui_set_size(button, 250, 50)
    ui_set_anchor_min(button, 0.5, 0.5)
    ui_set_anchor_max(button, 0.5, 0.5)
    ui_set_pivot(button, 0.5, 0.5)
    ui_set_color({entity = button, r = 0.3, g = 0.5, b = 0.8, a = 1.0})
    ui_set_name(button, "button_" .. text:lower():gsub(" ", "_"))
    
    -- Create button text
    local button_text = ui_create_text({
        parent = button,
        text = text,
        font_size = 18,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_anchor_min(button_text, 0.0, 0.0)
    ui_set_anchor_max(button_text, 1.0, 1.0)
    ui_set_position(button_text, 0, 0)
    ui_set_text_alignment(button_text, "MiddleCenter")
    
    -- Add hover effects
    ui_on_pointer_enter(button, "on_button_hover_enter")
    ui_on_pointer_exit(button, "on_button_hover_exit")
    
    return button
end

-- ============================================================================
-- Example 2: Creating a Settings Panel with Sliders and Toggles
-- ============================================================================

function create_settings_panel()
    print("\n=== Creating Settings Panel ===")
    
    local canvas = ui_create_canvas({
        render_mode = "ScreenSpaceOverlay",
        sort_order = 1
    })
    
    local panel = ui_create_panel({
        parent = canvas,
        background = "settings_bg"
    })
    ui_set_position(panel, 0, 0)
    ui_set_size(panel, 500, 600)
    ui_set_anchor_min(panel, 0.5, 0.5)
    ui_set_anchor_max(panel, 0.5, 0.5)
    ui_set_pivot(panel, 0.5, 0.5)
    
    -- Title
    local title = ui_create_text({
        parent = panel,
        text = "Settings",
        font_size = 32,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_position(title, 0, 250)
    ui_set_anchor_min(title, 0.5, 0.5)
    ui_set_anchor_max(title, 0.5, 0.5)
    ui_set_text_alignment(title, "MiddleCenter")
    
    -- Volume slider
    create_slider_setting(panel, "Master Volume", 150, "volume_slider")
    
    -- Brightness slider
    create_slider_setting(panel, "Brightness", 50, "brightness_slider")
    
    -- Fullscreen toggle
    create_toggle_setting(panel, "Fullscreen", -50, "fullscreen_toggle")
    
    -- VSync toggle
    create_toggle_setting(panel, "VSync", -120, "vsync_toggle")
    
    -- Close button
    local close_button = ui_create_button({
        parent = panel,
        on_click = "on_close_settings"
    })
    ui_set_position(close_button, 0, -230)
    ui_set_size(close_button, 150, 40)
    ui_set_anchor_min(close_button, 0.5, 0.5)
    ui_set_anchor_max(close_button, 0.5, 0.5)
    ui_set_pivot(close_button, 0.5, 0.5)
    
    local close_text = ui_create_text({
        parent = close_button,
        text = "Close",
        font_size = 16
    })
    ui_set_anchor_min(close_text, 0.0, 0.0)
    ui_set_anchor_max(close_text, 1.0, 1.0)
    ui_set_text_alignment(close_text, "MiddleCenter")
    
    print("Settings panel created successfully!")
    return canvas
end

function create_slider_setting(parent, label, y_pos, name)
    -- Label
    local label_text = ui_create_text({
        parent = parent,
        text = label,
        font_size = 16,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_position(label_text, -150, y_pos + 20)
    ui_set_anchor_min(label_text, 0.5, 0.5)
    ui_set_anchor_max(label_text, 0.5, 0.5)
    ui_set_text_alignment(label_text, "MiddleLeft")
    
    -- Slider (placeholder - actual slider component would be created here)
    local slider_bg = ui_create_panel({
        parent = parent
    })
    ui_set_position(slider_bg, 0, y_pos)
    ui_set_size(slider_bg, 300, 20)
    ui_set_anchor_min(slider_bg, 0.5, 0.5)
    ui_set_anchor_max(slider_bg, 0.5, 0.5)
    ui_set_pivot(slider_bg, 0.5, 0.5)
    ui_set_color({entity = slider_bg, r = 0.3, g = 0.3, b = 0.3, a = 1.0})
    ui_set_name(slider_bg, name)
    
    -- Value display
    local value_text = ui_create_text({
        parent = parent,
        text = "50%",
        font_size = 14,
        color = {r = 0.8, g = 0.8, b = 0.8, a = 1.0}
    })
    ui_set_position(value_text, 170, y_pos)
    ui_set_anchor_min(value_text, 0.5, 0.5)
    ui_set_anchor_max(value_text, 0.5, 0.5)
    ui_set_text_alignment(value_text, "MiddleLeft")
end

function create_toggle_setting(parent, label, y_pos, name)
    -- Label
    local label_text = ui_create_text({
        parent = parent,
        text = label,
        font_size = 16,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_position(label_text, -150, y_pos)
    ui_set_anchor_min(label_text, 0.5, 0.5)
    ui_set_anchor_max(label_text, 0.5, 0.5)
    ui_set_text_alignment(label_text, "MiddleLeft")
    
    -- Toggle box (placeholder - actual toggle component would be created here)
    local toggle_box = ui_create_panel({
        parent = parent
    })
    ui_set_position(toggle_box, 150, y_pos)
    ui_set_size(toggle_box, 40, 40)
    ui_set_anchor_min(toggle_box, 0.5, 0.5)
    ui_set_anchor_max(toggle_box, 0.5, 0.5)
    ui_set_pivot(toggle_box, 0.5, 0.5)
    ui_set_color({entity = toggle_box, r = 0.3, g = 0.3, b = 0.3, a = 1.0})
    ui_set_name(toggle_box, name)
end

-- ============================================================================
-- Example 3: Creating a HUD with Dynamic Elements
-- ============================================================================

function create_game_hud()
    print("\n=== Creating Game HUD ===")
    
    local canvas = ui_create_canvas({
        render_mode = "ScreenSpaceOverlay",
        sort_order = 10
    })
    
    -- Health bar (top-left)
    create_health_bar(canvas)
    
    -- Score display (top-right)
    create_score_display(canvas)
    
    -- Minimap (bottom-right)
    create_minimap(canvas)
    
    -- Ability cooldowns (bottom-center)
    create_ability_bar(canvas)
    
    print("Game HUD created successfully!")
    return canvas
end

function create_health_bar(parent)
    -- Background
    local bg = ui_create_panel({parent = parent})
    ui_set_position(bg, 20, -20)
    ui_set_size(bg, 200, 30)
    ui_set_anchor_min(bg, 0.0, 1.0)
    ui_set_anchor_max(bg, 0.0, 1.0)
    ui_set_pivot(bg, 0.0, 1.0)
    ui_set_color({entity = bg, r = 0.2, g = 0.2, b = 0.2, a = 0.8})
    ui_set_name(bg, "health_bar_bg")
    
    -- Fill
    local fill = ui_create_image({parent = bg})
    ui_set_position(fill, 0, 0)
    ui_set_size(fill, 200, 30)
    ui_set_anchor_min(fill, 0.0, 0.0)
    ui_set_anchor_max(fill, 0.0, 0.0)
    ui_set_pivot(fill, 0.0, 0.0)
    ui_set_color({entity = fill, r = 0.8, g = 0.2, b = 0.2, a = 1.0})
    ui_set_name(fill, "health_bar_fill")
    
    -- Text
    local text = ui_create_text({
        parent = bg,
        text = "HP: 100/100",
        font_size = 14,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_anchor_min(text, 0.0, 0.0)
    ui_set_anchor_max(text, 1.0, 1.0)
    ui_set_text_alignment(text, "MiddleCenter")
    ui_set_name(text, "health_bar_text")
end

function create_score_display(parent)
    local score_text = ui_create_text({
        parent = parent,
        text = "Score: 0",
        font_size = 24,
        color = {r = 1.0, g = 1.0, b = 0.0, a = 1.0}
    })
    ui_set_position(score_text, -20, -20)
    ui_set_anchor_min(score_text, 1.0, 1.0)
    ui_set_anchor_max(score_text, 1.0, 1.0)
    ui_set_pivot(score_text, 1.0, 1.0)
    ui_set_text_alignment(score_text, "MiddleRight")
    ui_set_name(score_text, "score_text")
end

function create_minimap(parent)
    local minimap = ui_create_panel({parent = parent})
    ui_set_position(minimap, -20, 20)
    ui_set_size(minimap, 150, 150)
    ui_set_anchor_min(minimap, 1.0, 0.0)
    ui_set_anchor_max(minimap, 1.0, 0.0)
    ui_set_pivot(minimap, 1.0, 0.0)
    ui_set_color({entity = minimap, r = 0.1, g = 0.1, b = 0.1, a = 0.7})
    ui_set_name(minimap, "minimap")
    
    -- Minimap border
    local border = ui_create_image({parent = minimap})
    ui_set_anchor_min(border, 0.0, 0.0)
    ui_set_anchor_max(border, 1.0, 1.0)
    ui_set_color({entity = border, r = 0.5, g = 0.5, b = 0.5, a = 1.0})
end

function create_ability_bar(parent)
    local ability_count = 4
    local ability_size = 50
    local ability_spacing = 10
    local total_width = (ability_size * ability_count) + (ability_spacing * (ability_count - 1))
    
    for i = 1, ability_count do
        local x_offset = (i - 1) * (ability_size + ability_spacing) - (total_width / 2) + (ability_size / 2)
        
        local ability = ui_create_panel({parent = parent})
        ui_set_position(ability, x_offset, 20)
        ui_set_size(ability, ability_size, ability_size)
        ui_set_anchor_min(ability, 0.5, 0.0)
        ui_set_anchor_max(ability, 0.5, 0.0)
        ui_set_pivot(ability, 0.5, 0.0)
        ui_set_color({entity = ability, r = 0.3, g = 0.3, b = 0.5, a = 0.9})
        ui_set_name(ability, "ability_" .. i)
        
        -- Ability key text
        local key_text = ui_create_text({
            parent = ability,
            text = tostring(i),
            font_size = 16,
            color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
        })
        ui_set_anchor_min(key_text, 0.0, 0.0)
        ui_set_anchor_max(key_text, 1.0, 1.0)
        ui_set_text_alignment(key_text, "MiddleCenter")
    end
end

-- ============================================================================
-- Example 4: Animated Notification System
-- ============================================================================

function show_notification(message, duration)
    print("\n=== Showing Notification: " .. message .. " ===")
    
    -- Find or create notification canvas
    local canvas = ui_find_by_name("notification_canvas")
    if not canvas then
        canvas = ui_create_canvas({
            render_mode = "ScreenSpaceOverlay",
            sort_order = 100
        })
        ui_set_name(canvas, "notification_canvas")
    end
    
    -- Create notification panel
    local notification = ui_create_panel({parent = canvas})
    ui_set_position(notification, 0, 100)
    ui_set_size(notification, 400, 80)
    ui_set_anchor_min(notification, 0.5, 1.0)
    ui_set_anchor_max(notification, 0.5, 1.0)
    ui_set_pivot(notification, 0.5, 1.0)
    ui_set_color({entity = notification, r = 0.2, g = 0.2, b = 0.2, a = 0.0})
    
    -- Create notification text
    local text = ui_create_text({
        parent = notification,
        text = message,
        font_size = 18,
        color = {r = 1.0, g = 1.0, b = 1.0, a = 1.0}
    })
    ui_set_anchor_min(text, 0.0, 0.0)
    ui_set_anchor_max(text, 1.0, 1.0)
    ui_set_text_alignment(text, "MiddleCenter")
    
    -- Animate in
    ui_animate_position({
        entity = notification,
        to_x = 0,
        to_y = -20,
        duration = 0.3,
        easing = "EaseOutBack"
    })
    
    ui_animate_alpha({
        entity = notification,
        to = 0.95,
        duration = 0.3,
        easing = "EaseOutQuad"
    })
    
    -- Schedule fade out (would need timer system)
    print("Notification will auto-dismiss after " .. duration .. " seconds")
    
    return notification
end

-- ============================================================================
-- Event Handlers
-- ============================================================================

function on_menu_button_click()
    print("Menu button clicked!")
    
    -- Get the button that was clicked (would need event system to pass entity)
    -- For now, just demonstrate animation
    print("Playing button click animation...")
end

function on_button_hover_enter()
    print("Button hover enter")
    -- Scale up animation would go here
    -- ui_animate_scale({entity = button, to_x = 1.1, to_y = 1.1, duration = 0.2})
end

function on_button_hover_exit()
    print("Button hover exit")
    -- Scale down animation would go here
    -- ui_animate_scale({entity = button, to_x = 1.0, to_y = 1.0, duration = 0.2})
end

function on_close_settings()
    print("Closing settings panel...")
    -- Fade out and destroy would go here
end

function update_health(current, max)
    local health_fill = ui_find_by_name("health_bar_fill")
    local health_text = ui_find_by_name("health_bar_text")
    
    if health_fill then
        local percentage = current / max
        ui_set_size(health_fill, 200 * percentage, 30)
        
        -- Animate color based on health
        local r = 1.0 - (percentage * 0.2)
        local g = percentage * 0.8
        ui_animate_color({
            entity = health_fill,
            to_r = r,
            to_g = g,
            to_b = 0.2,
            to_a = 1.0,
            duration = 0.3
        })
    end
    
    if health_text then
        ui_set_text(health_text, "HP: " .. current .. "/" .. max)
    end
end

function update_score(score)
    local score_text = ui_find_by_name("score_text")
    if score_text then
        ui_set_text(score_text, "Score: " .. score)
        
        -- Pulse animation on score change
        ui_animate_scale({
            entity = score_text,
            to_x = 1.2,
            to_y = 1.2,
            duration = 0.1,
            easing = "EaseOutQuad"
        })
        -- Would need to scale back down after delay
    end
end

-- ============================================================================
-- Main Execution
-- ============================================================================

function init_ui()
    print("=== Initializing UI System ===\n")
    
    -- Create main menu
    local menu_canvas = create_main_menu()
    
    -- Create settings panel (hidden by default)
    -- local settings_canvas = create_settings_panel()
    -- ui_set_active(settings_canvas, false)
    
    -- Create game HUD
    -- local hud_canvas = create_game_hud()
    
    -- Show a notification
    -- show_notification("Welcome to the game!", 3.0)
    
    print("\n=== UI Initialization Complete ===")
    print("\nAvailable functions:")
    print("  - create_main_menu()")
    print("  - create_settings_panel()")
    print("  - create_game_hud()")
    print("  - show_notification(message, duration)")
    print("  - update_health(current, max)")
    print("  - update_score(score)")
end

-- Run initialization
init_ui()

-- Example usage:
-- update_health(75, 100)
-- update_score(1250)
-- show_notification("Level Up!", 2.0)

