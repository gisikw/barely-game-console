mod animations;
mod app;
mod assets;
mod ui;

use crate::app::BarelyGameConsole;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Barely Game Console",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(Coordinator::new(BarelyGameConsole::new(cc))))),
    )
}

struct Coordinator {
    ui_app: Arc<Mutex<BarelyGameConsole>>,
}

impl Coordinator {
    fn new(ui_app: BarelyGameConsole) -> Self {
        let ui_app = Arc::new(Mutex::new(ui_app));

        let ui_app_clone = Arc::clone(&ui_app);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(5));
            if let Ok(mut app) = ui_app_clone.lock() {
                app.enqueue_rom(Some("Super Mario World".to_string()));
            }
        });

        Self { ui_app }
    }
}

impl eframe::App for Coordinator {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(mut app) = self.ui_app.lock() {
            app.update(ctx, frame);
        }
    }
}
