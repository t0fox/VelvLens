#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter("sublens=info")
        .init();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([520.0, 450.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "SubLens",
        options,
        Box::new(|cc| Ok(Box::new(sublens::app::SubLensApp::new(cc)))),
    )
}
