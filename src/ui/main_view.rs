use eframe::egui;

use crate::{model::Protocol, resolver::AnalysisReport};

use super::{config_card, details, inspector};

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
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new("SubLens").color(egui::Color32::from_rgb(110, 190, 255)),
            );
            ui.label(
                egui::RichText::new("visual proxy subscription inspector")
                    .color(egui::Color32::from_rgb(145, 158, 178)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    result.open_settings = true;
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label("Subscription / Share URL");
            ui.add(
                egui::TextEdit::singleline(input)
                    .desired_width(520.0)
                    .hint_text("https://example.com/subscription"),
            );
            if ui.button("Paste").clicked() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        *input = text;
                    }
                }
            }
            if !history.is_empty() {
                egui::ComboBox::from_id_salt("recent-history")
                    .selected_text("Recent")
                    .show_ui(ui, |ui| {
                        for url in history {
                            if ui.selectable_label(false, url).clicked() {
                                *input = url.clone();
                            }
                        }
                    });
            }
            if running {
                if ui.button("Cancel").clicked() {
                    result.cancel = true;
                }
            } else if ui.button("Analyze").clicked() {
                result.analyze = true;
            }
        });
        ui.label(
            egui::RichText::new(status)
                .small()
                .color(egui::Color32::from_rgb(155, 170, 190)),
        );
        ui.add_space(10.0);
    });

    egui::SidePanel::right("inspector")
        .resizable(true)
        .default_width(360.0)
        .show(ctx, |ui| {
            if let (Some(report), Some(index)) = (report, *selected) {
                if let Some(config) = report.configs.get(index) {
                    details::show(ui, config, show_sensitive, qr);
                }
            } else if let Some(report) = report {
                inspector::show(ui, report);
            } else {
                ui.heading("Inspector");
                ui.label("Analyze a subscription to inspect its pipeline.");
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(report) = report {
            inspector::show(ui, report);
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                filter_button(ui, "All", filter, None);
                for protocol in Protocol::ALL {
                    filter_button(ui, protocol.as_str(), filter, Some(protocol));
                }
                ui.separator();
                ui.add(
                    egui::TextEdit::singleline(search)
                        .desired_width(180.0)
                        .hint_text("Search host or name"),
                );
                if ui.button("Copy all for v2rayN").clicked() {
                    result.copy_all = true;
                }
            });
            ui.add_space(8.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
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
            empty_state(ui);
        }
    });

    if let Some(matrix) = qr.as_ref() {
        let mut close = false;
        egui::Window::new("QR Code")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                draw_qr(ui, matrix);
                if ui.button("Close").clicked() {
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
    filter: &mut Option<Protocol>,
    value: Option<Protocol>,
) {
    let active = *filter == value;
    if ui.selectable_label(active, label).clicked() {
        *filter = value;
    }
}

fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        ui.heading("Inspect a subscription locally");
        ui.label("Paste a share URL above. SubLens will show every decode and extraction stage without sending credentials to a backend.");
    });
}

fn draw_qr(ui: &mut egui::Ui, matrix: &crate::qr::QrMatrix) {
    let size = 280.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let cell = size / matrix.width() as f32;
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
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
