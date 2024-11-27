use eframe::egui::{self, Context};
use std::sync::Arc;

use crate::assets::load_texture;
use crate::rom_preview::RomPreview;
use crate::ui::{draw_background, draw_header};

pub struct BarelyGameConsole {
    rom_preview: RomPreview,
    ctx: Arc<Context>,
}

impl BarelyGameConsole {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            rom_preview: RomPreview::new(),
            ctx: Arc::new(cc.egui_ctx.clone()),
        }
    }

    pub fn enqueue_rom(&mut self, rom: Option<String>) {
        self.rom_preview.enqueue(rom);
        self.ctx.request_repaint();
    }
}

impl eframe::App for BarelyGameConsole {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(texture) = load_texture(&ctx, "assets/background.png") {
            draw_background(ctx, &texture);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    draw_header(ui, "Barely Game Console");
                    ui.add_space(20.0);
                    self.rom_preview.update(ctx, ui);
                });
            });
    }
}
