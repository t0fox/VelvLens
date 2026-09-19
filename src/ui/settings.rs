use eframe::egui;

use crate::settings::AppSettings;

use super::theme;

#[derive(Debug, Clone, Copy, Default)]
pub struct SettingsViewResult {
    pub close: bool,
    pub changed: bool,
    pub history_toggled: bool,
    pub clear_history: bool,
}

pub fn show(ui: &mut egui::Ui, settings: &mut AppSettings) -> SettingsViewResult {
    let mut result = SettingsViewResult::default();
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.label(egui::RichText::new("Settings").size(18.0).strong());
        ui.label(
            egui::RichText::new("Control history and resolver limits for this session.")
                .size(12.0)
                .color(theme::MUTED),
        );
        ui.add_space(14.0);
        let history = ui.checkbox(
            &mut settings.history_enabled,
            "Enable protected URL history",
        );
        result.changed |= history.changed();
        result.history_toggled |= history.changed();
        ui.label(
            egui::RichText::new(if settings.history_enabled {
                "Stored locally using Windows protection."
            } else {
                "History is disabled and no URLs are stored."
            })
            .size(11.0)
            .color(theme::MUTED),
        );
        ui.add_space(6.0);
        let clear = ui.add_enabled(
            settings.history_enabled,
            egui::Button::new("Clear history").fill(theme::SURFACE_RAISED),
        );
        result.clear_history = clear.clicked();
        ui.add_space(10.0);
        egui::Frame::none()
            .fill(theme::CANVAS)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                result.changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.resolver.max_depth, 1..=16)
                            .text("Max recursion depth"),
                    )
                    .changed();
                result.changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.resolver.max_discovered_urls, 8..=512)
                            .text("Max discovered URLs"),
                    )
                    .changed();
                result.changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.timeout_seconds, 2..=60)
                            .text("Request timeout (seconds)"),
                    )
                    .changed();
            });
        ui.add_space(14.0);
        if ui
            .add_sized(
                [110.0, 34.0],
                egui::Button::new("Close").fill(theme::SURFACE_RAISED),
            )
            .clicked()
        {
            result.close = true;
        }
    });
    result
}
