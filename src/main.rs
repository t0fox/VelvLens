fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter("sublens=info")
        .init();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SubLens",
        options,
        Box::new(|cc| Ok(Box::new(sublens::app::SubLensApp::new(cc)))),
    )
}
