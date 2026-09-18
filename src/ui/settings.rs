use eframe::egui;

use crate::settings::AppSettings;

use super::theme;

pub fn show(ui: &mut egui::Ui, settings: &mut AppSettings) -> bool {
    let mut close = false;
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.label(egui::RichText::new("Settings").size(18.0).strong());
        ui.label(
            egui::RichText::new("Control privacy and resolver limits for this session.")
                .size(12.0)
                .color(theme::MUTED),
        );
        ui.add_space(14.0);
        ui.checkbox(
            &mut settings.history_enabled,
            "Enable protected URL history",
        );
        ui.checkbox(
            &mut settings.show_sensitive_session,
            "Show sensitive fields for this session",
        );
        ui.add_space(10.0);
        egui::Frame::none()
            .fill(theme::CANVAS)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut settings.resolver.max_depth, 1..=16)
                        .text("Max recursion depth"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.resolver.max_discovered_urls, 8..=512)
                        .text("Max discovered URLs"),
                );
                ui.add(
                    egui::Slider::new(&mut settings.timeout_seconds, 2..=60)
                        .text("Request timeout (seconds)"),
                );
            });
        ui.add_space(14.0);
        if ui
            .add_sized(
                [110.0, 34.0],
                egui::Button::new("Close").fill(theme::SURFACE_RAISED),
            )
            .clicked()
        {
            close = true;
        }
    });
    close
}
