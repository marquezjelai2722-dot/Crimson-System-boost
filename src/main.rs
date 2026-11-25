#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 620.0])
            .with_title("Crimson System Boost"),
        ..Default::default()
    };

    eframe::run_native(
        "Crimson System Boost",
        options,
        Box::new(|_cc| Ok(Box::new(app::CrimsonApp::default()))),
    )
}
