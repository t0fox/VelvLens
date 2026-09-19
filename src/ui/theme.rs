use eframe::egui;

pub const CANVAS: egui::Color32 = egui::Color32::from_rgb(11, 17, 28);
pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(18, 26, 39);
pub const SURFACE_RAISED: egui::Color32 = egui::Color32::from_rgb(24, 34, 51);
pub const SURFACE_SELECTED: egui::Color32 = egui::Color32::from_rgb(35, 24, 70);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(39, 54, 75);
pub const BORDER_SELECTED: egui::Color32 = egui::Color32::from_rgb(124, 58, 237);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(244, 247, 251);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(163, 176, 196);
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(124, 58, 237);
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(53, 208, 127);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(231, 173, 60);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(241, 107, 107);

pub fn surface_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::same(16.0))
}

pub fn control_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(10.0, 7.0))
}

pub fn badge(ui: &mut egui::Ui, text: &str, fill: egui::Color32, color: egui::Color32) {
    egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.0).strong().color(color));
        });
}

pub fn draw_logo(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, egui::Rounding::same(size * 0.27), ACCENT);
    let stroke = egui::Stroke::new(size * 0.061, egui::Color32::WHITE);
    let center = rect.center();
    let offset = size * 0.125;
    let radius = size * 0.139;
    painter.circle_stroke(center - egui::vec2(offset, size * 0.097), radius, stroke);
    painter.circle_stroke(center + egui::vec2(offset, size * 0.097), radius, stroke);
    painter.line_segment(
        [
            center - egui::vec2(size * 0.042, size * 0.028),
            center + egui::vec2(size * 0.042, size * 0.028),
        ],
        stroke,
    );
}

pub fn draw_network_backdrop(ui: &egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    let offset = egui::pos2(rect.right() - 180.0, rect.top() + 38.0);
    let nodes = [
        egui::vec2(0.0, 0.0),
        egui::vec2(145.0, 42.0),
        egui::vec2(225.0, 156.0),
        egui::vec2(55.0, 210.0),
        egui::vec2(285.0, 280.0),
    ];
    let line = egui::Stroke::new(
        1.0_f32,
        egui::Color32::from_rgba_unmultiplied(95, 116, 156, 32),
    );
    for pair in nodes.windows(2) {
        painter.line_segment([offset + pair[0], offset + pair[1]], line);
    }
    painter.line_segment([offset + nodes[0], offset + nodes[3]], line);
    painter.line_segment([offset + nodes[2], offset + nodes[4]], line);
    for (index, node) in nodes.iter().enumerate() {
        let color = if index == 0 || index == 3 {
            egui::Color32::from_rgba_unmultiplied(236, 72, 153, 70)
        } else {
            egui::Color32::from_rgba_unmultiplied(96, 165, 250, 60)
        };
        painter.circle_filled(offset + *node, 4.0, color);
        painter.circle_stroke(offset + *node, 10.0, egui::Stroke::new(1.0_f32, color));
    }
}
