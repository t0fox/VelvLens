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
        ui.label(egui::RichText::new("Настройки").size(18.0).strong());
        ui.label(
            egui::RichText::new("История ссылок и лимиты локального анализа.")
                .size(12.0)
                .color(theme::TEXT_SECONDARY),
        );
        ui.add_space(14.0);
        let history = ui.checkbox(&mut settings.history_enabled, "Сохранять историю ссылок");
        result.changed |= history.changed();
        result.history_toggled |= history.changed();
        ui.label(
            egui::RichText::new(if settings.history_enabled {
                "Хранится локально с защитой Windows."
            } else {
                "История отключена, ссылки не сохраняются."
            })
            .size(11.0)
            .color(theme::TEXT_MUTED),
        );
        ui.add_space(6.0);
        let clear = ui.add_enabled(
            settings.history_enabled,
            egui::Button::new("Очистить историю").fill(theme::SURFACE_RAISED),
        );
        result.clear_history = clear.clicked();
        ui.add_space(10.0);
        egui::Frame::none()
            .fill(theme::CANVAS)
            .rounding(egui::Rounding::same(theme::RADIUS_INPUT))
            .inner_margin(egui::Margin::same(theme::SPACE_12))
            .show(ui, |ui| {
                result.changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.resolver.max_depth, 1..=16)
                            .text("Глубина рекурсии"),
                    )
                    .changed();
                result.changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.resolver.max_discovered_urls, 8..=512)
                            .text("Максимум найденных ссылок"),
                    )
                    .changed();
                result.changed |= ui
                    .add(
                        egui::Slider::new(&mut settings.timeout_seconds, 2..=60)
                            .text("Таймаут запроса (секунды)"),
                    )
                    .changed();
            });
        ui.add_space(14.0);
        if ui
            .add_sized(
                [110.0, 34.0],
                egui::Button::new("Закрыть").fill(theme::SURFACE_RAISED),
            )
            .clicked()
        {
            result.close = true;
        }
    });
    result
}
