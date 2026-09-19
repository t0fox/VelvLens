use eframe::egui;

// The product palette is intentionally small. Keeping every token here makes
// the compact and wide layouts feel like the same application.
pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(17, 23, 33);
pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(27, 35, 48);
pub const SURFACE_HOVER: egui::Color32 = egui::Color32::from_rgb(37, 46, 64);
pub const SURFACE_SELECTED: egui::Color32 = egui::Color32::from_rgb(41, 33, 63);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(51, 61, 80);
pub const BORDER_SELECTED: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(243, 244, 248);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(170, 181, 200);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(135, 147, 167);
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(124, 58, 237);
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(144, 97, 249);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(74, 222, 128);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(245, 185, 66);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(248, 113, 113);

// Compatibility aliases keep the rendering code readable while all values
// still come from the same token set.
pub const CANVAS: egui::Color32 = BACKGROUND;
pub const SURFACE_RAISED: egui::Color32 = SURFACE_HOVER;
pub const TEXT: egui::Color32 = TEXT_PRIMARY;
pub const MUTED: egui::Color32 = TEXT_SECONDARY;

pub const RADIUS_WINDOW: f32 = 12.0;
pub const RADIUS_PANEL: f32 = 12.0;
pub const RADIUS_CARD: f32 = 12.0;
pub const RADIUS_INPUT: f32 = 10.0;
pub const RADIUS_BUTTON: f32 = 10.0;
pub const RADIUS_BADGE: f32 = 6.0;

pub const SPACE_2: f32 = 2.0;
pub const SPACE_4: f32 = 4.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_24: f32 = 24.0;

// Two-column mode needs roughly 300 px for the catalog, 620 px for readable
// details, and the surrounding gutters. Below this layout budget, the list
// and inspector become separate pages instead of competing for width.
pub const WIDE_BREAKPOINT: f32 = 1000.0;

pub fn surface_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(RADIUS_PANEL))
        .inner_margin(egui::Margin::same(SPACE_16))
}

pub fn control_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0_f32, BORDER))
        .rounding(egui::Rounding::same(RADIUS_INPUT))
        .inner_margin(egui::Margin::symmetric(SPACE_12, SPACE_6))
}

pub fn badge(ui: &mut egui::Ui, text: &str, fill: egui::Color32, color: egui::Color32) {
    egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(RADIUS_BADGE))
        .inner_margin(egui::Margin::symmetric(SPACE_8, SPACE_4))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.0).color(color));
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
