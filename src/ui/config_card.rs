use eframe::egui;

use crate::model::{Protocol, ProxyConfig};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardResponse {
    pub clicked: bool,
    pub selection_toggled: bool,
}

/// Backwards-compatible normal card entry point used by the UI smoke test.
pub fn show(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    selected: bool,
    selected_for_export: bool,
) -> CardResponse {
    show_with_mode(ui, config, selected, selected_for_export, false)
}

pub fn show_with_mode(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    selected: bool,
    selected_for_export: bool,
    selection_mode: bool,
) -> CardResponse {
    let port = config
        .port_range
        .clone()
        .unwrap_or_else(|| config.port.to_string());
    let card_rect =
        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 68.0));
    let hovered = ui
        .input(|input| input.pointer.hover_pos())
        .is_some_and(|position| card_rect.contains(position));
    let fill = if selected {
        theme::SURFACE_SELECTED
    } else if hovered {
        theme::SURFACE_RAISED
    } else {
        theme::SURFACE
    };
    let stroke = if selected {
        egui::Stroke::new(1.0_f32, theme::BORDER_SELECTED)
    } else if hovered {
        egui::Stroke::new(1.0_f32, theme::ACCENT_HOVER)
    } else {
        egui::Stroke::new(1.0_f32, theme::BORDER)
    };
    let mut selection_toggled = false;
    egui::Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(egui::Rounding::same(theme::RADIUS_CARD))
        .inner_margin(egui::Margin::symmetric(theme::SPACE_12, theme::SPACE_6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                protocol_mark(ui, config.protocol);
                ui.add_space(theme::SPACE_8);
                ui.vertical(|ui| {
                    let name = config.name.as_deref().unwrap_or("Без названия");
                    ui.add(
                        egui::Label::new(egui::RichText::new(name).size(13.5).strong()).truncate(),
                    )
                    .on_hover_text(name);
                    let host = format!("{}:{port}", config.host);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&host)
                                .size(10.5)
                                .color(theme::TEXT_MUTED),
                        )
                        .truncate(),
                    )
                    .on_hover_text(host);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if selection_mode {
                        let mut checked = selected_for_export;
                        if ui.checkbox(&mut checked, "Выбрать").clicked() {
                            selection_toggled = true;
                        }
                    }
                });
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                theme::badge(
                    ui,
                    config.protocol.as_str(),
                    protocol_fill(config.protocol),
                    egui::Color32::WHITE,
                );
                if config.security != crate::model::Security::Unknown {
                    theme::badge(
                        ui,
                        config.security.as_str(),
                        theme::SURFACE_RAISED,
                        theme::TEXT_SECONDARY,
                    );
                }
                if config.transport != crate::model::Transport::Unknown {
                    theme::badge(
                        ui,
                        config.transport.as_str(),
                        theme::SURFACE_RAISED,
                        theme::TEXT_SECONDARY,
                    );
                }
                if config.metadata.exact_duplicate_count > 1 {
                    theme::badge(
                        ui,
                        &format!("×{}", config.metadata.exact_duplicate_count),
                        egui::Color32::from_rgba_unmultiplied(231, 173, 60, 34),
                        theme::WARNING,
                    );
                }
            });
        });
    let response = ui.interact(
        card_rect,
        ui.id().with(("config-card", config.id.as_str())),
        egui::Sense::click(),
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            card_rect.expand(1.0),
            egui::Rounding::same(theme::RADIUS_CARD),
            egui::Stroke::new(2.0_f32, theme::ACCENT_HOVER),
        );
    }
    CardResponse {
        clicked: !selection_mode && response.clicked(),
        selection_toggled: selection_toggled || (selection_mode && response.clicked()),
    }
}

fn protocol_mark(ui: &mut egui::Ui, protocol: Protocol) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        egui::Rounding::same(theme::RADIUS_BADGE + 2.0),
        protocol_fill(protocol),
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        protocol.as_str().chars().next().unwrap_or('?'),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );
}

fn protocol_fill(protocol: Protocol) -> egui::Color32 {
    match protocol {
        Protocol::Vless => egui::Color32::from_rgb(124, 58, 237),
        Protocol::Vmess => egui::Color32::from_rgb(37, 99, 235),
        Protocol::Trojan => egui::Color32::from_rgb(234, 88, 12),
        Protocol::Shadowsocks => egui::Color32::from_rgb(14, 165, 164),
        Protocol::Socks5 => egui::Color32::from_rgb(6, 182, 212),
        Protocol::Hysteria2 => egui::Color32::from_rgb(22, 163, 74),
        Protocol::Tuic => egui::Color32::from_rgb(219, 39, 119),
        Protocol::Unknown => egui::Color32::from_rgb(100, 116, 139),
    }
}
