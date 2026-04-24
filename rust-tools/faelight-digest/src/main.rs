// faelight-digest v1.0.0
// Forest morning digest — system stats + forest context
// Replaces faelight-fetch as the `fae` / `faelight` command

use colored::*;
use sysinfo::System;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let brief = args.iter().any(|a| a == "--brief" || a == "-b");
    let health_only = args.iter().any(|a| a == "--health");

    if health_only {
        println!("faelight-digest v1.0.0 — healthy");
        return;
    }

    if !brief {
        print_system_panel();
    }
    print_forest_context();
}

fn print_system_panel() {
    let home = std::env::var("HOME").unwrap_or_default();
    let version = std::fs::read_to_string(format!("{}/0-core/00-meta/VERSION", home))
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();
    let theme = {
        let changelog = std::fs::read_to_string(format!("{}/0-core/00-meta/CHANGELOG.md", home))
            .unwrap_or_default();
        changelog
            .lines()
            .find(|l| l.contains(&format!("[{}]", version)))
            .and_then(|l| l.split(" — ").nth(1))
            .and_then(|s| s.split('(').next())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "The Forest Grows".into())
    };

    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "fealight".into())
        .trim()
        .to_string();

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "zsh".into());
    let shell_name = shell.split('/').next_back().unwrap_or("zsh");

    let kernel = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    let rust_ver = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .to_string()
        })
        .unwrap_or_else(|_| "unknown".into());

    let uptime_raw = std::fs::read_to_string("/proc/uptime").unwrap_or_default();
    let secs: f64 = uptime_raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let mins = (secs / 60.0) as u64;
    let uptime = if mins >= 1440 {
        format!("{}d {}h", mins / 1440, (mins % 1440) / 60)
    } else if mins >= 60 {
        format!("{}h {}m", mins / 60, mins % 60)
    } else {
        format!("{}m", mins)
    };

    let total_mem = sys.total_memory() / 1024 / 1024 / 1024;
    let used_mem_mb = sys.used_memory() / 1024 / 1024;
    let used_mem = if used_mem_mb >= 1024 {
        format!("{:.1}G", used_mem_mb as f64 / 1024.0)
    } else {
        format!("{}M", used_mem_mb)
    };

    let cpu: f32 =
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len().max(1) as f32;

    let health_num: u32 =
        std::fs::read_to_string(format!("{}/0-core/runtime/cache/health.txt", home))
            .unwrap_or_else(|_| "95".into())
            .trim()
            .trim_end_matches('%')
            .parse()
            .unwrap_or(95);

    let health_icon = if health_num >= 95 {
        "🟢"
    } else if health_num >= 80 {
        "🟡"
    } else {
        "🔴"
    };

    let lock_status = std::process::Command::new("lsattr")
        .args(["-d", &format!("{}/0-core", home)])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("----i"))
        .unwrap_or(false);
    let lock_icon = if lock_status {
        "🔒 locked"
    } else {
        "🔓 unlocked"
    };

    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_default()
        .trim()
        .to_string();
    let tools = std::fs::read_dir(format!("{}/0-core/scripts", home))
        .map(|d| d.flatten().filter(|e| e.path().is_file()).count())
        .unwrap_or(0);

    let wm = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("WAYLAND_DISPLAY").map(|_| "niri".into()))
        .unwrap_or_else(|_| "niri".into());

    println!();
    println!(
        "  {}",
        format!("╭─ 🌲 Faelight Forest {} ─╮", version)
            .bright_green()
            .bold()
    );
    println!("  {}    ", "system".dimmed());
    println!("  {:>14}  {}", "host".dimmed(), hostname.bright_white());
    println!(
        "  {:>14}  {}",
        "health".dimmed(),
        format!("{} {}%", health_icon, health_num)
    );
    println!("  {:>14}  {}", "core".dimmed(), lock_icon.dimmed());
    println!("  {}    ", "env".dimmed());
    println!("  {:>14}  {}", "wm".dimmed(), wm.bright_white());
    println!("  {:>14}  {}", "shell".dimmed(), shell_name.bright_white());
    println!("  {:>14}  {}", "kernel".dimmed(), kernel.bright_white());
    println!("  {:>14}  {}", "rust".dimmed(), rust_ver.bright_white());
    println!("  {:>14}  {}", "uptime".dimmed(), uptime.bright_white());
    println!("  {}    ", "resources".dimmed());
    println!(
        "  {:>14}  {}",
        "cpu".dimmed(),
        format!("{:.0}%", cpu).bright_white()
    );
    println!(
        "  {:>14}  {}",
        "memory".dimmed(),
        format!("{} / {}G", used_mem, total_mem).bright_white()
    );
    println!("  {}    ", "0-core".dimmed());
    println!(
        "  {:>14}  {}",
        "version".dimmed(),
        format!("{} — {}", version, theme).bright_green()
    );
    println!("  {:>14}  {}", "commits".dimmed(), commits.bright_white());
    println!(
        "  {:>14}  {}",
        "tools".dimmed(),
        tools.to_string().bright_white()
    );
    println!();
}

fn print_forest_context() {
    let home = std::env::var("HOME").unwrap_or_default();
    let db_path = format!("{}/0-core/runtime/state.db", home);
    let core_root = format!("{}/0-core", home);

    let Ok(conn) = rusqlite::Connection::open(&db_path) else {
        return;
    };

    let hour = chrono::Local::now().hour();
    use chrono::Timelike;
    let greeting = match hour {
        5..=11 => "Good morning. The forest has been thinking.",
        12..=17 => "Good afternoon. Here is where things stand.",
        18..=21 => "Good evening. Here is what happened today.",
        _ => "Welcome back. The forest kept watch.",
    };

    println!("  {} {}", "🌲".normal(), greeting.bright_white().bold());
    println!();

    // Last session gap
    let last_session: Option<i64> = conn
        .query_row(
            "SELECT value FROM session_state WHERE key='last_session_ts'",
            [],
            |r| r.get(0),
        )
        .ok();

    if let Some(ts) = last_session {
        let now = chrono::Local::now().timestamp();
        let gap_h = (now - ts) / 3600;
        if gap_h > 0 {
            println!(
                "  {} Since {} ago:",
                "→".bright_cyan(),
                if gap_h >= 24 {
                    format!("{}d", gap_h / 24)
                } else {
                    format!("{}h", gap_h)
                }
            );
        }
    } else {
        println!("  {} Since last session:", "→".bright_cyan());
    }

    // Health
    let health_num: u32 =
        std::fs::read_to_string(format!("{}/runtime/cache/health.txt", core_root))
            .unwrap_or_else(|_| "95".into())
            .trim()
            .trim_end_matches('%')
            .parse()
            .unwrap_or(95);

    let health_str = if health_num >= 95 {
        format!("{}% healthy", health_num)
            .bright_green()
            .to_string()
    } else {
        format!("{}% advisory", health_num).yellow().to_string()
    };
    println!("    {} Health: {}", "·".dimmed(), health_str);

    // Commits
    let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    if commits > 0 {
        println!(
            "    {} {} commits total",
            "·".dimmed(),
            commits.to_string().bright_white()
        );
    }

    // Active intents
    let intents_path = std::path::Path::new(&core_root).join("intents/future");
    let mut active: Vec<String> = vec![];
    if let Ok(entries) = std::fs::read_dir(&intents_path) {
        for e in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(e.path()) {
                if content.contains("status: in-progress") {
                    if let Some(num) = e
                        .file_name()
                        .to_string_lossy()
                        .split('-')
                        .next()
                        .map(|s| s.to_string())
                    {
                        active.push(format!("INT-{}", num));
                    }
                }
            }
        }
    }
    if !active.is_empty() {
        println!(
            "    {} Working on: {}",
            "·".dimmed(),
            active.join(", ").bright_cyan()
        );
    }

    // Recently modified files (top 3 from file_index)
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name FROM file_index WHERE kind='file' AND extension IN ('rs','toml','kdl','zsh','md') ORDER BY modified DESC LIMIT 3"
    ) {
        let files: Vec<String> = stmt.query_map([], |r| r.get(0))
            .unwrap().filter_map(|r| r.ok()).collect();
        if !files.is_empty() {
            println!("    {} Recent: {}", "·".dimmed(), files.join(", ").dimmed());
        }
    }

    // Reactions fired today
    let today = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let reactions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM reaction_log WHERE triggered_at >= ?1",
            rusqlite::params![today],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if reactions > 0 {
        println!(
            "    {} {} reaction{} fired today",
            "·".yellow(),
            reactions.to_string().yellow(),
            if reactions == 1 { "" } else { "s" }
        );
    }

    // Pending decisions
    let old_decisions: i64 = conn
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM decisions WHERE outcome='pending' AND timestamp < {}",
                chrono::Utc::now().timestamp() - 7 * 86400
            ),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if old_decisions > 0 {
        println!(
            "    {} {} pending decision{} older than 7 days",
            "·".yellow(),
            old_decisions.to_string().yellow(),
            if old_decisions == 1 { "" } else { "s" }
        );
    }

    println!();
    println!(
        "  {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
}
