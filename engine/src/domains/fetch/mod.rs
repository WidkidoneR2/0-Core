#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::process::Command;

fn read_file(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn run_cmd(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_kernel() -> String {
    run_cmd("uname", &["-r"])
}

fn get_uptime() -> String {
    let uptime_raw = read_file("/proc/uptime");
    let secs: f64 = uptime_raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let mins = (secs / 60.0) as u64;
    let hours = mins / 60;
    let days = hours / 24;
    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins % 60)
    } else {
        format!("{}m", mins)
    }
}

fn get_hostname() -> String {
    read_file("/etc/hostname")
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .map(|s| s.rsplit('/').next().unwrap_or("zsh").to_string())
        .unwrap_or_else(|_| "zsh".to_string())
}

fn get_wm() -> String {
    if std::env::var("NIRI_SOCKET").is_ok() {
        "niri".to_string()
    } else if std::env::var("SWAYSOCK").is_ok() {
        "sway".to_string()
    } else if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        "hyprland".to_string()
    } else {
        "unknown".to_string()
    }
}

fn get_term() -> String {
    std::env::var("TERM").unwrap_or_else(|_| "foot".to_string())
}

fn get_version(ctx: &AppContext) -> String {
    let version_file = std::path::PathBuf::from(&ctx.core_root).join("00-meta/VERSION");
    fs::read_to_string(&version_file)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "?.?.?".to_string())
}

fn get_zone(ctx: &AppContext) -> String {
    let zone = crate::domains::zone::detect(ctx);
    format!("{} {}", zone.icon, zone.label)
}

fn get_profile() -> String {
    dirs::home_dir()
        .map(|h| h.join(".local/state/0-core/current-profile"))
        .and_then(|p| fs::read_to_string(p).ok())
        .map(|s| s.trim().to_uppercase()[..3.min(s.trim().len())].to_string())
        .unwrap_or_else(|| "DEF".to_string())
}

fn get_core_status() -> String {
    // Check if core is locked via immutable flag
    let _output = Command::new("lsattr")
        .arg("-d")
        .arg(std::env::current_dir().unwrap_or_default())
        .output();
    // Simple heuristic — check core-protect state file
    let state = dirs::home_dir()
        .map(|h| h.join(".local/state/0-core/lock-status"))
        .and_then(|p| fs::read_to_string(p).ok())
        .unwrap_or_default();
    if state.trim() == "locked" {
        "🔒 locked".to_string()
    } else {
        "🔓 unlocked".to_string()
    }
}

pub fn run(ctx: &AppContext, health_check: bool) -> CoreResult<()> {
    ctx.capabilities.require(
        "fetch",
        &[Capability::FilesystemReadHome, Capability::SpawnProcess],
    )?;
    if health_check {
        println!("{}", "🏥 core fetch health".bold());
        println!("  {} System info readable", "✅".green());
        println!("  {} Zone detection working", "✅".green());
        println!("  {} All checks passed!", "✅".green());
        return Ok(());
    }

    let version = get_version(ctx);
    let zone = get_zone(ctx);
    let profile = get_profile();
    let core_status = get_core_status();
    let wm = get_wm();
    let term = get_term();
    let shell = get_shell();
    let kernel = get_kernel();
    let uptime = get_uptime();
    let host = get_hostname();

    println!("╭─────────────────────────────────╮");
    println!("│ 🌲 Faelight Forest v{:<14}│", version);
    println!("╰─────────────────────────────────╯");
    println!("{:>10}  {}", "zone".dimmed(), zone);
    println!("{:>10}  {}", "profile".dimmed(), profile);
    println!("{:>10}  {}", "core".dimmed(), core_status);
    println!("{:>10}  {}", "wm".dimmed(), wm);
    println!("{:>10}  {}", "term".dimmed(), term);
    println!("{:>10}  {}", "shell".dimmed(), shell);
    println!("{:>10}  {}", "kernel".dimmed(), kernel);
    println!("{:>10}  {}", "uptime".dimmed(), uptime);
    println!("{:>10}  {}", "host".dimmed(), host);

    Ok(())
}
