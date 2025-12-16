pub mod exit_dialog;
pub mod layout_dialog;
// prefab dialog logic was deeply coupled with prefab_manager creation loop, 
// leaving it in editor_logic for now or standardizing it later is fine, 
// but let's try to extract it if possible? 
// Actually the existing code used editor_state.create_prefab_dialog.render() which is already somewhat modular, 
// but the handling of the RESULT was in editor_logic. Let's make a wrapper here too.

pub use exit_dialog::ExitDialog;
pub use layout_dialog::LayoutDialog;
