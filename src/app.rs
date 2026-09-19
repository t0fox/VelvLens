use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crate::{
    diagnostics::DiagnosticResult,
    history::HistoryStore,
    jobs::{JobEvent, JobHandle, JobManager},
    model::Protocol,
    resolver::AnalysisReport,
    settings::AppSettings,
    ui,
};
use eframe::egui;

use crate::ui::main_view::{CompactPage, ConnectivityFilter, DuplicateFilter, LteFilter};
use crate::ui::theme;

pub struct SubLensApp {
    input: String,
    status: String,
    job_manager: JobManager,
    job: Option<JobHandle>,
    diagnostic_job: Option<JobHandle>,
    diagnostic_target: Option<usize>,
    report: Option<AnalysisReport>,
    selected_config: Option<String>,
    protocol_filters: HashSet<Protocol>,
    selected_configs: HashSet<String>,
    search: String,
    security_filter: Option<crate::model::Security>,
    transport_filter: Option<crate::model::Transport>,
    connectivity_filter: ConnectivityFilter,
    duplicate_filter: DuplicateFilter,
    lte_filter: LteFilter,
    selection_mode: bool,
    compact_page: CompactPage,
    settings: AppSettings,
    history: HistoryStore,
    show_settings: bool,
    show_export: bool,
    qr: Option<crate::qr::QrMatrix>,
    diagnostics: HashMap<usize, DiagnosticResult>,
    copied_until: Option<Instant>,
}

impl SubLensApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);
        let settings = AppSettings::load();
        let history = HistoryStore::load(settings.history_enabled);
        Self {
            input: String::new(),
            status: "Готово · анализ выполняется локально".to_owned(),
            job_manager: JobManager::new().expect("Tokio runtime must start"),
            job: None,
            diagnostic_job: None,
            diagnostic_target: None,
            report: None,
            selected_config: None,
            protocol_filters: HashSet::new(),
            selected_configs: HashSet::new(),
            search: String::new(),
            security_filter: None,
            transport_filter: None,
            connectivity_filter: ConnectivityFilter::All,
            duplicate_filter: DuplicateFilter::All,
            lte_filter: LteFilter::All,
            selection_mode: false,
            compact_page: CompactPage::ConfigList,
            settings,
            history,
            show_settings: false,
            show_export: false,
            qr: None,
            diagnostics: HashMap::new(),
            copied_until: None,
        }
    }

    fn start(&mut self) {
        let url = self.input.trim().to_owned();
        if url.is_empty() {
            self.status = "Введите HTTP(S)-ссылку на подписку".to_owned();
            return;
        }
        self.report = None;
        self.show_export = false;
        self.selected_config = None;
        self.protocol_filters.clear();
        self.selected_configs.clear();
        self.security_filter = None;
        self.transport_filter = None;
        self.connectivity_filter = ConnectivityFilter::All;
        self.duplicate_filter = DuplicateFilter::All;
        self.lte_filter = LteFilter::All;
        self.selection_mode = false;
        self.compact_page = CompactPage::ConfigList;
        self.diagnostics.clear();
        if let Some(job) = &self.diagnostic_job {
            job.cancel.cancel();
        }
        self.diagnostic_job = None;
        self.diagnostic_target = None;
        if self.settings.history_enabled {
            self.history = HistoryStore::load(true);
            self.history.append(&url);
        }
        self.status = "Анализ · HTTP → Декодирование → Извлечение → Разбор".to_owned();
        let mut resolver = self.settings.resolver.clone();
        resolver.request_timeout = Duration::from_secs(self.settings.timeout_seconds);
        self.job = Some(self.job_manager.start_analysis(url, resolver));
    }

    fn poll_job(&mut self) -> bool {
        let Some(job) = &self.job else { return false };
        let mut finished = false;
        let mut changed = false;
        while let Ok(event) = job.receiver.try_recv() {
            changed = true;
            match event {
                JobEvent::Started => self.status = "Получение источника…".to_owned(),
                JobEvent::Stage(stage) => {
                    self.status = format!(
                        "{} · {}",
                        ui::inspector::format_stage(stage.kind),
                        stage.preview
                    );
                }
                JobEvent::Completed(report) => {
                    self.status = format!(
                        "HTTP готов · декодирование готово · найдено конфигураций: {}",
                        report.configs.len()
                    );
                    self.selected_config = report.configs.first().map(|config| config.id.clone());
                    self.compact_page = CompactPage::ConfigList;
                    self.report = Some(*report);
                    finished = true;
                }
                JobEvent::Failed(error) => {
                    self.status = format!("Ошибка анализа: {error}");
                    finished = true;
                }
                JobEvent::Cancelled => {
                    self.status = "Анализ отменён".to_owned();
                    finished = true;
                }
                JobEvent::DiagnosticCompleted(_) => {}
            }
        }
        if finished {
            self.job = None;
        }
        changed
    }

    fn poll_diagnostic_job(&mut self) -> bool {
        let Some(job) = &self.diagnostic_job else {
            return false;
        };
        let mut finished = false;
        let mut changed = false;
        while let Ok(event) = job.receiver.try_recv() {
            changed = true;
            match event {
                JobEvent::DiagnosticCompleted(result) => {
                    if let Some(index) = self.diagnostic_target {
                        self.diagnostics.insert(index, result);
                    }
                    self.status = "Проверка соединения завершена · только DNS/TCP".to_owned();
                    finished = true;
                }
                JobEvent::Failed(error) => {
                    self.status = format!("Ошибка проверки соединения: {error}");
                    finished = true;
                }
                JobEvent::Cancelled => {
                    self.status = "Проверка соединения отменена".to_owned();
                    finished = true;
                }
                JobEvent::Started | JobEvent::Stage(_) | JobEvent::Completed(_) => {}
            }
        }
        if finished {
            self.diagnostic_job = None;
            self.diagnostic_target = None;
        }
        changed
    }

    fn start_diagnostic(&mut self, index: usize) {
        let Some(config) = self
            .report
            .as_ref()
            .and_then(|report| report.configs.get(index))
            .cloned()
        else {
            return;
        };
        if let Some(job) = &self.diagnostic_job {
            job.cancel.cancel();
        }
        self.status = "Проверка соединения · DNS → TCP…".to_owned();
        self.diagnostic_job = Some(
            self.job_manager
                .start_diagnostic(config, Duration::from_secs(self.settings.timeout_seconds)),
        );
        self.diagnostic_target = Some(index);
    }
}

impl eframe::App for SubLensApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let job_changed = self.poll_job();
        let diagnostic_changed = self.poll_diagnostic_job();
        if job_changed || diagnostic_changed {
            ctx.request_repaint();
        }
        if self.job.is_some() || self.diagnostic_job.is_some() {
            ctx.request_repaint_after(crate::frame_pacing::repaint_interval_for_frame(frame));
        }
        let running = self.job.is_some();
        if self.show_settings {
            egui::Window::new("Настройки")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let settings_result = ui::settings::show(ui, &mut self.settings);
                    if settings_result.history_toggled {
                        self.history = HistoryStore::load(self.settings.history_enabled);
                    }
                    if settings_result.clear_history {
                        self.history.clear();
                        self.status = "История ссылок очищена".to_owned();
                    }
                    if settings_result.changed {
                        if let Err(error) = self.settings.save() {
                            self.status = format!("Не удалось сохранить настройки: {error}");
                        }
                    }
                    if settings_result.close {
                        self.show_settings = false;
                    }
                });
        }
        if self.show_export {
            let mut close_export = false;
            egui::Window::new("Экспорт конфигураций")
                .collapsible(false)
                .resizable(false)
                .frame(theme::surface_frame(theme::SURFACE))
                .show(ctx, |ui| {
                    if let Some(report) = &self.report {
                        let has_json_payload = report
                            .configs
                            .iter()
                            .any(|config| is_json_payload(&config.raw_uri));
                        ui.label(
                            egui::RichText::new(format!(
                                "Конфигураций: {} · выберите локальный экспорт",
                                report.configs.len()
                            ))
                            .size(12.0)
                            .color(theme::MUTED),
                        );
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                [250.0, 34.0],
                                egui::Button::new(if has_json_payload {
                                    "Скопировать исходные данные конфигураций"
                                } else {
                                    "Скопировать список URI"
                                })
                                .fill(theme::SURFACE_RAISED),
                            )
                            .clicked()
                        {
                            ui::details::copy_to_clipboard(&crate::export::raw_lines(
                                &report.configs,
                            ));
                        }
                        let base64_export = ui.add_enabled(
                            !has_json_payload,
                            egui::Button::new(if has_json_payload {
                                "Base64 недоступен для JSON"
                            } else {
                                "Скопировать подписку Base64"
                            })
                            .fill(theme::SURFACE_RAISED),
                        );
                        if base64_export.clicked() {
                            ui::details::copy_to_clipboard(&crate::export::base64_subscription(
                                &report.configs,
                            ));
                        }
                        if ui
                            .add_sized(
                                [250.0, 34.0],
                                egui::Button::new("Скопировать JSON").fill(theme::SURFACE_RAISED),
                            )
                            .clicked()
                        {
                            if let Ok(json) = crate::export::json_dump(&report.configs, true) {
                                ui::details::copy_to_clipboard(&json);
                            }
                        }
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "Исходный экспорт включает учётные данные только после этого действия.",
                            )
                            .size(11.0)
                            .color(theme::MUTED),
                        );
                    }
                    ui.add_space(10.0);
                    if ui
                        .add_sized(
                            [250.0, 34.0],
                            egui::Button::new("Закрыть").fill(theme::SURFACE_RAISED),
                        )
                        .clicked()
                    {
                        close_export = true;
                    }
                });
            if close_export {
                self.show_export = false;
            }
        }
        let result = ui::main_view::show(
            ctx,
            &mut self.input,
            self.history.urls(),
            self.report.as_ref(),
            &mut self.selected_config,
            &mut self.protocol_filters,
            &mut self.selected_configs,
            &mut self.search,
            &mut self.security_filter,
            &mut self.transport_filter,
            &mut self.connectivity_filter,
            &mut self.duplicate_filter,
            &mut self.lte_filter,
            &mut self.selection_mode,
            &mut self.compact_page,
            running,
            &self.status,
            &mut self.qr,
            &self.diagnostics,
            &mut self.copied_until,
        );
        if result.open_settings {
            self.show_settings = true;
        }
        if result.open_export {
            self.show_export = true;
        }
        if result.analyze {
            self.start();
        }
        if result.cancel {
            if let Some(job) = &self.job {
                job.cancel.cancel();
            }
        }
        if let Some(id) = result.test {
            if let Some(index) = self
                .report
                .as_ref()
                .and_then(|report| report.configs.iter().position(|config| config.id == id))
            {
                self.start_diagnostic(index);
            }
        }
        if result.copy_all {
            if let Some(report) = &self.report {
                ui::details::copy_to_clipboard(&crate::export::raw_lines(&report.configs));
            }
        }
        if result.copy_selected {
            if let Some(report) = &self.report {
                let mut seen = HashSet::new();
                let configs = report
                    .configs
                    .iter()
                    .filter(|config| self.selected_configs.contains(&config.id))
                    .filter(|config| seen.insert(config.raw_uri.clone()))
                    .cloned()
                    .collect::<Vec<_>>();
                ui::details::copy_to_clipboard(&crate::export::raw_lines(&configs));
            }
        }
    }
}

pub fn configure_style(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(font_bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts.font_data.insert(
            "Segoe UI".to_owned(),
            egui::FontData::from_owned(font_bytes),
        );
        if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            proportional.insert(0, "Segoe UI".to_owned());
        }
        ctx.set_fonts(fonts);
    }
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = theme::CANVAS;
    visuals.panel_fill = theme::CANVAS;
    visuals.window_rounding = egui::Rounding::same(theme::RADIUS_WINDOW);
    visuals.menu_rounding = egui::Rounding::same(theme::RADIUS_INPUT);
    visuals.window_stroke = egui::Stroke::new(1.0_f32, theme::BORDER);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 16, 24);
    visuals.faint_bg_color = theme::SURFACE;
    visuals.override_text_color = Some(theme::TEXT);
    visuals.widgets.noninteractive.bg_fill = theme::SURFACE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, theme::BORDER);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, theme::TEXT_SECONDARY);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(theme::RADIUS_INPUT);
    visuals.widgets.inactive.bg_fill = theme::SURFACE;
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, theme::BORDER);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, theme::TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = egui::Rounding::same(theme::RADIUS_BUTTON);
    visuals.widgets.hovered.bg_fill = theme::SURFACE_HOVER;
    visuals.widgets.hovered.weak_bg_fill = theme::SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, theme::ACCENT_HOVER);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, theme::TEXT_PRIMARY);
    visuals.widgets.hovered.rounding = egui::Rounding::same(theme::RADIUS_BUTTON);
    visuals.widgets.active.bg_fill = theme::ACCENT;
    visuals.widgets.active.weak_bg_fill = theme::ACCENT;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, theme::ACCENT_HOVER);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    visuals.widgets.active.rounding = egui::Rounding::same(theme::RADIUS_BUTTON);
    visuals.widgets.open = visuals.widgets.hovered;
    visuals.selection.bg_fill = theme::ACCENT;
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    visuals.hyperlink_color = theme::ACCENT_HOVER;
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(theme::SPACE_8, theme::SPACE_6);
    style.spacing.button_padding = egui::vec2(theme::SPACE_12, theme::SPACE_6);
    style.spacing.interact_size = egui::vec2(44.0, 34.0);
    style.spacing.menu_margin = egui::Margin::same(theme::SPACE_6);
    style.spacing.indent = 16.0;
    ctx.set_style(style);
}

fn is_json_payload(raw: &str) -> bool {
    matches!(raw.trim_start().chars().next(), Some('{') | Some('['))
}
