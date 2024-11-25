use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::HashMap;

pub struct TextureCache {
    cache: HashMap<String, TextureHandle>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_load(&mut self, ctx: &Context, path: &str) -> Option<&TextureHandle> {
        if !self.cache.contains_key(path) {
            let image = image::open(path).ok()?.to_rgba8();
            let size = [image.width() as usize, image.height() as usize];
            let color_image =
                ColorImage::from_rgba_unmultiplied(size, image.as_flat_samples().as_slice());

            let texture = ctx.load_texture(path, color_image, TextureOptions::LINEAR);
            self.cache.insert(path.to_string(), texture);
        }
        self.cache.get(path)
    }
}
