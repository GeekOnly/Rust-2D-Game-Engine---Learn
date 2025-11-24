use egui::Context;

pub struct EditorModule {
    pub visible: bool,
}

impl EditorModule {
    pub fn new() -> Self { Self { visible: true } }
    
    pub fn render_ui(&mut self, ctx: &Context) {
        if !self.visible { return; }
        
        egui::Window::new("Engine Editor").show(ctx, |ui| {
            ui.label("Hello from Editor!");
            if ui.button("Toggle Visibility").clicked() {
                // logic
            }
        });
    }
}
