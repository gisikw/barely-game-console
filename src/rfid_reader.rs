use evdev::{Device, InputEventKind, Key};

pub struct RFIDReader;

impl RFIDReader {
    pub fn new() -> Self {
        Self
    }

    pub fn run<F>(&self, mut on_id: F)
    where
        F: FnMut(String),
    {
        let device_path = "/dev/input/event18";
        let mut device = Device::open(device_path).expect("Failed to open it womp womp");

        let _ = device.grab();

        let mut id = String::new();
        loop {
            for event in device.fetch_events().expect("could not fetch event") {
                if let InputEventKind::Key(key) = event.kind() {
                    if event.value() == 0 {
                        match key {
                            Key::KEY_ENTER => {
                                on_id(id.clone());
                                id.clear();
                            }
                            Key::KEY_1 => {
                                id.push('1');
                            }
                            Key::KEY_2 => {
                                id.push('2');
                            }
                            Key::KEY_3 => {
                                id.push('3');
                            }
                            Key::KEY_4 => {
                                id.push('4');
                            }
                            Key::KEY_5 => {
                                id.push('5');
                            }
                            Key::KEY_6 => {
                                id.push('6');
                            }
                            Key::KEY_7 => {
                                id.push('7');
                            }
                            Key::KEY_8 => {
                                id.push('8');
                            }
                            Key::KEY_9 => {
                                id.push('9');
                            }
                            Key::KEY_0 => {
                                id.push('0');
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
