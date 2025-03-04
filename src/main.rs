mod app;
mod file_scanner;
mod file_utils;
mod preview;
mod tests;

use app::DuplicateFinderApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "DupFi - Duplicate File Finder",
        options,
        Box::new(|cc| Box::new(DuplicateFinderApp::new(cc)))
    )
}
