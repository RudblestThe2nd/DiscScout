mod crawler;
mod tree;
mod squarify;
mod app;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DiskScout")
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DiskScout",
        options,
        Box::new(|_cc| Ok(Box::new(app::DiskScoutApp::new()))),
    )
}
