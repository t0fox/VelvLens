use std::collections::{HashMap, HashSet};

use eframe::egui;

use crate::{
    diagnostics::{CheckStatus, DiagnosticResult},
    model::{Protocol, Security, Transport},
    resolver::AnalysisReport,
};

use super::{config_card, details, inspector, theme};

pub struct MainViewResult {
    pub analyze: bool,
    pub cancel: bool,
    pub selected: Option<usize>,
    pub copy_all: bool,
    pub copy_selected: bool,
    pub open_settings: bool,
    pub open_export: bool,
    pub test: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectivityFilter {
    #[default]
    All,
    Passed,
    Failed,
    NotChecked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicateFilter {
    #[default]
    All,
    Unique,
    Duplicates,
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ctx: &egui::Context,
    input: &mut String,
    history: &[String],
    report: Option<&AnalysisReport>,
    selected: &mut Option<usize>,
    protocol_filters: &mut HashSet<Protocol>,
    selected_configs: &mut HashSet<usize>,
    search: &mut String,
    security_filter: &mut Option<Security>,
    transport_filter: &mut Option<Transport>,
    connectivity_filter: &mut ConnectivityFilter,
    duplicate_filter: &mut DuplicateFilter,
    running: bool,
    status: &str,
    show_sensitive: bool,
    qr: &mut Option<crate::qr::QrMatrix>,
    diagnostics: &HashMap<usize, crate::diagnostics::DiagnosticResult>,
) -> MainViewResult {
    let mut result = MainViewResult {
        analyze: false,
        cancel: false,
        selected: *selected,
        copy_all: false,
        copy_selected: false,
        open_settings: false,
        open_export: false,
        test: None,
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
                    let export = ui
                        .add_enabled_ui(report.is_some(), |ui| {
                            ui.add_sized(
                                [82.0, 34.0],
                                egui::Button::new(egui::RichText::new("Export").size(12.0))
                                    .fill(theme::SURFACE_RAISED),
                            )
                        })
                        .inner;
                    if export.clicked() {
                        result.open_export = true;
                    }
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
                    if !selected_configs.is_empty() {
                        theme::badge(
                            ui,
                            &format!("{} selected", selected_configs.len()),
                            egui::Color32::from_rgba_unmultiplied(124, 58, 237, 36),
                            theme::ACCENT_HOVER,
                        );
                    }
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
                ui.label(
                    egui::RichText::new("PROTOCOL CATEGORIES")
                        .size(10.0)
                        .strong()
                        .color(theme::MUTED),
                );
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    filter_button(ui, "All", report.configs.len(), protocol_filters, None);
                    for protocol in Protocol::ALL {
                        let count = report
                            .configs
                            .iter()
                            .filter(|config| config.protocol == protocol)
                            .count();
                        filter_button(
                            ui,
                            protocol.as_str(),
                            count,
                            protocol_filters,
                            Some(protocol),
                        );
                    }
                });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    security_filter_combo(
                        ui,
                        security_filter,
                        &[
                            Security::Reality,
                            Security::Tls,
                            Security::None,
                            Security::Unknown,
                        ],
                    );
                    transport_filter_combo(
                        ui,
                        transport_filter,
                        &[
                            Transport::Tcp,
                            Transport::XHttp,
                            Transport::WebSocket,
                            Transport::Grpc,
                            Transport::Http2,
                            Transport::Quic,
                            Transport::Unknown,
                        ],
                    );
                    connectivity_filter_combo(ui, connectivity_filter);
                    duplicate_filter_combo(ui, duplicate_filter);
                });
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Filter configurations")
                        .size(10.0)
                        .strong()
                        .color(theme::MUTED),
                );
                let search_width = ui.available_width();
                ui.add_sized(
                    [search_width, 32.0],
                    egui::TextEdit::singleline(search)
                        .hint_text("Search name/host or exclude -LTE…"),
                );
                let search_query = search.to_ascii_lowercase();
                let visible_indices = report
                    .configs
                    .iter()
                    .enumerate()
                    .filter(|(index, config)| {
                        is_visible(
                            *index,
                            config,
                            protocol_filters,
                            *security_filter,
                            *transport_filter,
                            *connectivity_filter,
                            *duplicate_filter,
                            diagnostics,
                            &search_query,
                        )
                    })
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_sized(
                            [112.0, 28.0],
                            egui::Button::new("Select visible").fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        selected_configs.extend(visible_indices.iter().copied());
                    }
                    if ui
                        .add_sized(
                            [88.0, 28.0],
                            egui::Button::new("Clear selection").fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        selected_configs.clear();
                    }
                    let selected_count = selected_configs.len();
                    let copy_selected = ui.add_enabled(
                        selected_count > 0,
                        egui::Button::new(format!("Copy selected ({selected_count})")).fill(
                            if selected_count > 0 {
                                theme::ACCENT
                            } else {
                                theme::SURFACE_RAISED
                            },
                        ),
                    );
                    if copy_selected.clicked() {
                        result.copy_selected = true;
                    }
                    let filters_active = !protocol_filters.is_empty()
                        || security_filter.is_some()
                        || transport_filter.is_some()
                        || !matches!(connectivity_filter, ConnectivityFilter::All)
                        || !matches!(duplicate_filter, DuplicateFilter::All)
                        || !search.trim().is_empty();
                    if ui
                        .add_enabled(
                            filters_active,
                            egui::Button::new("Reset filters").fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        protocol_filters.clear();
                        *security_filter = None;
                        *transport_filter = None;
                        *connectivity_filter = ConnectivityFilter::All;
                        *duplicate_filter = DuplicateFilter::All;
                        search.clear();
                    }
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Configuration list")
                            .size(10.0)
                            .strong()
                            .color(theme::MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} visible", visible_indices.len()))
                                .size(10.0)
                                .color(theme::MUTED),
                        );
                    });
                });
                let list_height = ui.available_height().max(1.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), list_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        // The default egui scrollbar is intentionally subtle. In this dense
                        // catalog that makes the only scrollable region look clipped, so use a
                        // solid, high-contrast rail and reserve its width in the layout.
                        ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                        ui.spacing_mut().scroll.bar_width = 10.0;
                        egui::ScrollArea::vertical()
                            .id_salt("catalog-list")
                            .max_height(list_height)
                            .min_scrolled_height(list_height)
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                            )
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for index in visible_indices.iter().copied() {
                                    let config = &report.configs[index];
                                    let response = config_card::show(
                                        ui,
                                        config,
                                        *selected == Some(index),
                                        selected_configs.contains(&index),
                                    );
                                    if response.selection_toggled {
                                        if selected_configs.contains(&index) {
                                            selected_configs.remove(&index);
                                        } else {
                                            selected_configs.insert(index);
                                        }
                                    }
                                    if response.clicked {
                                        *selected = Some(index);
                                        result.selected = Some(index);
                                    }
                                    ui.add_space(8.0);
                                }
                            });
                    },
                );
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
                        // Salt the scroll state with the selected index. A newly selected
                        // profile always opens at its header, while long details remain scrollable.
                        egui::ScrollArea::vertical()
                            .id_salt(("workspace-inspector", index))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let content_width = ui.available_width().min(920.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(content_width, ui.available_height()),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        if details::show(
                                            ui,
                                            config,
                                            show_sensitive,
                                            qr,
                                            diagnostics.get(&index),
                                        ) {
                                            result.test = Some(index);
                                        }
                                    },
                                );
                                ui.add_space(12.0);
                                egui::CollapsingHeader::new("Decode pipeline")
                                    .default_open(false)
                                    .show(ui, |ui| inspector::show_stages(ui, report));
                            });
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
    protocol_filters: &mut HashSet<Protocol>,
    value: Option<Protocol>,
) {
    let active = value.is_none() && protocol_filters.is_empty()
        || value.is_some_and(|protocol| protocol_filters.contains(&protocol));
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
    let width = (format!("{label}  {count}").chars().count() as f32 * 6.2 + 28.0).max(62.0);
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
        if let Some(protocol) = value {
            if !protocol_filters.insert(protocol) {
                protocol_filters.remove(&protocol);
            }
        } else {
            protocol_filters.clear();
        }
    }
}

fn security_filter_combo(ui: &mut egui::Ui, current: &mut Option<Security>, values: &[Security]) {
    egui::ComboBox::from_id_salt("security-filter")
        .selected_text(format!(
            "Security: {}",
            current.map(Security::as_str).unwrap_or("All")
        ))
        .width(110.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "All").clicked() {
                *current = None;
            }
            for value in values {
                if ui
                    .selectable_label(*current == Some(*value), value.as_str())
                    .clicked()
                {
                    *current = Some(*value);
                }
            }
        });
}

fn transport_filter_combo(
    ui: &mut egui::Ui,
    current: &mut Option<Transport>,
    values: &[Transport],
) {
    egui::ComboBox::from_id_salt("transport-filter")
        .selected_text(format!(
            "Transport: {}",
            current.map(Transport::as_str).unwrap_or("All")
        ))
        .width(110.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "All").clicked() {
                *current = None;
            }
            for value in values {
                if ui
                    .selectable_label(*current == Some(*value), value.as_str())
                    .clicked()
                {
                    *current = Some(*value);
                }
            }
        });
}

fn connectivity_filter_combo(ui: &mut egui::Ui, current: &mut ConnectivityFilter) {
    egui::ComboBox::from_id_salt("connectivity-filter")
        .selected_text(format!(
            "Status: {}",
            match current {
                ConnectivityFilter::All => "All",
                ConnectivityFilter::Passed => "Passed",
                ConnectivityFilter::Failed => "Failed",
                ConnectivityFilter::NotChecked => "Not tested",
            }
        ))
        .width(110.0)
        .show_ui(ui, |ui| {
            for (value, label) in [
                (ConnectivityFilter::All, "All"),
                (ConnectivityFilter::Passed, "Passed"),
                (ConnectivityFilter::Failed, "Failed"),
                (ConnectivityFilter::NotChecked, "Not tested"),
            ] {
                if ui.selectable_label(*current == value, label).clicked() {
                    *current = value;
                }
            }
        });
}

fn duplicate_filter_combo(ui: &mut egui::Ui, current: &mut DuplicateFilter) {
    egui::ComboBox::from_id_salt("duplicate-filter")
        .selected_text(format!(
            "Duplicates: {}",
            match current {
                DuplicateFilter::All => "All",
                DuplicateFilter::Unique => "Unique",
                DuplicateFilter::Duplicates => "Duplicates",
            }
        ))
        .width(110.0)
        .show_ui(ui, |ui| {
            for (value, label) in [
                (DuplicateFilter::All, "All"),
                (DuplicateFilter::Unique, "Unique"),
                (DuplicateFilter::Duplicates, "Duplicates"),
            ] {
                if ui.selectable_label(*current == value, label).clicked() {
                    *current = value;
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn is_visible(
    index: usize,
    config: &crate::model::ProxyConfig,
    protocol_filters: &HashSet<Protocol>,
    security_filter: Option<Security>,
    transport_filter: Option<Transport>,
    connectivity_filter: ConnectivityFilter,
    duplicate_filter: DuplicateFilter,
    diagnostics: &HashMap<usize, DiagnosticResult>,
    search_query: &str,
) -> bool {
    if !protocol_filters.is_empty() && !protocol_filters.contains(&config.protocol) {
        return false;
    }
    if security_filter.is_some_and(|security| config.security != security)
        || transport_filter.is_some_and(|transport| config.transport != transport)
    {
        return false;
    }
    let is_duplicate = config.metadata.exact_duplicate_count > 1
        || config.metadata.semantic_duplicate_group.is_some();
    if matches!(duplicate_filter, DuplicateFilter::Unique) && is_duplicate
        || matches!(duplicate_filter, DuplicateFilter::Duplicates) && !is_duplicate
    {
        return false;
    }
    match connectivity_filter {
        ConnectivityFilter::All => {}
        ConnectivityFilter::NotChecked if diagnostics.contains_key(&index) => return false,
        ConnectivityFilter::NotChecked => {}
        ConnectivityFilter::Passed => {
            if !diagnostics.get(&index).is_some_and(diagnostic_passed) {
                return false;
            }
        }
        ConnectivityFilter::Failed => {
            if !diagnostics.get(&index).is_some_and(diagnostic_failed) {
                return false;
            }
        }
    }
    if search_query.is_empty() {
        return true;
    }
    let haystack = format!(
        "{} {} {} {}",
        config.name.as_deref().unwrap_or_default(),
        config.host,
        config.protocol.as_str(),
        config.raw_uri
    )
    .to_ascii_lowercase();
    search_query.split_whitespace().all(|term| {
        if let Some(excluded) = term.strip_prefix('-') {
            !excluded.is_empty() && !haystack.contains(excluded)
        } else {
            haystack.contains(term)
        }
    })
}

fn diagnostic_passed(result: &DiagnosticResult) -> bool {
    matches!(result.dns, CheckStatus::Passed) && matches!(result.tcp, CheckStatus::Passed)
}

fn diagnostic_failed(result: &DiagnosticResult) -> bool {
    matches!(result.dns, CheckStatus::Failed(_)) || matches!(result.tcp, CheckStatus::Failed(_))
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
