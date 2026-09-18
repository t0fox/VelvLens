use eframe::egui;

use crate::{model::Protocol, resolver::AnalysisReport};

use super::{config_card, details, inspector, theme};

pub struct MainViewResult {
    pub analyze: bool,
    pub cancel: bool,
    pub selected: Option<usize>,
    pub copy_all: bool,
    pub open_settings: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ctx: &egui::Context,
    input: &mut String,
    history: &[String],
    report: Option<&AnalysisReport>,
    selected: &mut Option<usize>,
    filter: &mut Option<Protocol>,
    search: &mut String,
    running: bool,
    status: &str,
    show_sensitive: bool,
    qr: &mut Option<crate::qr::QrMatrix>,
) -> MainViewResult {
    let mut result = MainViewResult {
        analyze: false,
        cancel: false,
        selected: *selected,
        copy_all: false,
        open_settings: false,
    };

    egui::TopBottomPanel::top("header")
        .frame(
            egui::Frame::none()
                .fill(theme::CANVAS)
                .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                theme::draw_logo(ui);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("SubLens").size(18.0).strong());
                    ui.label(
                        egui::RichText::new("visual proxy subscription inspector")
                            .size(11.0)
                            .color(theme::MUTED),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let settings = ui.add_sized(
                        [92.0, 34.0],
                        egui::Button::new(egui::RichText::new("Settings").size(12.0))
                            .fill(theme::SURFACE_RAISED),
                    );
                    if settings.clicked() {
                        result.open_settings = true;
                    }
                });
            });

            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Source URL")
                    .size(11.0)
                    .strong()
                    .color(theme::MUTED),
            );
            ui.horizontal(|ui| {
                let reserved = if history.is_empty() { 292.0 } else { 388.0 };
                let input_width = (ui.available_width() - reserved).max(240.0);
                ui.add_sized(
                    [input_width, 36.0],
                    egui::TextEdit::singleline(input)
                        .hint_text("Paste a subscription URL…")
                        .font(egui::TextStyle::Body),
                );
                if !history.is_empty() {
                    egui::ComboBox::from_id_salt("recent-history")
                        .selected_text("Recent")
                        .width(78.0)
                        .show_ui(ui, |ui| {
                            for url in history {
                                if ui.selectable_label(false, url).clicked() {
                                    *input = url.clone();
                                }
                            }
                        });
                }
                let paste = ui.add_sized(
                    [94.0, 36.0],
                    egui::Button::new(egui::RichText::new("Paste").size(12.0))
                        .fill(theme::SURFACE_RAISED),
                );
                if paste.clicked() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        if let Ok(text) = clipboard.get_text() {
                            *input = text;
                        }
                    }
                }
                let action = if running {
                    ui.add_sized(
                        [132.0, 36.0],
                        egui::Button::new(egui::RichText::new("Cancel").size(13.0).strong())
                            .fill(theme::SURFACE_RAISED),
                    )
                } else {
                    ui.add_sized(
                        [142.0, 36.0],
                        egui::Button::new(egui::RichText::new("Analyze").size(13.0).strong())
                            .fill(theme::ACCENT),
                    )
                };
                if action.clicked() {
                    if running {
                        result.cancel = true;
                    } else {
                        result.analyze = true;
                    }
                }
            });

            ui.add_space(10.0);
            inspector::pipeline_strip(ui, report, running, status);
        });

    egui::SidePanel::left("catalog")
        .resizable(true)
        .default_width(344.0)
        .min_width(300.0)
        .max_width(420.0)
        .frame(theme::surface_frame(theme::SURFACE))
        .show(ctx, |ui| {
            if let Some(report) = report {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Configurations").size(16.0).strong());
                    theme::badge(
                        ui,
                        &report.configs.len().to_string(),
                        theme::SURFACE_RAISED,
                        theme::MUTED,
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let copy = ui.add_sized(
                            [88.0, 30.0],
                            egui::Button::new(egui::RichText::new("Copy all").size(11.0))
                                .fill(theme::SURFACE_RAISED),
                        );
                        if copy.clicked() {
                            result.copy_all = true;
                        }
                    });
                });
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    filter_button(ui, "All", report.configs.len(), filter, None);
                    for protocol in Protocol::ALL {
                        let count = report
                            .configs
                            .iter()
                            .filter(|config| config.protocol == protocol)
                            .count();
                        filter_button(ui, protocol.as_str(), count, filter, Some(protocol));
                    }
                });
                ui.add_space(8.0);
                let search_width = ui.available_width();
                ui.add_sized(
                    [search_width, 32.0],
                    egui::TextEdit::singleline(search).hint_text("Search host or name…"),
                );
                ui.add_space(10.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (index, config) in report.configs.iter().enumerate() {
                            if filter
                                .as_ref()
                                .is_some_and(|active| *active != config.protocol)
                            {
                                continue;
                            }
                            if !search.is_empty()
                                && !format!(
                                    "{} {}",
                                    config.name.as_deref().unwrap_or_default(),
                                    config.host
                                )
                                .to_ascii_lowercase()
                                .contains(&search.to_ascii_lowercase())
                            {
                                continue;
                            }
                            let response = config_card::show(ui, config, *selected == Some(index));
                            if response.clicked() {
                                *selected = Some(index);
                                result.selected = Some(index);
                            }
                            ui.add_space(8.0);
                        }
                    });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("Configurations").size(15.0).strong());
                    ui.label(egui::RichText::new("No analysis yet").color(theme::MUTED));
                });
            }
        });

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(theme::CANVAS)
                .inner_margin(egui::Margin::same(18.0)),
        )
        .show(ctx, |ui| {
            if report.is_none() {
                theme::draw_network_backdrop(ui);
            }
            if let Some(report) = report {
                if let Some(index) = *selected {
                    if let Some(config) = report.configs.get(index) {
                        let content_width = ui.available_width().min(920.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(content_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| details::show(ui, config, show_sensitive, qr),
                        );
                        ui.add_space(12.0);
                        egui::CollapsingHeader::new("Decode pipeline")
                            .default_open(false)
                            .show(ui, |ui| inspector::show_stages(ui, report));
                    }
                } else {
                    inspector::show(ui, report);
                }
            } else {
                empty_state(ui, status, running);
            }
        });

    if let Some(matrix) = qr.as_ref() {
        let mut close = false;
        egui::Window::new("QR code")
            .collapsible(false)
            .resizable(false)
            .frame(theme::surface_frame(theme::SURFACE))
            .show(ctx, |ui| {
                draw_qr(ui, matrix);
                ui.add_space(8.0);
                if ui
                    .add_sized(
                        [280.0, 34.0],
                        egui::Button::new("Close").fill(theme::SURFACE_RAISED),
                    )
                    .clicked()
                {
                    close = true;
                }
            });
        if close {
            *qr = None;
        }
    }
    result
}

fn filter_button(
    ui: &mut egui::Ui,
    label: &str,
    count: usize,
    filter: &mut Option<Protocol>,
    value: Option<Protocol>,
) {
    let active = *filter == value;
    let fill = if active {
        theme::ACCENT
    } else {
        theme::SURFACE_RAISED
    };
    let color = if active {
        egui::Color32::WHITE
    } else {
        theme::MUTED
    };
    let width = match label {
        "All" => 76.0,
        "VLESS" | "VMess" | "Trojan" | "TUIC" => 92.0,
        "Hysteria2" => 112.0,
        "Shadowsocks" => 128.0,
        _ => 96.0,
    };
    let response = ui.add_sized(
        [width, 30.0],
        egui::Button::new(
            egui::RichText::new(format!("{label}  {count}"))
                .size(11.0)
                .strong()
                .color(color),
        )
        .fill(fill),
    );
    if response.clicked() {
        *filter = value;
    }
}

fn empty_state(ui: &mut egui::Ui, status: &str, running: bool) {
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(78.0);
            theme::draw_logo(ui);
            ui.add_space(14.0);
            if running {
                ui.spinner();
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Inspecting source").size(22.0).strong());
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Fetching and decoding locally. This can take a moment.")
                        .size(13.0)
                        .color(theme::MUTED),
                );
            } else if let Some(error) = status.strip_prefix("Analysis failed: ") {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(241, 107, 107, 20))
                    .stroke(egui::Stroke::new(1.0_f32, theme::ERROR))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Analysis couldn't complete")
                                .size(16.0)
                                .strong()
                                .color(theme::ERROR),
                        );
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(error).size(12.0).color(theme::MUTED));
                    });
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Check the source URL and try Analyze again.")
                        .size(13.0)
                        .color(theme::MUTED),
                );
            } else {
                ui.label(
                    egui::RichText::new("Inspect a subscription locally")
                        .size(22.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(
                        "Paste a source above to see its decode and extraction stages.",
                    )
                    .size(13.0)
                    .color(theme::MUTED),
                );
            }
            ui.add_space(78.0);
        });
    });
}

fn draw_qr(ui: &mut egui::Ui, matrix: &crate::qr::QrMatrix) {
    let size = 280.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let cell = size / matrix.width() as f32;
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, egui::Rounding::same(8.0), egui::Color32::WHITE);
    for y in 0..matrix.width() {
        for x in 0..matrix.width() {
            if matrix.is_dark(x, y) {
                let min = rect.min + egui::vec2(x as f32 * cell, y as f32 * cell);
                painter.rect_filled(
                    egui::Rect::from_min_size(min, egui::vec2(cell + 0.5, cell + 0.5)),
                    0.0,
                    egui::Color32::BLACK,
                );
            }
        }
    }
}
