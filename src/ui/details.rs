use std::time::{Duration, Instant};

use eframe::egui;

use crate::{
    model::{ProxyConfig, Security, Transport},
    qr::QrMatrix,
};

use super::theme;

pub fn show(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    qr: &mut Option<QrMatrix>,
    diagnostic: Option<&crate::diagnostics::DiagnosticResult>,
    copied_until: &mut Option<Instant>,
) -> bool {
    let json_payload = is_json_payload(config);
    if copied_until.is_some_and(|until| until <= Instant::now()) {
        *copied_until = None;
    }
    let mut test = false;
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.horizontal(|ui| {
            protocol_badge(ui, config.protocol.as_str());
            ui.add_space(2.0);
            ui.vertical(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(
                            config.name.as_deref().unwrap_or("Unnamed configuration"),
                        )
                        .size(19.0)
                        .strong(),
                    )
                    .truncate(),
                );
                ui.label(
                    egui::RichText::new(format!("{}:{}", config.host, display_port(config)))
                        .size(12.0)
                        .color(theme::MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if config.security != Security::Unknown {
                    theme::badge(
                        ui,
                        config.security.as_str(),
                        egui::Color32::from_rgba_unmultiplied(53, 208, 127, 35),
                        theme::SUCCESS,
                    );
                }
            });
        });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if config.transport != Transport::Unknown {
                theme::badge(
                    ui,
                    config.transport.as_str(),
                    theme::SURFACE_RAISED,
                    theme::MUTED,
                );
            }
            if let Some(flow) = &config.flow {
                theme::badge(ui, flow, theme::SURFACE_RAISED, theme::MUTED);
            }
        });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            let copied = copied_until.is_some();
            if ui
                .add_sized(
                    [190.0, 36.0],
                    egui::Button::new(if copied {
                        egui::RichText::new("✓ Copied").strong()
                    } else if json_payload {
                        egui::RichText::new("Copy JSON config").strong()
                    } else {
                        egui::RichText::new("Copy configuration").strong()
                    })
                    .fill(theme::ACCENT),
                )
                .clicked()
            {
                copy_config(config, copied_until);
            }
            if ui
                .add_sized(
                    [68.0, 36.0],
                    egui::Button::new("Copy").fill(theme::SURFACE_RAISED),
                )
                .clicked()
            {
                copy_config(config, copied_until);
            }
            let qr_response = ui.add_enabled(
                !json_payload,
                egui::Button::new(if json_payload { "QR unavailable" } else { "QR" })
                    .fill(theme::SURFACE_RAISED),
            );
            if qr_response.clicked() {
                *qr = Some(crate::qr::encode(&config.raw_uri));
            }
            if ui
                .add_sized(
                    [88.0, 36.0],
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
            egui::RichText::new("CONFIGURATION DETAILS")
                .size(10.0)
                .strong()
                .color(theme::MUTED),
        );
        ui.add_space(8.0);
        show_fields(ui, config);
        ui.add_space(12.0);
        egui::Frame::none()
            .fill(theme::CANVAS)
            .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(if json_payload {
                            "Configuration JSON"
                        } else {
                            "Configuration URI"
                        })
                        .size(11.0)
                        .strong()
                        .color(theme::MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [58.0, 24.0],
                                egui::Button::new("Copy").fill(theme::SURFACE_RAISED),
                            )
                            .clicked()
                        {
                            copy_config(config, copied_until);
                        }
                    });
                });
                let mut raw_uri = config.raw_uri.clone();
                ui.add(
                    egui::TextEdit::singleline(&mut raw_uri)
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

fn show_fields(ui: &mut egui::Ui, config: &ProxyConfig) {
    let mut fields = vec![
        ("Protocol", config.protocol.as_str().to_owned()),
        ("Address", config.host.clone()),
        ("Port", display_port(config)),
    ];
    if config.security != Security::Unknown {
        fields.push(("Security", config.security.as_str().to_owned()));
    }
    if config.transport != Transport::Unknown {
        fields.push(("Transport", config.transport.as_str().to_owned()));
    }
    add_optional(&mut fields, "SNI", config.sni.as_deref());
    add_optional(&mut fields, "Fingerprint", config.fingerprint.as_deref());
    add_optional(
        &mut fields,
        "Public key",
        config.reality_public_key.as_deref(),
    );
    add_optional(&mut fields, "Short ID", config.reality_short_id.as_deref());
    add_optional(&mut fields, "UUID", config.uuid.as_deref());
    add_optional(&mut fields, "Username", config.username.as_deref());
    add_optional(&mut fields, "Password", config.password.as_deref());
    add_optional(&mut fields, "Flow", config.flow.as_deref());
    add_optional(&mut fields, "Encryption", config.encryption.as_deref());
    add_optional(&mut fields, "Path", config.path.as_deref());
    add_optional(&mut fields, "Host", config.host_header.as_deref());
    add_optional(&mut fields, "Service name", config.service_name.as_deref());
    add_optional(&mut fields, "Mode", config.mode.as_deref());
    if !config.unknown_params.is_empty() {
        let extra = config
            .unknown_params
            .iter()
            .flat_map(|(key, values)| values.iter().map(move |value| format!("{key}={value}")))
            .collect::<Vec<_>>()
            .join(" · ");
        fields.push(("Additional parameters", extra));
    }

    egui::Grid::new(ui.id().with("configuration-fields"))
        .num_columns(4)
        .spacing(egui::vec2(18.0, 10.0))
        .show(ui, |ui| {
            for pair in fields.chunks(2) {
                for (label, value) in pair {
                    ui.label(egui::RichText::new(*label).size(11.0).color(theme::MUTED));
                    ui.label(
                        egui::RichText::new(compact_value(value, 42))
                            .size(12.0)
                            .strong(),
                    );
                }
                if pair.len() == 1 {
                    ui.label("");
                    ui.label("");
                }
                ui.end_row();
            }
        });
}

fn add_optional(
    fields: &mut Vec<(&'static str, String)>,
    label: &'static str,
    value: Option<&str>,
) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        fields.push((label, value.to_owned()));
    }
}

fn display_port(config: &ProxyConfig) -> String {
    config
        .port_range
        .clone()
        .unwrap_or_else(|| config.port.to_string())
}

fn copy_config(config: &ProxyConfig, copied_until: &mut Option<Instant>) {
    copy_to_clipboard(&config.raw_uri);
    *copied_until = Some(Instant::now() + Duration::from_secs(2));
}

fn is_json_payload(config: &ProxyConfig) -> bool {
    matches!(
        config.raw_uri.trim_start().chars().next(),
        Some('{') | Some('[')
    )
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
