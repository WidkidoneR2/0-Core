//! Ultra-smooth text rendering

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings};
use std::sync::OnceLock;

const FONT_SIZE: f32 = 24.0;

static MAIN_FONT: OnceLock<Font> = OnceLock::new();

fn get_main_font() -> &'static Font {
    MAIN_FONT.get_or_init(|| {
        let font_data = include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf");
        Font::from_bytes(font_data as &[u8], FontSettings::default()).expect("Failed to load font")
    })
}

/// Draw text left-aligned, returns the x position after the text (for chaining)
pub fn draw_text(canvas: &mut [u8], stride: i32, x: i32, y: i32, text: &str, color: u32) -> i32 {
    let font = get_main_font();

    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.0,
        y: 0.0,
        max_width: Some(3000.0),
        max_height: Some(200.0),
        ..LayoutSettings::default()
    });

    layout.append(&[font], &TextStyle::new(text, FONT_SIZE, 0));

    let color_r = ((color >> 16) & 0xFF) as u8;
    let color_g = ((color >> 8) & 0xFF) as u8;
    let color_b = (color & 0xFF) as u8;

    let mut max_x = x;

    for glyph in layout.glyphs() {
        let (metrics, bitmap) = font.rasterize_config(glyph.key);

        for (i, &alpha) in bitmap.iter().enumerate() {
            if alpha > 20 {
                let glyph_x = i % metrics.width;
                let glyph_y = i / metrics.width;

                let px = x + glyph.x as i32 + glyph_x as i32;
                let py = y + glyph.y as i32 + glyph_y as i32 - 4;

                if px >= 0 && py >= 0 {
                    let offset = (py * stride + px * 4) as usize;
                    if offset + 3 < canvas.len() {
                        let alpha_f = (alpha as f32 / 255.0).powf(1.0 / 2.2);
                        let inv_alpha = 1.0 - alpha_f;

                        let bg_b = canvas[offset] as f32;
                        let bg_g = canvas[offset + 1] as f32;
                        let bg_r = canvas[offset + 2] as f32;

                        canvas[offset] = (color_b as f32 * alpha_f + bg_b * inv_alpha) as u8;
                        canvas[offset + 1] = (color_g as f32 * alpha_f + bg_g * inv_alpha) as u8;
                        canvas[offset + 2] = (color_r as f32 * alpha_f + bg_r * inv_alpha) as u8;
                        canvas[offset + 3] = 0xFF;
                    }
                }
            }
        }

        let glyph_end = x + glyph.x as i32 + glyph.width as i32;
        if glyph_end > max_x {
            max_x = glyph_end;
        }
    }

    max_x
}

pub fn text_width(text: &str) -> i32 {
    let font = get_main_font();
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings::default());
    layout.append(&[font], &TextStyle::new(text, FONT_SIZE, 0));

    layout
        .glyphs()
        .last()
        .map(|g| (g.x + g.width as f32) as i32)
        .unwrap_or((text.len() * 14) as i32)
}
