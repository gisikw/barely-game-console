mod app;
mod assets;
mod config;
mod rfid_reader;
mod rom_preview;
mod ui;

use crate::app::BarelyGameConsole;
use crate::config::Config;
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
        let config = Config::load();

        let ui_app = Arc::new(Mutex::new(ui_app));
        automated_test(config.clone(), Arc::clone(&ui_app));
        rfid_listener(config.clone(), Arc::clone(&ui_app));

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

fn automated_test(config: Config, ui_app: Arc<Mutex<BarelyGameConsole>>) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        if let Ok(mut app) = ui_app.lock() {
            app.enqueue_rom(Some(
                config.rfid_cards.get("1234567890").unwrap().artwork.clone(),
            ));
        }

        thread::sleep(Duration::from_secs(5));
        if let Ok(mut app) = ui_app.lock() {
            app.enqueue_rom(Some(
                config.rfid_cards.get("0987654321").unwrap().artwork.clone(),
            ));
        }

        thread::sleep(Duration::from_secs(5));
        if let Ok(mut app) = ui_app.lock() {
            app.enqueue_rom(None);
        }

        thread::sleep(Duration::from_secs(5));
        if let Ok(mut app) = ui_app.lock() {
            app.enqueue_rom(None);
        }

        thread::sleep(Duration::from_secs(3));
        if let Ok(mut app) = ui_app.lock() {
            app.enqueue_rom(Some(
                config.rfid_cards.get("1234567890").unwrap().artwork.clone(),
            ));
        }
    });
}

fn rfid_listener(config: Config, ui_app: Arc<Mutex<BarelyGameConsole>>) {
    thread::spawn(move || {
        let reader = RFIDReader::new();
        reader.run(|id| {
            if let Ok(mut app) = ui_app.lock() {
                app.enqueue_rom(Some(id));
            }
            thread::sleep(Duration::from_secs(5));
            if let Ok(mut app) = ui_app.lock() {
                app.enqueue_rom(None);
            }
        });
    });
}
