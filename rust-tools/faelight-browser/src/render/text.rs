//! Text rendering - copied from faelight-bar

use faelight_core::GlyphCache;

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");

lazy_static::lazy_static! {
    static ref GLYPH_CACHE: std::sync::Mutex<GlyphCache> = {
        std::sync::Mutex::new(
            GlyphCache::new(FONT_DATA).expect("Failed to load font")
        )
    };
}

pub fn draw_text(canvas: &mut [u8], width: u32, text: &str, x: i32, y: i32, color: [u8; 4]) {
    let mut cache = GLYPH_CACHE.lock().unwrap();
    let mut cursor_x = x;
    let font_size = 14.0;
    let baseline = y + 12;

    for ch in text.chars() {
        let glyph = cache.rasterize(ch, font_size);
        let metrics = &glyph.metrics;
        let bitmap = &glyph.bitmap;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 {
                    continue;
                }

                let px = cursor_x + metrics.xmin + col as i32;
                let py = baseline - metrics.height as i32 - metrics.ymin + row as i32;

                if px >= 0 && px < width as i32 && py >= 0 {
                    let idx = (py as usize * width as usize + px as usize) * 4;
                    if idx + 3 < canvas.len() {
                        let a = alpha as f32 / 255.0;
                        canvas[idx] = ((1.0 - a) * canvas[idx] as f32 + a * color[0] as f32) as u8;
                        canvas[idx + 1] =
                            ((1.0 - a) * canvas[idx + 1] as f32 + a * color[1] as f32) as u8;
                        canvas[idx + 2] =
                            ((1.0 - a) * canvas[idx + 2] as f32 + a * color[2] as f32) as u8;
                        canvas[idx + 3] = 255;
                    }
                }
            }
        }
        cursor_x += metrics.advance_width as i32;
    }
}
