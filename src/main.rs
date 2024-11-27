mod animations;
mod app;
mod assets;
mod rfid_reader;
mod ui;

use crate::app::BarelyGameConsole;
use crate::rfid_reader::RFIDReader;
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
        thread::spawn(move || {
            let reader = RFIDReader::new();
            reader.run(|id| {
                if let Ok(mut app) = ui_app_clone.lock() {
                    app.enqueue_rom(Some(id));
                }
                thread::sleep(Duration::from_secs(5));
                if let Ok(mut app) = ui_app_clone.lock() {
                    app.enqueue_rom(None);
                }
            });
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
