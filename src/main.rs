mod app;
mod assets;
mod config;
mod rfid_reader;
mod rom_preview;
mod ui;

use crate::app::BarelyGameConsole;
use crate::config::{CardInfo, Config};
use crate::rfid_reader::RFIDReader;
use eframe::egui;
use evdev::{Device, InputEventKind, Key};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
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
        device_listener(config.clone(), Arc::clone(&ui_app));

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

// Overloaded to listen to rfid AND power
fn device_listener(config: Config, ui_app: Arc<Mutex<BarelyGameConsole>>) {
    let timer_version = Arc::new(AtomicUsize::new(0));
    let selected_rom: Arc<Mutex<Option<CardInfo>>> = Arc::new(Mutex::new(None));
    let game_pid: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));

    thread::spawn({
        let game_pid = Arc::clone(&game_pid);
        let selected_rom = Arc::clone(&selected_rom);
        let timer_version = Arc::clone(&timer_version);
        let ui_app = Arc::clone(&ui_app);
        move || {
            let device_path = "/dev/input/event2";
            let mut device = Device::open(device_path).expect("Failed to open the button");
            let _ = device.grab();

            loop {
                for event in device.fetch_events().expect("could not fetch event") {
                    if event.value() == 0 && event.kind() == InputEventKind::Key(Key::KEY_POWER) {
                        if let Ok(pid) = game_pid.lock() {
                            if let Some(pid) = *pid {
                                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
                            } else {
                                if let Ok(mut selected_rom) = selected_rom.lock() {
                                    let rom = (&*selected_rom).clone();
                                    if let Some(rom) = rom {
                                        *selected_rom = None;
                                        timer_version.fetch_add(1, Ordering::SeqCst);
                                        if let Ok(mut app) = ui_app.lock() {
                                            app.enqueue_rom(None);
                                        }

                                        thread::spawn({
                                            let game_pid = Arc::clone(&game_pid);
                                            move || {
                                                if let Ok(mut child) = Command::new("retroarch")
                                                    .arg("-L")
                                                    .arg(rom.emulator)
                                                    .arg(rom.rom_path)
                                                    .arg("--appendconfig")
                                                    .arg("retroarch.cfg")
                                                    .spawn()
                                                {
                                                    if let Ok(mut pid) = game_pid.lock() {
                                                        *pid = Some(child.id());
                                                    }
                                                    let _ = child.wait();
                                                    if let Ok(mut pid) = game_pid.lock() {
                                                        *pid = None;
                                                    }
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    thread::spawn({
        let game_pid = Arc::clone(&game_pid);
        let selected_rom = Arc::clone(&selected_rom);
        let timer_version = Arc::clone(&timer_version);
        let ui_app = Arc::clone(&ui_app);
        move || {
            let reader = RFIDReader::new();
            reader.run(move |id| {
                if let Ok(pid) = game_pid.lock() {
                    if (*pid).is_none() {
                        if let Some(rom) = config.rfid_cards.get(&id) {
                            if let Ok(mut sel_rom) = selected_rom.lock() {
                                *sel_rom = Some(rom.clone());

                                let current_version =
                                    timer_version.fetch_add(1, Ordering::SeqCst) + 1;
                                if let Ok(mut app) = ui_app.lock() {
                                    app.enqueue_rom(Some(rom.artwork.clone()));
                                }

                                let selected_rom = Arc::clone(&selected_rom);
                                let timer_version = Arc::clone(&timer_version);
                                let ui_app = Arc::clone(&ui_app);
                                thread::spawn(move || {
                                    thread::sleep(Duration::from_secs(5));
                                    if timer_version.load(Ordering::SeqCst) == current_version {
                                        if let Ok(mut selected_rom) = selected_rom.lock() {
                                            *selected_rom = None;
                                        }
                                        if let Ok(mut app) = ui_app.lock() {
                                            app.enqueue_rom(None);
                                        }
                                    }
                                });
                            }
                        } else {
                            println!("Unknown id: {:?}", id);
                        }
                    }
                }
            });
        }
    });
}
