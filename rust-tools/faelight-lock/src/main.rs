//! faelight-lock v0.2 - Screen Locker (swaylock wrapper)
//! 🌲 Faelight Forest
//!
//! Uses faelight-core Theme to provide consistent colors to swaylock

use faelight_core::Theme;
use std::process::Command;

fn main() {
    eprintln!("🔒 faelight-lock v0.2");
    
    let theme = Theme::faelight_default();
    
    // Convert colors to hex strings for swaylock
    let bg = format!("{:06x}", theme.bg_primary);
    let accent = format!("{:06x}", theme.accent);
    let blue = format!("{:06x}", theme.accent_hover);
    let text = format!("{:06x}", theme.text_primary);
    let danger = format!("{:06x}", theme.danger);
    
    // Use swaylock with Faelight Forest colors from theme
    Command::new("swaylock")
        .args([
            "-f",
            "--color", &bg,
            "--inside-color", &bg,
            "--ring-color", &accent,
            "--key-hl-color", &accent,
            "--text-color", &text,
            "--line-color", "00000000",
            "--separator-color", "00000000",
            "--inside-clear-color", &bg,
            "--ring-clear-color", &blue,
            "--inside-wrong-color", &bg,
            "--ring-wrong-color", &danger,
            "--text-wrong-color", &danger,
            "--inside-ver-color", &bg,
            "--ring-ver-color", &accent,
            "--indicator-radius", "100",
            "--indicator-thickness", "10",
        ])
        .status()
        .expect("Failed to run swaylock");
}
