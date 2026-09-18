use eframe::egui;

use crate::resolver::{stage::StageStatus, AnalysisReport};

use super::theme;

pub fn pipeline_strip(
    ui: &mut egui::Ui,
    report: Option<&AnalysisReport>,
    running: bool,
    status: &str,
) {
    if let Some(report) = report {
        ui.horizontal(|ui| {
            pipeline_step(
                ui,
                "HTTP",
                stage_status(report, |kind| {
                    matches!(
                        kind,
                        crate::resolver::stage::StageKind::Http
                            | crate::resolver::stage::StageKind::Redirect
                    )
                }),
            );
            pipeline_step(
                ui,
                "Decode",
                stage_status(report, |kind| {
                    matches!(kind, crate::resolver::stage::StageKind::Decode)
                }),
            );
            pipeline_step(
                ui,
                "Extracted",
                stage_status(report, |kind| {
                    matches!(
                        kind,
                        crate::resolver::stage::StageKind::Extract
                            | crate::resolver::stage::StageKind::Parse
                    )
                }),
            );
            pipeline_step(
                ui,
                "Deduplicated",
                stage_status(report, |kind| {
                    matches!(kind, crate::resolver::stage::StageKind::Deduplication)
                }),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{} configurations", report.configs.len()))
                        .size(11.0)
                        .color(theme::MUTED),
                );
            });
        });
    } else {
        let message = if running {
            status.to_owned()
        } else {
            "Ready · all analysis stays local".to_owned()
        };
        theme::control_frame(theme::SURFACE).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").color(if running {
                    theme::WARNING
                } else {
                    theme::SUCCESS
                }));
                ui.label(egui::RichText::new(message).size(11.0).color(theme::MUTED));
            });
        });
    }
}

pub fn show(ui: &mut egui::Ui, report: &AnalysisReport) {
    theme::surface_frame(theme::SURFACE).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Analysis details").size(18.0).strong());
            theme::badge(ui, "LOCAL", theme::SURFACE_RAISED, theme::MUTED);
        });
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "The source was processed without sending its contents to a backend.",
            )
            .size(12.0)
            .color(theme::MUTED),
        );
        ui.add_space(16.0);
        show_stages(ui, report);
    });
}

pub fn show_stages(ui: &mut egui::Ui, report: &AnalysisReport) {
    for stage in &report.stages {
        let (icon, color) = match stage.status {
            StageStatus::Success => ("✓", theme::SUCCESS),
            StageStatus::Failed => ("!", theme::ERROR),
            StageStatus::Running => ("…", theme::WARNING),
            StageStatus::Skipped => ("–", theme::MUTED),
            StageStatus::Cancelled => ("×", theme::WARNING),
        };
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).strong().color(color));
            ui.label(
                egui::RichText::new(format_stage(stage.kind))
                    .size(12.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(stage.preview.as_str())
                    .size(11.0)
                    .color(theme::MUTED),
            );
        });
        if let Some(error) = &stage.error {
            ui.label(egui::RichText::new(error).size(11.0).color(theme::ERROR));
        }
    }
    if !report.redirect_chain.is_empty() {
        ui.add_space(12.0);
        ui.label(egui::RichText::new("Redirect chain").size(12.0).strong());
        for hop in &report.redirect_chain {
            ui.label(
                egui::RichText::new(format!("HTTP {}  {} → {}", hop.status, hop.from, hop.to))
                    .size(11.0)
                    .color(theme::MUTED),
            );
        }
    }
}

fn pipeline_step(ui: &mut egui::Ui, label: &str, status: Option<StageStatus>) {
    let (icon, color) = match status {
        Some(StageStatus::Success) => ("✓", theme::SUCCESS),
        Some(StageStatus::Failed) => ("!", theme::ERROR),
        Some(StageStatus::Running) => ("…", theme::WARNING),
        Some(StageStatus::Cancelled) => ("×", theme::WARNING),
        Some(StageStatus::Skipped) | None => ("–", theme::MUTED),
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(icon).strong().color(color));
        ui.label(egui::RichText::new(label).size(11.0).color(theme::MUTED));
    });
}

fn stage_status(
    report: &AnalysisReport,
    matches: impl Fn(crate::resolver::stage::StageKind) -> bool,
) -> Option<StageStatus> {
    report
        .stages
        .iter()
        .filter(|stage| matches(stage.kind))
        .map(|stage| stage.status)
        .last()
}

fn format_stage(kind: crate::resolver::stage::StageKind) -> &'static str {
    match kind {
        crate::resolver::stage::StageKind::Http => "HTTP fetch",
        crate::resolver::stage::StageKind::Redirect => "Redirect",
        crate::resolver::stage::StageKind::Detection => "Content detection",
        crate::resolver::stage::StageKind::Decode => "Base64 decode",
        crate::resolver::stage::StageKind::Extract => "URI extraction",
        crate::resolver::stage::StageKind::Parse => "Protocol parse",
        crate::resolver::stage::StageKind::Deduplication => "Deduplication",
        crate::resolver::stage::StageKind::Diagnostics => "Diagnostics",
    }
}
