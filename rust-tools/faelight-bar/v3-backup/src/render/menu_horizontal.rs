//! Horizontal dmenu-style search rendering
use crate::state::MenuState;
use faelight_core::GlyphCache;

const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const SELECTED_COLOR: [u8; 4] = [0x77, 0xc1, 0xf5, 0xFF];
const DIM_COLOR: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF];
const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");

lazy_static::lazy_static! {
    static ref GLYPH_CACHE: std::sync::Mutex<GlyphCache> = {
        std::sync::Mutex::new(
            GlyphCache::new(FONT_DATA).expect("Failed to load font")
        )
    };
}

fn draw_text(
    cache: &mut GlyphCache,
    canvas: &mut [u8],
    width: u32,
    text: &str,
    x: i32,
    y: i32,
    color: [u8; 4],
) {
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

                if px >= 0 && px < width as i32 && (0..32).contains(&py) {
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

pub fn render(menu: &MenuState, canvas: &mut [u8], width: u32, _height: u32) {
    let mut cache = GLYPH_CACHE.lock().unwrap();

    // NO background clearing - transparent overlay!
    // Start right after search icon (~130px)
    let mut x_pos = 250;

    draw_text(&mut cache, canvas, width, "Search:", x_pos, 8, TEXT_COLOR);
    x_pos += 70;

    draw_text(
        &mut cache,
        canvas,
        width,
        &menu.input,
        x_pos,
        8,
        ACCENT_COLOR,
    );

    let cursor_x = x_pos + (menu.input.len() as i32 * 8);
    draw_text(&mut cache, canvas, width, "_", cursor_x, 8, ACCENT_COLOR);

    x_pos = cursor_x + 30;

    if !menu.filtered.is_empty() {
        draw_text(&mut cache, canvas, width, "→", x_pos, 8, DIM_COLOR);
        x_pos += 20;

        if let Some(&first_idx) = menu.filtered.first() {
            let item = &menu.items[first_idx];
            draw_text(
                &mut cache,
                canvas,
                width,
                &item.display,
                x_pos,
                8,
                SELECTED_COLOR,
            );
        }
    }
}
