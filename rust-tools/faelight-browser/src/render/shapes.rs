//! Shape rendering - gradient separators
//! EXACT copy from faelight-bar/src/render/bar.rs

use crate::ui::colors::BG_COLOR;

pub fn draw_gradient_separator(
    canvas: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    color: [u8; 4],
) {
    let start_y = 6;
    let end_y = height as i32 - 6;

    for y in start_y..end_y {
        let progress = (y - start_y) as f32 / (end_y - start_y) as f32;
        let alpha = if progress < 0.2 {
            progress / 0.2
        } else if progress > 0.8 {
            (1.0 - progress) / 0.2
        } else {
            1.0
        };

        if x >= 0 && x < width as i32 {
            let idx = (y as usize * width as usize + x as usize) * 4;
            if idx + 3 < canvas.len() {
                canvas[idx] = ((1.0 - alpha) * BG_COLOR[0] as f32 + alpha * color[0] as f32) as u8;
                canvas[idx + 1] =
                    ((1.0 - alpha) * BG_COLOR[1] as f32 + alpha * color[1] as f32) as u8;
                canvas[idx + 2] =
                    ((1.0 - alpha) * BG_COLOR[2] as f32 + alpha * color[2] as f32) as u8;
                canvas[idx + 3] = 255;
            }
        }
    }
}
