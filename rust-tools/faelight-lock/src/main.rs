//! faelight-lock v2.0.0 - Screen Locker (swaylock wrapper)
//! 🌲 Faelight Forest
//!
//! Uses faelight-core Theme to provide consistent colors to swaylock

use clap::Parser;
use faelight_core::Theme;
use std::process::Command;

#[derive(Parser)]
#[command(name = "faelight-lock")]
#[command(about = "Screen locker with Faelight Forest theming", long_about = None)]
#[command(version = "2.0.0")]
struct Args {
    /// Run health check and exit
    #[arg(long)]
    health_check: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.health_check {
        return health_check();
    }

    eprintln!("🔒 faelight-lock v2.0.0");

    let theme = Theme::faelight_default();

    // Convert colors to hex strings for swaylock
    let bg = format!("{:06x}", theme.bg_primary);
    let accent = format!("{:06x}", theme.accent);
    let blue = format!("{:06x}", theme.accent_hover);
    let text = format!("{:06x}", theme.text_primary);
    let danger = format!("{:06x}", theme.danger);

    // Use swaylock with Faelight Forest colors from theme
    let status = Command::new("swaylock")
        .args([
            "-f",
            "--color",
            &bg,
            "--inside-color",
            &bg,
            "--ring-color",
            &accent,
            "--key-hl-color",
            &accent,
            "--text-color",
            &text,
            "--line-color",
            "00000000",
            "--separator-color",
            "00000000",
            "--inside-clear-color",
            &bg,
            "--ring-clear-color",
            &blue,
            "--inside-wrong-color",
            &bg,
            "--ring-wrong-color",
            &danger,
            "--text-wrong-color",
            &danger,
            "--inside-ver-color",
            &bg,
            "--ring-ver-color",
            &accent,
            "--indicator-radius",
            "100",
            "--indicator-thickness",
            "10",
        ])
        .status()?;

    if !status.success() {
        return Err("swaylock exited with error".into());
    }

    Ok(())
}

fn health_check() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏥 faelight-lock health check");

    // Check if swaylock is installed
    let swaylock_check = Command::new("which").arg("swaylock").output()?;

    if !swaylock_check.status.success() {
        eprintln!("❌ swaylock: not found");
        return Err("swaylock not installed".into());
    }
    println!("✅ swaylock: installed");

    // Check if we can load theme
    let _theme = Theme::faelight_default();
    println!("✅ theme: loaded successfully");

    println!("\n✅ All checks passed!");
    Ok(())
}
