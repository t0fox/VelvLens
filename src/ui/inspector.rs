use eframe::egui;

use crate::resolver::{stage::StageStatus, AnalysisReport};

pub fn show(ui: &mut egui::Ui, report: &AnalysisReport) {
    ui.heading("Visual Decode Inspector");
    ui.add_space(6.0);
    if !report.source.is_empty() {
        ui.label(egui::RichText::new("Source").strong());
        let mut source = report.source.clone();
        ui.add(
            egui::TextEdit::singleline(&mut source)
                .interactive(false)
                .desired_width(f32::INFINITY),
        );
        ui.add_space(4.0);
    }
    for stage in &report.stages {
        let icon = match stage.status {
            StageStatus::Success => "✓",
            StageStatus::Failed => "!",
            StageStatus::Running => "…",
            StageStatus::Skipped => "–",
            StageStatus::Cancelled => "×",
        };
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).strong().color(
                if stage.status == StageStatus::Success {
                    egui::Color32::from_rgb(93, 211, 142)
                } else {
                    egui::Color32::YELLOW
                },
            ));
            ui.label(format!("{:?}", stage.kind));
            ui.label(stage.preview.as_str());
        });
        if let Some(error) = &stage.error {
            ui.label(
                egui::RichText::new(error)
                    .small()
                    .color(egui::Color32::from_rgb(255, 128, 128)),
            );
        }
    }
    if !report.redirect_chain.is_empty() {
        ui.separator();
        ui.label(egui::RichText::new("Redirect chain").strong());
        for hop in &report.redirect_chain {
            ui.label(format!("HTTP {}  {} → {}", hop.status, hop.from, hop.to));
        }
    }
}
