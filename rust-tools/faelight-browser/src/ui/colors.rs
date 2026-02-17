//! Color palette - EXACT copy from faelight-bar
//!
//! Faelight Forest colors for consistent visual identity

#![allow(dead_code)]

// EXACT colors from faelight-bar/src/render/bar.rs
pub const BG_COLOR: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF]; // Dark forest
pub const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF]; // Light fog
pub const TEXT_BRIGHT: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF]; // Same as TEXT_COLOR
pub const TEXT_DIM: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF]; // Muted
pub const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF]; // Faelight green
pub const ACCENT_GREEN: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF]; // Same as ACCENT_COLOR
pub const ACCENT_BLUE: [u8; 4] = [0x6b, 0xa3, 0xe3, 0xFF]; // Blue accent
pub const DIM_COLOR: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF]; // Muted
pub const BLUE_COLOR: [u8; 4] = [0xff, 0xc8, 0x5c, 0xFF]; // Blue accent
pub const AMBER_COLOR: [u8; 4] = [0x77, 0xc1, 0xf5, 0xFF]; // Amber accent
pub const RED_COLOR: [u8; 4] = [0x70, 0x87, 0xd0, 0xFF]; // Red accent

// Semantic colors
pub const SECURE_COLOR: [u8; 4] = ACCENT_COLOR; // Green = secure
pub const INSECURE_COLOR: [u8; 4] = RED_COLOR; // Red = insecure
pub const WARNING_COLOR: [u8; 4] = AMBER_COLOR; // Amber = warning
