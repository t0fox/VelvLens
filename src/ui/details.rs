use eframe::egui;

use crate::{model::ProxyConfig, qr::QrMatrix, security::redact_uri};

use super::theme;

pub fn show(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    show_sensitive: bool,
    qr: &mut Option<QrMatrix>,
) {
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.horizontal(|ui| {
            protocol_badge(ui, config.protocol.as_str());
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(config.name.as_deref().unwrap_or("Unnamed configuration"))
                        .size(19.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("{}:{}", config.host, config.port))
                        .size(12.0)
                        .color(theme::MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                theme::badge(
                    ui,
                    config.security.as_str(),
                    egui::Color32::from_rgba_unmultiplied(53, 208, 127, 35),
                    theme::SUCCESS,
                );
            });
        });
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            theme::badge(
                ui,
                config.transport.as_str(),
                theme::SURFACE_RAISED,
                theme::MUTED,
            );
            if let Some(flow) = &config.flow {
                theme::badge(ui, flow, theme::SURFACE_RAISED, theme::MUTED);
            }
        });
        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);
        let public_key = compact_value(config.reality_public_key.as_deref().unwrap_or("—"), 28);
        let short_id = compact_value(config.reality_short_id.as_deref().unwrap_or("—"), 18);
        let uuid = if show_sensitive {
            compact_value(config.uuid.as_deref().unwrap_or("—"), 24)
        } else {
            "••••••".to_owned()
        };
        let password = if show_sensitive {
            compact_value(config.password.as_deref().unwrap_or("—"), 24)
        } else {
            "••••••".to_owned()
        };
        egui::Grid::new(ui.id().with("configuration-fields"))
            .num_columns(4)
            .spacing(egui::vec2(20.0, 14.0))
            .show(ui, |ui| {
                detail_pair(
                    ui,
                    "Address",
                    &config.host,
                    "Transport",
                    config.transport.as_str(),
                );
                ui.end_row();
                detail_pair(
                    ui,
                    "Port",
                    &config.port.to_string(),
                    "SNI",
                    config.sni.as_deref().unwrap_or("—"),
                );
                ui.end_row();
                detail_pair(
                    ui,
                    "Security",
                    config.security.as_str(),
                    "Fingerprint",
                    config.fingerprint.as_deref().unwrap_or("—"),
                );
                ui.end_row();
                detail_pair(ui, "Public key", &public_key, "Short ID", &short_id);
                ui.end_row();
                detail_pair(ui, "UUID", &uuid, "Password", &password);
                ui.end_row();
            });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Credentials are masked by default")
                    .size(11.0)
                    .color(theme::MUTED),
            );
        });
        ui.add_space(8.0);
        egui::Frame::none()
            .fill(theme::CANVAS)
            .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Configuration URI")
                        .size(11.0)
                        .color(theme::MUTED),
                );
                let mut sanitized_uri = redact_uri(&config.raw_uri);
                ui.add(
                    egui::TextEdit::singleline(&mut sanitized_uri)
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [190.0, 36.0],
                    egui::Button::new(egui::RichText::new("Copy configuration").strong())
                        .fill(theme::ACCENT),
                )
                .clicked()
            {
                copy_to_clipboard(&config.raw_uri);
            }
            if ui
                .add_sized(
                    [182.0, 36.0],
                    egui::Button::new("Copy sanitized").fill(theme::SURFACE_RAISED),
                )
                .clicked()
            {
                copy_to_clipboard(&redact_uri(&config.raw_uri));
            }
            if ui
                .add_sized(
                    [108.0, 36.0],
                    egui::Button::new("QR code").fill(theme::SURFACE_RAISED),
                )
                .clicked()
            {
                *qr = Some(crate::qr::encode(&config.raw_uri));
            }
        });
    });
}

fn protocol_badge(ui: &mut egui::Ui, protocol: &str) {
    theme::badge(ui, protocol, theme::ACCENT, egui::Color32::WHITE);
}

fn detail_pair(ui: &mut egui::Ui, label: &str, value: &str, next_label: &str, next_value: &str) {
    ui.label(egui::RichText::new(label).size(11.0).color(theme::MUTED));
    ui.label(egui::RichText::new(value).size(12.0).strong());
    ui.label(
        egui::RichText::new(next_label)
            .size(11.0)
            .color(theme::MUTED),
    );
    ui.label(egui::RichText::new(next_value).size(12.0).strong());
}

fn compact_value(value: &str, max_chars: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max_chars {
        return value.to_owned();
    }
    let prefix: String = chars.iter().take(max_chars.saturating_sub(7)).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{prefix}…{suffix}")
}

pub fn copy_to_clipboard(value: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(value.to_owned());
    }
}
