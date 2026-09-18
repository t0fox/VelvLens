use qrcode::QrCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrMatrix {
    width: usize,
    cells: Vec<bool>,
}

impl QrMatrix {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn modules(&self) -> &[bool] {
        &self.cells
    }

    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        self.cells[y * self.width + x]
    }
}

pub fn encode(uri: &str) -> QrMatrix {
    let code = QrCode::new(uri.as_bytes()).expect("QR input is within qrcode capacity");
    let width = code.width();
    let code = &code;
    let cells = (0..width)
        .flat_map(|y| (0..width).map(move |x| code[(x, y)] == qrcode::types::Color::Dark))
        .collect();
    QrMatrix { width, cells }
}
