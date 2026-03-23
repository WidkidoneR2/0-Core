// faelight-notify v4 — text rendering
// Uses fontdue::layout::Layout — same approach as faelight-bar/render/text.rs
// This is the proven renderer. No manual baseline math.

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings};
use std::sync::OnceLock;

// Faelight Forest palette
pub const BG: [u8; 4] = [0x11, 0x14, 0x0f, 0xf0]; // dark bg, slight transparency
pub const BORDER_NORMAL: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xff]; // faelight green
pub const BORDER_CRITICAL: [u8; 4] = [0xe3, 0x6b, 0x6b, 0xff]; // red
pub const BORDER_LOW: [u8; 4] = [0x4a, 0x6a, 0x3a, 0xff]; // muted green
pub const TEXT_APP: [u8; 4] = [0x55, 0x60, 0x50, 0xff]; // dim
pub const TEXT_SUMMARY: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xff]; // bright green
pub const TEXT_BODY: [u8; 4] = [0xda, 0xe0, 0xd7, 0xff]; // light

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");

static FONT: OnceLock<Font> = OnceLock::new();

fn get_font() -> &'static Font {
    FONT.get_or_init(|| {
        Font::from_bytes(FONT_DATA, FontSettings::default()).expect("Failed to load HackNerdFont")
    })
}

/// Draw text using fontdue layout engine — proven correct baseline handling
/// Returns x position after text
pub fn draw_text(
    canvas: &mut [u8],
    stride: u32,
    x: i32,
    y: i32,
    text: &str,
    color: [u8; 4],
    size: f32,
) -> i32 {
    let font = get_font();
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.0,
        y: 0.0,
        max_width: Some(3000.0),
        max_height: Some(200.0),
        ..LayoutSettings::default()
    });
    layout.append(&[font], &TextStyle::new(text, size, 0));

    let mut max_x = x;
    for glyph in layout.glyphs() {
        let (metrics, bitmap) = font.rasterize_config(glyph.key);
        for (i, &alpha) in bitmap.iter().enumerate() {
            if alpha > 20 {
                let gx = i % metrics.width;
                let gy = i / metrics.width;
                let px = x + glyph.x as i32 + gx as i32;
                let py = y + glyph.y as i32 + gy as i32;
                if px >= 0 && py >= 0 {
                    let offset = (py as u32 * stride + px as u32 * 4) as usize;
                    if offset + 3 < canvas.len() {
                        let a = (alpha as f32 / 255.0).powf(1.0 / 2.2);
                        let ia = 1.0 - a;
                        canvas[offset] = (color[2] as f32 * a + canvas[offset] as f32 * ia) as u8;
                        canvas[offset + 1] =
                            (color[1] as f32 * a + canvas[offset + 1] as f32 * ia) as u8;
                        canvas[offset + 2] =
                            (color[0] as f32 * a + canvas[offset + 2] as f32 * ia) as u8;
                        canvas[offset + 3] = 0xff;
                    }
                }
            }
        }
        let end = x + glyph.x as i32 + glyph.width as i32;
        if end > max_x {
            max_x = end;
        }
    }
    max_x
}

/// Draw a filled rectangle
pub fn fill_rect(canvas: &mut [u8], stride: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for row in y..y + h {
        for col in x..x + w {
            let idx = (row * stride + col * 4) as usize;
            if idx + 3 < canvas.len() {
                canvas[idx] = color[2]; // B
                canvas[idx + 1] = color[1]; // G
                canvas[idx + 2] = color[0]; // R
                canvas[idx + 3] = color[3]; // A
            }
        }
    }
}

/// Draw notification popup onto canvas
pub fn draw_notification(
    canvas: &mut [u8],
    width: u32,
    height: u32,
    app_name: &str,
    summary: &str,
    body: &str,
    border_color: [u8; 4],
) {
    let stride = width * 4;

    // Background
    fill_rect(canvas, stride, 0, 0, width, height, BG);

    // Border — 2px all sides
    fill_rect(canvas, stride, 0, 0, width, 2, border_color); // top
    fill_rect(canvas, stride, 0, height - 2, width, 2, border_color); // bottom
    fill_rect(canvas, stride, 0, 0, 2, height, border_color); // left
    fill_rect(canvas, stride, width - 2, 0, 2, height, border_color); // right

    let pad = 12i32;
    let font_size = 13.5f32;

    // App name — dim, small
    draw_text(
        canvas,
        stride,
        pad,
        pad,
        app_name,
        TEXT_APP,
        font_size - 1.0,
    );

    // Summary — bright green, medium
    draw_text(
        canvas,
        stride,
        pad,
        pad + 18,
        summary,
        TEXT_SUMMARY,
        font_size + 1.0,
    );

    // Body — light, normal
    if !body.is_empty() {
        draw_text(canvas, stride, pad, pad + 38, body, TEXT_BODY, font_size);
    }
}
