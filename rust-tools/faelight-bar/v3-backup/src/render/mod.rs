//! Rendering abstraction for bar and menu modes

pub mod bar;

use crate::state::{AppState, ModeState};

/// Render the current mode to canvas and return click regions
pub fn render(
    state: &AppState,
    canvas: &mut [u8],
    width: u32,
    height: u32,
) -> Vec<(i32, i32, String)> {
    // Clear canvas to TRANSPARENT (not black!)
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Fully transparent
    }

    match &state.mode {
        ModeState::Bar(_) => {
            // Fill bar area with background, then render
            fill_rect(canvas, width, 0, 0, width, 32, [0x11, 0x14, 0x0f, 0xFF]);
            bar::render(canvas, width, height)
        }
        ModeState::Menu(menu) => {
            fill_rect(canvas, width, 0, 0, width, 32, [0x11, 0x14, 0x0f, 0xFF]);
            bar::render(canvas, width, height);
            menu_horizontal::render(menu, canvas, width, height);
            vec![]
        }
    }
}

fn fill_rect(canvas: &mut [u8], width: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for py in y..(y + h) {
        for px in x..(x + w) {
            let idx = (py as usize * width as usize + px as usize) * 4;
            if idx + 3 < canvas.len() {
                canvas[idx] = color[0];
                canvas[idx + 1] = color[1];
                canvas[idx + 2] = color[2];
                canvas[idx + 3] = color[3];
            }
        }
    }
}
pub mod menu_horizontal;
