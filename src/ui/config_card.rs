use eframe::egui;

use crate::model::{Protocol, ProxyConfig};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardResponse {
    pub clicked: bool,
    pub selection_toggled: bool,
}

pub fn show(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    selected: bool,
    selected_for_export: bool,
) -> CardResponse {
    let cursor = ui.cursor().min;
    let card_rect = egui::Rect::from_min_size(cursor, egui::vec2(ui.available_width(), 80.0));
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
    let mut details_clicked = false;
    let frame_response = egui::Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                protocol_mark(ui, config.protocol);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(
                            config.name.as_deref().unwrap_or("Unnamed configuration"),
                        )
                        .size(13.0)
                        .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("{}:{}", config.host, config.port))
                            .size(11.0)
                            .color(theme::MUTED),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui
                        .add_sized(
                            [62.0, 26.0],
                            egui::Button::new(egui::RichText::new("Details").size(10.0))
                                .fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        details_clicked = true;
                    }
                    let mut checked = selected_for_export;
                    if ui.checkbox(&mut checked, "Select").clicked() {
                        selection_toggled = true;
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
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                theme::badge(
                    ui,
                    config.protocol.as_str(),
                    protocol_fill(config.protocol),
                    egui::Color32::WHITE,
                );
                theme::badge(
                    ui,
                    config.security.as_str(),
                    theme::SURFACE_RAISED,
                    theme::MUTED,
                );
                theme::badge(
                    ui,
                    config.transport.as_str(),
                    theme::SURFACE_RAISED,
                    theme::MUTED,
                );
            });
        })
        .response;
    let response = ui.interact(
        frame_response.rect,
        ui.id().with(("config-card", config.id.as_str())),
        egui::Sense::click(),
    );
    CardResponse {
        clicked: response.clicked() || details_clicked,
        selection_toggled,
    }
}

fn protocol_mark(ui: &mut egui::Ui, protocol: Protocol) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, egui::Rounding::same(9.0), protocol_fill(protocol));
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
        Protocol::Hysteria2 => egui::Color32::from_rgb(22, 163, 74),
        Protocol::Tuic => egui::Color32::from_rgb(219, 39, 119),
        Protocol::Unknown => egui::Color32::from_rgb(100, 116, 139),
    }
}
