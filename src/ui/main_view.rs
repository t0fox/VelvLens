use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use eframe::egui;

use crate::{
    diagnostics::{CheckStatus, DiagnosticResult},
    model::{Protocol, ProxyConfig, Security, Transport},
    resolver::AnalysisReport,
};

use super::{config_card, details, inspector, theme};

pub struct MainViewResult {
    pub analyze: bool,
    pub cancel: bool,
    pub selected: Option<String>,
    pub catalog_scroll_offset: Option<f32>,
    pub copy_all: bool,
    pub copy_selected: bool,
    pub open_settings: bool,
    pub open_export: bool,
    pub test: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LteFilter {
    #[default]
    All,
    Exclude,
    Only,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompactPage {
    #[default]
    ConfigList,
    ConfigDetails,
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ctx: &egui::Context,
    input: &mut String,
    history: &[String],
    report: Option<&AnalysisReport>,
    selected_config: &mut Option<String>,
    protocol_filters: &mut HashSet<Protocol>,
    selected_configs: &mut HashSet<String>,
    search: &mut String,
    security_filter: &mut Option<Security>,
    transport_filter: &mut Option<Transport>,
    connectivity_filter: &mut ConnectivityFilter,
    duplicate_filter: &mut DuplicateFilter,
    lte_filter: &mut LteFilter,
    selection_mode: &mut bool,
    compact_page: &mut CompactPage,
    running: bool,
    status: &str,
    qr: &mut Option<crate::qr::QrMatrix>,
    diagnostics: &HashMap<usize, DiagnosticResult>,
    copied_until: &mut Option<Instant>,
) -> MainViewResult {
    let mut result = MainViewResult {
        analyze: false,
        cancel: false,
        selected: selected_config.clone(),
        catalog_scroll_offset: None,
        copy_all: false,
        copy_selected: false,
        open_settings: false,
        open_export: false,
        test: None,
    };
    let compact = ctx.screen_rect().width() < theme::WIDE_BREAKPOINT;

    let background_painter = ctx.layer_painter(egui::LayerId::background());
    if compact {
        let shell = ctx.screen_rect().shrink(8.0);
        background_painter.rect_filled(
            shell,
            egui::Rounding::same(theme::RADIUS_WINDOW),
            theme::SURFACE,
        );
        background_painter.rect_stroke(
            shell,
            egui::Rounding::same(theme::RADIUS_WINDOW),
            egui::Stroke::new(1.0_f32, theme::BORDER),
        );
    } else {
        background_painter.rect_filled(ctx.screen_rect(), egui::Rounding::ZERO, theme::CANVAS);
    }

    draw_header(
        ctx,
        input,
        history,
        report,
        running,
        status,
        compact,
        &mut result,
    );

    if compact {
        if *selection_mode {
            egui::TopBottomPanel::bottom("compact-selection-actions")
                .frame(compact_bar_frame())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Выбрано: {}", selected_configs.len()))
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    !selected_configs.is_empty(),
                                    egui::Button::new(format!(
                                        "Скопировать выбранные ({})",
                                        selected_configs.len()
                                    ))
                                    .fill(theme::ACCENT),
                                )
                                .clicked()
                            {
                                result.copy_selected = true;
                            }
                        });
                    });
                });
        } else if *compact_page == CompactPage::ConfigDetails {
            if let Some(config) = selected_config
                .as_ref()
                .and_then(|id| report.and_then(|r| r.configs.iter().find(|c| &c.id == id)))
            {
                egui::TopBottomPanel::bottom("compact-details-actions")
                    .frame(compact_bar_frame())
                    .show(ctx, |ui| {
                        let label = if copied_until.is_some() {
                            "Скопировано"
                        } else {
                            "Скопировать конфигурацию"
                        };
                        if ui
                            .add_sized(
                                [ui.available_width(), 36.0],
                                egui::Button::new(label).fill(theme::ACCENT),
                            )
                            .clicked()
                        {
                            details::copy_config(config, copied_until);
                        }
                    });
            }
        }
        show_compact(
            ctx,
            report,
            selected_config,
            protocol_filters,
            selected_configs,
            search,
            security_filter,
            transport_filter,
            connectivity_filter,
            duplicate_filter,
            lte_filter,
            selection_mode,
            compact_page,
            running,
            status,
            qr,
            diagnostics,
            copied_until,
            &mut result,
        );
    } else {
        show_wide(
            ctx,
            report,
            selected_config,
            protocol_filters,
            selected_configs,
            search,
            security_filter,
            transport_filter,
            connectivity_filter,
            duplicate_filter,
            lte_filter,
            selection_mode,
            running,
            status,
            qr,
            diagnostics,
            copied_until,
            &mut result,
        );
    }

    if let Some(matrix) = qr.clone() {
        show_qr(ctx, &matrix, qr);
    }
    result.selected = selected_config.clone();
    result
}

#[allow(clippy::too_many_arguments)]
fn draw_header(
    ctx: &egui::Context,
    input: &mut String,
    history: &[String],
    report: Option<&AnalysisReport>,
    running: bool,
    status: &str,
    compact: bool,
    result: &mut MainViewResult,
) {
    egui::TopBottomPanel::top("header")
        .resizable(false)
        .show_separator_line(false)
        .min_height(if compact { 120.0 } else { 0.0 })
        .frame(
            egui::Frame::none()
                .fill(if compact {
                    egui::Color32::TRANSPARENT
                } else {
                    theme::CANVAS
                })
                .inner_margin(egui::Margin::symmetric(
                    if compact { 12.0 } else { 20.0 },
                    if compact { 6.0 } else { 8.0 },
                )),
        )
        .show(ctx, |ui| {
            // egui can leave the newly reserved compact header band
            // transparent for one frame when its height changes. Paint the
            // panel background explicitly so the band never flashes as a
            // platform-default blue surface.
            if !compact {
                ui.painter().rect_filled(ui.max_rect(), 0.0, theme::CANVAS);
            }
            ui.horizontal(|ui| {
                theme::draw_logo(ui, if compact { 28.0 } else { 32.0 });
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("SubLens")
                        .size(if compact { 16.0 } else { 17.0 })
                        .strong(),
                );
                if compact {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.menu_button("...", |ui| {
                            if ui.button("Вставить ссылку").clicked() {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    if let Ok(text) = clipboard.get_text() {
                                        *input = text;
                                    }
                                }
                                ui.close_menu();
                            }
                            if ui.button("Настройки").clicked() {
                                result.open_settings = true;
                                ui.close_menu();
                            }
                            if ui
                                .add_enabled(report.is_some(), egui::Button::new("Экспорт"))
                                .clicked()
                            {
                                result.open_export = true;
                                ui.close_menu();
                            }
                            if ui
                                .add_enabled(report.is_some(), egui::Button::new("Скопировать все"))
                                .clicked()
                            {
                                result.copy_all = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            let inspector_id = egui::Id::new("inspector-visible");
                            let mut inspector_visible = ctx.data_mut(|data| {
                                data.get_persisted::<bool>(inspector_id).unwrap_or(false)
                            });
                            if ui
                                .add_enabled(
                                    report.is_some(),
                                    egui::Checkbox::new(
                                        &mut inspector_visible,
                                        "Инспектор обработки",
                                    ),
                                )
                                .changed()
                            {
                                ctx.data_mut(|data| {
                                    data.insert_persisted(inspector_id, inspector_visible);
                                });
                            }
                        });
                    });
                } else {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                report.is_some(),
                                egui::Button::new("Экспорт").fill(theme::SURFACE_RAISED),
                            )
                            .clicked()
                        {
                            result.open_export = true;
                        }
                        if ui
                            .add(egui::Button::new("Настройки").fill(theme::SURFACE_RAISED))
                            .clicked()
                        {
                            result.open_settings = true;
                        }
                        ui.menu_button("Ещё", |ui| {
                            if ui
                                .add_enabled(report.is_some(), egui::Button::new("Скопировать все"))
                                .clicked()
                            {
                                result.copy_all = true;
                                ui.close_menu();
                            }
                            let inspector_id = egui::Id::new("inspector-visible");
                            let mut inspector_visible = ctx.data_mut(|data| {
                                data.get_persisted::<bool>(inspector_id).unwrap_or(false)
                            });
                            if ui
                                .add_enabled(
                                    report.is_some(),
                                    egui::Checkbox::new(
                                        &mut inspector_visible,
                                        "Инспектор обработки",
                                    ),
                                )
                                .changed()
                            {
                                ctx.data_mut(|data| {
                                    data.insert_persisted(inspector_id, inspector_visible);
                                });
                            }
                        });
                    });
                }
            });

            if compact {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let analyze_width = if running { 72.0 } else { 34.0 };
                    let input_width =
                        (ui.available_width() - analyze_width - theme::SPACE_8).max(140.0);
                    ui.add_sized(
                        [input_width, 32.0],
                        egui::TextEdit::singleline(input).hint_text("Вставьте ссылку на подписку…"),
                    );
                    if ui
                        .add_sized(
                            [analyze_width, 32.0],
                            egui::Button::new(if running { "Отмена" } else { "▶" }).fill(
                                if running {
                                    theme::SURFACE_RAISED
                                } else {
                                    theme::ACCENT
                                },
                            ),
                        )
                        .on_hover_text(if running {
                            "Остановить анализ"
                        } else {
                            "Анализировать"
                        })
                        .clicked()
                    {
                        if running {
                            result.cancel = true;
                        } else {
                            result.analyze = true;
                        }
                    }
                });
                ui.add_space(6.0);
                if let Some(report) = report {
                    ui.horizontal(|ui| {
                        let (icon_rect, _) =
                            ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                        let painter = ui.painter_at(icon_rect);
                        painter.circle_filled(icon_rect.center(), 4.0, theme::SUCCESS);
                        let ink = egui::Stroke::new(1.0_f32, theme::CANVAS);
                        painter.line_segment(
                            [
                                icon_rect.center() + egui::vec2(-2.0, 0.0),
                                icon_rect.center() + egui::vec2(-0.5, 1.5),
                            ],
                            ink,
                        );
                        painter.line_segment(
                            [
                                icon_rect.center() + egui::vec2(-0.5, 1.5),
                                icon_rect.center() + egui::vec2(2.5, -2.0),
                            ],
                            ink,
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Готово · {} конфигураций",
                                report.configs.len()
                            ))
                            .size(11.0)
                            .color(theme::TEXT_SECONDARY),
                        );
                    });
                } else {
                    let message = if running {
                        if status.is_empty() {
                            "Анализ…"
                        } else {
                            status
                        }
                    } else {
                        "Готово"
                    };
                    ui.label(egui::RichText::new(message).size(11.0).color(theme::MUTED));
                }
                return;
            }
            ui.add_space(6.0);
            if !compact {
                ui.label(
                    egui::RichText::new("Ссылка источника")
                        .size(10.0)
                        .strong()
                        .color(theme::MUTED),
                );
            }
            ui.horizontal(|ui| {
                let history_width = if history.is_empty() {
                    0.0
                } else if compact {
                    70.0
                } else {
                    82.0
                };
                let action_width = if compact { 64.0 + 88.0 } else { 94.0 + 142.0 };
                let input_width =
                    (ui.available_width() - history_width - action_width - 16.0).max(150.0);
                ui.add_sized(
                    [input_width, if compact { 32.0 } else { 34.0 }],
                    egui::TextEdit::singleline(input).hint_text("Вставьте ссылку на подписку…"),
                );
                if !history.is_empty() {
                    egui::ComboBox::from_id_salt("recent-history")
                        .selected_text("Недавние")
                        .width(if compact { 66.0 } else { 78.0 })
                        .show_ui(ui, |ui| {
                            for url in history {
                                if ui.selectable_label(false, url).clicked() {
                                    *input = url.clone();
                                }
                            }
                        });
                }
                let paste = ui.add_sized(
                    [
                        if compact { 64.0 } else { 94.0 },
                        if compact { 32.0 } else { 34.0 },
                    ],
                    egui::Button::new("Вставить").fill(theme::SURFACE_RAISED),
                );
                if paste.clicked() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        if let Ok(text) = clipboard.get_text() {
                            *input = text;
                        }
                    }
                }
                let action = ui.add_sized(
                    [
                        if compact { 88.0 } else { 142.0 },
                        if compact { 32.0 } else { 34.0 },
                    ],
                    egui::Button::new(if running {
                        "Отмена"
                    } else {
                        "Анализировать"
                    })
                    .fill(if running {
                        theme::SURFACE_RAISED
                    } else {
                        theme::ACCENT
                    }),
                );
                if action.clicked() {
                    if running {
                        result.cancel = true;
                    } else {
                        result.analyze = true;
                    }
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let message = if let Some(report) = report {
                    format!("Готово · Конфигураций: {}", report.configs.len())
                } else if running {
                    if status.is_empty() {
                        "Анализ…".to_owned()
                    } else {
                        status.to_owned()
                    }
                } else {
                    "Готово".to_owned()
                };
                ui.label(egui::RichText::new(message).size(11.0).color(theme::MUTED));
            });
        });
}

fn compact_bar_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
        .rounding(egui::Rounding::same(theme::RADIUS_PANEL))
        .inner_margin(egui::Margin::symmetric(theme::SPACE_12, theme::SPACE_6))
}

#[allow(clippy::too_many_arguments)]
fn show_compact(
    ctx: &egui::Context,
    report: Option<&AnalysisReport>,
    selected_config: &mut Option<String>,
    protocol_filters: &mut HashSet<Protocol>,
    selected_configs: &mut HashSet<String>,
    search: &mut String,
    security_filter: &mut Option<Security>,
    transport_filter: &mut Option<Transport>,
    connectivity_filter: &mut ConnectivityFilter,
    duplicate_filter: &mut DuplicateFilter,
    lte_filter: &mut LteFilter,
    selection_mode: &mut bool,
    compact_page: &mut CompactPage,
    running: bool,
    status: &str,
    qr: &mut Option<crate::qr::QrMatrix>,
    diagnostics: &HashMap<usize, DiagnosticResult>,
    copied_until: &mut Option<Instant>,
    result: &mut MainViewResult,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(egui::Margin::same(10.0)),
        )
        .show(ctx, |ui| match compact_page {
            CompactPage::ConfigDetails => {
                let Some(report) = report else {
                    *compact_page = CompactPage::ConfigList;
                    empty_state(ui, status, running);
                    return;
                };
                if ui.button("‹  К списку конфигураций").clicked() {
                    *compact_page = CompactPage::ConfigList;
                }
                ui.add_space(8.0);
                let Some(index) = selected_config
                    .as_ref()
                    .and_then(|id| report.configs.iter().position(|config| &config.id == id))
                else {
                    *compact_page = CompactPage::ConfigList;
                    return;
                };
                egui::ScrollArea::vertical()
                    .id_salt("compact-details-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if details::show(
                            ui,
                            &report.configs[index],
                            qr,
                            diagnostics.get(&index),
                            copied_until,
                            true,
                        ) {
                            result.test = Some(report.configs[index].id.clone());
                        }
                        let inspector_id = egui::Id::new("inspector-visible");
                        let inspector_visible = ctx.data_mut(|data| {
                            data.get_persisted::<bool>(inspector_id).unwrap_or(false)
                        });
                        if inspector_visible {
                            ui.add_space(12.0);
                            inspector::show(ui, report);
                        }
                    });
            }
            CompactPage::ConfigList => {
                draw_catalog(
                    ui,
                    report,
                    selected_config,
                    protocol_filters,
                    selected_configs,
                    search,
                    security_filter,
                    transport_filter,
                    connectivity_filter,
                    duplicate_filter,
                    lte_filter,
                    selection_mode,
                    true,
                    Some(compact_page),
                    diagnostics,
                    result,
                );
                if report.is_none() {
                    empty_state(ui, status, running);
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn show_wide(
    ctx: &egui::Context,
    report: Option<&AnalysisReport>,
    selected_config: &mut Option<String>,
    protocol_filters: &mut HashSet<Protocol>,
    selected_configs: &mut HashSet<String>,
    search: &mut String,
    security_filter: &mut Option<Security>,
    transport_filter: &mut Option<Transport>,
    connectivity_filter: &mut ConnectivityFilter,
    duplicate_filter: &mut DuplicateFilter,
    lte_filter: &mut LteFilter,
    selection_mode: &mut bool,
    running: bool,
    status: &str,
    qr: &mut Option<crate::qr::QrMatrix>,
    diagnostics: &HashMap<usize, DiagnosticResult>,
    copied_until: &mut Option<Instant>,
    result: &mut MainViewResult,
) {
    egui::SidePanel::left("catalog")
        .resizable(true)
        .default_width(320.0)
        .min_width(280.0)
        .max_width(380.0)
        .frame(theme::surface_frame(theme::SURFACE))
        .show(ctx, |ui| {
            draw_catalog(
                ui,
                report,
                selected_config,
                protocol_filters,
                selected_configs,
                search,
                security_filter,
                transport_filter,
                connectivity_filter,
                duplicate_filter,
                lte_filter,
                selection_mode,
                false,
                None,
                diagnostics,
                result,
            );
        });

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(theme::CANVAS)
                .inner_margin(egui::Margin::same(14.0)),
        )
        .show(ctx, |ui| {
            let Some(report) = report else {
                theme::draw_network_backdrop(ui);
                empty_state(ui, status, running);
                return;
            };
            egui::ScrollArea::vertical()
                .id_salt("wide-details-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(index) = selected_config
                        .as_ref()
                        .and_then(|id| report.configs.iter().position(|config| &config.id == id))
                    {
                        if details::show(
                            ui,
                            &report.configs[index],
                            qr,
                            diagnostics.get(&index),
                            copied_until,
                            false,
                        ) {
                            result.test = Some(report.configs[index].id.clone());
                        }
                    }
                    let inspector_id = egui::Id::new("inspector-visible");
                    let inspector_visible = ctx
                        .data_mut(|data| data.get_persisted::<bool>(inspector_id).unwrap_or(false));
                    if inspector_visible {
                        ui.add_space(12.0);
                        inspector::show(ui, report);
                    }
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_catalog(
    ui: &mut egui::Ui,
    report: Option<&AnalysisReport>,
    selected_config: &mut Option<String>,
    protocol_filters: &mut HashSet<Protocol>,
    selected_configs: &mut HashSet<String>,
    search: &mut String,
    security_filter: &mut Option<Security>,
    transport_filter: &mut Option<Transport>,
    connectivity_filter: &mut ConnectivityFilter,
    duplicate_filter: &mut DuplicateFilter,
    lte_filter: &mut LteFilter,
    selection_mode: &mut bool,
    compact: bool,
    mut compact_page: Option<&mut CompactPage>,
    diagnostics: &HashMap<usize, DiagnosticResult>,
    result: &mut MainViewResult,
) {
    let Some(report) = report else {
        ui.label(egui::RichText::new("Конфигурации").size(15.0).strong());
        ui.label(
            egui::RichText::new("Запустите анализ, чтобы заполнить список.")
                .color(theme::TEXT_SECONDARY),
        );
        return;
    };

    let visible_indices = visible_indices(
        report,
        protocol_filters,
        *security_filter,
        *transport_filter,
        *connectivity_filter,
        *duplicate_filter,
        *lte_filter,
        diagnostics,
        search,
    );
    if !compact {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Конфигурации").size(16.0).strong());
            theme::badge(
                ui,
                &report.configs.len().to_string(),
                theme::SURFACE_RAISED,
                theme::TEXT_SECONDARY,
            );
            if !selected_configs.is_empty() {
                theme::badge(
                    ui,
                    &format!("Выбрано: {}", selected_configs.len()),
                    egui::Color32::from_rgba_unmultiplied(124, 58, 237, 36),
                    theme::ACCENT_HOVER,
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !*selection_mode && ui.button("Выбрать").clicked() {
                    *selection_mode = true;
                }
            });
        });
        ui.add_space(6.0);
    }
    // Keep quick protocol chips only when the resizable catalog is wide enough;
    // the Filters menu remains the canonical fallback at compact widths.
    if !compact && ui.available_width() >= 360.0 {
        protocol_chips(ui, report, protocol_filters);
        ui.add_space(4.0);
    }
    ui.horizontal(|ui| {
        let filter_width = if compact { 78.0 } else { 96.0 };
        let search_width = (ui.available_width() - filter_width).max(120.0);
        ui.allocate_ui(egui::vec2(search_width, 34.0), |ui| {
            egui::Frame::none()
                .fill(theme::SURFACE_HOVER)
                .stroke(egui::Stroke::new(1.0_f32, theme::BORDER))
                .rounding(egui::Rounding::same(theme::RADIUS_INPUT))
                .inner_margin(egui::Margin::symmetric(theme::SPACE_8, theme::SPACE_4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (icon_rect, _) =
                            ui.allocate_exact_size(egui::vec2(18.0, 22.0), egui::Sense::hover());
                        let painter = ui.painter();
                        let center = icon_rect.center() - egui::vec2(2.0, 2.0);
                        painter.circle_stroke(
                            center,
                            5.0,
                            egui::Stroke::new(1.5_f32, theme::TEXT_MUTED),
                        );
                        painter.line_segment(
                            [center + egui::vec2(3.5, 3.5), center + egui::vec2(7.0, 7.0)],
                            egui::Stroke::new(1.5_f32, theme::TEXT_MUTED),
                        );
                        ui.add_sized(
                            [ui.available_width(), 24.0],
                            egui::TextEdit::singleline(search)
                                .frame(false)
                                .hint_text(if compact {
                                    "Поиск..."
                                } else {
                                    "Поиск конфигурации..."
                                }),
                        );
                    });
                });
        });
        filter_menu(
            ui,
            report,
            protocol_filters,
            security_filter,
            transport_filter,
            connectivity_filter,
            duplicate_filter,
            lte_filter,
            search,
            compact,
        );
    });
    let active_filter_badges = !protocol_filters.is_empty()
        || security_filter.is_some()
        || transport_filter.is_some()
        || !matches!(connectivity_filter, ConnectivityFilter::All)
        || !matches!(duplicate_filter, DuplicateFilter::All)
        || !matches!(lte_filter, LteFilter::All);
    if active_filter_badges {
        ui.horizontal_wrapped(|ui| {
            for protocol in Protocol::ALL {
                if protocol_filters.contains(&protocol) {
                    theme::badge(
                        ui,
                        protocol.as_str(),
                        theme::SURFACE_SELECTED,
                        theme::ACCENT_HOVER,
                    );
                }
            }
            if let Some(security) = *security_filter {
                theme::badge(
                    ui,
                    security.as_str(),
                    theme::SURFACE_RAISED,
                    theme::TEXT_SECONDARY,
                );
            }
            if let Some(transport) = *transport_filter {
                theme::badge(
                    ui,
                    transport.as_str(),
                    theme::SURFACE_RAISED,
                    theme::TEXT_SECONDARY,
                );
            }
            if !matches!(*connectivity_filter, ConnectivityFilter::All) {
                theme::badge(
                    ui,
                    match *connectivity_filter {
                        ConnectivityFilter::Passed => "Успешные",
                        ConnectivityFilter::Failed => "Ошибки",
                        ConnectivityFilter::NotChecked => "Не проверены",
                        ConnectivityFilter::All => "",
                    },
                    theme::SURFACE_RAISED,
                    theme::TEXT_SECONDARY,
                );
            }
            if !matches!(*duplicate_filter, DuplicateFilter::All) {
                theme::badge(
                    ui,
                    match *duplicate_filter {
                        DuplicateFilter::Unique => "Уникальные",
                        DuplicateFilter::Duplicates => "Повторы",
                        DuplicateFilter::All => "",
                    },
                    theme::SURFACE_RAISED,
                    theme::TEXT_SECONDARY,
                );
            }
            if !matches!(*lte_filter, LteFilter::All) {
                theme::badge(
                    ui,
                    if matches!(*lte_filter, LteFilter::Exclude) {
                        "Без LTE"
                    } else {
                        "Только LTE"
                    },
                    theme::SURFACE_SELECTED,
                    theme::ACCENT_HOVER,
                );
            }
        });
    }
    if compact {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Конфигурации").size(15.0).strong());
            theme::badge(
                ui,
                &report.configs.len().to_string(),
                theme::SURFACE_RAISED,
                theme::TEXT_SECONDARY,
            );
            if !selected_configs.is_empty() {
                theme::badge(
                    ui,
                    &format!("Выбрано: {}", selected_configs.len()),
                    egui::Color32::from_rgba_unmultiplied(124, 58, 237, 36),
                    theme::ACCENT_HOVER,
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !*selection_mode && ui.button("Выбрать").clicked() {
                    *selection_mode = true;
                }
            });
        });
    }
    ui.add_space(if compact { 4.0 } else { 6.0 });
    if *selection_mode {
        selection_controls(
            ui,
            report,
            &visible_indices,
            selected_configs,
            selection_mode,
        );
    }
    ui.add_space(if compact { 4.0 } else { 6.0 });
    if visible_indices.is_empty() {
        empty_catalog(ui, report, protocol_filters);
        return;
    }

    let mut scroll_style = egui::style::ScrollStyle::solid();
    scroll_style.bar_width = 9.0;
    scroll_style.handle_min_length = 36.0;
    scroll_style.bar_inner_margin = 3.0;
    scroll_style.bar_outer_margin = 2.0;
    scroll_style.foreground_color = true;
    ui.spacing_mut().scroll = scroll_style;
    let row_height = if compact { 80.0 } else { 74.0 };
    let list_height = ui.available_height().max(120.0);
    let catalog_scroll_salt = if compact {
        "compact-config-list"
    } else {
        "wide-config-list"
    };
    let wheel_target = catalog_wheel_target(ui, catalog_scroll_salt);
    let mut catalog_scroll = egui::ScrollArea::vertical()
        .id_salt(catalog_scroll_salt)
        .max_height(list_height)
        .min_scrolled_height(list_height)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .auto_shrink([false, false]);
    if let Some(target) = wheel_target {
        catalog_scroll = catalog_scroll.vertical_scroll_offset(target);
    }
    let _scroll_output =
        catalog_scroll.show_rows(ui, row_height, visible_indices.len(), |ui, row_range| {
            for row in row_range {
                let index = visible_indices[row];
                let config = &report.configs[index];
                let card = ui
                    .push_id(config.id.clone(), |ui| {
                        config_card::show_with_mode(
                            ui,
                            config,
                            selected_config.as_deref() == Some(config.id.as_str()),
                            selected_configs.contains(&config.id),
                            *selection_mode,
                            compact,
                        )
                    })
                    .inner;
                if card.selection_toggled {
                    toggle_selection(selected_configs, &config.id);
                }
                if card.clicked {
                    *selected_config = Some(config.id.clone());
                    result.selected = selected_config.clone();
                    if compact {
                        // In compact mode a card is navigation, not a second
                        // Details button. The whole row is the hit target.
                        if let Some(page) = compact_page.as_deref_mut() {
                            *page = CompactPage::ConfigDetails;
                        }
                    }
                }
                ui.add_space(6.0);
            }
        });
    {
        result.catalog_scroll_offset = Some(_scroll_output.state.offset.y);
    }
}

/// Keep the catalog wheel target larger than the list itself. The filters and
/// search field are part of the same visual column, so scrolling over them
/// should continue moving the configuration list instead of doing nothing.
fn catalog_wheel_target(ui: &mut egui::Ui, scroll_salt: &str) -> Option<f32> {
    if !ui.rect_contains_pointer(ui.max_rect()) {
        return None;
    }
    let raw_delta = ui.input(|input| input.raw_scroll_delta.y);

    // Windows mouse wheels arrive as line events. egui deliberately spreads
    // those events over several frames, but this proxy lives outside the
    // ScrollArea and therefore cannot participate in that animation. Apply
    // the native event once and discard the smoothed remainder so the list
    // moves by a useful amount immediately and never scrolls twice.
    ui.input_mut(|input| input.smooth_scroll_delta.y = 0.0);
    if raw_delta.abs() <= f32::EPSILON {
        return None;
    }

    let scroll_id = ui.make_persistent_id(scroll_salt);
    let current_offset = ui.ctx().data_mut(|data| {
        data.get_persisted::<egui::scroll_area::State>(scroll_id)
            .map_or(0.0, |state| state.offset.y)
    });
    Some(current_offset - raw_delta)
}

fn protocol_chips(ui: &mut egui::Ui, report: &AnalysisReport, filters: &mut HashSet<Protocol>) {
    ui.horizontal_wrapped(|ui| {
        filter_button(ui, "Все", report.configs.len(), filters, None);
        for protocol in Protocol::ALL {
            let count = report
                .configs
                .iter()
                .filter(|c| c.protocol == protocol)
                .count();
            if count > 0 || filters.contains(&protocol) {
                let label = match protocol {
                    Protocol::Shadowsocks => "SS",
                    Protocol::Unknown => "Другое",
                    _ => protocol.as_str(),
                };
                filter_button(ui, label, count, filters, Some(protocol));
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn filter_menu(
    ui: &mut egui::Ui,
    report: &AnalysisReport,
    protocol_filters: &mut HashSet<Protocol>,
    security_filter: &mut Option<Security>,
    transport_filter: &mut Option<Transport>,
    connectivity_filter: &mut ConnectivityFilter,
    duplicate_filter: &mut DuplicateFilter,
    lte_filter: &mut LteFilter,
    search: &mut String,
    compact: bool,
) {
    let active = !protocol_filters.is_empty()
        || security_filter.is_some()
        || transport_filter.is_some()
        || !matches!(connectivity_filter, ConnectivityFilter::All)
        || !matches!(duplicate_filter, DuplicateFilter::All)
        || !matches!(lte_filter, LteFilter::All)
        || !search.trim().is_empty();
    ui.menu_button(
        if compact {
            if active {
                "Фильтры ·"
            } else {
                "Фильтры"
            }
        } else {
            "Фильтры"
        },
        |ui| {
            ui.label(
                egui::RichText::new("Протокол")
                    .strong()
                    .color(theme::TEXT_SECONDARY),
            );
            if ui
                .selectable_label(
                    protocol_filters.is_empty(),
                    format!("Все ({})", report.configs.len()),
                )
                .clicked()
            {
                protocol_filters.clear();
            }
            for protocol in Protocol::ALL {
                let count = report
                    .configs
                    .iter()
                    .filter(|c| c.protocol == protocol)
                    .count();
                let mut checked = protocol_filters.contains(&protocol);
                if ui
                    .checkbox(&mut checked, format!("{} ({count})", protocol.as_str()))
                    .clicked()
                {
                    if checked {
                        protocol_filters.insert(protocol);
                    } else {
                        protocol_filters.remove(&protocol);
                    }
                }
            }
            ui.separator();
            ui.label(
                egui::RichText::new("LTE в названии")
                    .strong()
                    .color(theme::MUTED),
            );
            for (value, label) in [
                (LteFilter::All, "Все"),
                (LteFilter::Exclude, "Без LTE"),
                (LteFilter::Only, "Только LTE"),
            ] {
                if ui.selectable_label(*lte_filter == value, label).clicked() {
                    *lte_filter = value;
                }
            }
            ui.separator();
            select_security(ui, security_filter);
            select_transport(ui, transport_filter);
            select_connectivity(ui, connectivity_filter);
            select_duplicates(ui, duplicate_filter);
            if ui.button("Сбросить фильтры").clicked() {
                protocol_filters.clear();
                search.clear();
                *security_filter = None;
                *transport_filter = None;
                *connectivity_filter = ConnectivityFilter::All;
                *duplicate_filter = DuplicateFilter::All;
                *lte_filter = LteFilter::All;
                ui.close_menu();
            }
        },
    );
}

fn selection_controls(
    ui: &mut egui::Ui,
    report: &AnalysisReport,
    visible_indices: &[usize],
    selected_configs: &mut HashSet<String>,
    selection_mode: &mut bool,
) {
    let all_visible = !visible_indices.is_empty()
        && visible_indices
            .iter()
            .all(|index| selected_configs.contains(&report.configs[*index].id));
    let some_visible = visible_indices
        .iter()
        .any(|index| selected_configs.contains(&report.configs[*index].id));
    ui.horizontal_wrapped(|ui| {
        let state_label = if all_visible {
            "Все видимые"
        } else if some_visible {
            "Часть видимых"
        } else {
            "Все видимые"
        };
        if ui
            .add_enabled(
                !visible_indices.is_empty(),
                egui::Button::new(state_label).fill(theme::SURFACE_RAISED),
            )
            .clicked()
        {
            for index in visible_indices {
                let id = &report.configs[*index].id;
                if all_visible {
                    selected_configs.remove(id);
                } else {
                    selected_configs.insert(id.clone());
                }
            }
        }
        ui.label(
            egui::RichText::new(format!("Выбрано: {}", selected_configs.len()))
                .size(11.0)
                .color(theme::TEXT_SECONDARY),
        );
        ui.menu_button("Действия", |ui| {
            if ui.button("Снять выбор с видимых").clicked() {
                for index in visible_indices {
                    selected_configs.remove(&report.configs[*index].id);
                }
                ui.close_menu();
            }
            if ui.button("Сбросить выбор").clicked() {
                selected_configs.clear();
                ui.close_menu();
            }
        });
        if ui.button("Готово").clicked() {
            *selection_mode = false;
        }
    });
}

fn toggle_selection(selected_configs: &mut HashSet<String>, id: &str) {
    if !selected_configs.insert(id.to_owned()) {
        selected_configs.remove(id);
    }
}

fn filter_button(
    ui: &mut egui::Ui,
    label: &str,
    count: usize,
    filters: &mut HashSet<Protocol>,
    value: Option<Protocol>,
) {
    let active = value.is_none() && filters.is_empty()
        || value.is_some_and(|protocol| filters.contains(&protocol));
    let text = format!("{label} {count}");
    let response = ui.add_sized(
        [(text.chars().count() as f32 * 5.4 + 18.0).max(58.0), 26.0],
        egui::Button::new(egui::RichText::new(text).size(10.5).color(if active {
            egui::Color32::WHITE
        } else {
            theme::MUTED
        }))
        .fill(if active {
            theme::ACCENT
        } else {
            theme::SURFACE_RAISED
        }),
    );
    if response.clicked() {
        if let Some(protocol) = value {
            if !filters.insert(protocol) {
                filters.remove(&protocol);
            }
        } else {
            filters.clear();
        }
    }
}

fn select_security(ui: &mut egui::Ui, current: &mut Option<Security>) {
    egui::ComboBox::from_id_salt("security-filter")
        .selected_text(format!(
            "Безопасность: {}",
            current.map(Security::as_str).unwrap_or("Все")
        ))
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "Все").clicked() {
                *current = None;
            }
            for value in [
                Security::Reality,
                Security::Tls,
                Security::None,
                Security::Unknown,
            ] {
                if ui
                    .selectable_label(*current == Some(value), value.as_str())
                    .clicked()
                {
                    *current = Some(value);
                }
            }
        });
}

fn select_transport(ui: &mut egui::Ui, current: &mut Option<Transport>) {
    egui::ComboBox::from_id_salt("transport-filter")
        .selected_text(format!(
            "Транспорт: {}",
            current.map(Transport::as_str).unwrap_or("Все")
        ))
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "Все").clicked() {
                *current = None;
            }
            for value in [
                Transport::Tcp,
                Transport::XHttp,
                Transport::WebSocket,
                Transport::Grpc,
                Transport::Http2,
                Transport::Quic,
                Transport::Unknown,
            ] {
                if ui
                    .selectable_label(*current == Some(value), value.as_str())
                    .clicked()
                {
                    *current = Some(value);
                }
            }
        });
}

fn select_connectivity(ui: &mut egui::Ui, current: &mut ConnectivityFilter) {
    egui::ComboBox::from_id_salt("connectivity-filter")
        .selected_text(format!(
            "Проверка: {}",
            match current {
                ConnectivityFilter::All => "Все",
                ConnectivityFilter::Passed => "Успешные",
                ConnectivityFilter::Failed => "Ошибки",
                ConnectivityFilter::NotChecked => "Не проверены",
            }
        ))
        .show_ui(ui, |ui| {
            for (value, label) in [
                (ConnectivityFilter::All, "Все"),
                (ConnectivityFilter::Passed, "Успешные"),
                (ConnectivityFilter::Failed, "Ошибки"),
                (ConnectivityFilter::NotChecked, "Не проверены"),
            ] {
                if ui.selectable_label(*current == value, label).clicked() {
                    *current = value;
                }
            }
        });
}

fn select_duplicates(ui: &mut egui::Ui, current: &mut DuplicateFilter) {
    egui::ComboBox::from_id_salt("duplicate-filter")
        .selected_text(format!(
            "Повторы: {}",
            match current {
                DuplicateFilter::All => "Все",
                DuplicateFilter::Unique => "Уникальные",
                DuplicateFilter::Duplicates => "Повторы",
            }
        ))
        .show_ui(ui, |ui| {
            for (value, label) in [
                (DuplicateFilter::All, "Все"),
                (DuplicateFilter::Unique, "Уникальные"),
                (DuplicateFilter::Duplicates, "Повторы"),
            ] {
                if ui.selectable_label(*current == value, label).clicked() {
                    *current = value;
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
pub fn visible_indices(
    report: &AnalysisReport,
    protocol_filters: &HashSet<Protocol>,
    security_filter: Option<Security>,
    transport_filter: Option<Transport>,
    connectivity_filter: ConnectivityFilter,
    duplicate_filter: DuplicateFilter,
    lte_filter: LteFilter,
    diagnostics: &HashMap<usize, DiagnosticResult>,
    search: &str,
) -> Vec<usize> {
    let search_query = search.to_lowercase();
    report
        .configs
        .iter()
        .enumerate()
        .filter_map(|(index, config)| {
            is_visible(
                index,
                config,
                protocol_filters,
                security_filter,
                transport_filter,
                connectivity_filter,
                duplicate_filter,
                lte_filter,
                diagnostics,
                &search_query,
            )
            .then_some(index)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn is_visible(
    index: usize,
    config: &ProxyConfig,
    protocol_filters: &HashSet<Protocol>,
    security_filter: Option<Security>,
    transport_filter: Option<Transport>,
    connectivity_filter: ConnectivityFilter,
    duplicate_filter: DuplicateFilter,
    lte_filter: LteFilter,
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
    let lte = is_lte_name(config.name.as_deref());
    if matches!(lte_filter, LteFilter::Exclude) && lte
        || matches!(lte_filter, LteFilter::Only) && !lte
    {
        return false;
    }
    let duplicate = config.metadata.exact_duplicate_count > 1
        || config.metadata.semantic_duplicate_group.is_some();
    if matches!(duplicate_filter, DuplicateFilter::Unique) && duplicate
        || matches!(duplicate_filter, DuplicateFilter::Duplicates) && !duplicate
    {
        return false;
    }
    match connectivity_filter {
        ConnectivityFilter::All => {}
        ConnectivityFilter::NotChecked if diagnostics.contains_key(&index) => return false,
        ConnectivityFilter::NotChecked => {}
        ConnectivityFilter::Passed if !diagnostics.get(&index).is_some_and(diagnostic_passed) => {
            return false
        }
        ConnectivityFilter::Failed if !diagnostics.get(&index).is_some_and(diagnostic_failed) => {
            return false
        }
        ConnectivityFilter::Passed | ConnectivityFilter::Failed => {}
    }
    search_query.split_whitespace().all(|term| {
        let haystack = format!(
            "{} {} {} {}",
            config.name.as_deref().unwrap_or_default(),
            config.host,
            config.protocol.as_str(),
            config.port_range.as_deref().unwrap_or("")
        );
        let haystack = format!("{haystack} {}", config.port).to_lowercase();
        if let Some(excluded) = term.strip_prefix('-') {
            if excluded.eq_ignore_ascii_case("lte") {
                return !lte;
            }
            !excluded.is_empty() && !haystack.contains(excluded)
        } else {
            haystack.contains(term)
        }
    })
}

/// LTE is a token in the display name, not a substring in a host or key.
/// Unicode separators (dashes, brackets, spaces, etc.) are accepted while
/// COMPLETE, DELETE, and other embedded strings remain normal names.
pub fn is_lte_name(name: Option<&str>) -> bool {
    name.map(|name| {
        name.split(|character: char| !character.is_alphanumeric())
            .any(|token| token.eq_ignore_ascii_case("lte"))
    })
    .unwrap_or(false)
}

fn diagnostic_passed(result: &DiagnosticResult) -> bool {
    matches!(result.dns, CheckStatus::Passed) && matches!(result.tcp, CheckStatus::Passed)
}
fn diagnostic_failed(result: &DiagnosticResult) -> bool {
    matches!(result.dns, CheckStatus::Failed(_)) || matches!(result.tcp, CheckStatus::Failed(_))
}

fn empty_catalog(ui: &mut egui::Ui, report: &AnalysisReport, filters: &HashSet<Protocol>) {
    theme::control_frame(theme::CANVAS).show(ui, |ui| {
        let title = if filters.len() == 1 && filters.contains(&Protocol::Hysteria2) {
            "Конфигурации Hysteria2 не найдены"
        } else if report.configs.is_empty() {
            "Конфигурации не найдены"
        } else {
            "Нет подходящих конфигураций"
        };
        ui.label(egui::RichText::new(title).strong());
        ui.label(
            egui::RichText::new("Сбросьте фильтр или измените поисковый запрос.")
                .size(11.0)
                .color(theme::MUTED),
        );
    });
}

fn empty_state(ui: &mut egui::Ui, status: &str, running: bool) {
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            if running {
                ui.spinner();
                ui.label(
                    egui::RichText::new("Проверяем источник…")
                        .size(17.0)
                        .strong(),
                );
                ui.label(egui::RichText::new(status).size(12.0).color(theme::MUTED));
            } else if let Some(error) = status.strip_prefix("Ошибка анализа: ") {
                ui.label(
                    egui::RichText::new("Не удалось завершить анализ")
                        .size(17.0)
                        .strong()
                        .color(theme::ERROR),
                );
                ui.label(egui::RichText::new(error).size(12.0).color(theme::MUTED));
            } else {
                ui.label(
                    egui::RichText::new("Проверьте подписку локально")
                        .size(17.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("Вставьте ссылку выше, чтобы увидеть конфигурации.")
                        .size(12.0)
                        .color(theme::MUTED),
                );
            }
            ui.add_space(24.0);
        });
    });
}

fn show_qr(
    ctx: &egui::Context,
    matrix: &crate::qr::QrMatrix,
    qr: &mut Option<crate::qr::QrMatrix>,
) {
    let mut close = false;
    egui::Window::new("QR-код")
        .collapsible(false)
        .resizable(false)
        .frame(theme::surface_frame(theme::SURFACE))
        .show(ctx, |ui| {
            draw_qr(ui, matrix);
            if ui
                .add_sized(
                    [280.0, 34.0],
                    egui::Button::new("Закрыть").fill(theme::SURFACE_RAISED),
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

fn draw_qr(ui: &mut egui::Ui, matrix: &crate::qr::QrMatrix) {
    let size = 280.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let cell = size / matrix.width() as f32;
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, egui::Rounding::same(8.0), egui::Color32::WHITE);
    for y in 0..matrix.width() {
        for x in 0..matrix.width() {
            if matrix.is_dark(x, y) {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        rect.min + egui::vec2(x as f32 * cell, y as f32 * cell),
                        egui::vec2(cell + 0.5, cell + 0.5),
                    ),
                    0.0,
                    egui::Color32::BLACK,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_lte_name, is_visible, ConnectivityFilter, DuplicateFilter, LteFilter};
    use crate::{model::Protocol, protocols::parse_uri};
    use std::collections::{HashMap, HashSet};

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
            LteFilter::All,
            &HashMap::new(),
            &search.to_lowercase(),
        )
    }

    #[test]
    fn search_is_unicode_aware_and_supports_token_exclusions() {
        let uri = "vless://00000000-0000-4000-8000-000000000001@nl.example.test:443#SMART-%D0%9D%D0%B8%D0%B4%D0%B5%D1%80%D0%BB%D0%B0%D0%BD%D0%B4%D1%8B-LTE";
        assert!(matches_filter(uri, HashSet::new(), "нидерланды"));
        assert!(!matches_filter(uri, HashSet::new(), "-LTE"));
        assert!(matches_filter(uri, HashSet::new(), "nl.example"));
    }

    #[test]
    fn lte_detection_uses_name_token_boundaries() {
        for name in [
            "NL SMART [LTE]",
            "NL-LTE-01",
            "LTE Germany",
            "Germany LTE",
            "NL_LTE_02",
        ] {
            assert!(is_lte_name(Some(name)), "{name}");
        }
        for name in ["COMPLETE", "DELETE", "Satellite"] {
            assert!(!is_lte_name(Some(name)), "{name}");
        }
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
    fn lte_filter_and_exclusion_use_only_display_name_tokens() {
        let lte = parse_uri(
            "vless://00000000-0000-4000-8000-000000000001@lte.example.test:443#NL-LTE-01",
            None,
            0,
        )
        .unwrap();
        let complete = parse_uri(
            "vless://00000000-0000-4000-8000-000000000002@lte.example.test:443#COMPLETE",
            None,
            0,
        )
        .unwrap();
        assert!(is_visible(
            0,
            &lte,
            &HashSet::new(),
            None,
            None,
            ConnectivityFilter::All,
            DuplicateFilter::All,
            LteFilter::Only,
            &HashMap::new(),
            "",
        ));
        assert!(!is_visible(
            0,
            &lte,
            &HashSet::new(),
            None,
            None,
            ConnectivityFilter::All,
            DuplicateFilter::All,
            LteFilter::Exclude,
            &HashMap::new(),
            "",
        ));
        assert!(matches_filter(&complete.raw_uri, HashSet::new(), "-LTE"));
    }
}
