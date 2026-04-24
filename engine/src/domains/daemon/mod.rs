//! daemon domain — query faelight-daemon v2 over Unix socket
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
fn socket_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/.local/state/0-core/daemon.sock", home)
}
fn send_command(cmd: serde_json::Value) -> Result<serde_json::Value, String> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)
        .map_err(|e| format!("Cannot connect to daemon socket at {}: {}", path, e))?;

    let msg = serde_json::json!({ "id": 1, "payload": cmd });
    let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;

    stream
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).map_err(|e| e.to_string())?;

    serde_json::from_str(&response).map_err(|e| format!("Parse error: {}", e))
}
/// core daemon status
pub fn status(_ctx: &AppContext) -> CoreResult<()> {
    println!();
    println!("{}", "🌲 faelight-daemon Status".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    // Check if daemon is responding
    match send_command(serde_json::json!("Ping")) {
        Ok(_) => println!(
            "  {} Daemon: {}",
            "●".bright_green(),
            "running".bright_green()
        ),
        Err(e) => {
            println!(
                "  {} Daemon: {} — {}",
                "●".bright_red(),
                "not running".bright_red(),
                e
            );
            println!();
            return Ok(());
        }
    }
    // Get forest context
    match send_command(serde_json::json!("GetForestContext")) {
        Ok(resp) => {
            if let Some(payload) = resp.get("payload") {
                if let Some(fc) = payload.get("ForestContext") {
                    let health = fc["health"].as_u64().unwrap_or(0);
                    let alignment = fc["alignment"].as_f64().unwrap_or(0.0);
                    let commits = fc["commits_today"].as_i64().unwrap_or(0);
                    let friday = fc["friday_status"].as_str().unwrap_or("unknown");
                    let intent = fc["active_intent"].as_str().unwrap_or("none");
                    let prediction = fc["top_prediction"].as_str().unwrap_or("none");
                    println!(
                        "  {} Health: {}%",
                        "→".dimmed(),
                        if health == 100 {
                            health.to_string().bright_green()
                        } else {
                            health.to_string().bright_yellow()
                        }
                    );
                    println!("  {} Alignment: {:.0}%", "→".dimmed(), alignment * 100.0);
                    println!("  {} Active intent: {}", "→".dimmed(), intent.bright_cyan());
                    println!(
                        "  {} Commits today: {}",
                        "→".dimmed(),
                        commits.to_string().bright_white()
                    );
                    println!("  {} Friday: {}", "→".dimmed(), friday.dimmed());
                    println!(
                        "  {} Top prediction: {}",
                        "→".dimmed(),
                        prediction.bright_yellow()
                    );
                }
            }
        }
        Err(e) => println!("  {} Context unavailable: {}", "⚠️ ".yellow(), e),
    }
    // Watchdog status
    if let Ok(resp) = send_command(serde_json::json!("WatchdogStatus")) {
        if let Some(payload) = resp.get("payload") {
            if let Some(wd) = payload.get("Watchdog") {
                let alerts = wd["alerts_today"].as_i64().unwrap_or(0);
                let last_health = wd["last_health"].as_u64().unwrap_or(0);
                if alerts > 0 {
                    println!(
                        "  {} Watchdog: {} alerts today, last health {}%",
                        "⚠️ ".yellow(),
                        alerts.to_string().bright_red(),
                        last_health
                    );
                } else {
                    println!(
                        "  {} Watchdog: {} alerts today",
                        "→".dimmed(),
                        "0".bright_green()
                    );
                }
            }
        }
    }
    println!();
    Ok(())
}
/// core daemon context
pub fn context(_ctx: &AppContext) -> CoreResult<()> {
    println!();
    println!("{}", "🌿 Forest Context (via daemon)".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    match send_command(serde_json::json!("GetForestContext")) {
        Ok(resp) => {
            if let Some(payload) = resp.get("payload") {
                if let Some(fc) = payload.get("ForestContext") {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(fc)
                            .unwrap_or_default()
                            .dimmed()
                    );
                } else {
                    println!("  {} Unexpected response: {}", "⚠️ ".yellow(), payload);
                }
            }
        }
        Err(e) => println!("  {} Error: {}", "❌".red(), e),
    }
    println!();
    Ok(())
}
/// core daemon signals [limit]
pub fn signals(_ctx: &AppContext, limit: u32) -> CoreResult<()> {
    println!();
    println!("{}", "📡 Engine Signals (via daemon)".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    match send_command(serde_json::json!({"GetEngineSignals": {"limit": limit}})) {
        Ok(resp) => {
            if let Some(payload) = resp.get("payload") {
                if let Some(es) = payload.get("EngineSignals") {
                    if let Some(signals) = es["signals"].as_array() {
                        if signals.is_empty() {
                            println!("  {} No signals yet", "○".dimmed());
                        }
                        for sig in signals {
                            let source = sig["source"].as_str().unwrap_or("?");
                            let kind = sig["signal_type"].as_str().unwrap_or("?");
                            let weight = sig["weight"].as_f64().unwrap_or(0.0);
                            let ts = sig["created_at"].as_i64().unwrap_or(0);
                            let time = chrono::DateTime::from_timestamp(ts, 0)
                                .map(|t| t.format("%H:%M:%S").to_string())
                                .unwrap_or_default();
                            println!(
                                "  {} {} → {} (weight: {:.2}) @ {}",
                                "·".dimmed(),
                                source.bright_cyan(),
                                kind.bright_white(),
                                weight,
                                time.dimmed()
                            );
                        }
                    }
                }
            }
        }
        Err(e) => println!("  {} Error: {}", "❌".red(), e),
    }
    println!();
    Ok(())
}
/// core daemon neovim <file>
pub fn neovim(_ctx: &AppContext, file_path: &str) -> CoreResult<()> {
    match send_command(serde_json::json!({"GetNeovimContext": {"file_path": file_path}})) {
        Ok(resp) => {
            if let Some(payload) = resp.get("payload") {
                if let Some(nc) = payload.get("NeovimContext") {
                    let intent = nc["active_intent"].as_str().unwrap_or("none");
                    let title = nc["intent_title"].as_str().unwrap_or("");
                    let suggestion = nc["suggestion"].as_str();
                    println!();
                    println!(
                        "  {} {}: {}",
                        "🌲".normal(),
                        intent.bright_cyan(),
                        title.dimmed()
                    );
                    if let Some(s) = suggestion {
                        println!("  {} {}", "💡".normal(), s.bright_yellow());
                    }
                    println!();
                }
            }
        }
        Err(e) => println!("  {} Daemon unavailable: {}", "⚠️ ".yellow(), e),
    }
    Ok(())
}
/// core daemon watchdog
pub fn watchdog(_ctx: &AppContext) -> CoreResult<()> {
    println!();
    println!("{}", "🔍 Health Watchdog Status".cyan().bold());
    println!("{}", "━".repeat(52).dimmed());
    println!();
    match send_command(serde_json::json!("WatchdogStatus")) {
        Ok(resp) => {
            if let Some(payload) = resp.get("payload") {
                if let Some(wd) = payload.get("Watchdog") {
                    let last_health = wd["last_health"].as_u64().unwrap_or(0);
                    let alerts = wd["alerts_today"].as_i64().unwrap_or(0);
                    let last_check = wd["last_check"].as_i64().unwrap_or(0);
                    let time = chrono::DateTime::from_timestamp(last_check, 0)
                        .map(|t| t.format("%H:%M:%S").to_string())
                        .unwrap_or_default();
                    println!(
                        "  {} Last health: {}%",
                        "→".dimmed(),
                        if last_health >= 100 {
                            last_health.to_string().bright_green()
                        } else {
                            last_health.to_string().bright_yellow()
                        }
                    );
                    println!("  {} Last check: {}", "→".dimmed(), time.dimmed());
                    println!(
                        "  {} Alerts today: {}",
                        "→".dimmed(),
                        if alerts == 0 {
                            "0".bright_green()
                        } else {
                            alerts.to_string().bright_red()
                        }
                    );
                }
            }
        }
        Err(e) => println!("  {} Error: {}", "❌".red(), e),
    }
    println!();
    Ok(())
}
