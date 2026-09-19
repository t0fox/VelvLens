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
    compact: bool,
) -> bool {
    let json_payload = config.original_is_json();
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
                        egui::RichText::new(config.name.as_deref().unwrap_or("Без названия"))
                            .size(19.0)
                            .strong(),
                    )
                    .truncate(),
                );
                ui.label(
                    egui::RichText::new(format!("{}:{}", config.host, display_port(config)))
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
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
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if config.transport != Transport::Unknown {
                theme::badge(
                    ui,
                    config.transport.as_str(),
                    theme::SURFACE_RAISED,
                    theme::TEXT_SECONDARY,
                );
            }
            if let Some(flow) = &config.flow {
                theme::badge(ui, flow, theme::SURFACE_RAISED, theme::TEXT_SECONDARY);
            }
            if !compact {
                show_share_copy_actions(ui, config, copied_until);
                let qr_response = ui.add_enabled(
                    config.share_uri.uri().is_some(),
                    egui::Button::new(match &config.share_uri {
                        crate::model::ShareUriResult::Available { .. } => "QR",
                        crate::model::ShareUriResult::Limited { .. } => "QR с ограничениями",
                        crate::model::ShareUriResult::Unavailable { .. } => "QR недоступен",
                    })
                    .fill(theme::SURFACE_RAISED),
                );
                if qr_response.clicked() {
                    if let Some(uri) = config.share_uri.uri() {
                        *qr = Some(crate::qr::encode(uri));
                    }
                }
                ui.menu_button("Ещё", |ui| {
                    if ui.button("Проверить соединение").clicked() {
                        test = true;
                        ui.close_menu();
                    }
                });
            } else {
                ui.menu_button("Действия", |ui| {
                    if ui
                        .add_enabled(
                            config.share_uri.uri().is_some(),
                            egui::Button::new(match &config.share_uri {
                                crate::model::ShareUriResult::Available { .. } => "QR-код",
                                crate::model::ShareUriResult::Limited { .. } => {
                                    "QR с ограничениями"
                                }
                                crate::model::ShareUriResult::Unavailable { .. } => "QR недоступен",
                            }),
                        )
                        .clicked()
                    {
                        if let Some(uri) = config.share_uri.uri() {
                            *qr = Some(crate::qr::encode(uri));
                        }
                        ui.close_menu();
                    }
                    if config.original_is_json()
                        && config.share_uri.is_unavailable()
                        && ui.button("Скопировать исходный JSON").clicked()
                    {
                        copy_original(config, copied_until);
                        ui.close_menu();
                    }
                    if ui.button("Проверить соединение").clicked() {
                        test = true;
                        ui.close_menu();
                    }
                });
            }
        });
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("ПАРАМЕТРЫ СОЕДИНЕНИЯ")
                .size(10.0)
                .strong()
                .color(theme::TEXT_MUTED),
        );
        ui.add_space(6.0);
        show_fields(ui, config, compact);
        ui.add_space(8.0);
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(0.0, 4.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(if json_payload {
                        "Исходный JSON"
                    } else {
                        "URI конфигурации"
                    })
                    .size(11.0)
                    .strong()
                    .color(theme::TEXT_MUTED),
                );
                let mut raw_uri = config.original_text().to_owned();
                ui.add(
                    egui::TextEdit::singleline(&mut raw_uri)
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                )
                .on_hover_text(config.original_text());
            });
        if let Some(result) = diagnostic {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("ПРОВЕРКА СОЕДИНЕНИЯ")
                    .size(10.0)
                    .strong()
                    .color(theme::TEXT_MUTED),
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
                egui::RichText::new("Проверяются только DNS и TCP, не работа прокси-протокола.")
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
            );
        }
    });
    test
}

fn show_fields(ui: &mut egui::Ui, config: &ProxyConfig, compact: bool) {
    let mut fields = vec![
        ("Протокол", config.protocol.as_str().to_owned()),
        ("Адрес", config.host.clone()),
        ("Порт", display_port(config)),
    ];
    if config.security != Security::Unknown {
        fields.push(("Безопасность", config.security.as_str().to_owned()));
    }
    if config.transport != Transport::Unknown {
        fields.push(("Транспорт", config.transport.as_str().to_owned()));
    }
    add_optional(&mut fields, "SNI", config.sni.as_deref());
    add_optional(&mut fields, "Fingerprint", config.fingerprint.as_deref());
    add_optional(
        &mut fields,
        "Открытый ключ",
        config.reality_public_key.as_deref(),
    );
    add_optional(&mut fields, "Short ID", config.reality_short_id.as_deref());
    add_optional(&mut fields, "UUID", config.uuid.as_deref());
    add_optional(&mut fields, "Имя пользователя", config.username.as_deref());
    add_optional(&mut fields, "Пароль", config.password.as_deref());
    add_optional(&mut fields, "Flow", config.flow.as_deref());
    add_optional(&mut fields, "Шифрование", config.encryption.as_deref());
    add_optional(&mut fields, "Путь", config.path.as_deref());
    add_optional(&mut fields, "Host", config.host_header.as_deref());
    add_optional(&mut fields, "Имя сервиса", config.service_name.as_deref());
    add_optional(&mut fields, "Режим", config.mode.as_deref());
    if !config.unknown_params.is_empty() {
        let extra = config
            .unknown_params
            .iter()
            .flat_map(|(key, values)| values.iter().map(move |value| format!("{key}={value}")))
            .collect::<Vec<_>>()
            .join(" · ");
        fields.push(("Дополнительные параметры", extra));
    }

    let two_columns = !compact && ui.available_width() >= 620.0;
    if compact || !two_columns {
        egui::Grid::new(ui.id().with("configuration-fields"))
            .num_columns(2)
            .spacing(egui::vec2(14.0, 8.0))
            .show(ui, |ui| {
                for (label, value) in &fields {
                    ui.label(
                        egui::RichText::new(*label)
                            .size(11.0)
                            .color(theme::TEXT_MUTED),
                    );
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(compact_value(value, 42))
                                .size(12.0)
                                .strong(),
                        )
                        .truncate(),
                    )
                    .on_hover_text(value);
                    ui.end_row();
                }
            });
    } else {
        let split = fields.len().div_ceil(2);
        let grid_id = ui.id();
        ui.columns(2, |columns| {
            for (column_index, chunk) in fields.chunks(split).enumerate() {
                egui::Grid::new(grid_id.with(("configuration-fields", column_index)))
                    .num_columns(2)
                    .spacing(egui::vec2(12.0, 8.0))
                    .show(&mut columns[column_index], |ui| {
                        for (label, value) in chunk {
                            ui.label(
                                egui::RichText::new(*label)
                                    .size(11.0)
                                    .color(theme::TEXT_MUTED),
                            );
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(compact_value(value, 80))
                                        .size(12.0)
                                        .strong(),
                                )
                                .truncate(),
                            )
                            .on_hover_text(value);
                            ui.end_row();
                        }
                    });
            }
        });
    }
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

pub fn copy_config(config: &ProxyConfig, copied_until: &mut Option<Instant>) {
    if let Some(uri) = config.share_uri.uri() {
        copy_to_clipboard(uri);
        *copied_until = Some(Instant::now() + Duration::from_secs(2));
    }
}

pub fn copy_original(config: &ProxyConfig, copied_until: &mut Option<Instant>) {
    copy_to_clipboard(config.original_text());
    *copied_until = Some(Instant::now() + Duration::from_secs(2));
}

fn show_share_copy_actions(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    copied_until: &mut Option<Instant>,
) {
    let copied = copied_until.is_some();
    match &config.share_uri {
        crate::model::ShareUriResult::Available { .. } => {
            if ui
                .add_sized(
                    [178.0, 34.0],
                    egui::Button::new(if copied {
                        "Скопировано"
                    } else {
                        "Скопировать конфигурацию"
                    })
                    .fill(theme::ACCENT),
                )
                .clicked()
            {
                copy_config(config, copied_until);
            }
        }
        crate::model::ShareUriResult::Limited { .. } => {
            ui.add_enabled(false, egui::Button::new("Есть ограничения"));
            if ui.button("Скопировать с ограничениями").clicked() {
                copy_config(config, copied_until);
            }
        }
        crate::model::ShareUriResult::Unavailable { .. } => {
            ui.add_enabled(false, egui::Button::new("Ссылка недоступна"));
            if config.original_is_json() && ui.button("Скопировать исходный JSON").clicked()
            {
                copy_original(config, copied_until);
            }
        }
    }
    if !config.share_uri.limitations().is_empty() {
        ui.label(
            egui::RichText::new(
                config
                    .share_uri
                    .limitations()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" · "),
            )
            .size(10.0)
            .color(theme::WARNING),
        );
    }
}

fn status_badge(ui: &mut egui::Ui, label: &str, status: &crate::diagnostics::CheckStatus) {
    let (text, fill, color) = match status {
        crate::diagnostics::CheckStatus::Passed => (
            format!("{label}  УСПЕШНО"),
            theme::SUCCESS,
            egui::Color32::WHITE,
        ),
        crate::diagnostics::CheckStatus::Failed(error) => (
            format!("{label}  ОШИБКА: {}", compact_value(error, 28)),
            egui::Color32::from_rgba_unmultiplied(241, 107, 107, 35),
            theme::ERROR,
        ),
        crate::diagnostics::CheckStatus::Skipped => (
            format!("{label}  ПРОПУЩЕНО"),
            theme::SURFACE_RAISED,
            theme::TEXT_SECONDARY,
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
