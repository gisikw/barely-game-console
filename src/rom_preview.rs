use crate::assets::load_texture;
use crate::ui::draw_preview;
use eframe::egui::{self, TextureHandle};

static ANIMATION_TIME: f64 = 0.6;
static TRAVEL_DISTANCE: f64 = 800.0;

#[derive(Debug)]
enum AnimationState {
    Active,
    FlyingIn,
    FlyingOut,
    Offscreen,
    ReverseFlyingIn(f64),
}

pub struct RomPreview {
    state: AnimationState,
    next_rom: Option<String>,
    texture: Option<TextureHandle>,
    start_time: Option<f64>,
}

impl RomPreview {
    pub fn new() -> Self {
        Self {
            state: AnimationState::Offscreen,
            next_rom: None,
            texture: None,
            start_time: None,
        }
    }

    pub fn enqueue(&mut self, next_rom: Option<String>) {
        self.next_rom = next_rom;
        match self.state {
            AnimationState::Active => {
                self.state = AnimationState::FlyingOut;
                self.start_time = None;
            }
            AnimationState::FlyingIn => {
                self.state = AnimationState::ReverseFlyingIn(self.start_time.unwrap());
                self.start_time = None;
            }
            AnimationState::Offscreen => {
                // Force a transition on update, to get texture loading
                self.state = AnimationState::FlyingOut;
                self.start_time = Some(-ANIMATION_TIME);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let current_time = ctx.input(|i| i.time);
        println!("In update ({:?}, {:?})", self.state, current_time);
        if self.start_time.is_none() {
            self.start_time = Some(current_time);
        }

        let offset = match self.state {
            AnimationState::Active => 0.0,
            AnimationState::Offscreen => TRAVEL_DISTANCE,
            _ => self.sample_animation(current_time),
        };

        draw_preview(ui, offset, &self.texture);

        self.resolve_animation_state(current_time, ctx);
    }

    fn resolve_animation_state(&mut self, current_time: f64, ctx: &egui::Context) {
        let should_transition = match self.state {
            AnimationState::ReverseFlyingIn(original_start_time) => {
                let start_time = self.start_time.unwrap();
                current_time - start_time >= start_time - original_start_time
            }
            _ => self.progress(current_time) >= 1.0,
        };

        if should_transition {
            self.transition_to_next_state(ctx);
        } else {
            match self.state {
                AnimationState::FlyingIn
                | AnimationState::FlyingOut
                | AnimationState::ReverseFlyingIn(_) => {
                    ctx.request_repaint();
                }
                _ => {}
            }
        }
    }

    fn transition_to_next_state(&mut self, ctx: &egui::Context) {
        self.start_time = None;
        self.state = match self.state {
            AnimationState::FlyingIn => AnimationState::Active,
            AnimationState::FlyingOut | AnimationState::ReverseFlyingIn(_) => {
                match &self.next_rom {
                    Some(path) => {
                        self.texture = load_texture(ctx, path);
                        self.next_rom = None;
                        ctx.request_repaint();
                        AnimationState::FlyingIn
                    }
                    None => AnimationState::Offscreen,
                }
            }
            _ => return,
        };
    }

    fn sample_animation(&self, current_time: f64) -> f64 {
        match self.state {
            AnimationState::FlyingIn => {
                bouncy_easing(1.0 - self.progress(current_time)) * TRAVEL_DISTANCE
            }
            AnimationState::FlyingOut => {
                bouncy_easing(self.progress(current_time)) * TRAVEL_DISTANCE
            }
            AnimationState::ReverseFlyingIn(original_start_time) => {
                let start_time = self
                    .start_time
                    .expect("start_time was not set before sampling animation");
                let progress = (start_time - original_start_time - current_time) / ANIMATION_TIME;
                bouncy_easing(1.0 - progress) * TRAVEL_DISTANCE
            }
            _ => 0.0,
        }
    }

    fn progress(&self, current_time: f64) -> f64 {
        let start_time = self
            .start_time
            .expect("start_time was not set before sampling animation");
        ((current_time - start_time) / ANIMATION_TIME).clamp(0.0, 1.0)
    }
}

fn bouncy_easing(progress: f64) -> f64 {
    let (x1, y1) = (0.68, -0.6);
    let (x2, y2) = (0.32, 1.2);

    let t = solve_cubic_bezier(progress, x1, x2);
    cubic_bezier(t, y1, y2)
}

fn solve_cubic_bezier(progress: f64, x1: f64, x2: f64) -> f64 {
    let mut t = progress;
    for _ in 0..10 {
        let x = cubic_bezier(t, x1, x2);
        let dx = cubic_bezier_derivative(t, x1, x2);
        if dx.abs() < 1e-6 {
            break;
        }
        t -= (x - progress) / dx;
        t = t.clamp(0.0, 1.0);
    }
    t
}

fn cubic_bezier(t: f64, p1: f64, p2: f64) -> f64 {
    (1.0 - t).powi(3) * 0.0
        + 3.0 * (1.0 - t).powi(2) * t * p1
        + 3.0 * (1.0 - t) * t.powi(2) * p2
        + t.powi(3) * 1.0
}

fn cubic_bezier_derivative(t: f64, p1: f64, p2: f64) -> f64 {
    3.0 * (1.0 - t).powi(2) * p1 + 6.0 * (1.0 - t) * t * p2 + 3.0 * t.powi(2) * 1.0
}
