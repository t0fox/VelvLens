use eframe::egui;

use crate::{model::ProxyConfig, qr::QrMatrix, security::redact_uri};

use super::theme;

pub fn show(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    show_sensitive: bool,
    qr: &mut Option<QrMatrix>,
    diagnostic: Option<&crate::diagnostics::DiagnosticResult>,
) -> bool {
    let mut test = false;
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
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [132.0, 34.0],
                    egui::Button::new(egui::RichText::new("Copy URI").strong()).fill(theme::ACCENT),
                )
                .clicked()
            {
                copy_to_clipboard(&config.raw_uri);
            }
            if ui
                .add_sized(
                    [122.0, 34.0],
                    egui::Button::new("Copy safe").fill(theme::SURFACE_RAISED),
                )
                .clicked()
            {
                copy_to_clipboard(&redact_uri(&config.raw_uri));
            }
            if ui
                .add_sized(
                    [82.0, 34.0],
                    egui::Button::new("QR").fill(theme::SURFACE_RAISED),
                )
                .clicked()
            {
                *qr = Some(crate::qr::encode(&config.raw_uri));
            }
            if ui
                .add_sized(
                    [88.0, 34.0],
                    egui::Button::new("Test").fill(theme::SURFACE_RAISED),
                )
                .clicked()
            {
                test = true;
            }
        });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("CONNECTION DETAILS")
                .size(10.0)
                .strong()
                .color(theme::MUTED),
        );
        ui.add_space(8.0);
        let address = compact_value(&config.host, 32);
        let sni = compact_value(config.sni.as_deref().unwrap_or("—"), 24);
        let fingerprint = compact_value(config.fingerprint.as_deref().unwrap_or("—"), 20);
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
            .spacing(egui::vec2(18.0, 10.0))
            .show(ui, |ui| {
                detail_pair(
                    ui,
                    "Address",
                    &address,
                    "Transport",
                    config.transport.as_str(),
                );
                ui.end_row();
                detail_pair(ui, "Port", &config.port.to_string(), "SNI", &sni);
                ui.end_row();
                detail_pair(
                    ui,
                    "Security",
                    config.security.as_str(),
                    "Fingerprint",
                    &fingerprint,
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
        if let Some(result) = diagnostic {
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("CONNECTIVITY CHECK")
                    .size(10.0)
                    .strong()
                    .color(theme::MUTED),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                status_badge(ui, "DNS", &result.dns);
                status_badge(ui, "TCP", &result.tcp);
                if let Some(latency) = result.latency_ms {
                    theme::badge(
                        ui,
                        &format!("{latency} ms"),
                        theme::SURFACE_RAISED,
                        theme::TEXT,
                    );
                }
            });
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new("Connectivity only — this does not test the proxy protocol.")
                    .size(11.0)
                    .color(theme::MUTED),
            );
        }
    });
    test
}

fn status_badge(ui: &mut egui::Ui, label: &str, status: &crate::diagnostics::CheckStatus) {
    let (text, fill, color) = match status {
        crate::diagnostics::CheckStatus::Passed => (
            format!("{label}  PASS"),
            theme::SUCCESS,
            egui::Color32::WHITE,
        ),
        crate::diagnostics::CheckStatus::Failed(error) => (
            format!("{label}  FAIL: {}", compact_value(error, 28)),
            egui::Color32::from_rgba_unmultiplied(241, 107, 107, 35),
            theme::ERROR,
        ),
        crate::diagnostics::CheckStatus::Skipped => (
            format!("{label}  SKIPPED"),
            theme::SURFACE_RAISED,
            theme::MUTED,
        ),
    };
    theme::badge(ui, &text, fill, color);
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
