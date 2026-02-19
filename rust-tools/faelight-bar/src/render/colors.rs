//! Faelight Forest color palette - FM/Palette Edition 🎨

#![allow(dead_code)]

// Core FM/Palette colors
pub const BG: u32 = 0xFF11140F; // BG_DARK: RGB(17, 20, 15)
pub const FG: u32 = 0xFFDAE0D7; // TEXT_BRIGHT: RGB(218, 224, 215)
pub const ACCENT: u32 = 0xFFA3E36B; // ACCENT_GREEN: RGB(163, 227, 107)
pub const SUCCESS: u32 = 0xFFA3E36B; // Same as ACCENT_GREEN
pub const WARNING: u32 = 0xFFFFAA00; // Orange
pub const ERROR: u32 = 0xFFFF6B6B; // Red
pub const BG_SELECTED: u32 = 0xFF2D3426; // BG_SELECTED: RGB(45, 52, 38)
pub const ACCENT_BLUE: u32 = 0xFF6BA3E3; // ACCENT_BLUE: RGB(107, 163, 227)
pub const SEPARATOR: u32 = 0xFF2D3426; // Subtle separator

/// Helper to blend two colors
pub fn blend(color1: u32, color2: u32, ratio: f32) -> u32 {
    let r1 = ((color1 >> 16) & 0xFF) as f32;
    let g1 = ((color1 >> 8) & 0xFF) as f32;
    let b1 = (color1 & 0xFF) as f32;

    let r2 = ((color2 >> 16) & 0xFF) as f32;
    let g2 = ((color2 >> 8) & 0xFF) as f32;
    let b2 = (color2 & 0xFF) as f32;

    let r = (r1 + (r2 - r1) * ratio) as u32;
    let g = (g1 + (g2 - g1) * ratio) as u32;
    let b = (b1 + (b2 - b1) * ratio) as u32;

    0xFF000000 | (r << 16) | (g << 8) | b
}
