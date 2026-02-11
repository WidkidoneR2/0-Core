//! Faelight Forest color palette

#![allow(dead_code)] // Colors used in Phase 2+

pub const BG: u32 = 0xFF1A1A2E; // Dark background
pub const FG: u32 = 0xFFE8E8F0; // Light text
pub const ACCENT: u32 = 0xFF4ECCA3; // Teal accent
pub const WARNING: u32 = 0xFFF39C12; // Orange
pub const ERROR: u32 = 0xFFE74C3C; // Red
pub const SUCCESS: u32 = 0xFF2ECC71; // Green
pub const BG_HOVER: u32 = 0xFF2A2A3E; // Hover state
pub const SEPARATOR: u32 = 0xFF3A3A4E; // Widget separator

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
