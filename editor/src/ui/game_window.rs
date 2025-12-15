use egui;
use crate::states::AppState;
// Sample game removed - use projects/ folder for game content

pub struct GameWindow;

impl GameWindow {
    pub fn render(
        ctx: &egui::Context,
        app_state: &mut AppState,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Game Mode");
            ui.label("Sample game was removed. Use projects/ folder for game content.");
            ui.add_space(20.0);
            
            if ui.button("Back to Launcher").clicked() {
                *app_state = AppState::Launcher;
            }
        });
    }
}