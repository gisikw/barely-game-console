use eframe::egui::{self, Context, TextureHandle};

pub fn draw_background(ctx: &Context, texture: &TextureHandle) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let screen_rect = ctx.screen_rect();

    let uv = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0));
    painter.image(texture.id(), screen_rect, uv, egui::Color32::WHITE);
}

pub fn draw_header(ui: &mut egui::Ui, text: &str) {
    let transparent = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0);

    let rect = ui.available_rect_before_wrap();
    let center_x = rect.center().x;
    let top_y = rect.top() + 40.0;

    // Invisible "shadow"
    ui.painter().text(
        egui::Pos2::new(center_x + 2.0, top_y + 2.0),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(72.0),
        transparent,
    );

    // Invisible "stroke"
    ui.painter().text(
        egui::Pos2::new(center_x, top_y),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(72.0),
        transparent,
    );

    ui.add_space(72.0);
}

pub fn draw_preview(ui: &mut egui::Ui, offset: f64, opacity: f64, texture: &Option<TextureHandle>) {
    let preview_size = egui::Vec2::new(400.0, 400.0);
    let available_size = ui.available_size();
    let center_y = (available_size.y - preview_size.y) / 2.0;

    ui.add_space((center_y + offset as f32).max(0.0));

    let mut painter = ui.painter().clone();
    painter.set_opacity(opacity as f32);

    let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
    painter.rect_filled(
        rect,
        egui::Rounding::same(20.0),
        egui::Color32::from_rgba_premultiplied(0, 0, 0, 51),
    );

    painter.rect_stroke(
        rect.expand(8.0),
        egui::Rounding::same(20.0),
        egui::Stroke::new(8.0, egui::Color32::from_rgb(238, 238, 187)),
    );

    if let Some(texture) = texture.as_ref() {
        let image_size = texture.size_vec2();
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

        painter.image(
            texture.id(),
            adjusted_rect,
            egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}
