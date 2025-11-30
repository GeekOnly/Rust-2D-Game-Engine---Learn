use std::collections::VecDeque;
use chrono::Local;

/// Log level for console messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::from_rgb(200, 200, 200),
            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 0),
            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
            LogLevel::Debug => egui::Color32::from_rgb(150, 200, 255),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Debug => "🔍",
        }
    }
}

/// Console log message
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
    pub count: usize,
}

impl LogMessage {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            level,
            message,
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            count: 1,
        }
    }
}

/// Console window for displaying logs
pub struct Console {
    messages: VecDeque<LogMessage>,
    max_messages: usize,
    show_info: bool,
    show_warning: bool,
    show_error: bool,
    show_debug: bool,
    collapse: bool,
    auto_scroll: bool,
    filter: String,
}

impl Console {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 1000,
            show_info: true,
            show_warning: true,
            show_error: true,
            show_debug: true,  // Enable debug logs by default for physics debugging
            collapse: false,
            auto_scroll: true,
            filter: String::new(),
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let message = message.into();

        // Check if we should collapse duplicate messages
        if self.collapse {
            if let Some(last) = self.messages.back_mut() {
                if last.level == level && last.message == message {
                    last.count += 1;
                    return;
                }
            }
        }

        self.messages.push_back(LogMessage::new(level, message));

        // Limit message count
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    #[allow(dead_code)]
    pub fn warning(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }

    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("🗑 Clear").clicked() {
                self.clear();
            }

            ui.separator();

            // Filter toggles
            let info_count = self.messages.iter().filter(|m| m.level == LogLevel::Info).count();
            let warning_count = self.messages.iter().filter(|m| m.level == LogLevel::Warning).count();
            let error_count = self.messages.iter().filter(|m| m.level == LogLevel::Error).count();
            let debug_count = self.messages.iter().filter(|m| m.level == LogLevel::Debug).count();

            ui.toggle_value(&mut self.show_info, format!("ℹ️ Info ({})", info_count));
            ui.toggle_value(&mut self.show_warning, format!("⚠️ Warning ({})", warning_count));
            ui.toggle_value(&mut self.show_error, format!("❌ Error ({})", error_count));
            ui.toggle_value(&mut self.show_debug, format!("🔍 Debug ({})", debug_count));

            ui.separator();

            ui.checkbox(&mut self.collapse, "Collapse");
            ui.checkbox(&mut self.auto_scroll, "Auto Scroll");

            ui.separator();

            // Search filter
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.filter);
        });

        ui.separator();

        // Message list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                for msg in &self.messages {
                    // Filter by level
                    let should_show = match msg.level {
                        LogLevel::Info => self.show_info,
                        LogLevel::Warning => self.show_warning,
                        LogLevel::Error => self.show_error,
                        LogLevel::Debug => self.show_debug,
                    };

                    if !should_show {
                        continue;
                    }

                    // Filter by search text
                    if !self.filter.is_empty() {
                        if !msg.message.to_lowercase().contains(&self.filter.to_lowercase()) {
                            continue;
                        }
                    }

                    // Render message
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(msg.level.icon()).color(msg.level.color()));
                        ui.label(egui::RichText::new(&msg.timestamp).color(egui::Color32::GRAY));

                        let text = if msg.count > 1 {
                            format!("{} ({})", msg.message, msg.count)
                        } else {
                            msg.message.clone()
                        };

                        let response = ui.label(egui::RichText::new(&text).color(msg.level.color()))
                            .on_hover_text("Click to copy");

                        // Copy to clipboard on click
                        if response.clicked() {
                            ui.output_mut(|o| o.copied_text = msg.message.clone());
                        }

                        // Context menu for copy
                        response.context_menu(|ui| {
                            if ui.button("📋 Copy").clicked() {
                                ui.output_mut(|o| o.copied_text = msg.message.clone());
                                ui.close_menu();
                            }
                            if ui.button("📋 Copy with timestamp").clicked() {
                                ui.output_mut(|o| o.copied_text = format!("{} {}", msg.timestamp, msg.message));
                                ui.close_menu();
                            }
                        });
                    });
                }

                // Show empty message
                if self.messages.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("No messages").color(egui::Color32::GRAY));
                    });
                }
            });
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
