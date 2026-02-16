//! Text rendering using faelight-core's GlyphCache (smooth fonts!)

use faelight_core::GlyphCache;
use std::sync::Mutex;

static CACHE: Mutex<Option<GlyphCache>> = Mutex::new(None);

fn get_cache() -> &'static Mutex<Option<GlyphCache>> {
    &CACHE
}

pub fn draw_text(canvas: &mut [u8], stride: i32, x: i32, y: i32, text: &str, color: u32) {
    let mut cache_guard = get_cache().lock().unwrap();

    if cache_guard.is_none() {
        let font_data = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");
        *cache_guard = Some(GlyphCache::new(font_data).expect("Failed to load font"));
    }

    let cache = cache_guard.as_mut().unwrap();

    let mut x_pos = x;
    for ch in text.chars() {
        let glyph = cache.rasterize(ch, 18.0);

        for (i, &alpha) in glyph.bitmap.iter().enumerate() {
            if alpha == 0 {
                continue;
            }

            let px = i % glyph.metrics.width;
            let py = i / glyph.metrics.width;
            let screen_x = x_pos + px as i32;
            let screen_y = y + py as i32; // Fixed baseline

            if screen_x < 0 || screen_y < 0 {
                continue;
            }

            let offset = (screen_y * stride + screen_x * 4) as usize;
            if offset + 3 < canvas.len() {
                let r = (color & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = ((color >> 16) & 0xFF) as u8;

                // Blend with alpha
                let blend = |old: u8, new: u8, a: u8| -> u8 {
                    ((old as u16 * (255 - a as u16) + new as u16 * a as u16) / 255) as u8
                };

                canvas[offset] = blend(canvas[offset], b, alpha);
                canvas[offset + 1] = blend(canvas[offset + 1], g, alpha);
                canvas[offset + 2] = blend(canvas[offset + 2], r, alpha);
            }
        }

        x_pos += glyph.metrics.advance_width as i32;
    }
}

pub mod colors {
    pub const FG: u32 = 0xE0E0E0; // Light gray
    pub const BG: u32 = 0x1E1E1E; // Dark
    pub const ACCENT: u32 = 0x00FFFF; // Cyan
    pub const SUCCESS: u32 = 0x00FF00; // Green
    pub const WARNING: u32 = 0xFFFF00; // Yellow
    pub const SELECTED: u32 = 0x0080FF; // Blue
}
