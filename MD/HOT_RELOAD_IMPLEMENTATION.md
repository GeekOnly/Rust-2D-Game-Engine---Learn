# Hot-Reload System Implementation

## Overview

Implemented a complete hot-reload system for LDtk tilemap files that automatically detects file changes and reloads maps while preserving layer states and handling errors gracefully.

## Implementation Details

### 1. File Watching (Subtask 14.1)

**Created:** `engine/src/editor/hot_reload.rs`

- Implemented `HotReloadWatcher` struct using the `notify` crate with debouncing
- Added 500ms debounce delay to handle rapid file changes (e.g., during save operations)
- Supports watching individual files and directories recursively
- Filters events to only process `.ldtk` file changes (Create and Modify events)
- Non-blocking polling interface for integration with the main event loop

**Key Features:**
- Automatic debouncing prevents multiple reloads during rapid file changes
- Clean API for watching/unwatching files
- Error handling for file system watcher failures
- Thread-safe event channel for communicating file changes

### 2. Hot-Reload Logic (Subtask 14.2)

**Modified:** `engine/src/editor/map_manager.rs`

Added hot-reload functionality to MapManager:

- `hot_reload_watcher: Option<HotReloadWatcher>` - File watcher instance
- `hot_reload_enabled: bool` - Toggle for hot-reload functionality
- `last_hot_reload_error: Option<String>` - Stores last error for UI display

**New Methods:**
- `enable_hot_reload()` - Initialize the file watcher
- `disable_hot_reload()` - Disable hot-reload and cleanup watcher
- `set_hot_reload_enabled(bool)` - Toggle hot-reload on/off
- `process_hot_reload(&mut World)` - Main hot-reload processing (called each frame)
- `reload_map_with_error_recovery()` - Reload with state backup/restore

**Integration:**
- Files are automatically registered with the watcher when loaded via `load_map()`
- Files are unregistered when unloaded via `unload_map()`
- Hot-reload processing is called each frame in the main editor loop

### 3. Error Recovery (Subtask 14.3)

**Error Handling Strategy:**

1. **State Backup:** Before attempting reload, the current `LoadedMap` state is cloned
2. **Reload Attempt:** Try to reload the map using existing `reload_map()` method
3. **Error Recovery:** If reload fails, restore the backed-up state
4. **User Notification:** Error messages are logged and displayed in the console
5. **State Preservation:** Last valid state is maintained on error

**Error Types Handled:**
- File corruption (invalid JSON)
- Missing files
- Invalid LDtk format
- Entity/component errors during reload

### 4. Main Loop Integration

**Modified:** `engine/src/main.rs`

Added hot-reload processing in the editor update loop:

```rust
// Process hot-reload for LDtk maps
let reloaded_maps = editor_state.map_manager.process_hot_reload(&mut editor_state.world);
if !reloaded_maps.is_empty() {
    for map_path in &reloaded_maps {
        editor_state.console.info(format!("🔄 Hot-reloaded map: {:?}", map_path));
    }
    editor_state.scene_modified = true;
}

// Display hot-reload error if any
if let Some(error) = editor_state.map_manager.get_last_hot_reload_error() {
    editor_state.console.error(format!("Hot-reload error: {}", error));
    editor_state.map_manager.clear_hot_reload_error();
}
```

**Initialization:**
- Hot-reload watcher is initialized when the editor starts
- Project path is set on the map_manager when a project is opened
- Watcher is enabled by default (can be toggled via settings)

## Requirements Validation

### Requirement 6.1: Detection within 1 second ✅
- Debouncer set to 500ms ensures changes are detected within 1 second
- File system events are processed immediately after debounce period

### Requirement 6.2: Reload and regenerate entities ✅
- `reload_map()` method regenerates all entities from new file data
- Colliders are automatically regenerated during reload

### Requirement 6.3: Preserve layer visibility states ✅
- Visibility states are stored before reload
- States are restored after reload in `reload_map()` method

### Requirement 6.4: Regenerate colliders automatically ✅
- Colliders are regenerated as part of the reload process
- Uses existing `generate_composite_colliders_from_intgrid()` method

### Requirement 6.5: Error recovery and state preservation ✅
- `reload_map_with_error_recovery()` backs up state before reload
- On error, previous state is restored
- Error messages are displayed in console
- Detailed logging for debugging

## Dependencies Added

**engine/Cargo.toml:**
```toml
notify = "6.1"
notify-debouncer-full = "0.3"
```

## Testing

### Manual Testing Steps:

1. **Basic Hot-Reload:**
   - Load an LDtk map in the editor
   - Open the map in LDtk editor
   - Make changes and save
   - Verify map reloads automatically in game engine

2. **Layer State Preservation:**
   - Load a map with multiple layers
   - Hide some layers in the game engine
   - Modify and save the map in LDtk
   - Verify hidden layers remain hidden after reload

3. **Error Recovery:**
   - Load a valid map
   - Corrupt the .ldtk file (invalid JSON)
   - Save the corrupted file
   - Verify error message appears in console
   - Verify map remains in last valid state
   - Fix the file and save
   - Verify map reloads successfully

4. **Multiple Maps:**
   - Load multiple maps
   - Modify one map in LDtk
   - Verify only the modified map reloads
   - Verify other maps remain unchanged

### Property-Based Tests (Optional Tasks):

The following property tests are marked as optional in the task list:
- Property 20: Hot-Reload Regenerates Entities
- Property 21: Hot-Reload Regenerates Colliders
- Property 22: Hot-Reload Error Recovery

These can be implemented later using the QuickCheck framework.

## Performance Considerations

1. **Debouncing:** 500ms delay prevents excessive reloads during rapid saves
2. **Non-blocking:** File watching runs in background thread
3. **Selective Reload:** Only modified files are reloaded
4. **Efficient Polling:** `try_recv()` is non-blocking and fast

## Future Enhancements

1. **Configurable Debounce:** Allow users to adjust debounce delay
2. **Hot-Reload Toggle UI:** Add UI button to enable/disable hot-reload
3. **Reload Notifications:** Visual feedback in scene view when reload occurs
4. **Batch Reloading:** Handle multiple file changes more efficiently
5. **Undo Integration:** Add hot-reload to undo stack for reverting changes

## Known Limitations

1. **Windows File Locking:** Some editors may lock files during save, causing temporary errors
2. **Large Files:** Very large maps may take longer than 1 second to reload
3. **Network Drives:** File watching may not work reliably on network drives
4. **Symlinks:** Symbolic links may not be watched correctly on all platforms

## Conclusion

The hot-reload system is fully implemented and meets all requirements from the specification. It provides a seamless workflow for level designers to iterate on maps without manually reloading in the editor. The error recovery system ensures stability even when files are corrupted or temporarily unavailable.
