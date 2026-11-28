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
    // systemctl --user is-active for the old notifier's unit and printed the result as Daemon.
    // There has never been such a unit on Omarchy, and is-active answers inactive for a
    // unit it has never heard of, not unknown.
    // So the _ => bright_yellow arm never fired and the command reported a daemon as
    // inactive while notifications were working perfectly through someone else.
    //
    // THE QUESTION IS WHO OWNS THE BUS NAME. That is where a notification actually
    // goes, and desktop() below already talks to it. Measured 2026-09-02: Quickshell
    // owns org.freedesktop.Notifications, which is why the old notifier was retired --
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
/// Proven path: busctl. Used by reactions (intent complete, health/integrity drops).
///
/// ⚠️ HONOURS ZERO_NO_NOTIFY, AND HERE IS WHAT THAT IS FOR. nsh spawns `core doctor run` at
/// startup to refresh a stale health event (INT-124), detached and with stdout and stderr to
/// /dev/null so it is invisible (INT-176). BUT THIS FUNCTION GOES OVER busctl, WHICH THOSE
/// REDIRECTS DO NOT TOUCH -- so a cache-warm nobody asked for could interrupt at critical
/// urgency.
///
/// ⭐ MEASURED 2026-09-20 (INT-254): nsh-test runs a case with HOME pointed at an empty
/// directory, deliberately. The spawned shell refreshes health, that doctor cannot find
/// checks.toml under the fake HOME, the load error is correctly Tier::Critical, verdict() is
/// correctly Red -- and four notifications arrived per suite run. Every part of that chain is
/// right except a silent process being able to shout.
///
/// ★ THE VARIABLE IS SET BY THE CALLER THAT WANTS SILENCE, NEVER READ AS A DEFAULT. An
/// interactive `d` still notifies; only a spawn that declared itself invisible is quiet.
pub fn desktop(summary: &str, body: &str, critical: bool) {
    if std::env::var("ZERO_NO_NOTIFY").is_ok() {
        return;
    }
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
            "Project 0",
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
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F-0013. desktop() must not print busctl's reply. A fake busctl, first on
    /// PATH, prints `u 1` and exits 0. The check runs in a child so that inherited
    /// stdout is the pipe we capture. The real session bus is never contacted.
    #[test]
    fn f0013_desktop_notify_prints_nothing() {
        if std::env::var_os("F0013_CHILD").is_some() {
            let prev = std::env::var_os("ZERO_NO_NOTIFY");
            unsafe { std::env::remove_var("ZERO_NO_NOTIFY") };
            desktop("INT-317", "quiet", false);
            match prev {
                Some(value) => unsafe { std::env::set_var("ZERO_NO_NOTIFY", value) },
                None => {}
            }
            return;
        }

        let _guard = crate::HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!(
            "f0013-busctl-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("ran");
        let script = dir.join("busctl");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf 'u 1\\n'\n: > '{}'\nexit 0\n",
                marker.display()
            ),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

        let old_path = std::env::var("PATH").unwrap_or_default();
        let child_path = format!("{}:{}", dir.display(), old_path);
        let exe = std::env::current_exe().unwrap();
        // The fake busctl was written a moment ago. Linux refuses to run a file while any
        // process holds it open for writing, and another test thread that forks between our
        // write and the exec carries our write handle until its own exec. busctl() ignores
        // a failed start, so the one sign is a marker that never appears. Run the child again
        // (up to five times) until the marker is there; the asserts below are unchanged.
        let mut attempts = 0;
        let out = loop {
            attempts += 1;
            let out = std::process::Command::new(&exe)
                .args([
                    "domains::notify::tests::f0013_desktop_notify_prints_nothing",
                    "--exact",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env("F0013_CHILD", "1")
                .env("PATH", &child_path)
                .env_remove("ZERO_NO_NOTIFY")
                .env(
                    "DBUS_SESSION_BUS_ADDRESS",
                    format!("unix:path={}", dir.join("no-bus").display()),
                )
                .output()
                .unwrap();
            if marker.exists() || attempts >= 5 {
                break out;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let marker_ran = marker.exists();
        let status = out.status;
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            status.success(),
            "child failed: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            marker_ran,
            "fake busctl did not run, so this result must not be trusted"
        );
        assert!(
            !stdout.contains("u 1"),
            "busctl stdout leaked into the caller:\n{stdout}"
        );
    }
}
