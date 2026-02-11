//! Simple bitmap text rendering

const CHAR_WIDTH: i32 = 8;
const CHAR_HEIGHT: i32 = 16;

/// Draw text at position (very simple, monospace)
pub fn draw_text(canvas: &mut [u8], stride: i32, x: i32, y: i32, text: &str, color: u32) {
    for (i, ch) in text.chars().enumerate() {
        draw_char(canvas, stride, x + (i as i32 * CHAR_WIDTH), y, ch, color);
    }
}

fn draw_char(canvas: &mut [u8], stride: i32, x: i32, y: i32, _ch: char, color: u32) {
    // Simple 8x16 bitmap font (just draw a rectangle for now as placeholder)
    // This is temporary - we'll use proper font rendering later

    // For now, just draw simple boxes to prove rendering works
    for dy in 0..CHAR_HEIGHT {
        for dx in 0..CHAR_WIDTH {
            let px = x + dx;
            let py = y + dy;

            if px >= 0 && py >= 0 {
                let offset = (py * stride + px * 4) as usize;
                if offset + 3 < canvas.len() {
                    // Simple pattern to show SOMETHING
                    if dx == 0 || dx == CHAR_WIDTH - 1 || dy == 0 || dy == CHAR_HEIGHT - 1 {
                        let bytes = color.to_le_bytes();
                        canvas[offset..offset + 4].copy_from_slice(&bytes);
                    }
                }
            }
        }
    }
}

pub fn text_width(text: &str) -> i32 {
    text.chars().count() as i32 * CHAR_WIDTH
}
