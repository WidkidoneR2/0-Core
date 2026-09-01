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
    // ⚠️ THIS ASKED ABOUT THE WRONG THING AND ANSWERED CONFIDENTLY. It ran
    // systemctl --user is-active faelight-notify and printed the result as Daemon.
    // There has never been such a unit on Omarchy -- the session target died with nix/
    // -- and is-active answers inactive for a unit it has never heard of, not unknown.
    // So the _ => bright_yellow arm never fired and the command reported a daemon as
    // inactive while notifications were working perfectly through someone else.
    //
    // THE QUESTION IS WHO OWNS THE BUS NAME. That is where a notification actually
    // goes, and desktop() below already talks to it. Measured 2026-09-02: Quickshell
    // owns org.freedesktop.Notifications, which is why faelight-notify was retired --
    // it checks the same name at startup and correctly declines to compete.
    let owner = Command::new("busctl")
        .args([
            "--user",
            "call",
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "GetNameOwner",
            "s",
            "org.freedesktop.Notifications",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        // busctl answers with its type marker and quotes: s ":1.46". The connection id is
        // the answer; the rest is transport.
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_start_matches("s ")
                .trim_matches(char::from(34))
                .to_string()
        });

    // A CONNECTION ID IS TRUE AND USELESS. :1.46 names nothing a person recognises, so ask
    // the bus which PID holds it and read that process own name. Best-effort: if either
    // lookup fails the id still prints, because a thin answer beats a wrong one.
    let owner_name = owner.as_ref().and_then(|id| {
        let pid = Command::new("busctl")
            .args([
                "--user",
                "call",
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
                "GetConnectionUnixProcessID",
                "s",
                id,
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .trim_start_matches("u ")
                    .to_string()
            })?;
        std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .ok()
            .map(|c| c.trim().to_string())
    });

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🔔 Notify Status".bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    match owner {
        Some(o) => {
            println!(
                "  Bus name: {}",
                "org.freedesktop.Notifications".bright_white()
            );
            match &owner_name {
                Some(n) => println!("  Owned by: {} ({})", n.bright_green(), o.dimmed()),
                None => println!("  Owned by: {}", o.bright_green()),
            }
            println!("  {} notifications have somewhere to go", "✅".green());
        }
        None => {
            println!(
                "  Bus name: {}",
                "org.freedesktop.Notifications".bright_white()
            );
            println!("  Owned by: {}", "nobody".bright_yellow());
            println!(
                "  {} a notification sent now goes nowhere",
                "⚠".bright_yellow()
            );
        }
    }
    Ok(())
}

/// Fire-and-forget desktop notification via D-Bus (org.freedesktop.Notifications).
/// Proven path: busctl. Used by forest reactions (intent complete, health/integrity drops).
pub fn desktop(summary: &str, body: &str, critical: bool) {
    let urgency = if critical { "2" } else { "1" };
    let _ = Command::new("busctl")
        .args([
            "--user",
            "call",
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
            "Notify",
            "susssasa{sv}i",
            "Zero Core",
            "0",
            "",
            summary,
            body,
            "0",
            "1",
            "urgency",
            "y",
            urgency,
            "0",
        ])
        .status();
}
