#![windows_subsystem = "windows"]

mod app;
mod data;
mod lang;
mod settings;
mod storage;
mod toast;
mod updater;

use app::NotaApp;
use eframe::egui;
use std::sync::Arc;

fn load_icon() -> egui::IconData {
    let bytes          = include_bytes!("../assets/icon.png");
    let image          = image::load_from_memory(bytes)
        .expect("Failed to decode icon.png")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData { rgba: image.into_raw(), width, height }
}

fn main() -> eframe::Result<()> {
    // Write panics to a file — critical since windows_subsystem hides the console
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        let path = std::env::temp_dir().join("nota_crash.log");
        let _ = std::fs::write(&path, &msg);
        // On debug builds, also show a message box
        #[cfg(all(debug_assertions, target_os = "windows"))]
        {
            eprintln!("PANIC: {}", msg);
        }
    }));

    let viewport = egui::ViewportBuilder::default()
        .with_title("Nota")
        .with_inner_size([1200.0, 650.0])
        .with_icon(Arc::new(load_icon()));

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Nota",
        options,
        Box::new(|cc| Ok(Box::new(NotaApp::new(cc)))),
    )
}