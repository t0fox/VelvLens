use eframe::egui;

use crate::{model::ProxyConfig, qr::QrMatrix, security::redact_uri};

pub fn show(
    ui: &mut egui::Ui,
    config: &ProxyConfig,
    show_sensitive: bool,
    qr: &mut Option<QrMatrix>,
) {
    ui.heading("Configuration Inspector");
    ui.add_space(8.0);
    field(ui, "Protocol", config.protocol.as_str());
    field(ui, "Address", &config.host);
    field(ui, "Port", &config.port.to_string());
    field(ui, "Security", config.security.as_str());
    field(ui, "Transport", config.transport.as_str());
    field(ui, "SNI", config.sni.as_deref().unwrap_or("—"));
    field(
        ui,
        "Fingerprint",
        config.fingerprint.as_deref().unwrap_or("—"),
    );
    field(
        ui,
        "UUID",
        if show_sensitive {
            config.uuid.as_deref().unwrap_or("—")
        } else {
            "••••••"
        },
    );
    field(
        ui,
        "Password",
        if show_sensitive {
            config.password.as_deref().unwrap_or("—")
        } else {
            "••••••"
        },
    );
    ui.separator();
    ui.label(egui::RichText::new("Raw URI").strong());
    let mut sanitized_uri = redact_uri(&config.raw_uri);
    ui.add(
        egui::TextEdit::multiline(&mut sanitized_uri)
            .desired_rows(2)
            .interactive(false),
    );
    ui.horizontal(|ui| {
        if ui.button("Copy URI").clicked() {
            copy_to_clipboard(&config.raw_uri);
        }
        if ui.button("Copy sanitized URI").clicked() {
            copy_to_clipboard(&redact_uri(&config.raw_uri));
        }
        if ui.button("QR Code").clicked() {
            *qr = Some(crate::qr::encode(&config.raw_uri));
        }
    });
}

fn field(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{label}: ")).strong());
        ui.label(value);
    });
}

pub fn copy_to_clipboard(value: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(value.to_owned());
    }
}
