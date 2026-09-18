use eframe::egui;

use crate::settings::AppSettings;

pub fn show(ui: &mut egui::Ui, settings: &mut AppSettings) -> bool {
    let mut close = false;
    ui.heading("Settings");
    ui.checkbox(
        &mut settings.history_enabled,
        "Enable protected URL history",
    );
    ui.checkbox(
        &mut settings.show_sensitive_session,
        "Show sensitive fields for this session",
    );
    ui.add(egui::Slider::new(&mut settings.resolver.max_depth, 1..=16).text("Max recursion depth"));
    ui.add(
        egui::Slider::new(&mut settings.resolver.max_discovered_urls, 8..=512)
            .text("Max discovered URLs"),
    );
    ui.add(
        egui::Slider::new(&mut settings.timeout_seconds, 2..=60).text("Request timeout (seconds)"),
    );
    if ui.button("Close").clicked() {
        close = true;
    }
    close
}
