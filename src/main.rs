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
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Shared state between the main loop, device listener threads, and the UI.
///
/// The main loop alternates between running eframe (the launcher UI) and running
/// a game process. When a game launches, eframe exits — destroying its Wayland
/// surface — so the game is the sole Wayland client under Cage. This eliminates
/// the multi-surface frame callback deadlock.
struct SharedState {
    /// The current UI app instance (None when eframe isn't running).
    ui_app: Mutex<Option<BarelyGameConsole>>,
    /// Set by the power button thread to signal a game launch.
    /// The main loop reads this after eframe exits.
    pending_launch: Mutex<Option<CardInfo>>,
    /// The currently-previewed ROM (set by RFID, consumed by power button).
    selected_rom: Mutex<Option<CardInfo>>,
    /// PID of the running game process (set by main loop, read by power button for kill).
    game_pid: Mutex<Option<u32>>,
    /// Timer version for debouncing card preview timeouts.
    timer_version: AtomicUsize,
}

impl SharedState {
    fn new() -> Self {
        Self {
            ui_app: Mutex::new(None),
            pending_launch: Mutex::new(None),
            selected_rom: Mutex::new(None),
            game_pid: Mutex::new(None),
            timer_version: AtomicUsize::new(0),
        }
    }

    fn enqueue_rom(&self, rom: Option<String>) {
        if let Ok(mut app) = self.ui_app.lock() {
            if let Some(app) = app.as_mut() {
                app.enqueue_rom(rom);
            }
        }
    }
}

/// The eframe App wrapper. Delegates rendering to BarelyGameConsole
/// and closes the window when a game launch is pending.
struct Launcher {
    shared: Arc<SharedState>,
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // If a game launch is pending, close the window to destroy the Wayland surface
        if let Ok(pending) = self.shared.pending_launch.lock() {
            if pending.is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        if let Ok(mut app) = self.shared.ui_app.lock() {
            if let Some(app) = app.as_mut() {
                app.update(ctx);
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let config = Config::load();
    eprintln!(
        "barely-game-console started, {} cards configured",
        config.rfid_cards.len()
    );

    let shared = Arc::new(SharedState::new());

    // Start device listeners once — they persist across eframe restarts
    device_listener(config, Arc::clone(&shared));

    loop {
        // Clear stale textures from the previous eframe instance
        assets::clear_texture_cache();

        eframe::run_native(
            "Barely Game Console",
            eframe::NativeOptions::default(),
            Box::new({
                let shared = Arc::clone(&shared);
                move |cc| {
                    let app = BarelyGameConsole::new(cc);
                    *shared.ui_app.lock().unwrap() = Some(app);
                    Ok(Box::new(Launcher {
                        shared: Arc::clone(&shared),
                    }))
                }
            }),
        )?;

        // eframe exited — clear the stale UI app reference
        *shared.ui_app.lock().unwrap() = None;

        // Check if there's a game to launch
        let game = shared.pending_launch.lock().unwrap().take();
        if let Some(card) = game {
            run_game(&card, &shared);
            // Game exited — loop back to restart eframe
        } else {
            // Window closed without a game launch (shouldn't happen in kiosk mode)
            break;
        }
    }

    Ok(())
}

fn run_game(card: &CardInfo, shared: &SharedState) {
    let (cmd_desc, mut cmd) = if let Some(command) = &card.command {
        let desc = command.join(" ");
        let mut iter = command.iter();
        let first = iter.next().expect("Empty command list");
        let mut cmd = Command::new(first);
        cmd.args(iter);
        (desc, cmd)
    } else {
        let emulator = card.emulator.as_ref().expect("Missing emulator");
        let rom_path = card.rom_path.as_ref().expect("Missing rom path");
        let desc = format!("retroarch -L {} {}", emulator, rom_path);
        let mut cmd = Command::new("retroarch");
        cmd.arg("-L").arg(emulator).arg(rom_path);
        let config_path = std::env::var("BGC_RETROARCH_CONFIG")
            .unwrap_or_else(|_| "retroarch.cfg".to_string());
        cmd.arg("--appendconfig").arg(&config_path);
        (desc, cmd)
    };

    if let Some(dir) = &card.working_dir {
        cmd.current_dir(dir);
    }

    eprintln!("[launch] {}", cmd_desc);
    cmd.stdin(Stdio::null());

    match cmd.spawn() {
        Ok(mut child) => {
            let child_pid = child.id();
            eprintln!("[launch] spawned pid={}", child_pid);
            if let Ok(mut pid) = shared.game_pid.lock() {
                *pid = Some(child_pid);
            }
            let started = Instant::now();
            let status = child.wait();
            let elapsed = started.elapsed();
            match status {
                Ok(s) => eprintln!(
                    "[exit] pid={} status={} after {:.0?}",
                    child_pid, s, elapsed
                ),
                Err(e) => eprintln!(
                    "[exit] pid={} wait error: {} after {:.0?}",
                    child_pid, e, elapsed
                ),
            }
            if let Ok(mut pid) = shared.game_pid.lock() {
                *pid = None;
            }
        }
        Err(e) => {
            eprintln!("[launch] failed to spawn: {}", e);
        }
    }
}

fn device_listener(config: Config, shared: Arc<SharedState>) {
    // Power button listener
    thread::spawn({
        let shared = Arc::clone(&shared);
        move || loop {
            let device_path = match rfid_reader::find_device_path_by_name("Power Button") {
                Some(path) => path,
                None => {
                    eprintln!("Power Button device not found, retrying...");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            let mut device = match Device::open(&device_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to open Power Button: {}, retrying...", e);
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            let _ = device.grab();
            eprintln!("Power button ready on {}", device_path);

            'read: loop {
                let events = match device.fetch_events() {
                    Ok(events) => events,
                    Err(e) => {
                        eprintln!("Power button error: {}, re-opening device...", e);
                        break 'read;
                    }
                };
                for event in events {
                    if event.value() == 0 && event.kind() == InputEventKind::Key(Key::KEY_POWER) {
                        // Snapshot game_pid with a brief lock, then release
                        let game_pid_val = shared.game_pid.lock().ok().and_then(|p| *p);

                        if let Some(pid) = game_pid_val {
                            // Game is running — kill it
                            eprintln!("[power] killing game pid={}", pid);
                            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGKILL);
                        } else {
                            // No game running — launch if a ROM is selected
                            let rom = shared.selected_rom.lock().ok().and_then(|mut s| s.take());
                            if let Some(rom) = rom {
                                shared.timer_version.fetch_add(1, Ordering::SeqCst);
                                // Signal the main loop: store the card and trigger eframe close
                                *shared.pending_launch.lock().unwrap() = Some(rom);
                                shared.enqueue_rom(None);
                            }
                        }
                    }
                }
            }
        }
    });

    // RFID listener
    thread::spawn({
        let shared = Arc::clone(&shared);
        move || {
            let reader = RFIDReader::new();
            reader.run(move |id| {
                let game_running = shared
                    .game_pid
                    .lock()
                    .ok()
                    .map_or(false, |p| p.is_some());

                if !game_running {
                    if let Some(rom) = config.rfid_cards.get(&id) {
                        eprintln!("[rfid] card={} artwork={}", id, rom.artwork);
                        let current_version = {
                            let mut sel_rom = shared.selected_rom.lock().unwrap();
                            *sel_rom = Some(rom.clone());
                            shared.timer_version.fetch_add(1, Ordering::SeqCst) + 1
                        };
                        shared.enqueue_rom(Some(rom.artwork.clone()));

                        let shared = Arc::clone(&shared);
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(5));
                            if shared.timer_version.load(Ordering::SeqCst) == current_version {
                                *shared.selected_rom.lock().unwrap() = None;
                                shared.enqueue_rom(None);
                            }
                        });
                    } else {
                        eprintln!("[rfid] unknown card={}", id);
                    }
                }
            });
        }
    });
}
