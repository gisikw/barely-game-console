use eframe::egui::{self, Context, TextureHandle};
use std::sync::Arc;

mod animations;
mod assets;
mod ui;
use crate::animations::{AnimationController, AnimationState};
use crate::assets::TextureCache;
use crate::ui::{draw_background, draw_header, draw_preview};

static ANIMATION_TIME: f64 = 0.6;
static TRAVEL_DISTANCE: f64 = 800.0;

static SMW_WORLD: &str = "/home/gisikw/Projects/barely-game-console/assets/SMWCase.jpg";

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Barely Game Console",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BarelyGameConsole::new(cc)))),
    )
}

struct BarelyGameConsole {
    animation_controller: AnimationController,
    rom_selected: Option<String>,
    pending_rom: Option<String>,
    texture_cache: TextureCache,
    rom_texture: Option<TextureHandle>,
    ctx: Arc<Context>,
}

impl BarelyGameConsole {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            animation_controller: AnimationController::new(TRAVEL_DISTANCE, ANIMATION_TIME),
            rom_selected: None,
            pending_rom: None,
            texture_cache: TextureCache::new(),
            rom_texture: None,
            ctx: Arc::new(cc.egui_ctx.clone()),
        }
    }

    fn enqueue_rom(&mut self, rom: Option<String>) {
        self.pending_rom = rom.clone();
        self.rom_texture = self
            .texture_cache
            .get_or_load(&self.ctx, SMW_WORLD)
            .cloned();
        let current_time = self.ctx.input(|i| i.time);
        match self.animation_controller.state {
            AnimationState::Offscreen => {
                self.animation_controller
                    .start(AnimationState::AnimatingIn, current_time, None);
            }
            _ => {
                let ctx = Arc::clone(&self.ctx);
                self.animation_controller.start(
                    AnimationState::AnimatingOut,
                    current_time,
                    Some(Box::new(move || {
                        ctx.request_repaint();
                        Some(AnimationState::AnimatingIn)
                    })),
                );
            }
        }
        self.ctx.request_repaint();
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        let current_time = self.ctx.input(|i| i.time);
        let offset = self.animation_controller.update(current_time);
        if !self
            .animation_controller
            .animation
            .is_complete(current_time)
        {
            self.ctx.request_repaint();
        }

        draw_preview(ui, offset, &self.rom_texture);
    }
}

impl eframe::App for BarelyGameConsole {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(texture) = &self.texture_cache.get_or_load(
            &ctx,
            "/home/gisikw/Projects/barely-game-console/assets/background.png",
        ) {
            draw_background(ctx, texture);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.button("Select SMW").clicked() {
                        self.enqueue_rom(Some("Super Mario World".to_string()));
                    }
                    if ui.button("Clear").clicked() {
                        self.enqueue_rom(None);
                    }
                    ui.add_space(40.0);
                    draw_header(ui, "Barely Game Console");
                    ui.add_space(20.0);
                    self.render_preview(ui);
                });
            });
    }
}
