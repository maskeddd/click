#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, TextWrapMode, Vec2};

fn main() -> eframe::Result {
    let size = Vec2::new(420.0, 274.0);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_maximize_button(false)
            .with_always_on_top()
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size),
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        "Click",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.all_styles_mut(|style| {
                style.wrap_mode = Some(TextWrapMode::Extend);
            });
            cc.egui_ctx.options_mut(|options| {
                options.max_passes = std::num::NonZeroUsize::new(2).unwrap();
            });
            Ok(Box::new(click::ClickApp::new(cc)))
        }),
    )
}
