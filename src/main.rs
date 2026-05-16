#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod crawler;
mod tree;
mod squarify;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DiskMapper")
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DiskMapper",
        options,
        Box::new(|_cc| Ok(Box::new(app::DiskMapperApp::new()))),
    )
}
