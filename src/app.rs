use std::time::Duration;

use crate::{
    history::HistoryStore,
    jobs::{JobEvent, JobHandle, JobManager},
    model::Protocol,
    resolver::AnalysisReport,
    settings::AppSettings,
    ui,
};
use eframe::egui;

pub struct SubLensApp {
    input: String,
    status: String,
    job_manager: JobManager,
    job: Option<JobHandle>,
    report: Option<AnalysisReport>,
    selected: Option<usize>,
    filter: Option<Protocol>,
    search: String,
    settings: AppSettings,
    history: HistoryStore,
    show_settings: bool,
    qr: Option<crate::qr::QrMatrix>,
}

impl SubLensApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);
        Self {
            input: String::new(),
            status: "Ready · all analysis stays local".to_owned(),
            job_manager: JobManager::new().expect("Tokio runtime must start"),
            job: None,
            report: None,
            selected: None,
            filter: None,
            search: String::new(),
            settings: AppSettings::default(),
            history: HistoryStore::load(false),
            show_settings: false,
            qr: None,
        }
    }

    fn start(&mut self) {
        let url = self.input.trim().to_owned();
        if url.is_empty() {
            self.status = "Enter an HTTP(S) subscription URL".to_owned();
            return;
        }
        self.report = None;
        self.selected = None;
        if self.settings.history_enabled {
            self.history = HistoryStore::load(true);
            self.history.append(&url);
        }
        self.status = "Analyzing · HTTP → Decode → Extract → Parse".to_owned();
        let mut resolver = self.settings.resolver.clone();
        resolver.request_timeout = Duration::from_secs(self.settings.timeout_seconds);
        self.job = Some(self.job_manager.start_analysis(url, resolver));
    }

    fn poll_job(&mut self, ctx: &egui::Context) {
        let Some(job) = &self.job else { return };
        let mut finished = false;
        while let Ok(event) = job.receiver.try_recv() {
            match event {
                JobEvent::Started => self.status = "Fetching source…".to_owned(),
                JobEvent::Completed(report) => {
                    self.status = format!(
                        "✓ HTTP → ✓ Decode → ✓ {} configurations found",
                        report.configs.len()
                    );
                    self.report = Some(report);
                    finished = true;
                }
                JobEvent::Failed(error) => {
                    self.status = format!("Analysis failed: {error}");
                    finished = true;
                }
                JobEvent::Cancelled => {
                    self.status = "Analysis cancelled".to_owned();
                    finished = true;
                }
            }
        }
        if finished {
            self.job = None;
        }
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

impl eframe::App for SubLensApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_job(ctx);
        let running = self.job.is_some();
        if self.show_settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if ui::settings::show(ui, &mut self.settings) {
                        self.show_settings = false;
                    }
                });
        }
        let result = ui::main_view::show(
            ctx,
            &mut self.input,
            self.history.urls(),
            self.report.as_ref(),
            &mut self.selected,
            &mut self.filter,
            &mut self.search,
            running,
            &self.status,
            self.settings.show_sensitive_session,
            &mut self.qr,
        );
        if result.open_settings {
            self.show_settings = true;
        }
        if result.analyze {
            self.start();
        }
        if result.cancel {
            if let Some(job) = &self.job {
                job.cancel.cancel();
            }
        }
        if result.copy_all {
            if let Some(report) = &self.report {
                ui::details::copy_to_clipboard(&crate::export::v2rayn_bulk(&report.configs));
            }
        }
    }
}

fn configure_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(18, 23, 30);
    visuals.panel_fill = egui::Color32::from_rgb(15, 20, 27);
    visuals.extreme_bg_color = egui::Color32::from_rgb(10, 14, 19);
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    ctx.set_style(style);
}
