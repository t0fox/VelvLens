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
        theme::control_frame(theme::SURFACE).show(ui, |ui| {
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
                ui.separator();
                pipeline_step(
                    ui,
                    &format!(
                        "Декодировано: {}",
                        stage_found(report, |kind| {
                            matches!(kind, crate::resolver::stage::StageKind::Decode)
                        })
                    ),
                    stage_status(report, |kind| {
                        matches!(kind, crate::resolver::stage::StageKind::Decode)
                    }),
                );
                ui.separator();
                pipeline_step(
                    ui,
                    &format!(
                        "Извлечено: {}",
                        stage_found(report, |kind| {
                            matches!(kind, crate::resolver::stage::StageKind::Extract)
                        })
                    ),
                    stage_status(report, |kind| {
                        matches!(kind, crate::resolver::stage::StageKind::Extract)
                    }),
                );
                ui.separator();
                pipeline_step(
                    ui,
                    &format!("Разобрано: {}", report.configs.len()),
                    stage_status(report, |kind| {
                        matches!(kind, crate::resolver::stage::StageKind::Parse)
                    }),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("Уникальных: {}", report.configs.len()))
                            .size(11.0)
                            .color(theme::TEXT_MUTED),
                    );
                });
            });
        });
    } else {
        let message = if running {
            status.to_owned()
        } else {
            "Готово · анализ выполнен локально".to_owned()
        };
        theme::control_frame(theme::SURFACE).show(ui, |ui| {
            ui.horizontal(|ui| {
                status_mark(
                    ui,
                    if running {
                        theme::WARNING
                    } else {
                        theme::SUCCESS
                    },
                    if running {
                        StatusMark::Pending
                    } else {
                        StatusMark::Success
                    },
                );
                ui.label(
                    egui::RichText::new(message)
                        .size(11.0)
                        .color(theme::TEXT_SECONDARY),
                );
            });
        });
    }
}

pub fn compact_pipeline(
    ui: &mut egui::Ui,
    report: Option<&AnalysisReport>,
    running: bool,
    status: &str,
) {
    theme::control_frame(theme::SURFACE).show(ui, |ui| {
        ui.horizontal(|ui| {
            if let Some(report) = report {
                let failed = report
                    .stages
                    .iter()
                    .any(|stage| matches!(stage.status, StageStatus::Failed));
                status_mark(
                    ui,
                    if failed { theme::ERROR } else { theme::SUCCESS },
                    if failed {
                        StatusMark::Failure
                    } else {
                        StatusMark::Success
                    },
                );
                ui.label(
                    egui::RichText::new(if failed {
                        "Анализ завершился с ошибкой"
                    } else {
                        "Анализ завершён"
                    })
                    .size(11.0)
                    .color(if failed { theme::ERROR } else { theme::TEXT }),
                );
                ui.label(
                    egui::RichText::new(format!("· Конфигураций: {}", report.configs.len()))
                        .size(11.0)
                        .color(theme::TEXT_MUTED),
                );
            } else {
                status_mark(
                    ui,
                    if running {
                        theme::WARNING
                    } else {
                        theme::SUCCESS
                    },
                    if running {
                        StatusMark::Pending
                    } else {
                        StatusMark::Success
                    },
                );
                ui.label(
                    egui::RichText::new(if running {
                        if status.is_empty() {
                            "Анализ…"
                        } else {
                            status
                        }
                    } else {
                        "Готово"
                    })
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
                );
            }
        });
    });
}

pub fn show(ui: &mut egui::Ui, report: &AnalysisReport) {
    egui::CollapsingHeader::new("Инспектор обработки")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "Источник обработан локально; содержимое подписки не отправлялось на сервер.",
                )
                .size(11.0)
                .color(theme::TEXT_MUTED),
            );
            ui.add_space(8.0);
            show_stages(ui, report);
        });
}

pub fn show_stages(ui: &mut egui::Ui, report: &AnalysisReport) {
    for stage in &report.stages {
        let (color, mark) = match stage.status {
            StageStatus::Success => (theme::SUCCESS, StatusMark::Success),
            StageStatus::Failed => (theme::ERROR, StatusMark::Failure),
            StageStatus::Running => (theme::WARNING, StatusMark::Pending),
            StageStatus::Skipped => (theme::TEXT_MUTED, StatusMark::Neutral),
            StageStatus::Cancelled => (theme::WARNING, StatusMark::Pending),
        };
        ui.horizontal(|ui| {
            status_mark(ui, color, mark);
            ui.label(
                egui::RichText::new(format_stage(stage.kind))
                    .size(12.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(stage.preview.as_str())
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
            );
        });
        if let Some(error) = &stage.error {
            ui.label(egui::RichText::new(error).size(11.0).color(theme::ERROR));
        }
    }
    if !report.redirect_chain.is_empty() {
        ui.add_space(12.0);
        ui.label(
            egui::RichText::new("Цепочка перенаправлений")
                .size(12.0)
                .strong(),
        );
        for hop in &report.redirect_chain {
            ui.label(
                egui::RichText::new(format!("HTTP {}  {} → {}", hop.status, hop.from, hop.to))
                    .size(11.0)
                    .color(theme::TEXT_MUTED),
            );
        }
    }
}

fn pipeline_step(ui: &mut egui::Ui, label: &str, status: Option<StageStatus>) {
    let (color, mark) = match status {
        Some(StageStatus::Success) => (theme::SUCCESS, StatusMark::Success),
        Some(StageStatus::Failed) => (theme::ERROR, StatusMark::Failure),
        Some(StageStatus::Running) => (theme::WARNING, StatusMark::Pending),
        Some(StageStatus::Cancelled) => (theme::WARNING, StatusMark::Pending),
        Some(StageStatus::Skipped) | None => (theme::TEXT_MUTED, StatusMark::Neutral),
    };
    ui.horizontal(|ui| {
        status_mark(ui, color, mark);
        ui.label(
            egui::RichText::new(label)
                .size(11.0)
                .color(theme::TEXT_MUTED),
        );
    });
}

#[derive(Clone, Copy)]
enum StatusMark {
    Success,
    Failure,
    Pending,
    Neutral,
}

fn status_mark(ui: &mut egui::Ui, color: egui::Color32, mark: StatusMark) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.circle_filled(rect.center(), 5.0, color);
    let ink = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(8, 12, 20));
    match mark {
        StatusMark::Success => {
            painter.line_segment(
                [
                    rect.center() + egui::vec2(-2.5, 0.0),
                    rect.center() + egui::vec2(-0.5, 2.0),
                ],
                ink,
            );
            painter.line_segment(
                [
                    rect.center() + egui::vec2(-0.5, 2.0),
                    rect.center() + egui::vec2(3.0, -2.5),
                ],
                ink,
            );
        }
        StatusMark::Failure => {
            painter.line_segment(
                [
                    rect.center() + egui::vec2(-2.5, -2.5),
                    rect.center() + egui::vec2(2.5, 2.5),
                ],
                ink,
            );
            painter.line_segment(
                [
                    rect.center() + egui::vec2(2.5, -2.5),
                    rect.center() + egui::vec2(-2.5, 2.5),
                ],
                ink,
            );
        }
        StatusMark::Pending => {
            painter.line_segment(
                [
                    rect.center() + egui::vec2(-2.5, 0.0),
                    rect.center() + egui::vec2(2.5, 0.0),
                ],
                ink,
            );
        }
        StatusMark::Neutral => {
            painter.circle_filled(rect.center(), 1.5, ink.color);
        }
    }
}

fn stage_status(
    report: &AnalysisReport,
    matches: impl Fn(crate::resolver::stage::StageKind) -> bool,
) -> Option<StageStatus> {
    let statuses = report
        .stages
        .iter()
        .filter(|stage| matches(stage.kind))
        .map(|stage| stage.status)
        .collect::<Vec<_>>();
    if statuses.contains(&StageStatus::Failed) {
        Some(StageStatus::Failed)
    } else if statuses.contains(&StageStatus::Running) {
        Some(StageStatus::Running)
    } else if statuses.contains(&StageStatus::Cancelled) {
        Some(StageStatus::Cancelled)
    } else if statuses.contains(&StageStatus::Success) {
        Some(StageStatus::Success)
    } else if statuses.contains(&StageStatus::Skipped) {
        Some(StageStatus::Skipped)
    } else {
        None
    }
}

fn stage_found(
    report: &AnalysisReport,
    matches: impl Fn(crate::resolver::stage::StageKind) -> bool,
) -> usize {
    report
        .stages
        .iter()
        .filter(|stage| matches(stage.kind))
        .map(|stage| stage.found)
        .sum()
}

pub(crate) fn format_stage(kind: crate::resolver::stage::StageKind) -> &'static str {
    match kind {
        crate::resolver::stage::StageKind::Http => "HTTP-запрос",
        crate::resolver::stage::StageKind::Redirect => "Перенаправление",
        crate::resolver::stage::StageKind::Detection => "Определение содержимого",
        crate::resolver::stage::StageKind::Decode => "Декодирование Base64",
        crate::resolver::stage::StageKind::Extract => "Извлечение URI",
        crate::resolver::stage::StageKind::Parse => "Разбор протокола",
        crate::resolver::stage::StageKind::Deduplication => "Удаление повторов",
        crate::resolver::stage::StageKind::Diagnostics => "Диагностика",
    }
}
