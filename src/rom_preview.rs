use eframe::egui::{self, TextureHandle};

enum AnimationState {
    Active,
    FlyingIn,
    FlyingOut,
    Offscreen,
    ReverseFlyingIn(f64),
}

pub struct RomPreview {
    state: AnimationState,
    current_rom: Option<String>,
    next_rom: Option<String>,
    texture: Option<TextureHandle>,
    start_time: Option<f64>,
    reverse_time: Option<f64>,
}

impl RomPreview {
    pub fn new() -> Self {
        Self {
            state: AnimationState::Offscreen,
            current_rom: None,
            next_rom: None,
            texture: None,
            start_time: None,
            reverse_time: None,
        }
    }

    pub fn enqueue(&mut self, next_rom: String) {
        self.next_rom = Some(next_rom);

        match self.state {
            AnimationState::Active => {
                self.state = AnimationState::FlyingOut;
                self.start_time = None;
            }
            AnimationState::FlyingIn => {
                self.state = AnimationState::ReverseFlyingIn(self.start_time.unwrap());
                self.start_time = None;
            }
            _ => {}
        };
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if self.start_time.is_none() {
            self.start_time = Some(ctx.input(|i| i.time));
        }

        let offset = match self.state = {

        }

        // Do the ui rendering
    }
}
