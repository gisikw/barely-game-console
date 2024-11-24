use eframe::egui::{self, ColorImage, Context, TextureHandle, TextureOptions};
use std::sync::Arc;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimationState {
    Offscreen,
    AnimatingIn,
    AnimatingOut,
    Idle,
}

struct BarelyGameConsole {
    animation_state: AnimationState,
    animation_start_time: Option<f64>,
    rom_selected: Option<String>,
    pending_rom: Option<String>,
    texture: Option<TextureHandle>,
    rom_texture: Option<TextureHandle>,
    ctx: Arc<Context>,
}

impl BarelyGameConsole {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let texture = load_image(
            cc,
            "/home/gisikw/Projects/barely-game-console/assets/background.png",
        );
        Self {
            animation_state: AnimationState::Offscreen,
            animation_start_time: None,
            rom_selected: None,
            pending_rom: None,
            texture,
            rom_texture: None,
            ctx: Arc::new(cc.egui_ctx.clone()),
        }
    }

    fn enqueue_rom(&mut self, rom: Option<String>) {
        // TODO: Ignore duplicates

        self.pending_rom = rom.clone();

        let current_time = self.ctx.input(|i| i.time);

        use AnimationState::*;

        match self.animation_state {
            AnimatingIn | AnimatingOut => {
                self.animation_state = AnimatingOut;
                let elapsed_time = match self.animation_start_time {
                    Some(t) => current_time - t,
                    None => 0.0,
                };
                self.animation_start_time = Some(current_time + elapsed_time);
            }
            Idle => {
                self.animation_state = AnimatingOut;
                self.animation_start_time = Some(current_time);
            }
            Offscreen => {
                if rom.is_some() {
                    self.rom_selected = self.pending_rom.clone();
                    self.animation_state = AnimatingIn;
                    self.animation_start_time = Some(current_time);
                }
            }
        }

        self.ctx.request_repaint();
    }

    // TODO: Split out rendering from state transition
    fn render_preview(&mut self, ui: &mut egui::Ui) {
        let preview_size = egui::Vec2::new(400.0, 400.0);

        let available_size = ui.available_size();
        let center_y = (available_size.y - preview_size.y) / 2.0;

        let current_time = self.ctx.input(|i| i.time);
        let offset_y = match self.animation_state {
            AnimationState::AnimatingIn | AnimationState::AnimatingOut => {
                let elapsed = self
                    .animation_start_time
                    .map_or(0.0, |start| current_time - start);
                let mut progress = (elapsed / ANIMATION_TIME).clamp(0.0, 1.0);
                if progress == 1.0 {
                    if self.animation_state == AnimationState::AnimatingOut {
                        if self.pending_rom.is_some() {
                            progress = 0.0;
                            self.rom_selected = self.pending_rom.clone();
                            self.pending_rom = None;
                            self.animation_start_time = Some(current_time);
                            self.animation_state = AnimationState::AnimatingIn;
                            self.ctx.request_repaint();
                        } else {
                            self.animation_state = AnimationState::Offscreen;
                        }
                    } else {
                        self.animation_state = AnimationState::Idle;
                    }
                } else {
                    self.ctx.request_repaint();
                }

                match self.animation_state {
                    AnimationState::AnimatingIn => {
                        (bouncy_easing(1.0 - progress) * TRAVEL_DISTANCE) as f32
                    }
                    AnimationState::AnimatingOut => {
                        (bouncy_easing(progress) * TRAVEL_DISTANCE) as f32
                    }
                    AnimationState::Offscreen => TRAVEL_DISTANCE as f32,
                    AnimationState::Idle => 0.0,
                }
            }
            AnimationState::Offscreen => TRAVEL_DISTANCE as f32,
            AnimationState::Idle => 0.0,
        };

        ui.add_space((center_y + offset_y).max(0.0));

        let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

        if let Some(_) = self.rom_selected {
            if self.rom_texture.is_none() {
                let rom_image = image::open(SMW_WORLD).unwrap().to_rgba8();
                let size = [rom_image.width() as usize, rom_image.height() as usize];
                let color_image = ColorImage::from_rgba_unmultiplied(
                    size,
                    rom_image.as_flat_samples().as_slice(),
                );
                self.rom_texture = Some(self.ctx.load_texture(
                    "rom_preview",
                    color_image,
                    TextureOptions::LINEAR,
                ));
                self.animation_start_time = Some(self.ctx.input(|i| i.time));
            }

            let image_size = self.rom_texture.as_ref().unwrap().size_vec2();
            let image_aspect = image_size.x / image_size.y;

            let rect_aspect = preview_size.x / preview_size.y;

            let adjusted_rect = if image_aspect > rect_aspect {
                let new_height = preview_size.x / image_aspect;
                let y_offset = (preview_size.y - new_height) / 2.0;
                egui::Rect::from_min_max(
                    rect.min + egui::vec2(0.0, y_offset),
                    rect.max - egui::vec2(0.0, preview_size.y - new_height - y_offset),
                )
            } else {
                let new_width = preview_size.y * image_aspect;
                let x_offset = (preview_size.x - new_width) / 2.0;
                egui::Rect::from_min_max(
                    rect.min + egui::vec2(x_offset, 0.0),
                    rect.max - egui::vec2(preview_size.x - new_width - x_offset, 0.0),
                )
            };

            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(20.0),
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 51),
            );
            ui.painter().image(
                self.rom_texture.as_ref().unwrap().id(),
                adjusted_rect,
                egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(20.0),
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 51),
            );
        }

        ui.painter().rect_stroke(
            rect.expand(8.0),
            egui::Rounding::same(20.0),
            egui::Stroke::new(8.0, egui::Color32::from_rgb(238, 238, 187)),
        );
    }
}

impl eframe::App for BarelyGameConsole {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(texture) = &self.texture {
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
                    render_header(ui, "Barely Game Console");
                    ui.add_space(20.0);
                    self.render_preview(ui);
                });
            });
    }
}

fn load_image(cc: &eframe::CreationContext<'_>, path: &str) -> Option<TextureHandle> {
    let image = image::open(path).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color_image = ColorImage::from_rgba_unmultiplied(size, image.as_flat_samples().as_slice());
    Some(
        cc.egui_ctx
            .load_texture("background", color_image, TextureOptions::LINEAR),
    )
}

fn draw_background(ctx: &Context, texture: &TextureHandle) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen_rect = ctx.screen_rect();

    let uv = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));
    painter.image(texture.id(), screen_rect, uv, egui::Color32::WHITE);
}

fn render_header(ui: &mut egui::Ui, text: &str) {
    let shadow_color = egui::Color32::from_rgb(153, 153, 153);
    let stroke_color = egui::Color32::from_rgb(102, 102, 136);

    let rect = ui.available_rect_before_wrap();
    let center_x = rect.center().x;
    let top_y = rect.top() + 40.0;

    ui.painter().text(
        egui::Pos2::new(center_x + 2.0, top_y + 2.0),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(72.0),
        shadow_color,
    );

    ui.painter().text(
        egui::Pos2::new(center_x, top_y),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(72.0),
        stroke_color,
    );

    ui.add_space(72.0);
}

fn bouncy_easing(progress: f64) -> f64 {
    // Hardcoded control points for `cubic-bezier(0.68, -0.6, 0.32, 1.2)`
    let (x1, y1) = (0.68, -0.6);
    let (x2, y2) = (0.32, 1.2);

    // Clamp progress to [0, 1]
    let progress = progress.clamp(0.0, 1.0);

    // Function to evaluate the cubic Bézier at t
    let bezier = |t: f64, p1: f64, p2: f64| -> f64 {
        (1.0 - t).powi(3) * 0.0
            + 3.0 * (1.0 - t).powi(2) * t * p1
            + 3.0 * (1.0 - t) * t.powi(2) * p2
            + t.powi(3) * 1.0
    };

    // Numerically solve for t using Newton's method
    let mut t = progress; // Initial guess
    for _ in 0..10 {
        let x = bezier(t, x1, x2); // Current x at t
        let dx = 3.0 * (1.0 - t).powi(2) * x1 + 6.0 * (1.0 - t) * t * x2 + 3.0 * t.powi(2) * 1.0; // Derivative of x wrt t
        if dx.abs() < 1e-6 {
            break; // Avoid division by zero
        }
        t -= (x - progress) / dx; // Newton's method
        t = t.clamp(0.0, 1.0); // Ensure t stays in [0, 1]
    }

    // Compute the corresponding y value
    bezier(t, y1, y2)
}
