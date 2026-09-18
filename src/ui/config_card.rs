use eframe::egui;

use crate::model::ProxyConfig;

pub fn show(ui: &mut egui::Ui, config: &ProxyConfig, selected: bool) -> egui::Response {
    let frame = egui::Frame::group(ui.style())
        .fill(if selected {
            egui::Color32::from_rgb(35, 48, 65)
        } else {
            egui::Color32::from_rgb(25, 31, 40)
        })
        .stroke(egui::Stroke::new(
            1.0_f32,
            if selected {
                egui::Color32::from_rgb(72, 154, 255)
            } else {
                egui::Color32::from_rgb(50, 58, 70)
            },
        ))
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::same(14.0));
    frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(config.protocol.as_str())
                        .strong()
                        .color(egui::Color32::from_rgb(110, 190, 255)),
                );
                if config.metadata.exact_duplicate_count > 1 {
                    ui.label(
                        egui::RichText::new(format!(
                            "Duplicate ×{}",
                            config.metadata.exact_duplicate_count
                        ))
                        .small()
                        .color(egui::Color32::YELLOW),
                    );
                }
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(config.name.as_deref().unwrap_or("Unnamed configuration"))
                    .strong(),
            );
            ui.label(format!("{}:{}", config.host, config.port));
            ui.horizontal_wrapped(|ui| {
                badge(ui, config.transport.as_str());
                badge(ui, config.security.as_str());
                if let Some(sni) = &config.sni {
                    badge(ui, &format!("SNI {sni}"));
                }
            });
        })
        .response
}

fn badge(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .small()
            .color(egui::Color32::from_rgb(170, 184, 204)),
    );
}
