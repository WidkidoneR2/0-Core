//! faelight-lock v0.1 - Screen Locker (swaylock wrapper)
//! 🌲 Faelight Forest
//!
//! Shows Faelight visual then delegates to swaylock for PAM auth

use std::process::Command;

fn main() {
    eprintln!("🔒 faelight-lock v0.1");
    
    // Use swaylock with Faelight Forest colors
    Command::new("swaylock")
        .args([
            "-f",
            "--color", "0f1411",
            "--inside-color", "0f1411",
            "--ring-color", "a3e36b",
            "--key-hl-color", "6be3a3",
            "--text-color", "d7e0da",
            "--line-color", "00000000",
            "--separator-color", "00000000",
            "--inside-clear-color", "0f1411",
            "--ring-clear-color", "5cc8ff",
            "--inside-wrong-color", "0f1411",
            "--ring-wrong-color", "e36b6b",
            "--text-wrong-color", "e36b6b",
            "--inside-ver-color", "0f1411",
            "--ring-ver-color", "a3e36b",
            "--indicator-radius", "100",
            "--indicator-thickness", "10",
        ])
        .status()
        .expect("Failed to run swaylock");
}
