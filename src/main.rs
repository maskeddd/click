use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([1.0, 1.0]),
        persist_window: false,
        ..Default::default()
    };
    eframe::run_native(
        "Click",
        native_options,
        Box::new(|cc| Ok(Box::new(click::ClickApp::new(cc)))),
    )
}
