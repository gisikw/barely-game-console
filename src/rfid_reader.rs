use evdev::{Device, InputEventKind, Key};
use std::fs;
use std::thread;
use std::time::Duration;

pub struct RFIDReader;

static DEVICE_NAME: &str = "HID 413d:2107";

impl RFIDReader {
    pub fn new() -> Self {
        Self
    }

    pub fn run<F>(&self, mut on_id: F)
    where
        F: FnMut(String),
    {
        let mut id = String::new();
        loop {
            let device_path = match find_device_path_by_name(DEVICE_NAME) {
                Some(path) => path,
                None => {
                    eprintln!("RFID device not found, retrying...");
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            let mut device = match Device::open(&device_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to open RFID device: {}, retrying...", e);
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };
            let _ = device.grab();
            eprintln!("RFID reader ready on {}", device_path);

            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if let InputEventKind::Key(key) = event.kind() {
                                if event.value() == 0 {
                                    match key {
                                        Key::KEY_ENTER => {
                                            on_id(id.clone());
                                            id.clear();
                                        }
                                        Key::KEY_1 => id.push('1'),
                                        Key::KEY_2 => id.push('2'),
                                        Key::KEY_3 => id.push('3'),
                                        Key::KEY_4 => id.push('4'),
                                        Key::KEY_5 => id.push('5'),
                                        Key::KEY_6 => id.push('6'),
                                        Key::KEY_7 => id.push('7'),
                                        Key::KEY_8 => id.push('8'),
                                        Key::KEY_9 => id.push('9'),
                                        Key::KEY_0 => id.push('0'),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("RFID reader error: {}, re-opening device...", e);
                        id.clear();
                        break;
                    }
                }
            }
        }
    }
}

pub fn find_device_path_by_name(target_name: &str) -> Option<String> {
    let entries = fs::read_dir("/dev/input").expect("Failed to read /dev/input");
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.to_str().unwrap().contains("event") {
                if let Ok(device) = Device::open(&path) {
                    if let Some(name) = device.name() {
                        if name == target_name {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
