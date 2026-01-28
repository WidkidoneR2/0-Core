//! Rendering subsystem
pub mod bar;
pub mod menu;

use crate::state::{AppState, ModeState};

pub fn render(state: &AppState, canvas: &mut [u8], width: u32, height: u32) -> Vec<(i32, i32, String)> {
    match &state.mode {
        ModeState::Bar(_) => {
            bar::render(canvas, width, height)
        }
        ModeState::Menu(menu) => {
            let click_regions = bar::render(canvas, width, height);
            menu::render(menu, canvas, width, height, 32);
            click_regions
        }
    }
}
