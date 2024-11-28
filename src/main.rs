mod app;
mod assets;
mod rfid_reader;
mod rom_preview;
mod ui;

use crate::app::BarelyGameConsole;
use crate::rfid_reader::RFIDReader;
use eframe::egui;
use serde::Deserialize;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use toml;

#[derive(Deserialize, Debug)]
struct Config {
    rfid_cards: std::collections::HashMap<String, CardInfo>,
}

#[derive(Deserialize, Debug)]
struct CardInfo {
    rom_path: String,
    emulator: String,
    artwork: String,
}

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
        let config = fs::read_to_string("config.toml").expect("Failed to read config.toml");
        let config: Config = toml::from_str(&config).expect("Failed to parse config.toml");
        println!("{:?}", config);

        let ui_app = Arc::new(Mutex::new(ui_app));

        let ui_app_clone = Arc::clone(&ui_app);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            if let Ok(mut app) = ui_app_clone.lock() {
                app.enqueue_rom(Some(
                    config.rfid_cards.get("1234567890").unwrap().artwork.clone(),
                ));
            }

            thread::sleep(Duration::from_secs(5));
            if let Ok(mut app) = ui_app_clone.lock() {
                app.enqueue_rom(Some(
                    config.rfid_cards.get("0987654321").unwrap().artwork.clone(),
                ));
            }

            thread::sleep(Duration::from_secs(5));
            if let Ok(mut app) = ui_app_clone.lock() {
                app.enqueue_rom(None);
            }

            thread::sleep(Duration::from_secs(5));
            if let Ok(mut app) = ui_app_clone.lock() {
                app.enqueue_rom(None);
            }

            thread::sleep(Duration::from_secs(3));
            if let Ok(mut app) = ui_app_clone.lock() {
                app.enqueue_rom(Some(
                    config.rfid_cards.get("1234567890").unwrap().artwork.clone(),
                ));
            }

            // TODO
            //if let Ok(mut app) = ui_app_clone.lock() {
            //    app.dismiss()
            //}

            //let reader = RFIDReader::new();
            //reader.run(|id| {
            //    if let Ok(mut app) = ui_app_clone.lock() {
            //        app.enqueue_rom(Some(id));
            //    }
            //    thread::sleep(Duration::from_secs(5));
            //    if let Ok(mut app) = ui_app_clone.lock() {
            //        app.enqueue_rom(None);
            //    }
            //});
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
