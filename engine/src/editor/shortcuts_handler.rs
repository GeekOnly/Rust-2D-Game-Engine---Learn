//! Keyboard Shortcuts Handler
//!
//! Centralized keyboard shortcut handling for all Priority 1 features:
//! - Undo/Redo (Ctrl+Z, Ctrl+Y)
//! - Selection (Ctrl+A, Escape, Delete)
//! - Clipboard (Ctrl+C/V/D/X)
//! - Snapping (Ctrl+G)

use crate::editor::{
    EditorState, SelectionMode,
    copy_selected, paste_from_clipboard, duplicate_selected,
    CreateEntityCommand, DeleteEntityCommand, BatchCommand,
};
use std::collections::HashMap;

/// Handle all editor keyboard shortcuts
pub fn handle_editor_shortcuts(
    ctx: &egui::Context,
    state: &mut EditorState,
) {
    ctx.input(|i| {
        // Skip if typing in text field
        if i.focused {
            return;
        }
        
        // ====================================================================
        // UNDO/REDO
        // ====================================================================
        
        // Ctrl+Z: Undo
        if i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Z) {
            if state.undo_stack.undo(&mut state.world, &mut state.entity_names) {
                if let Some(desc) = state.undo_stack.undo_description() {
                    state.console.info(format!("Undo: {}", desc));
                } else {
                    state.console.info("Undo");
                }
                state.scene_modified = true;
            }
        }
        
        // Ctrl+Shift+Z or Ctrl+Y: Redo
        if (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z)) ||
           (i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            if state.undo_stack.redo(&mut state.world, &mut state.entity_names) {
                if let Some(desc) = state.undo_stack.redo_description() {
                    state.console.info(format!("Redo: {}", desc));
                } else {
                    state.console.info("Redo");
                }
                state.scene_modified = true;
            }
        }
        
        // ====================================================================
        // SELECTION
        // ====================================================================
        
        // Ctrl+A: Select All
        if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
            let all_entities: Vec<_> = state.world.transforms.keys().copied().collect();
            if !all_entities.is_empty() {
                state.selection.select_all(&all_entities);
                state.console.info(format!("Selected {} entities", all_entities.len()));
            }
        }
        
        // Escape: Clear Selection
        if i.key_pressed(egui::Key::Escape) {
            if state.selection.has_selection() {
                let count = state.selection.count();
                state.selection.clear();
                state.console.info(format!("Deselected {} entities", count));
            }
        }
        
        // Delete: Delete Selected
        if i.key_pressed(egui::Key::Delete) {
            let selected = state.selection.get_selected();
            if !selected.is_empty() {
                let count = selected.len();
                let mut batch = BatchCommand::new("Delete");
                
                for &entity in &selected {
                    batch.add(Box::new(DeleteEntityCommand::new(
                        entity,
                        &state.world,
                        &state.entity_names,
                    )));
                }
                
                state.undo_stack.execute(
                    Box::new(batch),
                    &mut state.world,
                    &mut state.entity_names,
                );
                
                state.selection.clear();
                state.console.info(format!("Deleted {} entities", count));
                state.scene_modified = true;
            }
        }
        
        // ====================================================================
        // CLIPBOARD
        // ====================================================================
        
        let selected = state.selection.get_selected();
        
        // Ctrl+C: Copy
        if i.modifiers.ctrl && i.key_pressed(egui::Key::C) {
            if !selected.is_empty() {
                copy_selected(
                    &mut state.clipboard,
                    &selected,
                    &state.world,
                    &state.entity_names,
                );
                state.console.info(format!("Copied {} entities", selected.len()));
            }
        }
        
        // Ctrl+V: Paste
        if i.modifiers.ctrl && i.key_pressed(egui::Key::V) {
            if state.clipboard.has_data() {
                let new_entities = paste_from_clipboard(
                    &state.clipboard,
                    &mut state.world,
                    &mut state.entity_names,
                    Some([10.0, 10.0, 0.0]),
                );
                
                if !new_entities.is_empty() {
                    let count = new_entities.len();
                    let mut batch = BatchCommand::new("Paste");
                    
                    for &entity in &new_entities {
                        batch.add(Box::new(CreateEntityCommand::new(
                            entity,
                            &state.world,
                            &state.entity_names,
                        )));
                    }
                    
                    state.undo_stack.execute(
                        Box::new(batch),
                        &mut state.world,
                        &mut state.entity_names,
                    );
                    
                    state.selection.select_multiple(&new_entities, SelectionMode::Replace);
                    state.console.info(format!("Pasted {} entities", count));
                    state.scene_modified = true;
                }
            }
        }
        
        // Ctrl+D: Duplicate
        if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
            if !selected.is_empty() {
                let new_entities = duplicate_selected(
                    &state.clipboard,
                    &selected,
                    &mut state.world,
                    &mut state.entity_names,
                );
                
                if !new_entities.is_empty() {
                    let count = new_entities.len();
                    let mut batch = BatchCommand::new("Duplicate");
                    
                    for &entity in &new_entities {
                        batch.add(Box::new(CreateEntityCommand::new(
                            entity,
                            &state.world,
                            &state.entity_names,
                        )));
                    }
                    
                    state.undo_stack.execute(
                        Box::new(batch),
                        &mut state.world,
                        &mut state.entity_names,
                    );
                    
                    state.selection.select_multiple(&new_entities, SelectionMode::Replace);
                    state.console.info(format!("Duplicated {} entities", count));
                    state.scene_modified = true;
                }
            }
        }
        
        // Ctrl+X: Cut
        if i.modifiers.ctrl && i.key_pressed(egui::Key::X) {
            if !selected.is_empty() {
                let count = selected.len();
                
                // Copy first
                copy_selected(
                    &mut state.clipboard,
                    &selected,
                    &state.world,
                    &state.entity_names,
                );
                
                // Then delete
                let mut batch = BatchCommand::new("Cut");
                for &entity in &selected {
                    batch.add(Box::new(DeleteEntityCommand::new(
                        entity,
                        &state.world,
                        &state.entity_names,
                    )));
                }
                
                state.undo_stack.execute(
                    Box::new(batch),
                    &mut state.world,
                    &mut state.entity_names,
                );
                
                state.selection.clear();
                state.console.info(format!("Cut {} entities", count));
                state.scene_modified = true;
            }
        }
        
        // ====================================================================
        // SNAPPING
        // ====================================================================
        
        // Ctrl+G: Toggle Snapping
        if i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::G) {
            state.snap_settings.enabled = !state.snap_settings.enabled;
            state.console.info(format!(
                "Snapping: {}",
                if state.snap_settings.enabled { "ON" } else { "OFF" }
            ));
            
            // Save settings
            let _ = state.snap_settings.save();
        }
        
        // Ctrl+Shift+G: Toggle Grid
        if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::G) {
            state.snap_settings.show_grid = !state.snap_settings.show_grid;
            state.console.info(format!(
                "Grid: {}",
                if state.snap_settings.show_grid { "ON" } else { "OFF" }
            ));
            
            // Save settings
            let _ = state.snap_settings.save();
        }
    });
}

/// Get keyboard shortcut hints for UI
pub fn get_shortcut_hints() -> HashMap<&'static str, &'static str> {
    let mut hints = HashMap::new();
    
    // Undo/Redo
    hints.insert("Undo", "Ctrl+Z");
    hints.insert("Redo", "Ctrl+Y or Ctrl+Shift+Z");
    
    // Selection
    hints.insert("Select All", "Ctrl+A");
    hints.insert("Deselect", "Escape");
    hints.insert("Delete", "Delete");
    
    // Clipboard
    hints.insert("Copy", "Ctrl+C");
    hints.insert("Paste", "Ctrl+V");
    hints.insert("Duplicate", "Ctrl+D");
    hints.insert("Cut", "Ctrl+X");
    
    // Snapping
    hints.insert("Toggle Snap", "Ctrl+G");
    hints.insert("Toggle Grid", "Ctrl+Shift+G");
    
    hints
}

/// Render keyboard shortcuts help panel
pub fn render_shortcuts_help(ui: &mut egui::Ui) {
    ui.heading("Keyboard Shortcuts");
    
    ui.separator();
    
    ui.label("Undo/Redo:");
    ui.horizontal(|ui| {
        ui.label("  Ctrl+Z");
        ui.label("Undo");
    });
    ui.horizontal(|ui| {
        ui.label("  Ctrl+Y");
        ui.label("Redo");
    });
    
    ui.separator();
    
    ui.label("Selection:");
    ui.horizontal(|ui| {
        ui.label("  Ctrl+A");
        ui.label("Select All");
    });
    ui.horizontal(|ui| {
        ui.label("  Escape");
        ui.label("Clear Selection");
    });
    ui.horizontal(|ui| {
        ui.label("  Delete");
        ui.label("Delete Selected");
    });
    
    ui.separator();
    
    ui.label("Clipboard:");
    ui.horizontal(|ui| {
        ui.label("  Ctrl+C");
        ui.label("Copy");
    });
    ui.horizontal(|ui| {
        ui.label("  Ctrl+V");
        ui.label("Paste");
    });
    ui.horizontal(|ui| {
        ui.label("  Ctrl+D");
        ui.label("Duplicate");
    });
    ui.horizontal(|ui| {
        ui.label("  Ctrl+X");
        ui.label("Cut");
    });
    
    ui.separator();
    
    ui.label("Snapping:");
    ui.horizontal(|ui| {
        ui.label("  Ctrl+G");
        ui.label("Toggle Snap");
    });
    ui.horizontal(|ui| {
        ui.label("  Ctrl+Shift+G");
        ui.label("Toggle Grid");
    });
    ui.horizontal(|ui| {
        ui.label("  Hold Shift");
        ui.label("Disable Snap (temp)");
    });
    ui.horizontal(|ui| {
        ui.label("  Hold Ctrl");
        ui.label("Enable Snap (temp)");
    });
}
