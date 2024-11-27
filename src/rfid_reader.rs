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

        loop {
            for event in device.fetch_events().expect("could not fetch event") {
                if let InputEventKind::Key(key) = event.kind() {
                    if key == Key::KEY_ENTER && event.value() == 0 {
                        on_id("Super Mario World".to_string());
                    }
                }
            }
        }
    }
}
