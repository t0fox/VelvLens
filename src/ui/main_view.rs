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
    qr: &mut Option<crate::qr::QrMatrix>,
    diagnostics: &HashMap<usize, crate::diagnostics::DiagnosticResult>,
    copied_until: &mut Option<std::time::Instant>,
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
                .inner_margin(egui::Margin::symmetric(20.0, 8.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                theme::draw_logo(ui, 32.0);
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("SubLens").size(17.0).strong());
                    ui.label(
                        egui::RichText::new("visual proxy subscription inspector")
                            .size(10.0)
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

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Source URL")
                    .size(10.0)
                    .strong()
                    .color(theme::MUTED),
            );
            ui.horizontal(|ui| {
                let reserved = if history.is_empty() { 292.0 } else { 388.0 };
                let input_width = (ui.available_width() - reserved).max(240.0);
                ui.add_sized(
                    [input_width, 34.0],
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
                    [94.0, 34.0],
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
                        [132.0, 34.0],
                        egui::Button::new(egui::RichText::new("Cancel").size(13.0).strong())
                            .fill(theme::SURFACE_RAISED),
                    )
                } else {
                    ui.add_sized(
                        [142.0, 34.0],
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

            ui.add_space(6.0);
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
                            egui::Button::new(egui::RichText::new("Copy all configs").size(11.0))
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
                ui.add_space(3.0);
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.horizontal_wrapped(|ui| {
                        filter_button(ui, "All", report.configs.len(), protocol_filters, None);
                        for protocol in Protocol::ALL {
                            let count = report
                                .configs
                                .iter()
                                .filter(|config| config.protocol == protocol)
                                .count();
                            let label = match protocol {
                                Protocol::Shadowsocks => "SS",
                                Protocol::Unknown => "Other",
                                _ => protocol.as_str(),
                            };
                            filter_button(ui, label, count, protocol_filters, Some(protocol));
                        }
                    });
                });
                ui.add_space(6.0);
                let filters_active = !protocol_filters.is_empty()
                    || !search.trim().is_empty()
                    || security_filter.is_some()
                    || transport_filter.is_some()
                    || !matches!(connectivity_filter, ConnectivityFilter::All)
                    || !matches!(duplicate_filter, DuplicateFilter::All);
                egui::CollapsingHeader::new(if filters_active {
                    "More filters · active"
                } else {
                    "More filters"
                })
                .default_open(false)
                .show(ui, |ui| {
                    egui::Grid::new("catalog-filter-grid")
                        .num_columns(2)
                        .spacing(egui::vec2(6.0, 6.0))
                        .show(ui, |ui| {
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
                            ui.end_row();
                            connectivity_filter_combo(ui, connectivity_filter);
                            duplicate_filter_combo(ui, duplicate_filter);
                            ui.end_row();
                        });
                    if ui
                        .add_enabled(filters_active, egui::Button::new("Reset filters"))
                        .clicked()
                    {
                        protocol_filters.clear();
                        search.clear();
                        *security_filter = None;
                        *transport_filter = None;
                        *connectivity_filter = ConnectivityFilter::All;
                        *duplicate_filter = DuplicateFilter::All;
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Filter")
                            .size(10.0)
                            .strong()
                            .color(theme::MUTED),
                    );
                    let search_width = ui.available_width();
                    ui.add_sized(
                        [search_width, 30.0],
                        egui::TextEdit::singleline(search)
                            .hint_text("Search name/host or exclude -LTE…"),
                    );
                });
                let search_query = search.to_lowercase();
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
                reconcile_selection(selected, &visible_indices);
                result.selected = *selected;
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_sized(
                            [102.0, 26.0],
                            egui::Button::new("Select visible").fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        selected_configs.extend(visible_indices.iter().copied());
                    }
                    if ui
                        .add_sized(
                            [64.0, 26.0],
                            egui::Button::new("Clear selection").fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        selected_configs.clear();
                    }
                    let selected_count = selected_configs.len();
                    let copy_selected = ui
                        .add_enabled_ui(selected_count > 0, |ui| {
                            ui.add_sized(
                                [112.0, 26.0],
                                egui::Button::new(format!("Copy selected ({selected_count})"))
                                    .fill(if selected_count > 0 {
                                        theme::ACCENT
                                    } else {
                                        theme::SURFACE_RAISED
                                    }),
                            )
                        })
                        .inner;
                    if copy_selected.clicked() {
                        result.copy_selected = true;
                    }
                });
                ui.add_space(6.0);
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
                if visible_indices.is_empty() {
                    egui::Frame::none()
                        .fill(theme::CANVAS)
                        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(12.0, 12.0))
                        .show(ui, |ui| {
                            let protocol_only = protocol_filters.len() == 1
                                && protocol_filters.contains(&Protocol::Hysteria2);
                            let title = if protocol_only {
                                "No Hysteria2 configurations"
                            } else {
                                "No matching configurations"
                            };
                            let description = if protocol_only && report.configs.is_empty() {
                                "Analyze a source first to populate this category."
                            } else if protocol_only {
                                "This response contains no Hysteria2 entries."
                            } else if report.configs.is_empty() {
                                "The source did not contain inspectable configurations."
                            } else {
                                "Clear a filter or change the search query to see more."
                            };
                            ui.label(egui::RichText::new(title).strong());
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(description)
                                    .size(11.0)
                                    .color(theme::MUTED),
                            );
                        });
                } else {
                    let list_height = ui.available_height().max(140.0);
                    // This is the only vertical scroller in the catalog. Keep it as a direct
                    // child of the panel so egui can resolve the remaining height and paint the
                    // scrollbar instead of clipping the cards in a nested allocated rect.
                    let mut scroll_style = egui::style::ScrollStyle::solid();
                    scroll_style.bar_width = 9.0;
                    scroll_style.handle_min_length = 36.0;
                    scroll_style.bar_inner_margin = 3.0;
                    scroll_style.bar_outer_margin = 2.0;
                    scroll_style.foreground_color = true;
                    ui.spacing_mut().scroll = scroll_style;
                    egui::ScrollArea::vertical()
                        .id_salt("catalog-list")
                        .max_height(list_height)
                        .min_scrolled_height(list_height)
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .auto_shrink([false, false])
                        .show_rows(ui, 74.0, visible_indices.len(), |ui, row_range| {
                            for row in row_range {
                                let index = visible_indices[row];
                                let config = &report.configs[index];
                                let response = ui
                                    .push_id(index, |ui| {
                                        config_card::show(
                                            ui,
                                            config,
                                            *selected == Some(index),
                                            selected_configs.contains(&index),
                                        )
                                    })
                                    .inner;
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
                                ui.add_space(6.0);
                            }
                        });
                }
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
                let selected_index = *selected;
                // Salt the scroll state with the selected index. A newly selected profile
                // always opens at its header, while long details and the inspector remain
                // in one independently scrollable workspace.
                egui::ScrollArea::vertical()
                    .id_salt(("workspace-inspector", selected_index))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Let the details breathe on normal desktop widths, but keep a
                        // readable cap on ultra-wide windows so the compact utility does not
                        // turn into a stretched card with oversized scan lines.
                        let content_width = ui.available_width().min(1440.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(content_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                if let Some(index) = selected_index {
                                    if let Some(config) = report.configs.get(index) {
                                        if details::show(
                                            ui,
                                            config,
                                            qr,
                                            diagnostics.get(&index),
                                            copied_until,
                                        ) {
                                            result.test = Some(index);
                                        }
                                    }
                                }
                                ui.add_space(12.0);
                                inspector::show(ui, report);
                            },
                        );
                    });
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
    // Keep the complete protocol taxonomy visible without turning the catalog
    // header into a three-row wall of controls on compact windows.
    let label = format!("{label} {count}");
    let width = (label.chars().count() as f32 * 5.0 + 18.0).max(52.0);
    let response = ui.add_sized(
        [width, 25.0],
        egui::Button::new(egui::RichText::new(label).size(10.5).strong().color(color)).fill(fill),
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

fn reconcile_selection(selected: &mut Option<usize>, visible_indices: &[usize]) {
    if !selected.is_some_and(|index| visible_indices.contains(&index)) {
        *selected = visible_indices.first().copied();
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
        config
            .port_range
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| config.port.to_string())
    )
    .to_lowercase();
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
            theme::draw_logo(ui, 36.0);
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

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::{is_visible, reconcile_selection, ConnectivityFilter, DuplicateFilter};
    use crate::{model::Protocol, protocols::parse_uri};

    fn matches_filter(uri: &str, protocols: HashSet<Protocol>, search: &str) -> bool {
        let config = parse_uri(uri, None, 0).expect("fixture URI should parse");
        is_visible(
            0,
            &config,
            &protocols,
            None,
            None,
            ConnectivityFilter::All,
            DuplicateFilter::All,
            &HashMap::new(),
            &search.to_lowercase(),
        )
    }

    #[test]
    fn search_is_unicode_aware_and_supports_exclusions() {
        let uri = "vless://00000000-0000-4000-8000-000000000001@nl.example.test:443#SMART-%D0%9D%D0%B8%D0%B4%D0%B5%D1%80%D0%BB%D0%B0%D0%BD%D0%B4%D1%8B-LTE";
        assert!(matches_filter(uri, HashSet::new(), "нидерланды"));
        assert!(!matches_filter(uri, HashSet::new(), "-LTE"));
        assert!(matches_filter(uri, HashSet::new(), "nl.example"));
    }

    #[test]
    fn protocol_filter_selects_hysteria2_without_mixing_protocols() {
        let hysteria = "hysteria2://synthetic-password@hy.example.test:443#HY2-Torrent";
        assert!(matches_filter(
            hysteria,
            HashSet::from([Protocol::Hysteria2]),
            "torrent"
        ));
        assert!(!matches_filter(
            hysteria,
            HashSet::from([Protocol::Vless]),
            "torrent"
        ));
    }

    #[test]
    fn filtered_catalog_never_keeps_a_hidden_selected_profile() {
        let mut selected = Some(0);
        reconcile_selection(&mut selected, &[5, 8]);
        assert_eq!(selected, Some(5));

        reconcile_selection(&mut selected, &[]);
        assert_eq!(selected, None);
    }
}
