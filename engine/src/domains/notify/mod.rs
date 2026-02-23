#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::process::Command;

pub fn send(_ctx: &AppContext, summary: &str, body: Option<&str>, urgency: &str) -> CoreResult<()> {
    _ctx.capabilities
        .require("notify", &[Capability::SpawnProcess])?;
    let mut cmd = Command::new("notify-send");
    cmd.arg(format!("--urgency={}", urgency));
    cmd.arg(summary);
    if let Some(b) = body {
        cmd.arg(b);
    }
    let status = cmd.status()?;
    if status.success() {
        println!(
            "  {} Notification sent: {}",
            "✅".green(),
            summary.bright_white()
        );
    } else {
        println!("  {} notify-send failed", "✗".bright_red());
    }
    Ok(())
}

pub fn status(_ctx: &AppContext) -> CoreResult<()> {
    // Check if faelight-notify daemon is running
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "faelight-notify"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔔 Notify Status".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    let status_colored = match output.as_str() {
        "active" => output.bright_green().to_string(),
        "inactive" => output.dimmed().to_string(),
        _ => output.bright_yellow().to_string(),
    };
    println!("  Daemon: {}", status_colored);
    Ok(())
}
