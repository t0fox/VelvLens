fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter("sublens=info")
        .init();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([980.0, 640.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "SubLens",
        options,
        Box::new(|cc| Ok(Box::new(sublens::app::SubLensApp::new(cc)))),
    )
}
