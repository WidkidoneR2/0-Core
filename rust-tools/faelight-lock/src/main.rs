//! faelight-lock v2.1.0 - Screen Locker (swaylock wrapper)
//! 🌲 Faelight Forest
//!
//! Uses faelight-core Theme to provide consistent colors to swaylock

use clap::Parser;
use faelight_core::Theme;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "faelight-lock")]
#[command(about = "Screen locker with Faelight Forest theming", long_about = None)]
#[command(version = "2.1.0")]
struct Args {
    /// Run health check and exit
    #[arg(long)]
    health_check: bool,

    /// Custom lock screen message
    #[arg(short, long)]
    message: Option<String>,

    /// Grace period in seconds (unlock without password)
    #[arg(short, long, default_value = "0")]
    grace: u64,

    /// Urgent lock (skip grace period)
    #[arg(short, long)]
    urgent: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.health_check {
        return health_check();
    }

    eprintln!("🔒 faelight-lock v2.1.0");

    // Grace period (unless urgent)
    if args.grace > 0 && !args.urgent {
        eprintln!("⏳ Grace period: {} seconds", args.grace);
        thread::sleep(Duration::from_secs(args.grace));
    }

    let theme = Theme::faelight_default();

    // Convert colors to hex strings for swaylock
    let bg = format!("{:06x}", theme.bg_primary);
    let accent = format!("{:06x}", theme.accent);
    let blue = format!("{:06x}", theme.accent_hover);
    let text = format!("{:06x}", theme.text_primary);
    let danger = format!("{:06x}", theme.danger);

    // Build swaylock command
    let mut cmd = Command::new("swaylock");
    cmd.args([
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
    ]);

    // Add custom message if provided
    if let Some(msg) = args.message {
        eprintln!("💬 Message: {}", msg);
        cmd.args(["--text", &msg]);
    }

    let status = cmd.status()?;

    if !status.success() {
        eprintln!("❌ swaylock exited with error");
        eprintln!("💡 Make sure swaylock is installed: sudo pacman -S swaylock");
        return Err("swaylock exited with error".into());
    }

    Ok(())
}

fn health_check() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏥 faelight-lock v2.1.0 health check");

    // Check if swaylock is installed
    let swaylock_check = Command::new("which").arg("swaylock").output()?;

    if !swaylock_check.status.success() {
        eprintln!("❌ swaylock: not found");
        eprintln!("💡 Install with: sudo pacman -S swaylock");
        return Err("swaylock not installed".into());
    }
    println!("✅ swaylock: installed");

    // Check if we can load theme
    let _theme = Theme::faelight_default();
    println!("✅ theme: loaded successfully");

    println!("\n✅ All checks passed!");
    Ok(())
}
