# Debug HUD Not Showing

## Quick Debug Steps

### 1. Check Console Messages

Open **Console** tab (bottom panel) and look for:

✅ **Success Messages:**
```
✅ HUD bindings configured
✅ HUD asset is loaded
✅ HUD loaded successfully
HUD has 3 elements
```

❌ **Error Messages:**
```
⚠️ No HUD asset loaded yet
⚠️ HUD file not found
❌ Failed to load HUD: [error]
No HUD Manager provided to render_game_view!
HUD Manager has no HUD loaded!
```

### 2. Verify You're in Game View

- Look at the tab name - should say **"Game"** (not "Scene")
- Scene View doesn't show HUD by design
- Game View has a toolbar at the top with Resolution dropdown

### 3. Check HUD File Exists

File should be at:
```
projects/Celeste Demo/assets/ui/test_hud.hud
```

Or:
```
projects/Celeste Demo/assets/ui/celeste_hud.hud
```

### 4. Test with Simple HUD

The `test_hud.hud` file has 3 simple text elements:
- **Center**: "HUD IS WORKING!" (Red, 32px)
- **Top Left**: "TOP LEFT" (Green, 24px)
- **Bottom Right**: "BOTTOM RIGHT" (Blue, 24px)

If you see these, HUD system is working!

### 5. Check Resolution Settings

In Game View toolbar:
- **Resolution**: Try "Free (Fit to Window)" first
- **Scale**: Set to 100%
- **Info**: Enable to see resolution overlay

### 6. Rebuild and Restart

```bash
cargo build --package engine
cargo run --package engine
```

Then:
1. Open Celeste Demo
2. Go to Game tab
3. Check Console for messages

## Common Issues

### Issue: "No HUD Manager provided"
**Solution**: This is a code issue. HUD Manager should be passed to render_game_view.

### Issue: "HUD Manager has no HUD loaded"
**Solution**: 
- Check HUD file exists
- Check console for load errors
- Verify path is correct

### Issue: "HUD file not found"
**Solution**:
- Create `test_hud.hud` in `assets/ui/` folder
- Or check if `celeste_hud.hud` exists
- Verify project path is correct

### Issue: HUD loads but doesn't show
**Solution**:
- Check you're in Game View (not Scene View)
- Verify camera exists in scene
- Check resolution settings
- Look for render errors in console

### Issue: HUD shows in wrong position
**Solution**:
- Check anchor points in HUD file
- Verify offset values
- Try "Free" resolution mode
- Check screen dimensions

## Debug Checklist

- [ ] In Game View tab (not Scene)
- [ ] Console shows "HUD loaded successfully"
- [ ] Console shows "HUD has X elements"
- [ ] test_hud.hud file exists
- [ ] Camera entity exists in scene
- [ ] Resolution set to "Free" mode
- [ ] No errors in console
- [ ] Rebuilt after changes

## Still Not Working?

### Enable Debug Logging

The code now has debug logs. Check console for:
```
Rendering HUD: [width]x[height]
```

If you see this, HUD is trying to render.

### Check HUD Asset Content

Open `test_hud.hud` and verify:
- JSON is valid
- `visible: true` for all elements
- Anchor points are correct
- Colors have alpha > 0

### Verify Integration

Check that:
1. `EditorState` has `hud_manager` field
2. `render_game_view` receives `hud_manager` parameter
3. `TabContext` includes `hud_manager`
4. HUD is passed through all layers

## Test HUD File

If `test_hud.hud` doesn't exist, create it:

```json
{
  "name": "Test HUD",
  "elements": [
    {
      "id": "test_text",
      "element_type": {
        "type": "Text",
        "text": "HUD IS WORKING!",
        "font_size": 32.0,
        "color": [1.0, 0.0, 0.0, 1.0]
      },
      "anchor": "Center",
      "offset": [0.0, 0.0],
      "size": [300.0, 50.0],
      "visible": true
    }
  ]
}
```

This should show red text in the center of the screen.

## Next Steps

Once HUD shows:
1. Switch to `celeste_hud.hud`
2. Test different resolutions
3. Customize HUD layout
4. Add more elements

---

**If still not working, check the console output and report the error messages.**
